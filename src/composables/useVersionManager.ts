import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useMessages } from "./useMessages";
import type { PandocInfo } from "../types/pandoc";

export interface PandocRelease {
  tag_name: string;
  name: string;
  body: string;
  published_at: string;
  assets: Array<{
    name: string;
    download_url: string;
    size: number;
    content_type: string;
  }>;
}

export interface DownloadProgress {
  downloaded: number;
  total: number;
  progress: number;
  current_mirror: string;
}

const currentVersion = ref<string>("");
const latestVersion = ref<string>("");
const updateAvailable = ref<boolean>(false);
const downloading = ref<boolean>(false);
const checking = ref<boolean>(false);
const downloadProgress = ref<DownloadProgress>({
  downloaded: 0,
  total: 0,
  progress: 0,
  current_mirror: "",
});

const formatBytes = (bytes: number): string => {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
};

export function useVersionManager() {
  const { displayMessage } = useMessages();

  const checkForUpdates = async (): Promise<boolean> => {
    checking.value = true;

    try {
      displayMessage("Checking for updates...", "info");

      const release = await invoke<PandocRelease>("get_latest_pandoc_release");
      latestVersion.value = release.tag_name;

      // Get current version from actual Pandoc info (bundled, portable, or system)
      try {
        const pandocInfo = await invoke<PandocInfo>(
          "get_pandoc_info_with_portable",
          {
            custom_path: null,
          },
        );
        // Extract version number from the version string
        const versionMatch = pandocInfo.version.match(
          /(\d+\.\d+\.\d+(?:\.\d+)?)/,
        );
        currentVersion.value = versionMatch
          ? versionMatch[1]
          : pandocInfo.version;
      } catch {
        // Fallback to basic version check
        const currentVersion_str = await invoke<string>("check_pandoc_version");
        currentVersion.value = currentVersion_str;
      }

      // Clean up version strings for comparison (remove 'v' prefix if present)
      const cleanCurrent = currentVersion.value.replace(/^v/, "");
      const cleanLatest = latestVersion.value.replace(/^v/, "");

      updateAvailable.value = cleanLatest !== cleanCurrent;

      if (updateAvailable.value) {
        displayMessage(
          `Update available: ${cleanCurrent} → ${cleanLatest}`,
          "info",
        );
      } else {
        displayMessage("Already up to date", "success");
      }

      return updateAvailable.value;
    } catch (error) {
      displayMessage(`Failed to check for updates: ${error}`, "error");
      return false;
    } finally {
      checking.value = false;
    }
  };

  const downloadLatestVersion = async (): Promise<void> => {
    if (!updateAvailable.value) return;

    downloading.value = true;

    try {
      displayMessage("Starting download...", "info");

      // Download to application data directory
      const downloadPath = await invoke<string>("download_pandoc", {
        version: latestVersion.value,
        downloadDir: "./downloads",
      });

      displayMessage(
        `Download completed! File saved to: ${downloadPath}`,
        "success",
      );

      currentVersion.value = latestVersion.value;
      updateAvailable.value = false;
    } catch (error) {
      displayMessage(`Download failed: ${error}`, "error");
    } finally {
      downloading.value = false;
    }
  };

  return {
    currentVersion,
    latestVersion,
    updateAvailable,
    downloading,
    checking,
    downloadProgress,
    checkForUpdates,
    downloadLatestVersion,
    formatBytes,
  };
}
