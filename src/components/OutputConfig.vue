<template>
  <article>
    <header>⚙️ Output Configuration</header>

    <div class="config-grid">
      <!-- Directory Selection -->
      <div class="config-section">
        <label>Directory:</label>
        <button
          @click="selectOutputDirectory"
          class="secondary"
          :disabled="isSelecting"
          :aria-busy="isSelecting"
        >
          📁 {{ isSelecting ? "Selecting..." : "Select" }}
        </button>

        <div v-if="outputDirectory" class="path-display">
          <small>{{ getShortPath(outputDirectory) }}</small>
        </div>
      </div>

      <!-- Filename Input -->
      <div class="config-section">
        <label for="filename">Filename:</label>
        <div class="filename-container">
          <input
            id="filename"
            v-model="baseFileName"
            type="text"
            placeholder="Enter filename..."
            :disabled="!outputDirectory"
            @input="onFileNameChange"
          />
          <span class="file-extension" v-if="currentExtension">
            .{{ currentExtension }}
          </span>
        </div>

        <!-- File format info -->
        <div v-if="baseFileName && currentExtension" class="filename-preview">
          <small>
            📄 Full Name: <strong>{{ fullFileName }}</strong>
          </small>
        </div>

        <!-- Auto-generate suggestion -->
        <div
          v-if="inputFile && suggestedFileName !== fullFileName"
          class="filename-suggestion"
        >
          <small>
            💡 Suggested:
            <button @click="useSuggestedFileName" class="link-button">
              {{ suggestedFileName }}
            </button>
          </small>
        </div>
      </div>
    </div>
  </article>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { useFileHandling } from "../composables/useFileHandling";
import { useMessages } from "../composables/useMessages";
import {
  getBaseName,
  getFileExtension,
  generateOutputFilenameWithExt,
  getOutputFormatByValue,
} from "../composables/useUtils";

const { inputFile, outputDirectory, outputFileName, outputFormat } =
  useFileHandling();
const { displayMessage } = useMessages();

const isSelecting = ref(false);
const baseFileName = ref("");

// Computed properties for smart filename handling
const currentExtension = computed(() => {
  const formatObj = getOutputFormatByValue(outputFormat.value);
  return formatObj?.ext || outputFormat.value;
});

const fullFileName = computed(() => {
  if (!baseFileName.value) return "";
  return `${baseFileName.value}.${currentExtension.value}`;
});

const suggestedFileName = computed(() => {
  if (!inputFile.value) return "";
  const inputBaseName = getBaseName(inputFile.value);
  return generateOutputFilenameWithExt(inputBaseName, outputFormat.value);
});

// Watch for changes
watch(outputFileName, (newName) => {
  if (newName) {
    // Extract base name without extension
    const ext = getFileExtension(newName);
    if (ext) {
      baseFileName.value = getBaseName(newName);
    } else {
      baseFileName.value = newName;
    }
  }
});

watch(fullFileName, (newFullName) => {
  outputFileName.value = newFullName;
});

// Initialize base filename when input file changes
watch(inputFile, (newFile) => {
  if (newFile && !baseFileName.value) {
    baseFileName.value = getBaseName(newFile);
  }
});

// Methods
const selectOutputDirectory = async () => {
  isSelecting.value = true;

  try {
    const result = await open({
      multiple: false,
      directory: true,
    });

    if (result) {
      outputDirectory.value = result as string;
      displayMessage("Directory selected successfully", "success");
    }
  } catch (error) {
    displayMessage(`Failed to select directory: ${error}`, "error");
  } finally {
    isSelecting.value = false;
  }
};

const onFileNameChange = () => {
  // Just update the reactive value, watch will handle the rest
};

const useSuggestedFileName = () => {
  if (inputFile.value) {
    baseFileName.value = getBaseName(inputFile.value);
    displayMessage("Filename updated from suggestion", "success");
  }
};

const getShortPath = (path: string): string => {
  const parts = path.split(/[/\\]/);
  if (parts.length <= 3) return path;
  return `.../${parts.slice(-2).join("/")}`;
};
</script>

<style scoped>
/* Minimal custom styles for grid layout */
.config-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
}

/* Button animations */
button {
  transition: all 0.2s ease;
}

button:hover:not(:disabled) {
  transform: translateY(-1px);
}

.config-section {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.config-section label {
  font-size: 0.875rem;
  font-weight: 500;
  margin-bottom: 0;
}

.filename-container {
  display: flex;
  align-items: center;
  border: 1px solid var(--pico-border-color);
  border-radius: var(--pico-border-radius);
  overflow: hidden;
  height: 2.5rem;
}

.filename-container input {
  border: none;
  margin: 0;
  border-radius: 0;
  flex: 1;
  height: 100%;
  padding: 0 0.75rem;
}

.filename-container input:focus {
  box-shadow: none;
}

.file-extension {
  background: var(--pico-secondary-background);
  color: var(--pico-secondary-inverse);
  padding: 0 0.75rem;
  font-size: 0.875rem;
  border-left: 1px solid var(--pico-border-color);
  height: 100%;
  display: flex;
  align-items: center;
  min-width: fit-content;
}

.path-display {
  padding: 0.75rem;
  background: var(--pico-card-sectioning-background-color);
  border: 1px solid var(--pico-border-color);
  border-radius: var(--pico-border-radius);
  word-break: break-all;
  min-height: 2.5rem;
  display: flex;
  align-items: center;
  margin-top: 0.5rem;
}

.filename-preview {
  padding: 0.75rem;
  background: var(--pico-card-sectioning-background-color);
  border: 1px solid var(--pico-border-color);
  border-radius: var(--pico-border-radius);
  min-height: 2.5rem;
  display: flex;
  align-items: center;
  margin-top: 0.5rem;
}

.filename-suggestion {
  padding: 0.75rem;
  background: var(--pico-primary-background);
  border: 1px solid var(--pico-primary-border);
  border-radius: var(--pico-border-radius);
  color: var(--pico-primary-inverse);
  min-height: 2.5rem;
  display: flex;
  align-items: center;
  margin-top: 0.5rem;
}

.link-button {
  background: none;
  border: none;
  color: inherit;
  text-decoration: underline;
  cursor: pointer;
  font: inherit;
  padding: 0;
  margin: 0;
  transition: all 0.2s ease;
}

.link-button:hover {
  text-decoration: none;
  transform: none; /* Override general button hover effect */
  opacity: 0.8;
}

/* Responsive Design */
@media (max-width: 768px) {
  .config-grid {
    grid-template-columns: 1fr;
  }
}
</style>
