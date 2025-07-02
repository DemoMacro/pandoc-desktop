<template>
  <dialog :open="showPandocManager" @click="closePandocManager">
    <article @click.stop>
      <header>
        📦 Pandoc Manager
        <button
          aria-label="Close"
          rel="prev"
          @click="closePandocManager"
        ></button>
      </header>

      <section v-if="pandocInfo">
        <h4>Current Installation</h4>
        <div class="info-grid">
          <div class="info-item">
            <h6>Version</h6>
            <p>{{ pandocInfo.version }}</p>
          </div>
          <div class="info-item">
            <h6>Status</h6>
            <p :class="{ 'status-ready': isReady, 'status-error': !isReady }">
              {{ isReady ? "✅ Working" : "❌ Not Working" }}
            </p>
          </div>
          <div class="info-item">
            <h6>Latest Available</h6>
            <p>{{ latestVersion || "Checking..." }}</p>
          </div>
        </div>

        <details>
          <summary>Path Information</summary>
          <p>
            <small
              ><code>{{ cleanWindowsPath(pandocInfo.path) }}</code></small
            >
          </p>
        </details>

        <details v-if="pandocInfo.detected_paths.length > 1">
          <summary>
            All Detected Installations ({{ pandocInfo.detected_paths.length }})
          </summary>
          <div v-for="(path, index) in pandocInfo.detected_paths" :key="index">
            <p>
              <small>
                <span>{{ path === pandocInfo.path ? "🟢" : "⚪" }}</span>
                <code>{{ cleanWindowsPath(path) }}</code>
                <button
                  v-if="path !== pandocInfo.path"
                  @click="switchPandocPath(path)"
                  :disabled="switching"
                  :aria-busy="switching"
                  class="outline secondary"
                  style="margin-left: 0.5rem; padding: 0.25rem 0.5rem"
                >
                  {{ switching ? "Switching..." : "Use This" }}
                </button>
              </small>
            </p>
          </div>
        </details>
      </section>

      <section v-else>
        <h4>Pandoc Not Found</h4>
        <p>Please install Pandoc or configure a custom path below.</p>
        <div class="grid" style="grid-template-columns: 1fr">
          <button @click="openGitHubReleases" class="secondary">
            🌐 Download Pandoc
          </button>
        </div>
      </section>

      <section>
        <h4>Custom Path</h4>
        <div class="grid custom-path-grid">
          <input
            v-model="customPandocPath"
            type="text"
            placeholder="Enter custom Pandoc path..."
          />
          <button
            @click="browseCustomPath"
            class="secondary"
            :disabled="browsing"
            :aria-busy="browsing"
          >
            {{ browsing ? "📁 Browsing..." : "📁 Browse" }}
          </button>
          <button
            @click="validateAndUseCustomPath"
            :disabled="!customPandocPath || validating"
            :aria-busy="validating"
            class="secondary"
          >
            {{ validating ? "✅ Validating..." : "✅ Validate" }}
          </button>
        </div>
      </section>

      <section>
        <h4>Version Management</h4>
        <div class="grid">
          <button
            @click="checkForUpdates"
            :disabled="checking"
            :aria-busy="checking"
            class="secondary"
          >
            {{ checking ? "Checking..." : "🔍 Check for Updates" }}
          </button>

          <button
            v-if="updateAvailable"
            @click="downloadLatestVersion"
            :disabled="downloading"
            :aria-busy="downloading"
          >
            {{ downloading ? "Downloading..." : "🚀 Update Now" }}
          </button>

          <button
            @click="loadPandocInfo"
            class="secondary"
            :disabled="reDetecting"
            :aria-busy="reDetecting"
          >
            {{ reDetecting ? "🔍 Re-detecting..." : "🔍 Re-detect" }}
          </button>
        </div>
      </section>

      <section>
        <h4>📦 Managed Pandoc</h4>
        <p>
          <small
            >Install Pandoc directly into this application (no system
            installation required)</small
          >
        </p>

        <div v-if="portablePandocStatus.checking" class="status-checking">
          <p>🔍 Checking managed Pandoc...</p>
        </div>

        <div
          v-else-if="portablePandocStatus.available"
          class="status-available"
        >
          <p>✅ Managed Pandoc is installed and ready</p>
          <details>
            <summary>Managed Installation Info</summary>
            <p>
              <small
                >Managed Pandoc provides a self-contained installation that's
                automatically managed by the application. This includes both
                bundled Pandoc (shipped with the app) and portable installations
                (downloaded separately). It's automatically prioritized when
                available, ensuring consistent behavior across different
                systems.</small
              >
            </p>
          </details>
        </div>

        <div v-else class="status-not-available">
          <p>📦 Managed Pandoc not installed</p>
          <div class="grid">
            <button
              @click="installPortablePandoc"
              :disabled="portablePandocStatus.installing"
              :aria-busy="portablePandocStatus.installing"
              class="secondary"
            >
              {{
                portablePandocStatus.installing
                  ? "⬇️ Installing..."
                  : "📥 Install Managed Pandoc"
              }}
            </button>
            <button
              @click="checkPortablePandoc"
              class="secondary"
              :disabled="portablePandocStatus.checking"
              :aria-busy="portablePandocStatus.checking"
            >
              {{
                portablePandocStatus.checking
                  ? "🔍 Checking..."
                  : "🔍 Check Again"
              }}
            </button>
          </div>
        </div>
      </section>

      <section v-if="downloading">
        <h4>Download Progress</h4>
        <progress
          :value="downloadProgress.progress"
          max="100"
          :aria-label="`Download progress: ${downloadProgress.progress}%`"
        >
          {{ downloadProgress.progress }}%
        </progress>
        <div class="grid">
          <p>
            <small
              >{{ formatBytes(downloadProgress.downloaded) }} /
              {{ formatBytes(downloadProgress.total) }}</small
            >
          </p>
          <p>
            <small>{{ downloadProgress.progress }}%</small>
          </p>
        </div>
        <p v-if="downloadProgress.current_mirror">
          <small>Mirror: {{ downloadProgress.current_mirror }}</small>
        </p>
      </section>

      <section v-if="pandocInfo">
        <h4>Format Support</h4>

        <details>
          <summary>
            📥 Input Formats ({{ pandocInfo.supported_input_formats.length }})
          </summary>
          <div class="format-list">
            <span
              v-for="format in pandocInfo.supported_input_formats"
              :key="format"
              class="format-tag"
            >
              {{ format }}
            </span>
          </div>
        </details>

        <details>
          <summary>
            📤 Output Formats ({{ pandocInfo.supported_output_formats.length }})
          </summary>
          <div class="format-list">
            <span
              v-for="format in pandocInfo.supported_output_formats"
              :key="format"
              class="format-tag"
            >
              {{ format }}
            </span>
          </div>
        </details>
      </section>

      <section>
        <h4>About Pandoc Manager</h4>
        <p>
          <small>
            This manager handles all Pandoc-related operations including
            installation detection, version management, and managed
            installation. It automatically checks for the latest Pandoc releases
            using GitHub's API with multiple download mirrors for reliability.
          </small>
        </p>
      </section>
    </article>
  </dialog>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { usePandocManager } from "../composables/usePandocManager";
