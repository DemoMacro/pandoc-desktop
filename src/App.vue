<template>
  <div class="app-container">
    <header class="app-header">
      <nav>
        <div class="nav-brand">
          <strong>📄 Pandoc Desktop</strong>
        </div>
        <div class="nav-actions">
          <button
            class="action-button"
            @click="toggleTheme"
            aria-label="Toggle Theme"
            title="Toggle Theme"
          >
            {{ isDarkMode ? "☀️" : "🌙" }}
          </button>
          <button
            class="action-button"
            @click="toggleSettings"
            aria-label="Settings"
            title="Settings"
          >
            ⚙️
          </button>
          <button
            class="action-button"
            @click="togglePandocManager"
            aria-label="Pandoc Manager"
            title="Pandoc Manager"
          >
            📦
          </button>
        </div>
      </nav>

      <div class="status-bar">
        <div class="status-item" v-if="pandocInfo">
          <span
            :class="['status-indicator', { ready: isReady, error: !isReady }]"
          >
            {{ isReady ? "✅" : "❌" }}
          </span>
          <span class="status-text">{{ pandocInfo.version }}</span>
        </div>
        <div class="status-item" v-if="pandocInfo">
          <span class="file-indicator">📁</span>
          <span class="status-text">{{ getFileName(pandocInfo.path) }}</span>
        </div>
        <div class="status-item" v-if="!pandocInfo">
          <span class="status-indicator error">❌</span>
          <span class="status-text">Pandoc not found</span>
        </div>
      </div>
    </header>

    <main class="app-main">
      <div class="unified-grid">
        <FileInput class="grid-file-input" />
        <FormatSelector class="grid-format-selector" />
        <OutputConfig class="grid-output-config" />
        <ConvertButton class="grid-convert-button" />
      </div>
    </main>

    <!-- Modals and Overlays -->
    <StatusMessage />
    <SettingsPanel />
    <PandocManager />
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue";
import { usePandoc } from "./composables/usePandoc";
import { useUI } from "./composables/useUI";

import FileInput from "./components/FileInput.vue";
import FormatSelector from "./components/FormatSelector.vue";
import OutputConfig from "./components/OutputConfig.vue";
import ConvertButton from "./components/ConvertButton.vue";
import StatusMessage from "./components/StatusMessage.vue";
import SettingsPanel from "./components/SettingsPanel.vue";
import PandocManager from "./components/PandocManager.vue";

// Use composables for state management
const { pandocInfo, isReady, initializePandoc } = usePandoc();
const { toggleSettings, togglePandocManager } = useUI();
const isDarkMode = ref(false);

// Helper function
function getFileName(path: string): string {
  return path.split(/[/\\]/).pop() || "";
}

// Theme management
const loadThemeSettings = () => {
  try {
    const saved = localStorage.getItem("pandoc-desktop-settings");
    if (saved) {
      const settings = JSON.parse(saved);
      return settings.theme || "auto";
    }
  } catch (error) {
    console.warn("Failed to load theme settings:", error);
  }
  return "auto";
};

const applyTheme = (themeSetting: "auto" | "light" | "dark") => {
  let actualTheme: "light" | "dark";

  if (themeSetting === "auto") {
    actualTheme = window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  } else {
    actualTheme = themeSetting;
  }

  isDarkMode.value = actualTheme === "dark";
  document.documentElement.setAttribute("data-theme", actualTheme);
};

// Theme toggle
const toggleTheme = () => {
  isDarkMode.value = !isDarkMode.value;
  const newTheme = isDarkMode.value ? "dark" : "light";

  // Save to localStorage
  const settings = {
    theme: newTheme,
  };
  localStorage.setItem("pandoc-desktop-settings", JSON.stringify(settings));

  document.documentElement.setAttribute("data-theme", newTheme);
};

// Initialize on mount
onMounted(() => {
  initializePandoc();

  // Load and apply saved theme settings
  const savedTheme = loadThemeSettings();
  applyTheme(savedTheme);
});
</script>

