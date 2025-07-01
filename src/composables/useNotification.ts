import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";

const getIconForType = (type: "success" | "error" | "info") => {
  // 可以根据需要添加不同类型的图标
  switch (type) {
    case "success":
      return undefined; // 使用默认图标
    case "error":
      return undefined;
    case "info":
    default:
      return undefined;
  }
};

export function useNotification() {
  let permissionGranted = false;

  const initializeNotifications = async () => {
    try {
      permissionGranted = await isPermissionGranted();

      if (!permissionGranted) {
        const permission = await requestPermission();
        permissionGranted = permission === "granted";
      }
    } catch (error) {
      console.warn("Failed to initialize notifications:", error);
    }
  };

  const sendNotificationMessage = async (
    title: string,
    body: string,
    type: "success" | "error" | "info" = "info",
  ) => {
    if (!permissionGranted) {
      await initializeNotifications();
    }

    if (permissionGranted) {
      try {
        await sendNotification({
          title,
          body,
          icon: getIconForType(type),
        });
      } catch (error) {
        console.warn("Failed to send notification:", error);
      }
    }
  };

  const notifyConversionSuccess = (fileName: string) => {
    sendNotificationMessage(
      "Conversion Completed",
      `Successfully converted ${fileName}`,
      "success",
    );
  };

  const notifyConversionError = (error: string) => {
    sendNotificationMessage("Conversion Failed", error, "error");
  };

  const notifyConversionStarted = (fileName: string) => {
    sendNotificationMessage(
      "Conversion Started",
      `Converting ${fileName}...`,
      "info",
    );
  };

  return {
    initializeNotifications,
    sendNotificationMessage,
    notifyConversionSuccess,
    notifyConversionError,
    notifyConversionStarted,
  };
}