import { useUI } from "../composables/useUI";
import { useMessages } from "../composables/useMessages";
import { cleanWindowsPath } from "../composables/useUtils";

// Composables
const {
  isReady,
  pandocInfo,
  useCustomPath,
  updateBundledPandoc,
  checkBundledUpdate,
  initializePandoc,
} = usePandocManager();

const { showPandocManager, closePandocManager } = useUI();
const { displayMessage } = useMessages();

// Local state for PandocManager component
const customPandocPath = ref<string>("");
const checking = ref<boolean>(false);
const downloading = ref<boolean>(false);
const updateAvailable = ref<boolean>(false);
const latestVersion = ref<string>("");
const reDetecting = ref<boolean>(false);
const browsing = ref<boolean>(false);
const validating = ref<boolean>(false);
const switching = ref<boolean>(false);

// Download progress for compatibility
const downloadProgress = ref({
  downloaded: 0,
  total: 0,
  progress: 0,
  current_mirror: "",
});

// Managed Pandoc status (unified bundled/portable concept)
const portablePandocStatus = ref({
  available: false,
  checking: false,
  installing: false,
});

// Format bytes utility
const formatBytes = (bytes: number): string => {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
};

// Adapted functions for new manager system
const loadPandocInfo = async () => {
  reDetecting.value = true;
  try {
    await initializePandoc();
    displayMessage("Pandoc detection completed", "success");
  } catch (error) {
    displayMessage(`Failed to re-detect pandoc: ${error}`, "error");
  } finally {
    reDetecting.value = false;
  }
};

