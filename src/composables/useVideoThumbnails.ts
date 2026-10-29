import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { withTimeout, THUMBNAIL_TIMEOUT_MS } from "../lib/tauri-timeout";

export function useVideoThumbnails() {
  const thumbnails = ref<Map<string, string>>(new Map());
  const isLoading = ref(false);

  const generate = async (videoPath: string) => {
    if (thumbnails.value.has(videoPath)) {
      return; // Already cached
    }
    
    try {
      isLoading.value = true;
      const thumbnailDataUrl = await withTimeout(
        invoke<string>("generate_video_thumbnail", { videoPath }),
        THUMBNAIL_TIMEOUT_MS,
        "Loading video thumbnail timed out"
      );
      thumbnails.value = new Map(thumbnails.value).set(videoPath, thumbnailDataUrl);
    } catch (err) {
      console.error("Failed to generate video thumbnail:", err);
    } finally {
      isLoading.value = false;
    }
  };

  const get = (videoPath: string) => thumbnails.value.get(videoPath) ?? null;
  const has = (videoPath: string) => thumbnails.value.has(videoPath);
  const clear = () => thumbnails.value.clear();

  return {
    thumbnails,
    isLoading,
    generate,
    get,
    has,
    clear
  };
}
