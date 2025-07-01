import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useFileHandling } from "./useFileHandling";
import { usePandoc } from "./usePandoc";
import { useMessages } from "./useMessages";
import { useNotification } from "./useNotification";
import { generateOutputFilenameWithExt } from "../types/pandoc";

const isConverting = ref<boolean>(false);
const progress = ref<number>(0);

const getFileName = (path: string): string => {
  return path.split(/[/\\]/).pop() || "";
};

export function useConversion() {
  const { inputFile, outputDirectory, outputFileName, outputFormat } =
    useFileHandling();
  const { isReady, customPandocPath } = usePandoc();
  const { displayMessage } = useMessages();
  const {
    initializeNotifications,
    notifyConversionSuccess,
    notifyConversionError,
    notifyConversionStarted,
  } = useNotification();

  const canConvert = computed(() => {
    return (
      inputFile.value &&
      outputDirectory.value &&
      outputFileName.value &&
      isReady.value
    );
  });

  const convertDocument = async () => {
    if (!canConvert.value) {
      displayMessage(
        "Please select input file, output directory and filename",
        "error",
      );
      return;
    }

    // 初始化通知权限
    await initializeNotifications();

    isConverting.value = true;
    progress.value = 0;

    const fileName = getFileName(inputFile.value);

    try {
      const outputFileNameWithExt = generateOutputFilenameWithExt(
        outputFileName.value,
        outputFormat.value,
      );
      const outputPath = `${outputDirectory.value}/${outputFileNameWithExt}`;

      progress.value = 25;
      displayMessage("Starting conversion...", "info");
      notifyConversionStarted(fileName);

      const result = await invoke<string>("convert_with_pandoc", {
        inputFile: inputFile.value,
        outputFile: outputPath,
        inputFormat: null,
        outputFormat: outputFormat.value,
        customPandocPath: customPandocPath.value || null,
      });

      progress.value = 100;
      displayMessage(result, "success");
      notifyConversionSuccess(fileName);
    } catch (error) {
      const errorMessage = `Conversion failed: ${error}`;
      displayMessage(errorMessage, "error");
      notifyConversionError(errorMessage);
    } finally {
      isConverting.value = false;
      progress.value = 0;
    }
  };

  return {
    isConverting,
    progress,
    canConvert,
    convertDocument,
  };
}
