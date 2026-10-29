import { computed, type Ref } from "vue";
import type { PipelinePhase, PipelineItem, StepStatus } from "../types";

type VideoProgress = {
  index: number;
  audioPath: string;
  audioName: string;
  progress: number;
  message: string;
  status: "pending" | "processing" | "completed" | "failed";
  error?: string;
};

type UploadProgress = {
  index: number;
  progress: number;
  message: string;
  status: "pending" | "uploading" | "completed" | "failed";
  error?: string;
};

type UnifiedVideoMetadata = { title?: string };

export type UnifiedPipelineInput = {
  normalizationProgress: Ref<{ total: number; completed: number; message: string }>;
  videoProgresses: Ref<Map<number, VideoProgress>>;
  uploadProgresses: Ref<Map<number, UploadProgress>>;
  itemDisplayProgress: Ref<Map<number, number>>;
  uploadQueue: Ref<unknown[]>;
  videoMetadata: Ref<Map<string, UnifiedVideoMetadata>>;
  audioTracks: Ref<Array<{ path: string }>>;
  normalizeAudio: Ref<boolean>;
  isNormalizing: Ref<boolean>;
  isCancelled: Ref<boolean>;
  saveVideosLocally: Ref<boolean>;
  uploadToYouTube: Ref<boolean>;
  /** Track index -> thumbnail data URL for pipeline item thumbnails (video frame, once encoded). */
  uploadIndexToThumbnailUrl?: Ref<Map<number, string>>;
  /** Track index -> image preview data URL for pipeline item thumbnails (from image assignment). */
  trackIndexToImagePreviewUrl?: Ref<Map<number, string>>;
};