// Initial version check (without user feedback)
const getLatestVersion = async () => {
  try {
    const release = await invoke<any>("get_latest_pandoc_release");
    latestVersion.value = release.tag_name;
  } catch (error) {
    console.warn("Failed to fetch latest version:", error);
    latestVersion.value = "Unknown";
  }
};

const validatePandocPath = async (path: string): Promise<boolean> => {
  return await useCustomPath(path);
};

const checkForUpdates = async (): Promise<boolean> => {
  checking.value = true;
  try {
    // Always get latest version first
    const release = await invoke<any>("get_latest_pandoc_release");
    latestVersion.value = release.tag_name;

    // Then check if update is needed
    const needsUpdate = await checkBundledUpdate();
    updateAvailable.value = needsUpdate;

    if (needsUpdate) {
      displayMessage(`Update available: ${latestVersion.value}`, "info");
    } else {
      displayMessage(
        "Already up to date! You have the latest version.",
        "success",
      );
    }

    return needsUpdate;
  } catch (error) {
    displayMessage(`Failed to check for updates: ${error}`, "error");
    latestVersion.value = "Failed to fetch";
    return false;
  } finally {
    checking.value = false;
  }
};

const downloadLatestVersion = async (): Promise<void> => {
  downloading.value = true;
  try {
    await updateBundledPandoc();
    updateAvailable.value = false;
  } finally {
    downloading.value = false;
  }
};

// Managed Pandoc functions
const checkPortablePandoc = async () => {
  portablePandocStatus.value.checking = true;
  try {
    const isAvailable = await invoke<boolean>("check_portable_pandoc");
    portablePandocStatus.value.available = isAvailable;
  } catch (error) {
    console.warn("Failed to check managed Pandoc:", error);
    portablePandocStatus.value.available = false;
  } finally {
    portablePandocStatus.value.checking = false;
  }
};

const installPortablePandoc = async () => {
  portablePandocStatus.value.installing = true;
  try {
    const result = await invoke<string>("install_portable_pandoc");
    displayMessage(result, "success");
    portablePandocStatus.value.available = true;
    // Reload Pandoc info to pick up the new managed installation
    await loadPandocInfo();
  } catch (error) {
    displayMessage(`Failed to install managed Pandoc: ${error}`, "error");
  } finally {
    portablePandocStatus.value.installing = false;
  }
};

// Path management functions
const switchPandocPath = async (newPath: string) => {
  switching.value = true;
  customPandocPath.value = newPath;
  try {
    await loadPandocInfo();
    displayMessage("Switched to new Pandoc path", "success");
  } catch (error) {
    displayMessage(`Failed to switch path: ${error}`, "error");
  } finally {
    switching.value = false;
  }
};

const browseCustomPath = async () => {
  browsing.value = true;
  try {
    const result = await open({
      multiple: false,
      directory: false,
      filters: [
        {
          name: "Executable Files",
          extensions: ["exe", "app", "*"],
        },
      ],
    });

    if (result) {
      customPandocPath.value = result as string;
      displayMessage("File selected successfully", "success");
    }
  } catch (error) {
    displayMessage(`Failed to browse for path: ${error}`, "error");
  } finally {
    browsing.value = false;
  }
};

const validateAndUseCustomPath = async () => {
  if (!customPandocPath.value) {
    displayMessage("Please enter a path", "error");
    return;
  }

  validating.value = true;
  try {
    const isValid = await validatePandocPath(customPandocPath.value);

    if (isValid) {
      await loadPandocInfo();
      displayMessage(
        "Custom path validated and applied successfully",
        "success",
      );
    } else {
      displayMessage("Invalid pandoc path or file not executable", "error");
    }
  } catch (error) {
    displayMessage(`Path validation failed: ${error}`, "error");
  } finally {
    validating.value = false;
  }
};

const openGitHubReleases = () => {
  window.open("https://github.com/jgm/pandoc/releases", "_blank");
};

