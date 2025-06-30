<script setup lang="ts">
import { ref, onMounted, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import {
  type PandocInfo,
  detectInputFormat,
  getCompatibleOutputFormats,
  getBaseName,
  generateOutputFilenameWithExt,
} from "./types/pandoc";

// Reactive state
const inputFile = ref<string>("");
const outputDirectory = ref<string>("");
const outputFileName = ref<string>("");
const outputFormat = ref<string>("html");
const inputFormat = ref<string>("markdown");
const isConverting = ref<boolean>(false);
const message = ref<string>("");
const messageType = ref<"success" | "error" | "info">("info");
const pandocInfo = ref<PandocInfo | null>(null);
const showSettings = ref<boolean>(false);
const customPandocPath = ref<string>("");

// Computed properties
const isReady = computed(() => pandocInfo.value?.is_working ?? false);
const supportedOutputFormats = computed(
  () => pandocInfo.value?.supported_output_formats ?? [],
);
const availableOutputFormats = computed(() =>
  getCompatibleOutputFormats(inputFormat.value, supportedOutputFormats.value),
);
const formatCount = computed(() => availableOutputFormats.value.length);

// Initialize Pandoc info on startup
onMounted(async () => {
  await loadPandocInfo();
});

// Watch for input file changes to detect format
watch(inputFile, (newFile) => {
  if (newFile) {
    const detectedFormat = detectInputFormat(newFile);
    inputFormat.value = detectedFormat;

    // Set output filename to base name (without extension)
    outputFileName.value = getBaseName(newFile);
  }
});

// Watch for output format changes to update filename
watch(outputFormat, (newFormat) => {
  if (inputFile.value && outputFileName.value) {
    // Keep the current base name, don't regenerate from input file
    // User might have customized the filename
    const currentBaseName =
      outputFileName.value.replace(/\.[^/.]+$/, "") ||
      getBaseName(inputFile.value);
    outputFileName.value = currentBaseName;
  }
});

// Load Pandoc information
async function loadPandocInfo() {
  try {
    const info = await invoke<PandocInfo>("get_pandoc_info", {
      customPath: customPandocPath.value || null,
    });
    pandocInfo.value = info;
    showMessage(`Pandoc ready: ${info.version}`, "success");
  } catch (error) {
    pandocInfo.value = null;
    showMessage(`Pandoc not found: ${error}`, "error");
  }
}

// Show message helper
function showMessage(msg: string, type: "success" | "error" | "info") {
  message.value = msg;
  messageType.value = type;
  setTimeout(() => {
    message.value = "";
  }, 5000);
}

// Select input file
async function selectInputFile() {
  try {
    const result = await open({
      multiple: false,
      directory: false,
      filters: [
        {
          name: "All Supported",
          extensions: [
            "md",
            "html",
            "docx",
            "tex",
            "rst",
            "txt",
            "org",
            "epub",
            "rtf",
            "odt",
          ],
        },
        {
          name: "Markdown",
          extensions: ["md", "markdown", "mdown", "mkd"],
        },
        {
          name: "HTML",
          extensions: ["html", "htm"],
        },
        {
          name: "Word Document",
          extensions: ["docx"],
        },
        {
          name: "LaTeX",
          extensions: ["tex", "latex"],
        },
        {
          name: "reStructuredText",
          extensions: ["rst"],
        },
        {
          name: "Plain Text",
          extensions: ["txt", "text"],
        },
        {
          name: "Org Mode",
          extensions: ["org"],
        },
        {
          name: "EPUB",
          extensions: ["epub"],
        },
        {
          name: "OpenDocument",
          extensions: ["odt"],
        },
        {
          name: "Rich Text",
          extensions: ["rtf"],
        },
      ],
    });

    if (result) {
      inputFile.value = result as string;
    }
  } catch (error) {
    showMessage(`Failed to select file: ${error}`, "error");
  }
}

// Select output directory
async function selectOutputDirectory() {
  try {
    const result = await open({
      multiple: false,
      directory: true,
    });

    if (result) {
      outputDirectory.value = result as string;
    }
  } catch (error) {
    showMessage(`Failed to select directory: ${error}`, "error");
  }
}

// Toggle settings panel
function toggleSettings() {
  showSettings.value = !showSettings.value;
}

// Open GitHub releases page
function openGitHubReleases() {
  window.open("https://github.com/jgm/pandoc/releases", "_blank");
}

// Switch to a different detected pandoc path
async function switchPandocPath(newPath: string) {
  customPandocPath.value = newPath;
  await loadPandocInfo();
}

// Browse for custom pandoc path
async function browseCustomPath() {
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
    }
  } catch (error) {
    showMessage(`Failed to browse for path: ${error}`, "error");
  }
}