<style scoped>
/* App Container */
.app-container {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: var(--pico-background-color);
  color: var(--pico-color);
  overflow: hidden;
}

/* Header Styles */
.app-header {
  flex-shrink: 0;
  border-bottom: var(--pico-border-width) solid var(--pico-border-color);
  background: var(--pico-card-background-color);
  color: var(--pico-color);
}

nav {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--pico-form-element-spacing-vertical) var(--pico-spacing);
  margin: 0;
}

.nav-brand {
  font-size: 1rem;
  font-weight: 600;
  color: var(--pico-color);
}

.nav-actions {
  display: flex;
  gap: calc(var(--pico-spacing) * 0.375);
}

.action-button {
  background: var(--pico-secondary-background);
  color: var(--pico-secondary-inverse);
  border: 1px solid var(--pico-secondary-border);
  padding: var(--pico-form-element-spacing-vertical);
  border-radius: var(--pico-border-radius);
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 0.875rem;
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
}

.action-button:hover {
  background: var(--pico-secondary-hover-background);
  color: var(--pico-secondary-inverse);
  transform: translateY(-1px);
}

.action-button:active,
.action-button:focus {
  background: var(--pico-secondary-background);
  color: var(--pico-secondary-inverse);
}

/* Status Bar */
.status-bar {
  display: flex;
  align-items: center;
  gap: var(--pico-spacing);
  padding: calc(var(--pico-form-element-spacing-vertical) * 0.67)
    var(--pico-spacing);
  background: var(--pico-card-sectioning-background-color);
  color: var(--pico-color);
  border-top: 1px solid var(--pico-border-color);
  font-size: 0.75rem;
}

.status-item {
  display: flex;
  align-items: center;
  gap: calc(var(--pico-spacing) * 0.375);
}

.status-indicator {
  font-size: 0.875rem;
}

.status-indicator.ready {
  color: var(--pico-color);
}

.status-indicator.error {
  color: var(--pico-del-color);
}

.status-text {
  color: var(--pico-color);
  font-weight: 500;
}

.file-indicator {
  color: var(--pico-primary);
}

/* Main Content */
.app-main {
  flex: 1;
  overflow: auto;
  padding: var(--pico-spacing);
}

.unified-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  grid-template-rows: 1fr 1fr auto;
  gap: var(--pico-grid-column-gap);
  height: 100%;
}

.grid-file-input {
  grid-column: 1;
  grid-row: 1;
}

.grid-format-selector {
  grid-column: 2;
  grid-row: 1;
}

.grid-output-config {
  grid-column: 1 / -1;
  grid-row: 2;
}

.grid-convert-button {
  grid-column: 1 / -1;
  grid-row: 3;
}

/* Responsive Design */
@media (max-width: 768px) {
  .app-main {
    padding: calc(var(--pico-spacing) * 0.67);
  }

  .unified-grid {
    grid-template-columns: 1fr;
    grid-template-rows: auto auto auto auto;
    gap: calc(var(--pico-grid-column-gap) * 0.67);
  }

  .grid-file-input {
    grid-column: 1;
    grid-row: 1;
  }

  .grid-format-selector {
    grid-column: 1;
    grid-row: 2;
  }

  .grid-output-config {
    grid-column: 1;
    grid-row: 3;
  }

  .grid-convert-button {
    grid-column: 1;
    grid-row: 4;
  }

  nav {
    padding: calc(var(--pico-form-element-spacing-vertical) * 0.67)
      var(--pico-spacing);
  }

  .status-bar {
    padding: calc(var(--pico-form-element-spacing-vertical) * 0.5)
      var(--pico-spacing);
    flex-wrap: wrap;
    gap: var(--pico-spacing);
  }

  .nav-brand {
    font-size: 0.875rem;
  }
}

@media (max-width: 480px) {
  .status-bar {
    flex-direction: column;
    align-items: flex-start;
    gap: calc(var(--pico-spacing) * 0.375);
  }
}
</style>
