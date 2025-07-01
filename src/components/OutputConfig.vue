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
        <input
          id="filename"
          v-model="outputFileName"
          type="text"
          placeholder="Enter filename..."
          :disabled="!outputDirectory"
        />
      </div>
    </div>
  </article>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { useFileHandling } from "../composables/useFileHandling";
import { useMessages } from "../composables/useMessages";

const { outputDirectory, outputFileName } = useFileHandling();
const { displayMessage } = useMessages();

const isSelecting = ref(false);

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

.path-display {
  padding: 0.5rem;
  background: var(--pico-card-sectioning-background-color);
  border: 1px solid var(--pico-border-color);
  border-radius: var(--pico-border-radius);
  font-family: monospace;
  word-break: break-all;
}

/* Responsive Design */
@media (max-width: 768px) {
  .config-grid {
    grid-template-columns: 1fr;
  }
}
</style>