onMounted(async () => {
  await Promise.all([
    loadPandocInfo(),
    getLatestVersion(),
    checkPortablePandoc(),
  ]);
});
</script>

<style scoped>
.grid {
  display: grid;
  grid-template-columns: 3fr auto auto;
  gap: calc(var(--pico-grid-column-gap) * 0.75);
  align-items: stretch;
}

/* Custom Path specific grid - wider input */
.custom-path-grid {
  grid-template-columns: 6fr auto auto !important;
}

.grid > * {
  height: 2.5rem;
  min-height: 2.5rem;
  display: flex;
  align-items: center;
  border-radius: var(--pico-border-radius);
  font-size: 0.875rem;
}

.grid input {
  padding: 0 var(--pico-form-element-spacing-horizontal);
  border: 1px solid var(--pico-form-element-border-color);
  background: var(--pico-form-element-background-color);
  color: var(--pico-form-element-color);
  flex: 1;
}

.grid button {
  background: var(--pico-secondary-background);
  color: var(--pico-secondary-inverse);
  border: 1px solid var(--pico-secondary-border);
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
  padding: 0 var(--pico-form-element-spacing-horizontal);
  justify-content: center;
  font-weight: 500;
}

.grid button:hover:not(:disabled) {
  background: var(--pico-secondary-hover-background);
  color: var(--pico-secondary-inverse);
  transform: translateY(-1px);
}

.grid button:disabled {
  background: var(--pico-form-element-disabled-background-color);
  color: var(--pico-muted-color);
  cursor: not-allowed;
  transform: none;
  opacity: 0.6;
}

/* Information display grid */
.info-grid {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: var(--pico-spacing);
  margin-bottom: var(--pico-spacing);
}

.info-item {
  display: flex;
  flex-direction: column;
  gap: calc(var(--pico-spacing) * 0.25);
}

.info-item h6 {
  margin: 0;
  color: var(--pico-muted-color);
  font-size: 0.75rem;
  text-transform: uppercase;
  font-weight: 600;
}

.info-item p {
  margin: 0;
  font-weight: 500;
  font-size: 0.875rem;
  word-break: break-word;
}

/* Version Management section - equal width buttons */
section h4 + .grid:has(button:only-child) {
  grid-template-columns: 1fr;
}

section h4 + .grid:has(button:nth-child(2)) {
  grid-template-columns: 1fr 1fr;
}

section h4 + .grid:has(button:nth-child(3)) {
  grid-template-columns: 1fr 1fr 1fr;
}

/* Status indicators */
.status-ready {
  color: var(--pico-color-green-500);
}

.status-error {
  color: var(--pico-color-red-500);
}

.status-checking {
  color: var(--pico-muted-color);
  font-style: italic;
}

.status-available {
  color: var(--pico-color-green-500);
}

.status-not-available {
  color: var(--pico-muted-color);
}

/* Format support */
.format-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
  margin-top: 0.5rem;
}

.format-tag {
  display: inline-block;
  padding: 0.25rem 0.5rem;
  border-radius: var(--pico-border-radius);
  font-size: 0.75rem;
  font-weight: 500;
  margin: 0.125rem;
  border: 1px solid var(--pico-muted-border-color);
  background: var(--pico-card-sectioning-background-color);
  color: var(--pico-color);
}

/* Section spacing */
section {
  border-bottom: 1px solid var(--pico-border-color);
  padding-bottom: var(--pico-spacing);
  margin-bottom: var(--pico-spacing);
}

section:last-child {
  border-bottom: none;
  margin-bottom: 0;
}

h6 {
  margin-bottom: calc(var(--pico-spacing) * 0.25);
  color: var(--pico-muted-color);
  font-size: 0.75rem;
  text-transform: uppercase;
  font-weight: 600;
}

/* Responsive design */
@media (max-width: 768px) {
  .grid {
    grid-template-columns: 1fr;
    gap: calc(var(--pico-grid-column-gap) * 0.5);
  }

  .custom-path-grid {
    grid-template-columns: 1fr !important;
  }

  .info-grid {
    grid-template-columns: 1fr;
    gap: calc(var(--pico-spacing) * 0.75);
  }

  section h4 + .grid:has(button:nth-child(2)),
  section h4 + .grid:has(button:nth-child(3)) {
    grid-template-columns: 1fr;
  }
}
</style>