// Validate and use custom pandoc path
async function validateAndUseCustomPath() {
  if (!customPandocPath.value) {
    showMessage("Please enter a path", "error");
    return;
  }

  try {
    const isValid = await invoke<boolean>("validate_pandoc_path", {
      path: customPandocPath.value,
    });

    if (isValid) {
      await loadPandocInfo();
      showMessage("Custom path validated successfully", "success");
    } else {
      showMessage("Invalid pandoc path or file not executable", "error");
    }
  } catch (error) {
    showMessage(`Path validation failed: ${error}`, "error");
  }
}

// Convert file
async function convertFile() {
  if (!inputFile.value) {
    showMessage("Please select an input file", "error");
    return;
  }
  if (!outputDirectory.value) {
    showMessage("Please select an output directory", "error");
    return;
  }
  if (!outputFileName.value) {
    showMessage("Please enter an output filename", "error");
    return;
  }
  if (!pandocInfo.value?.is_working) {
    showMessage(
      "Pandoc is not ready. Please check your installation.",
      "error",
    );
    return;
  }

  isConverting.value = true;

  // Generate the complete output filename with correct extension
  const outputFileNameWithExt = generateOutputFilenameWithExt(
    outputFileName.value,
    outputFormat.value,
  );
  const outputPath = `${outputDirectory.value}/${outputFileNameWithExt}`;

  try {
    const result = await invoke<string>("convert_with_pandoc", {
      inputFile: inputFile.value,
      outputFile: outputPath,
      inputFormat: null, // Let Pandoc auto-detect based on file extension
      outputFormat: outputFormat.value,
      customPandocPath: customPandocPath.value || null,
    });
    showMessage(result, "success");
  } catch (error) {
    showMessage(`Conversion failed: ${error}`, "error");
  } finally {
    isConverting.value = false;
  }
}

// Get filename from path
function getFileName(path: string): string {
  return path.split(/[/\\]/).pop() || "";
}
</script>

