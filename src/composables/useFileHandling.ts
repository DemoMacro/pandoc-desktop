import { ref, watch } from "vue";
import { detectInputFormat, getBaseName } from "./useUtils";

const inputFile = ref<string>("");
const outputDirectory = ref<string>("");
const outputFileName = ref<string>("");
const outputFormat = ref<string>("html");
const inputFormat = ref<string>("markdown");

export function useFileHandling() {
  // Watch for input file changes to detect format
  watch(inputFile, (newFile) => {
    if (newFile) {
      inputFormat.value = detectInputFormat(newFile);
      outputFileName.value = getBaseName(newFile);
    }
  });

  // Watch for output format changes to update filename
  watch(outputFormat, () => {
    if (inputFile.value && outputFileName.value) {
      const currentBaseName =
        outputFileName.value.replace(/\.[^/.]+$/, "") ||
        getBaseName(inputFile.value);
      outputFileName.value = currentBaseName;
    }
  });

  const clearFiles = () => {
    inputFile.value = "";
    outputDirectory.value = "";
    outputFileName.value = "";
  };

  return {
    inputFile,
    outputDirectory,
    outputFileName,
    outputFormat,
    inputFormat,
    clearFiles,
  };
}
