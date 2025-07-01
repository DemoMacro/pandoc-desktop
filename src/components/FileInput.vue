<template>
  <article>
    <header>📁 Input File</header>

    <button
      @click="selectInputFile"
      class="secondary"
      :disabled="isSelecting"
      :aria-busy="isSelecting"
    >
      📂 {{ isSelecting ? "Selecting..." : "Select File" }}
    </button>

    <div v-if="inputFile" class="file-preview">
      <div class="file-info">
        <div class="file-icon">📄</div>
        <div class="file-details">
          <div class="file-name">{{ getFileName(inputFile) }}</div>
          <div class="file-format">Format: {{ inputFormat }}</div>
        </div>
      </div>
    </div>

    <div v-else class="empty-state">No file selected</div>
  </article>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { useFileHandling } from "../composables/useFileHandling";
import { useMessages } from "../composables/useMessages";

const { inputFile, inputFormat } = useFileHandling();
const { displayMessage } = useMessages();

const isSelecting = ref(false);

const selectInputFile = async () => {
  isSelecting.value = true;

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
      displayMessage("File selected successfully", "success");
    }
  } catch (error) {
    displayMessage(`Failed to select file: ${error}`, "error");
  } finally {
    isSelecting.value = false;
  }
};

const getFileName = (path: string): string => {
  return path.split(/[/\\]/).pop() || "";
};
</script>

<style scoped>
/* Minimal custom styles - let Pico CSS handle the card */
button.secondary {
  width: 100%;
  margin-bottom: 1rem;
  transition: all 0.2s ease;
}

button.secondary:hover:not(:disabled) {
  transform: translateY(-1px);
}

.file-preview {
  margin-top: 1rem;
}

.file-info {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem;
  background: var(--pico-card-sectioning-background-color);
  border: 1px solid var(--pico-border-color);
  border-radius: var(--pico-border-radius);
}

.file-icon {
  font-size: 1.25rem;
  color: var(--pico-primary);
  flex-shrink: 0;
}

.file-details {
  flex: 1;
  min-width: 0;
}

.file-name {
  font-weight: 500;
  margin-bottom: 0.25rem;
  word-break: break-all;
  font-size: 0.875rem;
}

.file-format {
  font-size: 0.75rem;
  color: var(--pico-muted-color);
}

.empty-state {
  text-align: center;
  padding: 2rem 1rem;
  color: var(--pico-muted-color);
  font-size: 0.875rem;
}
</style>
