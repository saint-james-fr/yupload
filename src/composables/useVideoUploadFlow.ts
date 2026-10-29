import { ref, computed, watch, onMounted, onUnmounted, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { withTimeout, FOLDER_LIST_TIMEOUT_MS } from "../lib/tauri-timeout";
import { listen } from "@tauri-apps/api/event";
import type {
  FolderContents,
  BulkVideoUploadResult,
  DiscogsRelease,
  UploadHistoryEntry
} from "../types";
import type { VideoUploadMetadata } from "../types";
import type { PipelinePhase, PipelineItem, StepStatus } from "../types";
import { usePreferences } from "./usePreferences";
import { useYouTube } from "./useYouTube";
import { useDiscogs } from "./useDiscogs";
import { useDescriptionTemplate } from "./useDescriptionTemplate";
import { useVideoThumbnails } from "./useVideoThumbnails";
import { useUploadHistory } from "./useUploadHistory";
import { useVideoProcessingStatus } from "./useVideoProcessingStatus";
import { useNotifications } from "./useNotifications";
import { useSharedModals } from "./useSharedModals";

const MAX_FILES_PER_OPERATION = 100;

export function useVideoUploadFlow() {
  const { preferences, loadPreferences, updatePreferences } = usePreferences();
  const youtube = useYouTube();
  const discogs = useDiscogs();
  const { autofillMetadataFromRelease, applyToVideo } = useDescriptionTemplate();
  const {
    generate: generateVideoThumbnail,
    has: hasVideoThumbnail,
    get: getVideoThumbnail,
    clear: clearVideoThumbnails
  } = useVideoThumbnails();
  const uploadHistory = useUploadHistory();
  const { checkStatus } = useVideoProcessingStatus();
  const { notify } = useNotifications();
  const modals = useSharedModals();

  const videos = ref<VideoUploadMetadata[]>([]);
  const selectedVideoIndex = ref<number | null>(null);
  const selectedFolder = ref<string | null>(null);
  const isUploading = ref(false);
  const uploadProgress = ref<
    Map<number, { progress: number; message: string; status: "pending" | "uploading" | "completed" | "failed"; error?: string }>
  >(new Map());
  const error = ref<string | null>(null);
  const success = ref<string | null>(null);
  const isCancelled = ref(false);
  const showApplyDescriptionSuccess = ref(false);
  const showApplyPrivacySuccess = ref(false);
  const showApplyCategorySuccess = ref(false);
  const showApplyTagsSuccess = ref(false);
  const isLoadingVideos = ref(false);
  const isLoadingFolder = ref(false);
  const showLinkToOtherVideosModal = ref(false);
  const pendingRelease = ref<DiscogsRelease | null>(null);
  const selectedVideosForLinking = ref<Set<number>>(new Set());
  const noVideosFoundModalOpen = ref(false);
  const forcePrivateUploads = ref(false);
  const rightPanelStartRef = ref<HTMLElement | null>(null);
  const progressSectionRef = ref<HTMLElement | null>(null);
  const successfulUploads = ref<Map<number, { videoId: string; videoUrl: string }>>(new Map());

  const selectedVideo = computed(() => {
    if (selectedVideoIndex.value === null) return null;
    return videos.value[selectedVideoIndex.value] || null;
  });

  const isAuthenticated = computed(() => youtube.isAuthenticated.value);
  const oauthError = computed(() => youtube.error.value);
  const isDiscogsAuthenticated = computed(() => discogs.isAuthenticated.value);
  const discogsError = computed(() => discogs.error.value);

  const canUpload = computed(() => {
    if (videos.value.length === 0 || isUploading.value || !isAuthenticated.value) return false;
    return videos.value.every(v => v.videoPath && v.title.trim().length > 0);
  });

  const videoStatus = (v: VideoUploadMetadata): "success" | "warning" | "muted" => {
    const hasTitle = (v.title?.trim() ?? "").length > 0;
    const hasDescription = (v.description?.trim() ?? "").length > 0;
    if (!hasTitle) return "muted";
    if (!hasDescription) return "warning";
    return "success";
  };

  const hasPreviousVideo = computed(
    () => selectedVideoIndex.value !== null && selectedVideoIndex.value > 0
  );
  const hasNextVideo = computed(
    () =>
      selectedVideoIndex.value !== null &&
      videos.value.length > 1 &&
      selectedVideoIndex.value < videos.value.length - 1
  );

  const otherVideos = computed(() =>
    videos.value
      .map((video, index) => ({ video, index }))
      .filter(({ index }) => index !== selectedVideoIndex.value)
  );

  const uploadPipeline = computed<{
    phases: PipelinePhase[];
    itemsByPhase: Record<string, PipelineItem[]>;
  }>(() => {
    const phases: PipelinePhase[] = [];
    const itemsByPhase: Record<string, PipelineItem[]> = {};
    const total = uploadProgress.value.size;
    const completed = Array.from(uploadProgress.value.values()).filter(
      p => p.status === "completed"
    ).length;
    const cancelled = Array.from(uploadProgress.value.values()).filter(
      p => p.status === "failed" && p.message === "Cancelled"
    ).length;
    const allDoneOrCancelled = total > 0 && completed + cancelled >= total;
    const phaseStatus: StepStatus =
      isCancelled.value || (allDoneOrCancelled && cancelled > 0 && completed === 0)
        ? "cancelled"
        : allDoneOrCancelled
          ? "done"
          : total === 0
            ? "pending"
            : "active";
    phases.push({
      id: "upload",
      label: "Upload to YouTube",
      description: total > 0 ? `Uploading ${total} video${total === 1 ? "" : "s"}…` : "Preparing…",
      status: phaseStatus,
      total,
      completed
    });
    const items: PipelineItem[] = Array.from(uploadProgress.value.entries())
      .sort((a, b) => a[0] - b[0])
      .map(([index, p]) => {
        let status: StepStatus = "pending";
        if (isCancelled.value && (p.status === "uploading" || p.status === "pending")) {
          status = "cancelled";
        } else if (p.status === "uploading") {
          status = "active";
        } else if (p.status === "completed") {
          status = "done";
        } else if (p.status === "failed") {
          status = "failed";
        }
        const videoPath = videos.value[index]?.videoPath;
        return {
          id: index,
          name: videos.value[index]?.title || `Video ${index + 1}`,
          status,
          progress: p.progress,
          message: status === "cancelled" ? undefined : p.message,
          stepLabel: status === "cancelled" ? "Cancelled" : p.progress >= 100 ? "Done" : "Uploading…",
          error: p.error,
          thumbnailUrl: videoPath ? getVideoThumbnail(videoPath) ?? undefined : undefined
        };
      });
    if (items.length > 0) itemsByPhase["upload"] = items;
    return { phases, itemsByPhase };
  });

  watch(selectedVideoIndex, () => {
    nextTick(() => {
      rightPanelStartRef.value?.scrollIntoView({ behavior: "smooth", block: "start" });
    });
  });

  watch(isUploading, uploading => {
    if (uploading) {
      nextTick(() => {
        progressSectionRef.value?.scrollIntoView({ behavior: "smooth", block: "start" });
      });
    }
  });

  watch(selectedVideo, newVideo => {
    if (newVideo && isDiscogsAuthenticated.value && !newVideo.discogsRelease) {
      const title = newVideo.title.trim();
      if (title && title.length >= 3) {
        discogs.searchQuery.value = title;
      }
    } else if (newVideo && !isDiscogsAuthenticated.value) {
      discogs.clearSearch();
    }
  });

  watch(
    [forcePrivateUploads, () => selectedVideo.value?.privacyStatus, selectedVideoIndex],
    ([fp]) => {
      if (!fp || !selectedVideo.value) return;
      if (selectedVideo.value.privacyStatus !== "private") {
        const i = selectedVideoIndex.value;
        if (i !== null) {
          videos.value[i] = { ...videos.value[i], privacyStatus: "private" };
        }
      }
    },
    { immediate: true }
  );

  const unlistens: Array<() => void> = [];

  const setupEventListeners = async () => {
    const unlisten1 = await listen("youtube-upload-start", (event: { payload: { index: number; video_path: string; title: string } }) => {
      const data = event.payload;
      uploadProgress.value.set(data.index, {
        progress: 0,
        message: "Starting upload...",
        status: "uploading"
      });
    });
    unlistens.push(unlisten1);

    const unlisten2 = await listen("youtube-upload-progress", (event: { payload: { video_path: string; progress: number; message: string } }) => {
      const data = event.payload;
      const index = videos.value.findIndex(v => v.videoPath === data.video_path);
      if (index !== -1) {
        const existing = uploadProgress.value.get(index);
        if (existing) {
          existing.progress = data.progress;
          existing.message = data.message;
        } else {
          uploadProgress.value.set(index, {
            progress: data.progress,
            message: data.message,
            status: "uploading"
          });
        }
      }
    });
    unlistens.push(unlisten2);

    const unlisten3 = await listen("youtube-upload-complete", (event: { payload: { index: number; success: boolean; video_id?: string; error?: string } }) => {
      const data = event.payload;
      const existing = uploadProgress.value.get(data.index);
      if (existing) {
        existing.status = data.success ? "completed" : "failed";
        existing.progress = data.success ? 100 : 0;
        existing.message = data.success ? "Upload complete!" : (data.error || "Upload failed");
        if (data.error) existing.error = data.error;
      }
      if (data.success && data.video_id) {
        successfulUploads.value.set(data.index, {
          videoId: data.video_id,
          videoUrl: `https://www.youtube.com/watch?v=${data.video_id}`
        });
      }
    });
    unlistens.push(unlisten3);
  };

  const createDefaultVideo = (videoPath: string, name: string): VideoUploadMetadata => {
    const prefs = preferences.value.preferences;
    return {
      videoPath,
      title: name.replace(/\.[^/.]+$/, "") || "Untitled",
      description: "",
      tags: [],
      categoryId: prefs?.default_category_id ?? "10",
      privacyStatus: (forcePrivateUploads.value
        ? "private"
        : (prefs?.default_privacy_status ?? "public")) as VideoUploadMetadata["privacyStatus"],
      selfDeclaredMadeForKids: false,
      publishAt: null,
      playlistId: (prefs?.default_playlist_id ?? null) as string | null,
      discogsReleaseId: null,
      discogsRelease: null
    };
  };

  const selectVideos = async () => {
    try {
      isLoadingVideos.value = true;
      error.value = null;
      const paths = await open({
        multiple: true,
        filters: [{ name: "Video", extensions: ["mp4", "mov", "avi", "mkv", "webm"] }]
      });
      if (paths && Array.isArray(paths)) {
        const newVideos = (paths as string[]).map(path =>
          createDefaultVideo(path, path.split("/").pop() ?? path)
        );
        const totalVideos = videos.value.length + newVideos.length;
        if (totalVideos > MAX_FILES_PER_OPERATION) {
          error.value = `Cannot select more than ${MAX_FILES_PER_OPERATION} videos. You already have ${videos.value.length} video(s) selected, and you're trying to add ${newVideos.length} more.`;
          return;
        }
        videos.value = [...videos.value, ...newVideos];
        if (selectedVideoIndex.value === null && newVideos.length > 0) {
          selectedVideoIndex.value = videos.value.length - newVideos.length;
          const first = newVideos[0];
          if (first) await generateVideoThumbnail(first.videoPath);
        }
      }
    } catch (err) {
      error.value = `Failed to select videos: ${err}`;
    } finally {
      isLoadingVideos.value = false;
    }
  };

  const loadFolderContents = async (folderPath: string) => {
    try {
      isLoadingFolder.value = true;
      const contents = await withTimeout(
        invoke<FolderContents>("list_folder_contents", { folderPath }),
        FOLDER_LIST_TIMEOUT_MS,
        "Loading folder timed out"
      );
      if (contents.videos.length > 0) {
        const newVideos = contents.videos.map(v =>
          createDefaultVideo(v.path, v.name)
        );
        const totalVideos = videos.value.length + newVideos.length;
        if (totalVideos > MAX_FILES_PER_OPERATION) {
          error.value = `Cannot select more than ${MAX_FILES_PER_OPERATION} videos. You already have ${videos.value.length} video(s) selected, and this folder contains ${contents.videos.length} video(s).`;
          return;
        }
        videos.value = [...videos.value, ...newVideos];
        if (selectedVideoIndex.value === null && newVideos.length > 0) {
          selectedVideoIndex.value = videos.value.length - newVideos.length;
          const first = newVideos[0];
          if (first) await generateVideoThumbnail(first.videoPath);
        }
      } else {
        error.value = "No video files found in the selected folder";
        noVideosFoundModalOpen.value = true;
      }
    } catch (err) {
      error.value = `Failed to load folder contents: ${err}`;
    } finally {
      isLoadingFolder.value = false;
    }
  };

  const selectFolder = async () => {
    try {
      isLoadingFolder.value = true;
      error.value = null;
      const path = await open({
        directory: true,
        multiple: false,
        defaultPath: preferences.value.preferences?.last_upload_source ?? undefined
      });
      if (path && typeof path === "string") {
        selectedFolder.value = path;
        await loadFolderContents(path);
        updatePreferences({ last_upload_source: path });
      }
    } catch (err) {
      error.value = `Failed to select folder: ${err}`;
    } finally {
      isLoadingFolder.value = false;
    }
  };

  const selectVideo = async (index: number) => {
    selectedVideoIndex.value = index;
    const video = videos.value[index];
    if (video && !hasVideoThumbnail(video.videoPath)) {
      await generateVideoThumbnail(video.videoPath);
    }
  };

  const removeVideo = (index: number) => {
    videos.value.splice(index, 1);
    uploadProgress.value.delete(index);
    if (selectedVideoIndex.value === index) {
      if (videos.value.length > 0) {
        selectedVideoIndex.value = Math.min(index, videos.value.length - 1);
      } else {
        selectedVideoIndex.value = null;
      }
    } else if (selectedVideoIndex.value !== null && selectedVideoIndex.value > index) {
      selectedVideoIndex.value--;
    }
  };

  const updateVideo = (field: keyof VideoUploadMetadata, value: unknown) => {
    if (selectedVideo.value && selectedVideoIndex.value !== null) {
      const index = selectedVideoIndex.value;
      videos.value[index] = { ...videos.value[index], [field]: value };
    }
  };

  const resetUploadState = () => {
    isUploading.value = false;
    isCancelled.value = false;
    uploadProgress.value.clear();
    error.value = null;
    success.value = null;
  };

  const uploadVideos = async () => {
    if (!canUpload.value) return;
    isUploading.value = true;
    isCancelled.value = false;
    error.value = null;
    success.value = null;
    uploadProgress.value.clear();
    successfulUploads.value.clear();
    videos.value.forEach((_, index) => {
      uploadProgress.value.set(index, {
        progress: 0,
        message: "Waiting...",
        status: "pending"
      });
    });

    // Generate thumbnails in background so they appear in the pipeline as they load
    videos.value.forEach(v => {
      if (v.videoPath && !hasVideoThumbnail(v.videoPath)) {
        generateVideoThumbnail(v.videoPath);
      }
    });

    try {
      const uploadRequest = {
        videos: videos.value.map(v => {
          const isDraft = v.privacyStatus === "draft";
          const privacy = forcePrivateUploads.value ? "private" : (isDraft ? "private" : v.privacyStatus);
          const publishAt =
            !isDraft && v.privacyStatus === "private" && v.publishAt
              ? (v.publishAt.includes("Z") ? v.publishAt : new Date(v.publishAt).toISOString())
              : null;
          return {
            video_path: v.videoPath,
            title: v.title,
            description: v.description,
            tags: v.tags,
            category_id: v.categoryId,
            privacy_status: privacy,
            self_declared_made_for_kids: v.selfDeclaredMadeForKids,
            publish_at: publishAt || undefined,
            playlist_id: v.playlistId || undefined
          };
        })
      };

      const result = await invoke<BulkVideoUploadResult>("upload_bulk_videos_to_youtube", {
        request: uploadRequest
      });

      await new Promise(resolve => setTimeout(resolve, 500));

      if (successfulUploads.value.size > 0) {
        try {
          const newEntries: UploadHistoryEntry[] = await Promise.all(
            Array.from(successfulUploads.value.entries()).map(async ([index, uploadData]) => {
              const videoMetadata = videos.value[index];
              const uploadResult = result.successful.find(r => r.video_id === uploadData.videoId);
              const videoUrl = uploadResult?.video_url || uploadData.videoUrl;
              const discogsRelease = videoMetadata?.discogsRelease;
              const discogsReleaseUrl = discogsRelease
                ? (discogsRelease.uri || `https://www.discogs.com/release/${discogsRelease.id}`)
                : null;
              let processingStatus: UploadHistoryEntry["processingStatus"];
              let processingStatusCheckedAt: string | undefined;
              let processingFailureReason: string | null | undefined;
              try {
                const statusResult = await checkStatus(uploadData.videoId);
                processingStatus = statusResult.status;
                processingStatusCheckedAt = statusResult.checked_at;
                processingFailureReason = statusResult.failure_reason || null;
              } catch {
                // continue without status
              }
              return {
                id: `${Date.now()}-${index}-${Math.random().toString(36).slice(2, 11)}`,
                title: videoMetadata?.title || uploadData.videoId,
                youtubeVideoId: uploadData.videoId,
                youtubeUrl: videoUrl,
                discogsReleaseId: discogsRelease?.id ?? null,
                discogsReleaseUrl: discogsReleaseUrl ?? null,
                discogsReleaseTitle: discogsRelease?.title ?? null,
                uploadedAt: new Date().toISOString(),
                processingStatus,
                processingStatusCheckedAt,
                processingFailureReason
              };
            })
          );
          await uploadHistory.addMultiple(newEntries);
        } catch (err) {
          console.error("Failed to save upload history:", err);
        }
      }

      let navigatePayload: { from: "upload"; title: string; summary: string } | null = null;
      if (result.failed.length === 0) {
        const message = `Successfully uploaded ${result.successful.length} video(s)!`;
        success.value = message;
        navigatePayload = { from: "upload", title: "Upload complete", summary: message };
      } else {
        success.value = `Uploaded ${result.successful.length} video(s), ${result.failed.length} failed`;
        error.value = `Failed videos:\n${result.failed.map(([path, err]: [string, string]) => `${path.split("/").pop()}: ${err}`).join("\n")}`;
      }

      const notifyEnabled = preferences.value.preferences?.notify_on_upload_complete ?? false;
      if (notifyEnabled) {
        await notify(
          "Upload complete",
          result.failed.length === 0
            ? `Successfully uploaded ${result.successful.length} video(s).`
            : `Uploaded ${result.successful.length} video(s), ${result.failed.length} failed.`
        );
      }
      return navigatePayload;
    } catch (err) {
      const errMsg = String(err);
      if (errMsg.includes("cancelled") || errMsg.includes("Operation cancelled")) {
        isCancelled.value = true;
        error.value = "Upload cancelled";
        uploadProgress.value.forEach(p => {
          if (p.status === "pending" || p.status === "uploading") {
            p.status = "failed";
            p.message = "Cancelled";
          }
        });
      } else {
        error.value = `Failed to upload videos: ${err}`;
      }
    } finally {
      isUploading.value = false;
    }
    return null;
  };

  const goBack = (): boolean => {
    if (isUploading.value || uploadProgress.value.size > 0) {
      modals.showCancelConfirmation("youtube_upload");
      return false;
    }
    return true;
  };

  const authenticate = async () => {
    await youtube.authenticate();
    if (youtube.isAuthenticated.value) {
      success.value = "Successfully authenticated with YouTube!";
      setTimeout(() => { success.value = null; }, 5000);
    }
  };

  const authenticateDiscogs = async () => {
    await discogs.authenticate();
    if (discogs.isAuthenticated.value) {
      success.value = "Successfully authenticated with Discogs!";
      setTimeout(() => { success.value = null; }, 5000);
    }
  };

  const cancelOperation = async (phaseId: string) => {
    if (phaseId !== "upload") return;
    try {
      await invoke("cancel_operation", { operationType: "youtube_upload" });
      resetUploadState();
    } catch (err) {
      error.value = `Failed to cancel upload: ${err}`;
      resetUploadState();
    }
  };

  const cancelItem = async (phaseId: string, itemId: string | number) => {
    if (phaseId !== "upload") return;
    try {
      await invoke("cancel_item", { operationType: "youtube_upload", itemId: String(itemId) });
      const existing = uploadProgress.value.get(Number(itemId));
      if (existing && (existing.status === "pending" || existing.status === "uploading")) {
        existing.status = "failed";
        existing.message = "Cancelled";
        existing.progress = 0;
      }
      const allItems = Array.from(uploadProgress.value.values());
      const allDoneOrCancelled =
        allItems.length > 0 && allItems.every(p => p.status === "completed" || p.status === "failed");
      if (allDoneOrCancelled) {
        setTimeout(() => resetUploadState(), 500);
      }
    } catch (err) {
      error.value = `Failed to cancel item: ${err}`;
    }
  };

  const clearSource = () => {
    videos.value = [];
    selectedVideoIndex.value = null;
    selectedFolder.value = null;
    uploadProgress.value.clear();
    clearVideoThumbnails();
    error.value = null;
    success.value = null;
  };

  const applyDescriptionTemplate = () => {
    if (!selectedVideo.value) return;
    applyToVideo(selectedVideo.value);
  };

  const handleApplyDescriptionToAll = () => {
    if (!selectedVideo.value || selectedVideoIndex.value === null) return;
    const currentDescription = selectedVideo.value.description;
    videos.value.forEach((video, index) => {
      if (index !== selectedVideoIndex.value) {
        videos.value[index] = { ...video, description: currentDescription };
      }
    });
    showApplyDescriptionSuccess.value = true;
    setTimeout(() => {
      showApplyDescriptionSuccess.value = false;
    }, 2000);
  };

  const handleApplyPrivacyToAll = () => {
    if (!selectedVideo.value || selectedVideoIndex.value === null) return;
    const { privacyStatus, publishAt } = selectedVideo.value;
    videos.value.forEach((video, index) => {
      if (index !== selectedVideoIndex.value) {
        videos.value[index] = { ...video, privacyStatus, publishAt: publishAt ?? null };
      }
    });
    showApplyPrivacySuccess.value = true;
    setTimeout(() => {
      showApplyPrivacySuccess.value = false;
    }, 2000);
  };

  const handleApplyCategoryToAll = () => {
    if (!selectedVideo.value || selectedVideoIndex.value === null) return;
    const categoryId = selectedVideo.value.categoryId;
    videos.value.forEach((video, index) => {
      if (index !== selectedVideoIndex.value) {
        videos.value[index] = { ...video, categoryId };
      }
    });
    showApplyCategorySuccess.value = true;
    setTimeout(() => {
      showApplyCategorySuccess.value = false;
    }, 2000);
  };

  const handleApplyTagsToAll = () => {
    if (!selectedVideo.value || selectedVideoIndex.value === null) return;
    const tags = [...selectedVideo.value.tags];
    videos.value.forEach((video, index) => {
      if (index !== selectedVideoIndex.value) {
        videos.value[index] = { ...video, tags };
      }
    });
    showApplyTagsSuccess.value = true;
    setTimeout(() => {
      showApplyTagsSuccess.value = false;
    }, 2000);
  };

  const handleDiscogsReleaseSelected = async (release: DiscogsRelease) => {
    if (videos.value.length > 1) {
      pendingRelease.value = release;
      selectedVideosForLinking.value = new Set();
      showLinkToOtherVideosModal.value = true;
    }
  };

  const applyReleaseToSelectedVideos = () => {
    if (!pendingRelease.value) return;
    selectedVideosForLinking.value.forEach(index => {
      if (index !== selectedVideoIndex.value && index < videos.value.length) {
        videos.value[index].discogsReleaseId = pendingRelease.value!.id;
        videos.value[index].discogsRelease = pendingRelease.value!;
        autofillMetadataFromRelease(pendingRelease.value!, videos.value[index] as Parameters<typeof autofillMetadataFromRelease>[1]);
      }
    });
    showLinkToOtherVideosModal.value = false;
    pendingRelease.value = null;
    selectedVideosForLinking.value = new Set();
  };

  const cancelLinkToOtherVideos = () => {
    showLinkToOtherVideosModal.value = false;
    pendingRelease.value = null;
    selectedVideosForLinking.value = new Set();
  };

  const dismissNoVideosModal = () => {
    noVideosFoundModalOpen.value = false;
    error.value = null;
  };

  const toggleVideoForLinking = (index: number) => {
    if (index === selectedVideoIndex.value) return;
    if (selectedVideosForLinking.value.has(index)) {
      selectedVideosForLinking.value.delete(index);
    } else {
      selectedVideosForLinking.value.add(index);
    }
  };

  onMounted(async () => {
    await loadPreferences();
    await youtube.checkAuthStatus();
    await discogs.checkAuthStatus();
    await youtube.loadPlaylists();
    await setupEventListeners();
  });

  onUnmounted(() => {
    unlistens.forEach(fn => fn());
    unlistens.length = 0;
  });

  return {
    videos,
    selectedVideoIndex,
    selectedFolder,
    isUploading,
    uploadProgress,
    error,
    success,
    showApplyDescriptionSuccess,
    showApplyPrivacySuccess,
    showApplyCategorySuccess,
    showApplyTagsSuccess,
    isLoadingVideos,
    isLoadingFolder,
    showLinkToOtherVideosModal,
    pendingRelease,
    selectedVideosForLinking,
    noVideosFoundModalOpen,
    forcePrivateUploads,
    rightPanelStartRef,
    progressSectionRef,
    selectedVideo,
    isAuthenticated,
    oauthError,
    isDiscogsAuthenticated,
    discogsError,
    canUpload,
    videoStatus,
    hasPreviousVideo,
    hasNextVideo,
    otherVideos,
    uploadPipeline,
    modals,
    selectVideos,
    selectFolder,
    selectVideo,
    removeVideo,
    updateVideo,
    uploadVideos,
    cancelOperation,
    cancelItem,
    clearSource,
    applyDescriptionTemplate,
    handleApplyDescriptionToAll,
    handleApplyPrivacyToAll,
    handleApplyCategoryToAll,
    handleApplyTagsToAll,
    handleDiscogsReleaseSelected,
    applyReleaseToSelectedVideos,
    cancelLinkToOtherVideos,
    dismissNoVideosModal,
    toggleVideoForLinking,
    getVideoThumbnail,
    goBack,
    authenticate,
    authenticateDiscogs
  };
}
