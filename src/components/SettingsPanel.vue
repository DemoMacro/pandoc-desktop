<template>
  <dialog :open="showSettings" @click="closeSettings">
    <article @click.stop>
      <header>
        ⚙️ Application Settings
        <button aria-label="Close" rel="prev" @click="closeSettings"></button>
      </header>

      <section>
        <h4>Appearance</h4>
        <div class="grid">
          <div>
            <h6>Theme</h6>
            <select v-model="selectedTheme" @change="applyTheme">
              <option value="auto">🔄 Auto (System)</option>
              <option value="light">☀️ Light</option>
              <option value="dark">🌙 Dark</option>
            </select>
          </div>
        </div>
      </section>

      <section>
        <h4>About</h4>
        <p>
          <small>
            <strong>Pandoc Desktop</strong><br />
            A modern desktop application for document conversion using
            Pandoc.<br />
            <a
              href="https://github.com/DemoMacro/pandoc-desktop"
              target="_blank"
            >
              🌐 View on GitHub
            </a>
          </small>
        </p>
      </section>
    </article>
  </dialog>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useUI } from "../composables/useUI";
import { useMessages } from "../composables/useMessages";

const { showSettings, closeSettings } = useUI();
const { displayMessage } = useMessages();

// Settings state
const selectedTheme = ref<"auto" | "light" | "dark">("auto");

// Theme management
const applyTheme = () => {
  let actualTheme: "light" | "dark";

  if (selectedTheme.value === "auto") {
    actualTheme = window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  } else {
    actualTheme = selectedTheme.value;
  }

  document.documentElement.setAttribute("data-theme", actualTheme);
  saveSettings();
  displayMessage(`Theme switched to ${selectedTheme.value}`, "success");
};

// Settings persistence
const saveSettings = () => {
  const settings = {
    theme: selectedTheme.value,
  };

  localStorage.setItem("pandoc-desktop-settings", JSON.stringify(settings));
};

const loadSettings = () => {
  try {
    const saved = localStorage.getItem("pandoc-desktop-settings");
    if (saved) {
      const settings = JSON.parse(saved);
      selectedTheme.value = settings.theme || "auto";
    }
  } catch (error) {
    console.warn("Failed to load settings:", error);
  }
};

onMounted(() => {
  loadSettings();
  applyTheme();
});
</script>

<style scoped>
.grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: var(--pico-spacing);
}

select {
  width: 100%;
}

label {
  display: flex;
  align-items: center;
  gap: calc(var(--pico-spacing) * 0.5);
  margin-bottom: calc(var(--pico-spacing) * 0.5);
  cursor: pointer;
}

label input[type="checkbox"] {
  margin: 0;
}

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
</style>
