<template>
  <article>
    <header>🚀 Convert Document</header>

    <div v-if="!canConvert" class="requirements-info">
      <p><small>Required:</small></p>
      <div class="requirements-grid">
        <span :class="['requirement', { met: inputFile }]">
          {{ inputFile ? "✅" : "⏳" }} File
        </span>
        <span :class="['requirement', { met: outputDirectory }]">
          {{ outputDirectory ? "✅" : "⏳" }} Directory
        </span>
        <span :class="['requirement', { met: outputFileName }]">
          {{ outputFileName ? "✅" : "⏳" }} Filename
        </span>
      </div>
    </div>

    <button
      @click="handleConvert"
      :disabled="!canConvert || isConverting"
      :aria-busy="isConverting"
      :class="['primary', { converting: isConverting }]"
    >
      {{ isConverting ? "⏳ Converting..." : "🔄 Convert Document" }}
    </button>
  </article>
</template>

<script setup lang="ts">
import { useConversion } from "../composables/useConversion";
import { useFileHandling } from "../composables/useFileHandling";

const { convertDocument, isConverting, canConvert } = useConversion();
const { inputFile, outputDirectory, outputFileName } = useFileHandling();

const handleConvert = async () => {
  if (!canConvert.value) return;
  await convertDocument();
};
</script>

<style scoped>
/* Minimal custom styles */
.requirements-info {
  margin-bottom: 1rem;
}

.requirements-grid {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
  margin-top: 0.5rem;
}

.requirement {
  display: inline-flex;
  align-items: center;
  padding: 0.25rem 0.5rem;
  border-radius: var(--pico-border-radius);
  font-size: 0.75rem;
  background: var(--pico-card-sectioning-background-color);
  color: var(--pico-muted-color);
  border: 1px solid var(--pico-border-color);
}

.requirement.met {
  background: var(--pico-form-element-valid-background-color);
  color: var(--pico-form-element-valid-color);
  border-color: var(--pico-form-element-valid-border-color);
}

button {
  width: 100%;
}

.converting {
  cursor: wait;
  opacity: 0.8;
}
</style>
