import { ref, computed, readonly } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useMessages } from "./useMessages";
import type { PandocSource, PandocManager } from "../types/pandoc";

const availableSources = ref<PandocManager[]>([]);
const currentManager = ref<PandocManager | null>(null);
const isLoading = ref(false);

// Get source type display name (moved to outer scope)
const getSourceDisplayName = (source: PandocSource): string => {
  if ("Custom" in source) return `Custom: ${source.Custom}`;
  if ("Bundled" in source) return "Bundled/Portable";
  if ("System" in source) return `System: ${source.System}`;
  return "Unknown";
};

export function usePandocManager() {
  const { displayMessage } = useMessages();

  const isReady = computed(() => currentManager.value?.available ?? false);
  const pandocInfo = computed(() => currentManager.value?.info ?? null);
  const supportedOutputFormats = computed(
    () => pandocInfo.value?.supported_output_formats ?? [],
  );

  // Discover all available pandoc sources
  const discoverSources = async (): Promise<PandocManager[]> => {
    try {
      isLoading.value = true;
      const sources = await invoke<PandocManager[]>("discover_pandoc_sources");
      availableSources.value = sources;
      return sources;
    } catch (error) {
      displayMessage(`Failed to discover pandoc sources: ${error}`, "error");
      return [];
    } finally {
      isLoading.value = false;
    }
  };

  // Get the best available pandoc manager
  const getBestManager = async (): Promise<PandocManager | null> => {
    try {
      isLoading.value = true;
      const manager = await invoke<PandocManager | null>(
        "get_best_pandoc_manager",
      );
      currentManager.value = manager;
      return manager;
    } catch (error) {
      displayMessage(`Failed to get pandoc manager: ${error}`, "error");
      return null;
    } finally {
      isLoading.value = false;
    }
  };

  // Create and validate custom manager
  const useCustomPath = async (customPath: string): Promise<boolean> => {
    try {
      isLoading.value = true;
      const manager = await invoke<PandocManager>(
        "create_and_validate_custom_manager",
        {
          customPath,
        },
      );
      currentManager.value = manager;
      displayMessage("Custom pandoc path validated successfully", "success");
      return true;
    } catch (error) {
      displayMessage(`Failed to validate custom path: ${error}`, "error");
      return false;
    } finally {
      isLoading.value = false;
    }
  };

  // Update bundled pandoc
  const updateBundledPandoc = async (): Promise<boolean> => {
    try {
      isLoading.value = true;
      const result = await invoke<string>("update_bundled_pandoc");
      displayMessage(result, "success");
      // Refresh current manager after update
      await getBestManager();
      return true;
    } catch (error) {
      displayMessage(`Failed to update bundled pandoc: ${error}`, "error");
      return false;
    } finally {
      isLoading.value = false;
    }
  };

  // Check if bundled pandoc needs update
  const checkBundledUpdate = async (): Promise<boolean> => {
    try {
      const needsUpdate = await invoke<boolean>("check_bundled_pandoc_update");
      return needsUpdate;
    } catch (error) {
      displayMessage(`Failed to check for updates: ${error}`, "error");
      return false;
    }
  };

  // Initialize pandoc manager
  const initializePandoc = async () => {
    try {
      // Get best manager first (internally calls discover_pandoc_sources)
      await getBestManager();

      // If no best manager found, explicitly discover all sources
      if (!currentManager.value?.available) {
        await discoverSources();
      }
    } catch (error) {
      displayMessage(`Failed to initialize pandoc: ${error}`, "error");
    }
  };

  // Switch to a different pandoc source
  const switchSource = (manager: PandocManager) => {
    if (manager.available) {
      currentManager.value = manager;
      displayMessage(
        `Switched to ${getSourceDisplayName(manager.source)}`,
        "success",
      );
    } else {
      displayMessage("Cannot switch to unavailable pandoc source", "error");
    }
  };

  return {
    // State
    availableSources: readonly(availableSources),
    currentManager: readonly(currentManager),
    isLoading: readonly(isLoading),

    // Computed
    isReady,
    pandocInfo,
    supportedOutputFormats,

    // Methods
    discoverSources,
    getBestManager,
    useCustomPath,
    updateBundledPandoc,
    checkBundledUpdate,
    initializePandoc,
    getSourceDisplayName,
    switchSource,
  };
}
