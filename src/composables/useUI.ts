import { ref } from "vue";

const showSettings = ref<boolean>(false);
const showPandocManager = ref<boolean>(false);

export function useUI() {
  const toggleSettings = () => {
    showSettings.value = !showSettings.value;
  };

  const closeSettings = () => {
    showSettings.value = false;
  };

  const togglePandocManager = () => {
    showPandocManager.value = !showPandocManager.value;
  };

  const closePandocManager = () => {
    showPandocManager.value = false;
  };

  return {
    showSettings,
    showPandocManager,
    toggleSettings,
    closeSettings,
    togglePandocManager,
    closePandocManager,
  };
}
