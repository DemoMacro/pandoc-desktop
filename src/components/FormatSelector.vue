<template>
  <article>
    <header>🔄 Output Format</header>

    <div class="form-group">
      <label for="format-select">Choose format:</label>
      <select
        id="format-select"
        v-model="outputFormat"
        :disabled="availableOutputFormats.length === 0"
      >
        <option
          v-for="format in availableOutputFormats"
          :key="format.value"
          :value="format.value"
        >
          {{ format.label }}
        </option>
      </select>
    </div>

    <footer>
      <small>
        📊 <strong>{{ formatCount }}</strong> available
      </small>
    </footer>
  </article>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useFileHandling } from "../composables/useFileHandling";
import { usePandoc } from "../composables/usePandoc";
import { getCompatibleOutputFormats } from "../types/pandoc";

const { outputFormat, inputFormat } = useFileHandling();
const { supportedOutputFormats } = usePandoc();

const availableOutputFormats = computed(() =>
  getCompatibleOutputFormats(inputFormat.value, supportedOutputFormats.value),
);

const formatCount = computed(() => availableOutputFormats.value.length);
</script>

<style scoped>
/* Minimal custom styles - let Pico CSS handle the card */
.form-group {
  margin-bottom: 1rem;
}

.form-group label {
  display: block;
  margin-bottom: 0.5rem;
  font-size: 0.875rem;
  font-weight: 500;
}

footer {
  text-align: center;
  color: var(--pico-muted-color);
  padding: 0.5rem var(--pico-spacing) calc(var(--pico-spacing) * 0.75);
  margin-top: 0;
}
</style>