<template>
  <div class="app">
    <header class="header">
      <div class="header-content">
        <h1>Pandoc Desktop</h1>
        <button @click="toggleSettings" class="btn btn-icon" title="Settings">
          ⚙️
        </button>
      </div>

      <!-- Pandoc Status -->
      <div class="status-bar">
        <div v-if="pandocInfo" class="status-item">
          <span
            class="status-icon"
            :class="{ 'status-ready': isReady, 'status-error': !isReady }"
          >
            {{ isReady ? "✅" : "❌" }}
          </span>
          <span class="status-text">{{ pandocInfo.version }}</span>
        </div>
        <div v-if="pandocInfo" class="status-item">
          <span class="status-label">📁</span>
          <span class="status-path">{{ getFileName(pandocInfo.path) }}</span>
        </div>
        <div v-if="!pandocInfo" class="status-item status-error">
          <span class="status-icon">❌</span>
          <span class="status-text">Pandoc not found</span>
        </div>
      </div>
    </header>

    <main class="main">
      <!-- Input File Section -->
      <section class="section">
        <h2>Input File</h2>
        <div class="input-group">
          <button @click="selectInputFile" class="btn btn-secondary">
            Select File
          </button>
          <div v-if="inputFile" class="file-info">
            <span class="file-name">{{ getFileName(inputFile) }}</span>
            <span class="file-format">Format: {{ inputFormat }}</span>
          </div>
          <span class="placeholder" v-else> No file selected </span>
        </div>
      </section>

      <!-- Output Format Section -->
      <section class="section">
        <h2>Output Format</h2>
        <div class="format-group">
          <select v-model="outputFormat" class="select">
            <option
              v-for="format in availableOutputFormats"
              :key="format.value"
              :value="format.value"
            >
              {{ format.label }}
            </option>
          </select>
          <span class="format-count">{{ formatCount }} formats available</span>
        </div>
      </section>

      <!-- Output Directory Section -->
      <section class="section">
        <h2>Output Directory</h2>
        <div class="input-group">
          <button @click="selectOutputDirectory" class="btn btn-secondary">
            Select Directory
          </button>
          <span class="file-path" v-if="outputDirectory">
            {{ outputDirectory }}
          </span>
          <span class="placeholder" v-else> No directory selected </span>
        </div>
      </section>

      <!-- Output Filename Section -->
      <section class="section">
        <h2>Output Filename</h2>
        <input
          v-model="outputFileName"
          type="text"
          class="input"
          placeholder="Enter filename..."
        />
      </section>

      <!-- Convert Button -->
      <section class="section">
        <button
          @click="convertFile"
          :disabled="isConverting || !isReady"
          class="btn btn-primary btn-large"
        >
          {{ isConverting ? "Converting..." : "Convert" }}
        </button>
      </section>
    </main>

    <!-- Message Display -->
    <div v-if="message" :class="['message', `message-${messageType}`]">
      {{ message }}
    </div>

    <!-- Settings Panel -->
    <div
      v-if="showSettings"
      class="settings-overlay"
      @click="showSettings = false"
    >
      <div class="settings-panel" @click.stop>
        <div class="settings-header">
          <h3>Pandoc Settings</h3>
          <button @click="showSettings = false" class="btn btn-close">×</button>
        </div>

        <div class="settings-content">
          <div class="setting-group">
            <h4>Current Installation</h4>
            <div v-if="pandocInfo" class="info-grid">
              <div class="info-item">
                <span class="info-label">Version:</span>
                <span class="info-value">{{ pandocInfo.version }}</span>
              </div>
              <div class="info-item">
                <span class="info-label">Path:</span>
                <span class="info-value" :title="pandocInfo.path">{{
                  pandocInfo.path
                }}</span>
              </div>
              <div class="info-item">
                <span class="info-label">Status:</span>
                <span
                  :class="[
                    'info-value',
                    { 'status-ready': isReady, 'status-error': !isReady },
                  ]"
                >
                  {{ isReady ? "✅ Working" : "❌ Not Working" }}
                </span>
              </div>
            </div>
            <div v-else class="info-error">Pandoc not found or not working</div>
          </div>

          <!-- Path Detection Details -->
          <div
            v-if="
              pandocInfo &&
              (pandocInfo.detected_paths.length > 1 ||
                pandocInfo.search_paths.length > 0)
            "
            class="setting-group"
          >
            <h4>Path Detection Details</h4>

            <div
              v-if="pandocInfo.detected_paths.length > 1"
              class="path-section"
            >
              <h5>
                🔍 All Detected Installations ({{
                  pandocInfo.detected_paths.length
                }})
              </h5>
              <div class="path-list">
                <div
                  v-for="(path, index) in pandocInfo.detected_paths"
                  :key="index"
                  :class="[
                    'path-item',
                    { 'path-active': path === pandocInfo.path },
                  ]"
                >
                  <span class="path-icon">{{
                    path === pandocInfo.path ? "🟢" : "⚪"
                  }}</span>
                  <span class="path-text" :title="path">{{ path }}</span>
                  <button
                    v-if="path !== pandocInfo.path"
                    @click="switchPandocPath(path)"
                    class="btn btn-mini"
                  >
                    Use This
                  </button>
                </div>
              </div>
            </div>

            <details
              v-if="pandocInfo.search_paths.length > 0"
              class="search-details"
            >
              <summary class="search-summary">
                🔎 Searched Locations ({{ pandocInfo.search_paths.length }})
              </summary>
              <div class="search-list">
                <div
                  v-for="(path, index) in pandocInfo.search_paths"
                  :key="index"
                  :class="[
                    'search-item',
                    {
                      'search-found': pandocInfo.detected_paths.includes(path),
                    },
                  ]"
                >
                  <span class="search-icon">{{
                    pandocInfo.detected_paths.includes(path) ? "✅" : "❌"
                  }}</span>
                  <span class="search-text">{{ path }}</span>
                </div>
              </div>
            </details>
          </div>

          <!-- Custom Path Configuration -->
          <div class="setting-group">
            <h4>Custom Path</h4>
            <div class="custom-path-group">
              <input
                v-model="customPandocPath"
                type="text"
                class="input custom-path-input"
                placeholder="Enter custom Pandoc path..."
              />
              <button @click="browseCustomPath" class="btn btn-secondary">
                📁 Browse
              </button>
              <button
                @click="validateAndUseCustomPath"
                :disabled="!customPandocPath"
                class="btn btn-secondary"
              >
                ✅ Validate
              </button>
            </div>
          </div>

          <div class="setting-group">
            <h4>Version Management</h4>
            <div class="button-group">
              <button @click="loadPandocInfo" class="btn btn-secondary">
                🔍 Re-detect All
              </button>
              <button @click="openGitHubReleases" class="btn btn-secondary">
                🌐 GitHub Releases
              </button>
            </div>
          </div>

          <div class="setting-group">
            <h4>Format Support</h4>
            <div v-if="pandocInfo" class="format-info">
              <div class="format-stats">
                <div class="format-stat">
                  <span class="stat-number">{{
                    pandocInfo.supported_input_formats.length
                  }}</span>
                  <span class="stat-label">Input Formats</span>
                </div>
                <div class="format-stat">
                  <span class="stat-number">{{
                    pandocInfo.supported_output_formats.length
                  }}</span>
                  <span class="stat-label">Output Formats</span>
                </div>
              </div>

              <details class="format-details">
                <summary class="format-summary">View Supported Formats</summary>
                <div class="format-lists">
                  <div class="format-list">
                    <h6>Input Formats</h6>
                    <div class="format-tags">
                      <span
                        v-for="format in pandocInfo.supported_input_formats"
                        :key="format"
                        class="format-tag input-format"
                      >
                        {{ format }}
                      </span>
                    </div>
                  </div>
                  <div class="format-list">
                    <h6>Output Formats</h6>
                    <div class="format-tags">
                      <span
                        v-for="format in pandocInfo.supported_output_formats"
                        :key="format"
                        class="format-tag output-format"
                      >
                        {{ format }}
                      </span>
                    </div>
                  </div>
                </div>
              </details>
            </div>
            <div v-else class="format-error">
              Unable to load format information
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 100vh Grid Layout - No Scroll Design */
.app {
  height: 100vh;
  display: grid;
  grid-template-rows: auto 1fr;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  background: #f8f9fa;
  overflow: hidden;
}

