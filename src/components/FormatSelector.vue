<template>
  <article>
    <header>🔄 Output Format</header>

    <div class="form-group">
      <label for="format-search">Choose format:</label>
      <div class="format-selector">
        <input
          id="format-search"
          v-model="searchQuery"
          type="text"
          :placeholder="
            selectedFormat
              ? `Selected: ${selectedFormat.label} (.${selectedFormat.ext})`
              : 'Search formats...'
          "
          class="format-search"
          @focus="handleFocus"
          @blur="handleBlur"
          autocomplete="off"
        />

        <div
          v-if="showDropdown && filteredFormats.length > 0"
          class="format-dropdown"
        >
          <div
            v-for="format in filteredFormats"
            :key="format.value"
            class="format-option"
            :class="{ active: format.value === outputFormat }"
            @mousedown.prevent="selectFormat(format)"
          >
            <span class="format-label">{{ format.label }}</span>
            <span class="format-description">{{ format.description }}</span>
          </div>
        </div>

        <div
          v-if="showDropdown && filteredFormats.length === 0 && searchQuery"
          class="no-results"
        >
          No formats found for "{{ searchQuery }}"
        </div>
      </div>
    </div>

    <footer>
      <small>
        📊 <strong>{{ formatCount }}</strong> available
      </small>
    </footer>
  </article>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useFileHandling } from "../composables/useFileHandling";
import { usePandocManager } from "../composables/usePandocManager";
import { getSupportedOutputFormats } from "../composables/useUtils";

const { outputFormat } = useFileHandling();
const { supportedOutputFormats } = usePandocManager();

const searchQuery = ref("");
const showDropdown = ref(false);

const availableOutputFormats = computed(() =>
  getSupportedOutputFormats(supportedOutputFormats.value),
);

const filteredFormats = computed(() => {
  if (!searchQuery.value.trim()) {
    return availableOutputFormats.value;
  }

  const query = searchQuery.value.toLowerCase();
  return availableOutputFormats.value.filter(
    (format) =>
      format.label.toLowerCase().includes(query) ||
      format.value.toLowerCase().includes(query) ||
      format.description?.toLowerCase().includes(query),
  );
});

const selectedFormat = computed(() =>
  availableOutputFormats.value.find((f) => f.value === outputFormat.value),
);

const formatCount = computed(() => availableOutputFormats.value.length);

const selectFormat = (format: any) => {
  outputFormat.value = format.value;
  searchQuery.value = ""; // Clear search after selection
  showDropdown.value = false;
};

const handleBlur = () => {
  // Delay to allow click events on dropdown items
  setTimeout(() => {
    showDropdown.value = false;
    // Keep search empty, don't auto-fill
  }, 150);
};

const handleFocus = () => {
  showDropdown.value = true;
  // Clear search when focusing to allow fresh search
  searchQuery.value = "";
};

// Watch for outputFormat changes to update display
watch(outputFormat, () => {
  if (!showDropdown.value) {
    searchQuery.value = "";
  }
});
</script>

<style scoped>
/* Format selector styles */
.form-group {
  margin-bottom: 1rem;
}

.form-group label {
  display: block;
  margin-bottom: 0.5rem;
  font-size: 0.875rem;
  font-weight: 500;
}

.format-selector {
  position: relative;
}

.format-search {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid var(--pico-border-color);
  border-radius: var(--pico-border-radius);
  background: var(--pico-form-element-background-color);
  color: var(--pico-form-element-color);
  font-size: 1rem;
}

.format-search:focus {
  outline: none;
  border-color: var(--pico-primary-color);
  box-shadow: 0 0 0 2px rgba(var(--pico-primary-rgb), 0.2);
}

.format-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  background: var(--pico-card-background-color);
  border: 1px solid var(--pico-border-color);
  border-top: none;
  border-radius: 0 0 var(--pico-border-radius) var(--pico-border-radius);
  max-height: 300px;
  overflow-y: auto;
  z-index: 1000;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

.format-option {
  padding: 0.5rem 0.75rem;
  cursor: pointer;
  border-bottom: 1px solid var(--pico-border-color);
  transition: background-color 0.2s ease;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 1rem;
}

.format-option:last-child {
  border-bottom: none;
}

.format-option:hover {
  background: var(--pico-secondary-background);
}

.format-option.active {
  background: var(--pico-primary-background);
  color: var(--pico-primary-inverse);
}

.format-label {
  font-weight: 500;
  flex-shrink: 0;
}

.format-description {
  font-size: 0.875rem;
  color: var(--pico-muted-color);
  text-align: right;
  opacity: 0.8;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.format-option:hover .format-description {
  color: var(--pico-color);
  opacity: 0.9;
}

.format-option.active .format-description {
  color: var(--pico-primary-inverse);
  opacity: 0.9;
}

.no-results {
  padding: 1rem;
  text-align: center;
  color: var(--pico-muted-color);
  font-style: italic;
}

footer {
  text-align: center;
  color: var(--pico-muted-color);
  padding: 0.5rem var(--pico-spacing) calc(var(--pico-spacing) * 0.75);
  margin-top: 0;
}
</style>
