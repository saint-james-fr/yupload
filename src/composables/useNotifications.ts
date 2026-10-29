import { ref, type Ref } from "vue";
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";

type NotificationPermissionState = "unknown" | "granted" | "denied";

/**
 * Composable for managing desktop notifications following Tauri v2 best practices.
 * 
 * Provides:
 * - Permission state management
 * - Permission request handling
 * - Safe notification sending with error handling
 */
export const useNotifications = () => {
  const permissionState = ref<NotificationPermissionState>("unknown");

  /**
   * Check current permission state and update internal state.
   * @returns Promise<boolean> - true if permission is granted
   */
  const checkPermission = async (): Promise<boolean> => {
    try {
      const granted = await isPermissionGranted();
      permissionState.value = granted ? "granted" : "denied";
      return granted;
    } catch (err) {
      console.error("Failed to check notification permission:", err);
      permissionState.value = "unknown";
      return false;
    }
  };

  /**
   * Ensure permission is granted, requesting if necessary.
   * Follows Tauri v2 documentation pattern:
   * 1. Check if permission is granted
   * 2. Request permission if not granted
   * 3. Return final permission state
   * 
   * @returns Promise<boolean> - true if permission is granted after request
   */
  const ensurePermission = async (): Promise<boolean> => {
    try {
      // Check current permission state
      let granted = await isPermissionGranted();
      
      // If not granted, request permission
      if (!granted) {
        const permission = await requestPermission();
        granted = permission === "granted";
      }
      
      // Update internal state
      permissionState.value = granted ? "granted" : "denied";
      return granted;
    } catch (err) {
      console.error("Failed to ensure notification permission:", err);
      permissionState.value = "unknown";
      return false;
    }
  };

  /**
   * Request notification permission explicitly.
   * Useful for user-initiated permission requests (e.g., in settings).
   * 
   * @returns Promise<boolean> - true if permission was granted
   */
  const requestPermissionExplicit = async (): Promise<boolean> => {
    try {
      const permission = await requestPermission();
      const granted = permission === "granted";
      permissionState.value = granted ? "granted" : "denied";
      return granted;
    } catch (err) {
      console.error("Failed to request notification permission:", err);
      permissionState.value = "unknown";
      return false;
    }
  };

  /**
   * Send a notification with proper permission checking and error handling.
   * Follows Tauri v2 documentation pattern:
   * 1. Check if permission is granted
   * 2. Request permission if not granted
   * 3. Send notification if permission granted
   * 
   * @param title - Notification title
   * @param body - Notification body text
   * @returns Promise<boolean> - true if notification was sent successfully
   */
  const notify = async (title: string, body: string): Promise<boolean> => {
    try {
      // Ensure permission is granted before sending
      const hasPermission = await ensurePermission();
      
      if (!hasPermission) {
        console.warn("Notification permission not granted, cannot send notification");
        return false;
      }

      // Send notification
      sendNotification({ title, body });
      return true;
    } catch (err) {
      console.error("Failed to send notification:", err);
      return false;
    }
  };

  /**
   * Initialize permission state by checking current status.
   * Call this on component mount to sync permission state.
   */
  const initializePermission = async (): Promise<void> => {
    await checkPermission();
  };

  return {
    permissionState: permissionState as Readonly<Ref<NotificationPermissionState>>,
    checkPermission,
    ensurePermission,
    requestPermissionExplicit,
    notify,
    initializePermission,
  };
};