/* Compact Header */
.header {
  background: #2c3e50;
  color: white;
  padding: 1rem 2rem;
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.5rem;
}

.header h1 {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 600;
}

.btn-icon {
  background: transparent;
  border: 1px solid rgba(255, 255, 255, 0.3);
  color: white;
  padding: 0.5rem;
  border-radius: 0.25rem;
  cursor: pointer;
  font-size: 1rem;
  transition: background-color 0.2s;
}

.btn-icon:hover {
  background: rgba(255, 255, 255, 0.1);
}

.status-bar {
  display: flex;
  gap: 1.5rem;
  font-size: 0.8rem;
  padding: 0.25rem 0;
  border-top: 1px solid rgba(255, 255, 255, 0.2);
}

.status-item {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  color: white;
  opacity: 0.9;
}

.status-ready {
  color: #2ecc71;
}
.status-error {
  color: #e74c3c;
}

/* Main Grid Layout */
.main {
  display: grid;
  grid-template-columns: 1fr 1fr;
  grid-template-rows: 1fr 1fr 1fr;
  gap: 1rem;
  padding: 1rem 2rem;
  max-width: 80rem;
  margin: 0 auto;
  width: 100%;
  overflow: hidden;
}

/* Compact Sections */
.section {
  background: white;
  border: 1px solid #e2e8f0;
  border-radius: 0.5rem;
  padding: 1rem;
  display: flex;
  flex-direction: column;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.section h2 {
  margin: 0 0 0.75rem 0;
  font-size: 0.9rem;
  font-weight: 600;
  color: #2c3e50;
  text-transform: uppercase;
  letter-spacing: 0.025em;
}

/* Input Group Layouts */
.input-group {
  display: flex;
  gap: 0.75rem;
  align-items: center;
  flex: 1;
}

/* Modern Button Styles */
.btn {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 0.375rem;
  cursor: pointer;
  font-size: 0.875rem;
  font-weight: 500;
  transition: all 0.2s;
  white-space: nowrap;
}

.btn-primary {
  background: #3b82f6;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: #2563eb;
}

.btn-secondary {
  background: #6b7280;
  color: white;
}

.btn-secondary:hover:not(:disabled) {
  background: #4b5563;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Convert Button - Full Width */
.section:nth-child(5) {
  grid-column: 1 / -1;
  justify-content: center;
}

.btn-large {
  padding: 0.75rem 2rem;
  font-size: 1rem;
  font-weight: 600;
  width: 100%;
  max-width: 20rem;
  margin: 0 auto;
}

/* Input and Select Styles */
.input,
.select {
  padding: 0.5rem 0.75rem;
  border: 1px solid #d1d5db;
  border-radius: 0.375rem;
  font-size: 0.875rem;
  flex: 1;
  min-width: 0;
}

.input:focus,
.select:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

/* File Info Compact Display */
.file-info {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
}

.file-name {
  color: #374151;
  font-weight: 500;
  font-size: 0.875rem;
  truncate: true;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-format {
  color: #6b7280;
  font-size: 0.75rem;
}

.file-path {
  color: #374151;
  font-weight: 500;
  font-size: 0.75rem;
  font-family: ui-monospace, monospace;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}

.placeholder {
  color: #9ca3af;
  font-style: italic;
  font-size: 0.875rem;
  text-align: center;
  flex: 1;
}

/* Format Group */
.format-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  flex: 1;
}

.format-count {
  color: #6b7280;
  font-size: 0.75rem;
  text-align: center;
}

/* Messages - Compact */
.message {
  padding: 0.75rem;
  border-radius: 0.375rem;
  font-size: 0.875rem;
  font-weight: 500;
  margin: 1rem 2rem;
  border-left: 3px solid;
  max-width: 80rem;
  margin-left: auto;
  margin-right: auto;
  width: calc(100% - 4rem);
}

.message-success {
  background: #ecfdf5;
  color: #065f46;
  border-left-color: #10b981;
}

.message-error {
  background: #fef2f2;
  color: #991b1b;
  border-left-color: #ef4444;
}

.message-info {
  background: #eff6ff;
  color: #1e40af;
  border-left-color: #3b82f6;
}

/* Settings Panel - Minimal */
.settings-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.settings-panel {
  background: white;
  border-radius: 0.75rem;
  width: 90%;
  max-width: 40rem;
  max-height: 85vh;
  overflow-y: auto;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1);
}

.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1.5rem;
  border-bottom: 1px solid #e5e7eb;
  background: #f9fafb;
  border-radius: 0.75rem 0.75rem 0 0;
}