export function useUnifiedPipeline(input: UnifiedPipelineInput) {
  const unifiedPipeline = computed<{
    phases: PipelinePhase[];
    itemsByPhase: Record<string, PipelineItem[]>;
  }>(() => {
    const phases: PipelinePhase[] = [];
    const itemsByPhase: Record<string, PipelineItem[]> = {};
    const {
      normalizationProgress,
      videoProgresses,
      uploadProgresses,
      itemDisplayProgress,
      uploadQueue,
      videoMetadata,
      audioTracks,
      normalizeAudio: normalizeAudioRef,
      isNormalizing,
      isCancelled,
      saveVideosLocally,
      uploadToYouTube,
      uploadIndexToThumbnailUrl,
      trackIndexToImagePreviewUrl
    } = input;

    const norm = normalizationProgress.value;
    const hasNormalize = normalizeAudioRef.value;
    const isNorm = isNormalizing.value;

    if (hasNormalize) {
      const normStatus: StepStatus = isCancelled.value
        ? "cancelled"
        : isNorm
          ? "active"
          : norm.total > 0 && norm.completed >= norm.total
            ? "done"
            : "pending";
      phases.push({
        id: "normalize",
        label: "Normalize audio",
        description: norm.message || undefined,
        status: normStatus,
        message: norm.message,
        total: norm.total,
        completed: norm.completed
      });
    }

    if (saveVideosLocally.value || uploadToYouTube.value) {
      const completedCount = Array.from(videoProgresses.value.values()).filter(
        vp => vp.status === "completed"
      ).length;
      const cancelledCount = Array.from(videoProgresses.value.values()).filter(
        vp => vp.status === "failed" && vp.message === "Cancelled"
      ).length;
      const totalItems = videoProgresses.value.size || 0;
      const allDoneOrCancelled = totalItems > 0 && completedCount + cancelledCount >= totalItems;
      const shouldWaitForNormalization = hasNormalize && (isNorm || (norm.total > 0 && norm.completed < norm.total));

      const videoStatus: StepStatus = isCancelled.value || (allDoneOrCancelled && cancelledCount > 0 && completedCount === 0)
        ? "cancelled"
        : allDoneOrCancelled
          ? "done"
          : shouldWaitForNormalization
            ? "pending"
            : totalItems > 0
              ? "active"
              : "pending";

      phases.push({
        id: "videos",
        label: "Create videos",
        description: totalItems > 0 ? undefined : "Preparing...",
        status: videoStatus,
        total: totalItems,
        completed: completedCount
      });

      const videoItems: PipelineItem[] = Array.from(videoProgresses.value.values())
        .sort((a, b) => a.index - b.index)
        .map(vp => {
          let status: StepStatus = "pending";
          const isItemCancelled = vp.message === "Cancelled" || (isCancelled.value && (vp.status === "processing" || vp.status === "pending"));
          if (isItemCancelled) status = "cancelled";
          else if (vp.status === "processing") status = "active";
          else if (vp.status === "completed") status = "done";
          else if (vp.status === "failed") status = "failed";
          return {
            id: vp.index,
            name: vp.audioName,
            status,
            progress: status === "cancelled" ? 0 : Math.round(itemDisplayProgress.value.get(vp.index) ?? vp.progress),
            message: vp.status === "pending" || status === "cancelled" ? undefined : vp.message,
            stepLabel: status === "cancelled" ? "Cancelled" : vp.status === "completed" || vp.progress >= 100 ? "Done" : vp.status === "pending" ? "Waiting..." : vp.progress >= 50 ? "Encoding..." : "Processing...",
            error: vp.error,
            thumbnailUrl: trackIndexToImagePreviewUrl?.value?.get(vp.index)
              ?? uploadIndexToThumbnailUrl?.value?.get(vp.index)
              ?? undefined
          };
        });
      if (videoItems.length > 0) itemsByPhase["videos"] = videoItems;
    }

    if (uploadToYouTube.value) {
      const completedCount = Array.from(uploadProgresses.value.values()).filter(p => p.status === "completed").length;
      const cancelledCount = Array.from(uploadProgresses.value.values()).filter(
        p => p.status === "failed" && p.message === "Cancelled"
      ).length;
      const totalItems = uploadProgresses.value.size || 0;
      const allDoneOrCancelled = totalItems > 0 && completedCount + cancelledCount >= totalItems;
      const hasCompletedVideos = Array.from(videoProgresses.value.values()).some(vp => vp.status === "completed");
      const shouldWaitForVideoCreation = (saveVideosLocally.value || uploadToYouTube.value) && !hasCompletedVideos && uploadQueue.value.length === 0;

      const uploadStatus: StepStatus = isCancelled.value || (allDoneOrCancelled && cancelledCount > 0 && completedCount === 0)
        ? "cancelled"
        : allDoneOrCancelled
          ? "done"
          : shouldWaitForVideoCreation
            ? "pending"
            : totalItems > 0
              ? "active"
              : "pending";

      phases.push({
        id: "upload",
        label: "Upload to YouTube",
        description: shouldWaitForVideoCreation
          ? "Waiting for first video to complete…"
          : totalItems > 0
            ? `Uploading ${totalItems} video${totalItems === 1 ? "" : "s"}…`
            : uploadQueue.value.length > 0
              ? "Videos ready for upload…"
              : "Preparing…",
        status: uploadStatus,
        total: totalItems,
        completed: completedCount
      });

      const uploadItems: PipelineItem[] = Array.from(uploadProgresses.value.entries())
        .sort((a, b) => a[0] - b[0])
        .map(([index, p]) => {
          let status: StepStatus = "pending";
          if (isCancelled.value && (p.status === "uploading" || p.status === "pending")) status = "cancelled";
          else if (p.status === "uploading") status = "active";
          else if (p.status === "completed") status = "done";
          else if (p.status === "failed") status = "failed";
          return {
            id: index,
            name: videoMetadata.value.get(audioTracks.value[index]?.path)?.title || `Video ${index + 1}`,
            status,
            progress: p.progress,
            message: status === "cancelled" ? undefined : p.message,
            stepLabel: status === "cancelled" ? "Cancelled" : p.progress >= 100 ? "Done" : "Uploading…",
            error: p.error,
            thumbnailUrl: trackIndexToImagePreviewUrl?.value?.get(index)
              ?? uploadIndexToThumbnailUrl?.value?.get(index)
              ?? undefined
          };
        });
      if (uploadItems.length > 0) itemsByPhase["upload"] = uploadItems;
    }

    return { phases, itemsByPhase };
  });

  return { unifiedPipeline };
}
