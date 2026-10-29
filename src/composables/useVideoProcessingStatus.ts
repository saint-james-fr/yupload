import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { VideoProcessingStatus } from "../types";

export type VideoProcessingStatusResponse = {
  video_id: string;
  status: VideoProcessingStatus;
  failure_reason?: string | null;
  checked_at: string;
};

export function useVideoProcessingStatus() {
  const isChecking = ref(false);
  const error = ref<string | null>(null);

  const checkStatus = async (videoId: string): Promise<VideoProcessingStatusResponse> => {
    try {
      isChecking.value = true;
      error.value = null;
      const result = await invoke<VideoProcessingStatusResponse>("check_video_processing_status", {
        videoId,
      });
      return result;
    } catch (err) {
      const errorMsg = `Failed to check status: ${err}`;
      error.value = errorMsg;
      throw new Error(errorMsg);
    } finally {
      isChecking.value = false;
    }
  };

  return {
    isChecking,
    error,
    checkStatus,
  };
}