.settings-header h3 {
  margin: 0;
  color: #111827;
  font-size: 1.125rem;
  font-weight: 600;
}

.btn-close {
  background: transparent;
  border: none;
  font-size: 1.25rem;
  cursor: pointer;
  color: #6b7280;
  padding: 0.25rem;
  border-radius: 0.25rem;
  transition: all 0.2s;
}

.btn-close:hover {
  background: #e5e7eb;
  color: #374151;
}

.settings-content {
  padding: 1.5rem;
}

.setting-group {
  margin-bottom: 1.5rem;
  padding: 1rem;
  background: #f9fafb;
  border-radius: 0.5rem;
  border: 1px solid #e5e7eb;
}

.setting-group h4 {
  margin: 0 0 0.75rem 0;
  color: #374151;
  font-size: 0.875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.025em;
}

/* Grid System for Info Display */
.info-grid {
  display: grid;
  gap: 0.5rem;
}

.info-item {
  display: grid;
  grid-template-columns: 5rem 1fr;
  gap: 0.75rem;
  align-items: center;
}

.info-label {
  font-size: 0.75rem;
  font-weight: 500;
  color: #6b7280;
  text-transform: uppercase;
}

.info-value {
  color: #374151;
  font-family: ui-monospace, monospace;
  font-size: 0.75rem;
  word-break: break-all;
}

