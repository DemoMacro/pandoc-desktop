import { ref, computed, readonly } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { PandocInfo } from "../types/pandoc";

const pandocInfo = ref<PandocInfo | null>(null);
const customPandocPath = ref<string>("");

export function usePandoc() {
  const isReady = computed(() => pandocInfo.value?.is_working ?? false);
  const supportedOutputFormats = computed(
    () => pandocInfo.value?.supported_output_formats ?? [],
  );

  const loadPandocInfo = async () => {
    try {
      // Try the new portable-aware command first
      const info = await invoke<PandocInfo>("get_pandoc_info_with_portable", {
        custom_path: customPandocPath.value || null,
      });
      pandocInfo.value = info;
      return info;
    } catch {
      // Fallback to original command if the new one fails
      try {
        const info = await invoke<PandocInfo>("get_pandoc_info", {
          custom_path: customPandocPath.value || null,
        });
        pandocInfo.value = info;
        return info;
      } catch (fallbackError) {
        pandocInfo.value = null;
        throw fallbackError;
      }
    }
  };

  const validatePandocPath = async (path: string): Promise<boolean> => {
    return await invoke<boolean>("validate_pandoc_path", { path });
  };

  const initializePandoc = async () => {
    try {
      // First try to setup bundled Pandoc if available
      try {
        const setupResult = await invoke<string>("setup_bundled_pandoc");
        console.log("Bundled Pandoc setup:", setupResult);
      } catch (setupError) {
        console.warn("Bundled Pandoc not available:", setupError);
        // This is not a critical error, continue with normal initialization
      }

      // Then load Pandoc info (this will check bundled, portable, and system installations)
      await loadPandocInfo();
    } catch (error) {
      console.error("Failed to initialize Pandoc:", error);
    }
  };

  return {
    pandocInfo: readonly(pandocInfo),
    customPandocPath,
    isReady,
    supportedOutputFormats,
    loadPandocInfo,
    validatePandocPath,
    initializePandoc,
  };
}
