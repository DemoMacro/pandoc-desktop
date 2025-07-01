import { ref } from "vue";

type MessageType = "success" | "error" | "info" | "warning";

const message = ref<string>("");
const messageType = ref<MessageType>("info");
const showMessage = ref<boolean>(false);

export function useMessages() {
  let timeoutId: number | null = null;

  const displayMessage = (
    msg: string,
    type: MessageType = "info",
    duration = 5000,
  ) => {
    message.value = msg;
    messageType.value = type;
    showMessage.value = true;

    if (timeoutId) {
      clearTimeout(timeoutId);
    }

    timeoutId = setTimeout(() => {
      showMessage.value = false;
      message.value = "";
    }, duration);
  };

  const clearMessage = () => {
    if (timeoutId) {
      clearTimeout(timeoutId);
      timeoutId = null;
    }
    showMessage.value = false;
    message.value = "";
  };

  return {
    message,
    messageType,
    showMessage,
    displayMessage,
    clearMessage,
  };
}