/* Additional compact styles for settings */
.path-section h5 {
  margin: 0 0 0.5rem 0;
  font-size: 0.8rem;
  color: #374151;
  font-weight: 600;
}

.path-list,
.button-group,
.custom-path-group {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.path-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem;
  background: white;
  border: 1px solid #e5e7eb;
  border-radius: 0.375rem;
  font-size: 0.75rem;
}

.btn-mini {
  padding: 0.25rem 0.5rem;
  font-size: 0.7rem;
  background: #3b82f6;
  color: white;
  border: none;
  border-radius: 0.25rem;
  cursor: pointer;
}

.format-stats {
  display: flex;
  gap: 0.75rem;
  margin-bottom: 0.75rem;
}

.format-stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 0.75rem;
  background: #3b82f6;
  color: white;
  border-radius: 0.5rem;
  flex: 1;
  text-align: center;
}

.stat-number {
  font-size: 1.25rem;
  font-weight: 700;
}

.stat-label {
  font-size: 0.7rem;
  opacity: 0.9;
}

/* Format Details and Tags */
.format-details {
  border: 1px solid #e5e7eb;
  border-radius: 0.5rem;
  overflow: hidden;
  margin-top: 0.75rem;
}

.format-summary {
  cursor: pointer;
  padding: 0.75rem 1rem;
  background: #f9fafb;
  font-weight: 500;
  color: #374151;
  outline: none;
  user-select: none;
  transition: background-color 0.2s;
}

.format-summary:hover {
  background: #f3f4f6;
}

.format-lists {
  padding: 1rem;
  display: grid;
  gap: 1.25rem;
}

.format-list h6 {
  margin: 0 0 0.75rem 0;
  color: #374151;
  font-size: 0.875rem;
  font-weight: 600;
}

.format-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
}

.format-tag {
  padding: 0.25rem 0.75rem;
  border-radius: 1rem;
  font-size: 0.75rem;
  font-weight: 500;
  border: 1px solid;
  transition: all 0.2s;
  cursor: default;
}

.format-tag.input-format {
  background: #dbeafe;
  color: #1e40af;
  border-color: #93c5fd;
}

.format-tag.input-format:hover {
  background: #bfdbfe;
  border-color: #60a5fa;
}

.format-tag.output-format {
  background: #f3e8ff;
  color: #7c3aed;
  border-color: #c4b5fd;
}

.format-tag.output-format:hover {
  background: #e9d5ff;
  border-color: #a78bfa;
}

.format-error {
  color: #dc2626;
  font-style: italic;
  padding: 1rem;
  background: #fef2f2;
  border-radius: 0.5rem;
  border: 1px solid #fecaca;
  text-align: center;
}

.info-error {
  color: #dc2626;
  font-style: italic;
  padding: 1rem;
  background: #fef2f2;
  border-radius: 0.5rem;
  border: 1px solid #fecaca;
  text-align: center;
}

/* Responsive Design */
@media (max-width: 768px) {
  .main {
    grid-template-columns: 1fr;
    grid-template-rows: repeat(5, auto);
    gap: 0.75rem;
    padding: 0.75rem;
  }

  .header {
    padding: 0.75rem 1rem;
  }

  .header h1 {
    font-size: 1.25rem;
  }

  .input-group {
    flex-direction: column;
    align-items: stretch;
  }

  .section {
    padding: 0.75rem;
  }
}

@media (max-width: 480px) {
  .header {
    padding: 0.5rem 0.75rem;
  }

  .main {
    padding: 0.5rem;
    gap: 0.5rem;
  }

  .section {
    padding: 0.5rem;
  }

  .btn {
    padding: 0.375rem 0.75rem;
    font-size: 0.8rem;
  }
}
</style>
