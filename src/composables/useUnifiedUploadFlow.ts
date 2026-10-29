import { ref, computed, watch, onMounted, onScopeDispose } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { withTimeout, FOLDER_LIST_TIMEOUT_MS, THUMBNAIL_TIMEOUT_MS } from "../lib/tauri-timeout";
import { listen } from "@tauri-apps/api/event";
import type {
  FolderContents,
  BulkVideoRequest,
  BulkVideoResult,
  BulkVideoUploadResult,
  ImageStyle,
  FitMode,
  ImageInput,
  AudioImagePair,
  DiscogsSearchResult,
  DiscogsRelease,
  DiscogsImage,
  AudioMetadata,
  UploadHistoryEntry
} from "../types";
import { DEFAULT_IMAGE_STYLE, DEFAULT_FIT_MODE } from "../types";
import { defaultPair, shouldUseAlbumArt } from "../lib/creation-helpers";
import { usePreferences } from "./usePreferences";
import { useYouTube } from "./useYouTube";
import { useDiscogs } from "./useDiscogs";
import { useDescriptionTemplate } from "./useDescriptionTemplate";
import { useUploadHistory } from "./useUploadHistory";
import { useVideoProcessingStatus } from "./useVideoProcessingStatus";
import { useNotifications } from "./useNotifications";
import { usePredefinedBackgrounds } from "./usePredefinedBackgrounds";
import { useOutputFolder } from "./useOutputFolder";
import { useUnifiedPipeline } from "./useUnifiedPipeline";
import { useVideoThumbnails } from "./useVideoThumbnails";

type PrivacyStatus = "public" | "private" | "unlisted" | "draft";

type UnifiedVideoMetadata = {
  audioPath: string;
  videoPath: string | null; // null until created
  title: string;
  description: string;
  tags: string[];
  categoryId: string;
  privacyStatus: PrivacyStatus;
  selfDeclaredMadeForKids: boolean;
  publishAt: string | null;
  playlistId: string | null;
  discogsReleaseId: number | null;
  discogsRelease: DiscogsRelease | null;
  isTempFile: boolean; // true if created in temp folder
  youtubeVideoId?: string; // set after successful upload
  youtubeUrl?: string; // set after successful upload
};

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

export function useUnifiedUploadFlow() {
  const { preferences, loadPreferences, updatePreferences } = usePreferences();
  const youtube = useYouTube();
  const discogs = useDiscogs();
  const { autofillMetadataFromRelease, applyToVideo } = useDescriptionTemplate();
  const uploadHistory = useUploadHistory();
  const {
    predefinedBackgrounds,
    isLoadingPredefinedBackgrounds,
    predefinedBackgroundsLoadError,
    userBackgrounds,
    isLoadingUserBackgrounds,
    userBackgroundsLoadError,
    loadPredefinedBackgrounds,
    loadUserBackgrounds,
    reloadUserBackgrounds
  } = usePredefinedBackgrounds();
  const { checkStatus } = useVideoProcessingStatus();
  const audioExportFolder = useOutputFolder({ 
    preferenceKey: "last_audio_export_output" 
  });
  const { notify } = useNotifications();
  const {
    generate: generateVideoThumbnail,
    get: getVideoThumbnail,
    has: hasVideoThumbnail
  } = useVideoThumbnails();

  // Step 1: Audio tracks with metadata (from creation flow)
  const audioTracks = ref<Array<{ path: string; name: string; metadata: AudioMetadata | null }>>([]);
  const selectedFolder = ref<string | null>(null);
  const folderContents = ref<FolderContents | null>(null);
  const selectedAudioIndex = ref<number | null>(null);
  const isLoadingFolder = ref(false);
  const isLoadingFiles = ref(false);

  // Step 2: Audio–image pairs (from creation flow)
  const audioImagePairs = ref<Map<string, AudioImagePair>>(new Map());

  // Step 3: Video metadata (from upload flow)
  const videoMetadata = ref<Map<string, UnifiedVideoMetadata>>(new Map());

  // Current image (right panel)
  const currentImagePath = ref<string | null>(null);
  const currentImageUrl = ref<string>("");
  const usePreviousImage = ref(false);
  const currentImageStyle = ref<ImageStyle>(DEFAULT_IMAGE_STYLE);
  const currentFitMode = ref<FitMode>(DEFAULT_FIT_MODE);
  const currentBackgroundPath = ref<string | null>(null);
  const currentBackgroundUrl = ref<string>("");
  const currentBackgroundPredefinedId = ref<string | null>(null);
  const previewImage = ref<string | null>(null);

  // Image editor state
  const isImageEditorOpen = ref(false);
  const editingImageSrc = ref<string>("");


  // Preview cache
  const imageThumbnails = ref<Map<string, string>>(new Map());
  const imagePreviews = ref<Map<string, string>>(new Map());
  const isGeneratingPreview = ref(false);

  // Discogs art (per-track, for image source)
  // Note: We use the discogs composable for auth, but keep local state for images
  const discogsImages = ref<DiscogsImage[]>([]);
  const isLoadingDiscogsImages = ref(false);
  const discogsError = ref<string | null>(null);
  const discogsSearchResults = ref<DiscogsSearchResult[]>([]);
  // Store the release selected in images section (per-track)
  const discogsReleaseForImage = ref<Map<string, DiscogsRelease>>(new Map());

  const noFilesFoundModalOpen = ref(false);

  // Step 4: Options
  // Note: uploadToYouTube is always true in this flow (it's the upload flow)
  const saveVideosLocally = ref(false);
  const uploadToYouTube = ref(true); // Always true - this is the upload flow
  const outputFolder = ref<string | null>(null);

  // Step 5: Processing
  const currentStep = ref<"source" | "images" | "metadata" | "options" | "processing">("source");
  const isProcessing = ref(false);
  const isCreatingVideos = ref(false);
  const isUploading = ref(false);
  const videoProgresses = ref<Map<number, VideoProgress>>(new Map());
  const uploadProgresses = ref<Map<number, UploadProgress>>(new Map());
  const itemDisplayProgress = ref<Map<number, number>>(new Map());
  const error = ref<string | null>(null);
  const success = ref<string | null>(null);
  const bulkResults = ref<BulkVideoResult | null>(null);
  const normalizeAudio = ref(false);
  const highPassFilter = ref(false);
  const isNormalizing = ref(false);
  const normalizationProgress = ref({ total: 0, completed: 0, message: "" });
  const exportNormalizedAudio = ref(false);
  const isCancelled = ref(false);
  const tempVideoPaths = ref<string[]>([]); // Track temp files for cleanup

  const selectedAudio = computed(() => {
    if (selectedAudioIndex.value === null) return null;
    return audioTracks.value[selectedAudioIndex.value] || null;
  });

  const selectedVideoMetadata = computed(() => {
    if (selectedAudioIndex.value === null) return null;
    const track = audioTracks.value[selectedAudioIndex.value];
    if (!track) return null;
    return videoMetadata.value.get(track.path) || null;
  });

  const favoriteBackgroundId = computed(() => {
    return preferences.value.preferences?.default_background_predefined_id ?? null;
  });

  const setFavoriteBackground = async (id: string | null) => {
    await updatePreferences({ default_background_predefined_id: id || null });
  };

  const canProceedToImages = computed(() => {
    return audioTracks.value.length > 0;
  });

  const canProceedToMetadata = computed(() => {
    if (audioTracks.value.length === 0) return false;
    return audioTracks.value.every(track => {
      const pair = audioImagePairs.value.get(track.path);
      if (!pair) return false;
      const style = pair.imageStyle ?? DEFAULT_IMAGE_STYLE;
      const hasMain = !!(pair.imagePath || pair.imageUrl || pair.usePrevious);
      const hasBackground =
        !!pair.backgroundPath || !!pair.backgroundUrl || !!pair.backgroundPredefinedId;
      if (style === "background-only") {
        return hasBackground;
      }
      return hasMain;
    });
  });

  const canProceedToOptions = computed(() => {
    if (audioTracks.value.length === 0) return false;
    // Check that all videos have titles
    return audioTracks.value.every(track => {
      const metadata = videoMetadata.value.get(track.path);
      return metadata && metadata.title.trim().length > 0;
    });
  });

  const canCreateAndUpload = computed(() => {
    if (audioTracks.value.length === 0) return false;
    // Upload is always enabled in this flow
    if (!youtube.isAuthenticated.value) return false;
    // If saving locally, output folder is required
    if (saveVideosLocally.value && !outputFolder.value) return false;
    // If exporting audio, audio export folder is required
    if (exportNormalizedAudio.value && normalizeAudio.value && !audioExportFolder.outputFolder.value) return false;
    if (isProcessing.value) return false;
    return true;
  });

  // Initialize video metadata from audio tracks
  const initializeVideoMetadata = () => {
    audioTracks.value.forEach(track => {
      if (!videoMetadata.value.has(track.path)) {
        const prefs = preferences.value.preferences;
        const metadata = track.metadata;
        
        // Auto-fill title from audio metadata
        let title = "";
        if (metadata) {
          if (metadata.artist && metadata.title) {
            title = `${metadata.artist} - ${metadata.title}`;
          } else if (metadata.title) {
            title = metadata.title;
          } else if (metadata.artist) {
            title = metadata.artist;
          }
        }
        if (!title) {
          title = track.name.replace(/\.[^/.]+$/, "") || "Untitled";
        }

        const videoMeta: UnifiedVideoMetadata = {
          audioPath: track.path,
          videoPath: null,
          title,
          description: "",
          tags: [],
          categoryId: prefs?.default_category_id ?? "10", // Music
          privacyStatus: (prefs?.default_privacy_status ?? "public") as PrivacyStatus,
          selfDeclaredMadeForKids: false,
          publishAt: null,
          playlistId: (prefs?.default_playlist_id ?? null) as string | null,
          discogsReleaseId: null,
          discogsRelease: null,
          isTempFile: false
        };

        // Auto-fill description from template if available
        // Note: We don't auto-fill from template here since there's no Discogs release linked yet
        // Template will be applied when user links a Discogs release or manually applies template

        videoMetadata.value.set(track.path, videoMeta);
      }
    });
  };

  // Watch audio tracks and initialize metadata
  watch(audioTracks, () => {
    initializeVideoMetadata();
    // Auto-advance to images step when tracks are selected
    if (audioTracks.value.length > 0 && currentStep.value === "source") {
      currentStep.value = "images";
    }
  }, { deep: true });

  // When entering metadata step, check if any tracks have Discogs releases from images section
  watch(currentStep, () => {
    if (currentStep.value === "metadata") {
      // Apply Discogs releases that were linked in images section to metadata
      audioTracks.value.forEach(track => {
        const releaseFromImages = discogsReleaseForImage.value.get(track.path);
        if (releaseFromImages) {
          const metadata = videoMetadata.value.get(track.path);
          if (metadata && (!metadata.discogsRelease || metadata.discogsRelease.id !== releaseFromImages.id)) {
            // Link the release and auto-fill if not already linked
            metadata.discogsReleaseId = releaseFromImages.id;
            metadata.discogsRelease = releaseFromImages;
            autofillMetadataFromRelease(releaseFromImages, metadata as any);
            videoMetadata.value.set(track.path, metadata);
          }
        }
      });
    }
  });

  // Auto-advance when conditions are met
  watch(canProceedToMetadata, (canProceed) => {
    if (canProceed && currentStep.value === "images" && audioTracks.value.length > 0) {
      // Don't auto-advance, let user click continue
    }
  });

  watch(canProceedToOptions, (canProceed) => {
    if (canProceed && currentStep.value === "metadata" && audioTracks.value.length > 0) {
      // Don't auto-advance, let user click continue
    }
  });

  // Load folder contents (reuse from creation flow)
  const loadFolderContents = async (folderPath: string) => {
    try {
      isLoadingFolder.value = true;
      const contents = await withTimeout(
        invoke<FolderContents>("list_folder_contents", { folderPath }),
        FOLDER_LIST_TIMEOUT_MS,
        "Loading folder timed out"
      );
      folderContents.value = contents;
      if (contents.audios.length === 0) {
        error.value = "No audio files found in the selected folder.";
        noFilesFoundModalOpen.value = true;
        return;
      }
      
      const MAX_FILES_PER_OPERATION = 100;
      const totalAudios = audioTracks.value.length + contents.audios.length;
      if (totalAudios > MAX_FILES_PER_OPERATION) {
        error.value = `Cannot select more than ${MAX_FILES_PER_OPERATION} audio files. You already have ${audioTracks.value.length} file(s) selected, and this folder contains ${contents.audios.length} file(s).`;
        return;
      }
      
      const tracksWithMetadata = await Promise.all(
        contents.audios.map(async (audio) => {
          let metadata: AudioMetadata | null = null;
          try {
            metadata = await invoke<AudioMetadata>("get_audio_metadata", {
              audioPath: audio.path
            });
          } catch (err) {
            console.warn(`Failed to extract metadata for ${audio.path}:`, err);
          }
          return {
            path: audio.path,
            name: audio.name,
            metadata
          };
        })
      );
      audioTracks.value = tracksWithMetadata;
      audioTracks.value.forEach(track => {
        if (!audioImagePairs.value.has(track.path)) {
          const pair: AudioImagePair = shouldUseAlbumArt(track.metadata)
            ? {
                audioPath: track.path,
                imagePath: null,
                imageUrl: track.metadata!.image_data_url!,
                usePrevious: false,
                imageStyle: preferences.value.preferences?.default_image_style ?? DEFAULT_IMAGE_STYLE,
                fitMode: preferences.value.preferences?.default_fit_mode ?? DEFAULT_FIT_MODE,
                backgroundPath: null,
                backgroundUrl: null,
                backgroundPredefinedId: preferences.value.preferences?.default_background_predefined_id ?? null
              }
            : defaultPair(track.path, preferences.value.preferences);
          audioImagePairs.value.set(track.path, pair);
        }
      });
      if (selectedAudioIndex.value === null && audioTracks.value.length > 0) {
        selectAudioTrack(0);
      }
      if (contents.images.length > 0) {
        for (const img of contents.images) {
          if (!imageThumbnails.value.has(img.path)) {
            try {
              const thumbnail = await withTimeout(
                invoke<string>("generate_image_thumbnail", {
                  imagePath: img.path,
                  size: 384
                }),
                THUMBNAIL_TIMEOUT_MS,
                "Loading image thumbnail timed out"
              );
              imageThumbnails.value.set(img.path, thumbnail);
            } catch (err) {
              console.error(`Failed to generate thumbnail for ${img.path}:`, err);
            }
          }
        }
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
        multiple: false
      });
      if (path && typeof path === "string") {
        selectedFolder.value = path;
        await loadFolderContents(path);
        updatePreferences({ last_creation_output: path });
      }
    } catch (err) {
      error.value = `Failed to select folder: ${err}`;
    } finally {
      isLoadingFolder.value = false;
    }
  };

  const MAX_FILES_PER_OPERATION = 100;

  const selectAudioFiles = async () => {
    try {
      isLoadingFiles.value = true;
      error.value = null;
      const paths = await open({
        multiple: true,
        filters: [{
          name: "Audio",
          extensions: ["mp3", "wav", "flac", "aac", "m4a", "ogg"]
        }]
      });
      if (paths && Array.isArray(paths)) {
        const newTracks = await Promise.all(
          (paths as string[]).map(async (p) => {
            let metadata: AudioMetadata | null = null;
            try {
              metadata = await invoke<AudioMetadata>("get_audio_metadata", {
                audioPath: p
              });
            } catch (err) {
              console.warn(`Failed to extract metadata for ${p}:`, err);
            }
            return {
              path: p,
              name: p.split("/").pop() || p,
              metadata
            };
          })
        );
        
        const totalAudios = audioTracks.value.length + newTracks.length;
        if (totalAudios > MAX_FILES_PER_OPERATION) {
          error.value = `Cannot select more than ${MAX_FILES_PER_OPERATION} audio files. You already have ${audioTracks.value.length} file(s) selected, and you're trying to add ${newTracks.length} more.`;
          return;
        }
        
        audioTracks.value = [...audioTracks.value, ...newTracks];
        newTracks.forEach(track => {
          if (!audioImagePairs.value.has(track.path)) {
            const pair: AudioImagePair = shouldUseAlbumArt(track.metadata)
              ? {
                  audioPath: track.path,
                  imagePath: null,
                  imageUrl: track.metadata!.image_data_url!,
                  usePrevious: false,
                  imageStyle: preferences.value.preferences?.default_image_style ?? DEFAULT_IMAGE_STYLE,
                  fitMode: preferences.value.preferences?.default_fit_mode ?? DEFAULT_FIT_MODE,
                  backgroundPath: null,
                  backgroundUrl: null,
                  backgroundPredefinedId: preferences.value.preferences?.default_background_predefined_id ?? null
                }
              : defaultPair(track.path, preferences.value.preferences);
            audioImagePairs.value.set(track.path, pair);
          }
        });
        if (newTracks.length > 0) {
          selectAudioTrack(audioTracks.value.length - newTracks.length);
        }
      }
    } catch (err) {
      error.value = `Failed to select audio files: ${err}`;
    } finally {
      isLoadingFiles.value = false;
    }
  };

  const selectAudioTrack = (index: number) => {
    selectedAudioIndex.value = index;
    const track = audioTracks.value[index];
    if (track) {
      let pair = audioImagePairs.value.get(track.path);
      
      if (!pair || (!pair.imagePath && !pair.imageUrl && !pair.usePrevious)) {
        if (shouldUseAlbumArt(track.metadata)) {
          const newPair: AudioImagePair = {
            audioPath: track.path,
            imagePath: null,
            imageUrl: track.metadata!.image_data_url!,
            usePrevious: false,
            imageStyle: pair?.imageStyle ?? DEFAULT_IMAGE_STYLE,
            fitMode: pair?.fitMode ?? DEFAULT_FIT_MODE,
            backgroundPath: pair?.backgroundPath ?? null,
            backgroundUrl: pair?.backgroundUrl ?? null,
            backgroundPredefinedId: pair?.backgroundPredefinedId ?? null
          };
          audioImagePairs.value.set(track.path, newPair);
          pair = newPair;
        } else if (!pair) {
          const defaultPairValue = defaultPair(track.path, preferences.value.preferences);
          audioImagePairs.value.set(track.path, defaultPairValue);
          pair = defaultPairValue;
        }
      }
      
      if (pair) {
        currentImagePath.value = pair.imagePath;
        currentImageUrl.value = pair.imageUrl || "";
        usePreviousImage.value = pair.usePrevious;
        const imageStyle = pair.imageStyle ?? DEFAULT_IMAGE_STYLE;
        currentImageStyle.value = imageStyle;
        currentFitMode.value = pair.fitMode ?? DEFAULT_FIT_MODE;
        currentBackgroundPath.value = pair.backgroundPath ?? null;
        currentBackgroundUrl.value = pair.backgroundUrl ?? "";
        currentBackgroundPredefinedId.value = pair.backgroundPredefinedId ?? null;
        
        // Load backgrounds when needed
        if (imageStyle === "custom-background" || imageStyle === "background-only") {
          loadPredefinedBackgrounds();
          loadUserBackgrounds();
        }
        
        const isBackgroundOnly = imageStyle === "background-only";
        let main = pair.imagePath || pair.imageUrl || null;
        
        if (isBackgroundOnly && !main) {
          if (pair.backgroundPath) {
            main = pair.backgroundPath;
          } else if (pair.backgroundUrl) {
            main = pair.backgroundUrl;
          } else if (pair.backgroundPredefinedId) {
            const bg = predefinedBackgrounds.value.find((b) => b.id === pair.backgroundPredefinedId);
            if (bg) {
              main = bg.imagePath;
            }
          }
        }
        
        if (main) {
          const imageKey = previewCacheKey(
            main,
            pair.imageStyle ?? DEFAULT_IMAGE_STYLE,
            pair.fitMode ?? DEFAULT_FIT_MODE,
            pair.backgroundPath ?? null,
            pair.backgroundUrl ?? null,
            pair.backgroundPredefinedId ?? null
          );
          if (imagePreviews.value.has(imageKey)) {
            previewImage.value = imagePreviews.value.get(imageKey)!;
          } else {
            previewImage.value = null;
            generatePreviewForPair(pair);
          }
        } else {
          previewImage.value = null;
        }
      }
    }
  };

  const previewCacheKey = (
    main: string,
    style: ImageStyle,
    fit: FitMode,
    bgPath: string | null,
    bgUrl: string | null,
    bgPredefId: string | null
  ): string => [main, style, fit, bgPath ?? "", bgUrl ?? "", bgPredefId ?? ""].join("|");

  const saveCurrentPair = () => {
    if (selectedAudio.value) {
      const pair: AudioImagePair = {
        audioPath: selectedAudio.value.path,
        imagePath: currentImagePath.value,
        imageUrl: currentImageUrl.value || null,
        usePrevious: usePreviousImage.value,
        imageStyle: currentImageStyle.value,
        fitMode: currentFitMode.value,
        backgroundPath: currentBackgroundPath.value,
        backgroundUrl: currentBackgroundUrl.value || null,
        backgroundPredefinedId: currentBackgroundPredefinedId.value
      };
      audioImagePairs.value.set(selectedAudio.value.path, pair);
    }
  };

  const generatePreview = async () => {
    const isBackgroundOnly = currentImageStyle.value === "background-only";
    let main = currentImagePath.value || currentImageUrl.value || null;

    if (isBackgroundOnly && !main) {
      if (currentBackgroundPath.value) {
        main = currentBackgroundPath.value;
      } else if (currentBackgroundUrl.value) {
        main = currentBackgroundUrl.value;
      } else if (currentBackgroundPredefinedId.value) {
        // Check both predefined and user backgrounds
        const bg = predefinedBackgrounds.value.find(
          (b) => b.id === currentBackgroundPredefinedId.value
        ) || userBackgrounds.value.find(
          (b) => b.id === currentBackgroundPredefinedId.value
        );
        if (bg) {
          main = bg.imagePath;
        }
      }
    }

    if (!main) {
      previewImage.value = null;
      isGeneratingPreview.value = false;
      return;
    }
    const hasBackground =
      !!currentBackgroundPath.value ||
      !!currentBackgroundUrl.value ||
      !!currentBackgroundPredefinedId.value;
    if (isBackgroundOnly && !hasBackground) {
      previewImage.value = null;
      isGeneratingPreview.value = false;
      return;
    }
    const effectiveStyle: ImageStyle =
      currentImageStyle.value === "custom-background" && !hasBackground
        ? DEFAULT_IMAGE_STYLE
        : currentImageStyle.value;
    const key = previewCacheKey(
      main,
      effectiveStyle,
      currentFitMode.value,
      currentBackgroundPath.value,
      currentBackgroundUrl.value || null,
      currentBackgroundPredefinedId.value
    );
    if (imagePreviews.value.has(key)) {
      previewImage.value = imagePreviews.value.get(key)!;
      isGeneratingPreview.value = false;
      return;
    }
    try {
      isGeneratingPreview.value = true;
      const previewDataUrl = await invoke<string>("generate_image_preview", {
        args: {
          imagePath: main,
          width: 1920,
          height: 1080,
          imageStyle: effectiveStyle,
          fitMode: currentFitMode.value,
          backgroundPath: currentBackgroundPath.value ?? undefined,
          backgroundUrl: currentBackgroundUrl.value || undefined,
          backgroundPredefinedId: currentBackgroundPredefinedId.value ?? undefined
        }
      });
      previewImage.value = previewDataUrl;
      imagePreviews.value = new Map(imagePreviews.value).set(key, previewDataUrl);
    } catch (err) {
      console.error("Failed to generate preview:", err);
      previewImage.value = null;
    } finally {
      isGeneratingPreview.value = false;
    }
  };

  const generatePreviewForPair = async (pair: AudioImagePair) => {
    const isBackgroundOnly = (pair.imageStyle ?? DEFAULT_IMAGE_STYLE) === "background-only";
    let main = pair.imagePath || pair.imageUrl || null;

    if (isBackgroundOnly && !main) {
      if (pair.backgroundPath) {
        main = pair.backgroundPath;
      } else if (pair.backgroundUrl) {
        main = pair.backgroundUrl;
      } else if (pair.backgroundPredefinedId) {
        // Check both predefined and user backgrounds
        const bg = predefinedBackgrounds.value.find((b) => b.id === pair.backgroundPredefinedId)
          || userBackgrounds.value.find((b) => b.id === pair.backgroundPredefinedId);
        if (bg) {
          main = bg.imagePath;
        }
      }
    }

    if (!main) {
      previewImage.value = null;
      isGeneratingPreview.value = false;
      return;
    }
    const hasBackground =
      !!pair.backgroundPath ||
      !!pair.backgroundUrl ||
      !!pair.backgroundPredefinedId;
    if (isBackgroundOnly && !hasBackground) {
      previewImage.value = null;
      isGeneratingPreview.value = false;
      return;
    }
    const style: ImageStyle =
      (pair.imageStyle ?? DEFAULT_IMAGE_STYLE) === "custom-background" && !hasBackground
        ? DEFAULT_IMAGE_STYLE
        : (pair.imageStyle ?? DEFAULT_IMAGE_STYLE);
    const fit = pair.fitMode ?? DEFAULT_FIT_MODE;
    const bgPath = pair.backgroundPath ?? null;
    const bgUrl = pair.backgroundUrl ?? null;
    const bgPredef = pair.backgroundPredefinedId ?? null;
    const key = previewCacheKey(main, style, fit, bgPath, bgUrl, bgPredef);
    if (imagePreviews.value.has(key)) {
      previewImage.value = imagePreviews.value.get(key)!;
      isGeneratingPreview.value = false;
      return;
    }
    try {
      isGeneratingPreview.value = true;
      const previewDataUrl = await invoke<string>("generate_image_preview", {
        args: {
          imagePath: main,
          width: 1920,
          height: 1080,
          imageStyle: style,
          fitMode: fit,
          backgroundPath: bgPath ?? undefined,
          backgroundUrl: bgUrl ?? undefined,
          backgroundPredefinedId: bgPredef ?? undefined
        }
      });
      previewImage.value = previewDataUrl;
      imagePreviews.value = new Map(imagePreviews.value).set(key, previewDataUrl);
    } catch (err) {
      console.error("Failed to generate preview:", err);
      previewImage.value = null;
    } finally {
      isGeneratingPreview.value = false;
    }
  };

  const selectImageFile = async () => {
    try {
      error.value = null;
      const path = await open({
        multiple: false,
        filters: [{
          name: "Images",
          extensions: ["png", "jpg", "jpeg", "gif", "bmp", "webp"]
        }]
      });
      if (path && typeof path === "string" && selectedAudio.value) {
        currentImagePath.value = path;
        currentImageUrl.value = "";
        usePreviousImage.value = false;
        previewImage.value = null;
        
        await generatePreview();
        saveCurrentPair();
      }
    } catch (err) {
      error.value = `Failed to select image: ${err}`;
      isGeneratingPreview.value = false;
    }
  };

  const selectImageFromFolder = async (imagePath: string) => {
    if (selectedAudio.value) {
      currentImagePath.value = imagePath;
      currentImageUrl.value = "";
      usePreviousImage.value = false;
      previewImage.value = null;
      await generatePreview();
      saveCurrentPair();
    }
  };

  /** Apply main image from URL (blur on URL tab). Clears file path so URL is used, then generates preview and saves pair. */
  const applyImageFromUrl = () => {
    currentImagePath.value = null;
    usePreviousImage.value = false;
    const url = currentImageUrl.value?.trim() || "";
    if (url) {
      previewImage.value = null;
      isGeneratingPreview.value = true; // Show loading state immediately while backend fetches/processes
      generatePreview();
    } else {
      previewImage.value = null;
    }
    saveCurrentPair();
  };

  // Image editor functions
  const getCurrentImageSrc = (): string | null => {
    if (currentImageUrl.value) return currentImageUrl.value;
    if (currentImagePath.value) return currentImagePath.value;
    return null;
  };

  const hasSelectedImage = computed(() => {
    return !!(currentImagePath.value || currentImageUrl.value);
  });

  const openImageEditor = () => {
    const imageSrc = getCurrentImageSrc();
    if (!imageSrc) return;
    
    editingImageSrc.value = imageSrc;
    isImageEditorOpen.value = true;
  };

  const closeImageEditor = () => {
    isImageEditorOpen.value = false;
    editingImageSrc.value = "";
  };

  const handleEditedImage = async (editedDataUrl: string) => {
    // Replace current image with edited version
    currentImagePath.value = null; // Clear file path
    currentImageUrl.value = editedDataUrl; // Set data URL
    usePreviousImage.value = false;
    previewImage.value = null;
    
    // Close editor
    closeImageEditor();
    
    // Trigger preview regeneration
    await generatePreview();
    saveCurrentPair();
  };


  const toggleUsePrevious = () => {
    if (usePreviousImage.value) {
      if (selectedAudioIndex.value !== null && selectedAudioIndex.value > 0) {
        let effectivePair: AudioImagePair | null = null;
        for (let i = selectedAudioIndex.value - 1; i >= 0; i--) {
          const track = audioTracks.value[i];
          const pair = audioImagePairs.value.get(track.path);
          if (pair && (pair.imagePath || pair.imageUrl)) {
            effectivePair = pair;
            break;
          }
        }
        if (effectivePair) {
          const main = effectivePair.imagePath || effectivePair.imageUrl!;
          currentImagePath.value = main.startsWith("http") ? null : main;
          currentImageUrl.value = main.startsWith("http") ? main : "";
          currentImageStyle.value = effectivePair.imageStyle ?? DEFAULT_IMAGE_STYLE;
          currentFitMode.value = effectivePair.fitMode ?? DEFAULT_FIT_MODE;
          currentBackgroundPath.value = effectivePair.backgroundPath ?? null;
          currentBackgroundUrl.value = effectivePair.backgroundUrl ?? "";
          currentBackgroundPredefinedId.value = effectivePair.backgroundPredefinedId ?? null;
          previewImage.value = null;
          generatePreview();
        }
      }
    } else {
      currentImagePath.value = null;
      currentImageUrl.value = "";
      currentImageStyle.value = DEFAULT_IMAGE_STYLE;
      currentFitMode.value = DEFAULT_FIT_MODE;
      currentBackgroundPath.value = null;
      currentBackgroundUrl.value = "";
      currentBackgroundPredefinedId.value = null;
      previewImage.value = null;
    }
    saveCurrentPair();
  };

  const previousGeneratedImages = computed(() => {
    const idx = selectedAudioIndex.value;
    if (idx === null || audioTracks.value.length === 0) return [];
    const currentMain = (currentImagePath.value ?? currentImageUrl.value) || "";
    const seen = new Set<string>(currentMain ? [currentMain] : []);
    const list: Array<{ trackName: string; pair: AudioImagePair; previewDataUrl: string | null }> = [];
    for (let i = 0; i < audioTracks.value.length; i++) {
      if (i === idx) continue;
      const track = audioTracks.value[i];
      const pair = audioImagePairs.value.get(track.path);
      if (!pair || (!pair.imagePath && !pair.imageUrl)) continue;
      const main = pair.imagePath ?? pair.imageUrl ?? "";
      if (seen.has(main)) continue;
      seen.add(main);
      const key = previewCacheKey(
        main,
        pair.imageStyle ?? DEFAULT_IMAGE_STYLE,
        pair.fitMode ?? DEFAULT_FIT_MODE,
        pair.backgroundPath ?? null,
        pair.backgroundUrl ?? null,
        pair.backgroundPredefinedId ?? null
      );
      list.push({
        trackName: track.name,
        pair,
        previewDataUrl: imagePreviews.value.get(key) ?? null
      });
    }
    return list;
  });

  const selectPreviousGeneratedImage = (pair: AudioImagePair) => {
    if (!selectedAudio.value) return;
    const main = pair.imagePath ?? pair.imageUrl ?? null;
    if (!main) return;
    const isUrl = main.startsWith("http");
    currentImagePath.value = isUrl ? null : main;
    currentImageUrl.value = isUrl ? main : "";
    usePreviousImage.value = false;
    currentImageStyle.value = pair.imageStyle ?? DEFAULT_IMAGE_STYLE;
    currentFitMode.value = pair.fitMode ?? DEFAULT_FIT_MODE;
    currentBackgroundPath.value = pair.backgroundPath ?? null;
    currentBackgroundUrl.value = pair.backgroundUrl ?? "";
    currentBackgroundPredefinedId.value = pair.backgroundPredefinedId ?? null;
    previewImage.value = null;
    generatePreview();
    saveCurrentPair();
  };

  const applyCurrentImageToAllTracks = () => {
    if (!selectedAudio.value || audioTracks.value.length <= 1) return;
    
    // For background-only style, we need at least a background
    if (currentImageStyle.value === 'background-only') {
      const hasBackground = !!(currentBackgroundPath.value || currentBackgroundUrl.value || currentBackgroundPredefinedId.value);
      if (!hasBackground) return;
    } else {
      // For other styles, we need a main image
      const main = currentImagePath.value || currentImageUrl.value;
      if (!main) return;
    }
    
    const main = currentImagePath.value || currentImageUrl.value;
    const isUrl = main?.startsWith("http") ?? false;
    
    audioTracks.value.forEach(track => {
      const pair: AudioImagePair = {
        audioPath: track.path,
        imagePath: isUrl ? null : (main || null),
        imageUrl: isUrl ? (main || null) : null,
        usePrevious: false,
        imageStyle: currentImageStyle.value,
        fitMode: currentFitMode.value,
        backgroundPath: currentBackgroundPath.value,
        backgroundUrl: currentBackgroundUrl.value || null,
        backgroundPredefinedId: currentBackgroundPredefinedId.value
      };
      audioImagePairs.value.set(track.path, pair);
    });
  };


  const importBackground = async () => {
    try {
      isLoadingUserBackgrounds.value = true;
      userBackgroundsLoadError.value = null;
      const path = await open({
        multiple: false,
        filters: [{
          name: "Images",
          extensions: ["png", "jpg", "jpeg", "gif", "bmp", "webp"]
        }]
      });
      if (path && typeof path === "string") {
        await invoke<string>("save_predefined_background", { imagePath: path });
        // Reload user backgrounds to show the new one
        await reloadUserBackgrounds();
      }
    } catch (err) {
      console.error("Failed to import background:", err);
      userBackgroundsLoadError.value = err instanceof Error ? err.message : "Could not import background.";
    } finally {
      isLoadingUserBackgrounds.value = false;
    }
  };

  const selectBackgroundFile = async () => {
    try {
      const path = await open({
        multiple: false,
        filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "gif", "bmp", "webp"] }]
      });
      if (path && typeof path === "string") {
        currentBackgroundPath.value = path;
        currentBackgroundUrl.value = "";
        currentBackgroundPredefinedId.value = null;
        if (currentImagePath.value || currentImageUrl.value) {
          previewImage.value = null;
          await generatePreview();
        }
        saveCurrentPair();
      }
    } catch (err) {
      error.value = `Failed to select background: ${err}`;
    }
  };

  const setBackgroundFromUrl = () => {
    currentBackgroundPath.value = null;
    currentBackgroundPredefinedId.value = null;
    if (currentImagePath.value || currentImageUrl.value) {
      previewImage.value = null;
      generatePreview();
    }
    saveCurrentPair();
  };

  const setBackgroundPredefined = (id: string) => {
    currentBackgroundPredefinedId.value = id;
    currentBackgroundPath.value = null;
    currentBackgroundUrl.value = "";
    previewImage.value = null;
    generatePreview();
    saveCurrentPair();
  };

  // Discogs functions (reuse from creation flow)
  // Note: We use the discogs composable's checkAuthStatus instead

  const authenticateDiscogsForImages = async () => {
    await discogs.authenticate();
  };

  const searchDiscogsReleasesForArt = async () => {
    if (!discogs.searchQuery.value.trim() || !discogs.isAuthenticated.value) return;
    await discogs.search(discogs.searchQuery.value);
  };

  let discogsSearchTimeout: ReturnType<typeof setTimeout> | null = null;

  const setDiscogsSearchQuery = (value: string) => {
    discogs.searchQuery.value = value;
    discogsImages.value = [];
    if (discogsSearchTimeout) {
      clearTimeout(discogsSearchTimeout);
    }
    if (!value.trim() || value.trim().length < 3 || !discogs.isAuthenticated.value) {
      discogs.clearSearch();
      return;
    }
    discogsSearchTimeout = setTimeout(() => {
      searchDiscogsReleasesForArt();
    }, 500);
  };

  const selectDiscogsReleaseForImage = async (result: DiscogsSearchResult) => {
    if (!discogs.isAuthenticated.value || !selectedAudio.value) return;
    try {
      isLoadingDiscogsImages.value = true;
      discogsError.value = null;
      const release = await invoke<DiscogsRelease>("get_discogs_release", {
        releaseId: result.id
      });
      const images = release.images ?? [];
      discogsImages.value = images.filter(img => !!img.uri);
      
      // Store the release for this track so we can use it in metadata section
      discogsReleaseForImage.value.set(selectedAudio.value.path, release);
      
      // Link the release to metadata and auto-fill description + tags
      const metadata = videoMetadata.value.get(selectedAudio.value.path);
      if (metadata) {
        metadata.discogsReleaseId = release.id;
        metadata.discogsRelease = release;
        autofillMetadataFromRelease(release, metadata as any);
        videoMetadata.value.set(selectedAudio.value.path, metadata);
      }
      
      // Clear search results but keep query so user knows what was searched
      discogs.searchResults.value = [];
    } catch (err) {
      discogsError.value = `Failed to load Discogs release: ${err}`;
      discogsImages.value = [];
    } finally {
      isLoadingDiscogsImages.value = false;
    }
  };

  const selectDiscogsImageForCurrentTrack = (url: string) => {
    if (!selectedAudio.value) return;
    currentImagePath.value = null;
    currentImageUrl.value = url;
    usePreviousImage.value = false;
    previewImage.value = null;
    saveCurrentPair(); // Save the pair so status updates correctly
    generatePreview();
  };

  // Video metadata functions
  const updateVideoMetadata = (field: keyof UnifiedVideoMetadata, value: any) => {
    if (selectedAudio.value) {
      const metadata = videoMetadata.value.get(selectedAudio.value.path);
      if (metadata) {
        videoMetadata.value.set(selectedAudio.value.path, {
          ...metadata,
          [field]: value
        });
      }
    }
  };

  const selectDiscogsReleaseForMetadata = async (result: DiscogsSearchResult) => {
    if (!selectedAudio.value) return;
    
    const release = await discogs.selectRelease(result);
    if (!release) return;
    
    const metadata = videoMetadata.value.get(selectedAudio.value.path);
    if (metadata) {
      metadata.discogsReleaseId = release.id;
      metadata.discogsRelease = release;
      autofillMetadataFromRelease(release, metadata as any);
      videoMetadata.value.set(selectedAudio.value.path, metadata);
    }
  };

  const applyDescriptionTemplate = () => {
    if (!selectedAudio.value) return;
    const metadata = videoMetadata.value.get(selectedAudio.value.path);
    if (metadata) {
      // Only apply template if there's a Discogs release linked
      // The template will only include discogsUrl if release exists
      applyToVideo(metadata as any);
      videoMetadata.value.set(selectedAudio.value.path, metadata);
    }
  };

  // Queue for videos ready to upload
  const uploadQueue = ref<Array<{ trackIndex: number; videoPath: string; metadata: UnifiedVideoMetadata }>>([]);
  const isUploadProcessorRunning = ref(false);
  /** Track index -> video path for pipeline thumbnails (set when video is created and queued for upload). */
  const uploadIndexToVideoPath = ref<Map<number, string>>(new Map());

  // Process upload queue concurrently
  const processUploadQueue = async () => {
    if (isUploadProcessorRunning.value || !uploadToYouTube.value || !youtube.isAuthenticated.value) {
      return;
    }

    isUploadProcessorRunning.value = true;
    isUploading.value = true;

    try {
      while (uploadQueue.value.length > 0) {
        // Take up to 3 videos from queue for concurrent upload
        const batch = uploadQueue.value.splice(0, 3);
        if (batch.length === 0) break;

        // Initialize upload progress for this batch
        batch.forEach((item) => {
          uploadProgresses.value.set(item.trackIndex, {
            index: item.trackIndex,
            progress: 0,
            message: "Starting upload...",
            status: "pending"
          });
        });

        // Create upload request for this batch
        const uploadRequest = {
          videos: batch.map((item) => {
            const v = item.metadata;
            const isDraft = v.privacyStatus === "draft";
            const privacy = isDraft ? "private" : v.privacyStatus;
            const publishAt = !isDraft && v.privacyStatus === "private" && v.publishAt
              ? (v.publishAt.includes("Z") ? v.publishAt : new Date(v.publishAt).toISOString())
              : null;
            
            return {
              video_path: item.videoPath,
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

        // Store upload index mapping for progress tracking
        batch.forEach((item, uploadIdx) => {
          uploadIndexMap.value.set(uploadIdx, item.trackIndex);
        });

        // Upload this batch
        try {
          const uploadResult = await invoke<BulkVideoUploadResult>("upload_bulk_videos_to_youtube", {
            request: uploadRequest
          });
          
          // Process upload results and save to history
          if (uploadResult.successful.length > 0) {
            try {
              const newEntries: UploadHistoryEntry[] = [];
              
              for (let i = 0; i < uploadResult.successful.length; i++) {
                const result = uploadResult.successful[i];
                const batchItem = batch[i]; // Use index mapping since results are in same order as batch
                
                if (batchItem) {
                  const metadata = batchItem.metadata;
                  const discogsRelease = metadata.discogsRelease;
                  const discogsReleaseUrl = discogsRelease 
                    ? (discogsRelease.uri || `https://www.discogs.com/release/${discogsRelease.id}`)
                    : null;
                  
                  let processingStatus: UploadHistoryEntry["processingStatus"] = undefined;
                  let processingStatusCheckedAt: string | undefined = undefined;
                  let processingFailureReason: string | null | undefined = undefined;
                  
                  try {
                    const statusResult = await checkStatus(result.video_id);
                    processingStatus = statusResult.status;
                    processingStatusCheckedAt = statusResult.checked_at;
                    processingFailureReason = statusResult.failure_reason || null;
                  } catch (err) {
                    console.warn(`Failed to check processing status for video ${result.video_id}:`, err);
                  }

                  // Update metadata with YouTube info
                  metadata.youtubeVideoId = result.video_id;
                  metadata.youtubeUrl = result.video_url;
                  
                  newEntries.push({
                    id: `${Date.now()}-${result.video_id}-${Math.random().toString(36).substr(2, 9)}`,
                    title: metadata.title || result.video_id,
                    youtubeVideoId: result.video_id,
                    youtubeUrl: result.video_url,
                    discogsReleaseId: discogsRelease?.id || null,
                    discogsReleaseUrl: discogsReleaseUrl,
                    discogsReleaseTitle: discogsRelease?.title || null,
                    uploadedAt: new Date().toISOString(),
                    processingStatus,
                    processingStatusCheckedAt,
                    processingFailureReason,
                  });
                }
              }
              
              if (newEntries.length > 0) {
                await uploadHistory.addMultiple(newEntries);
              }
            } catch (err) {
              console.error("Failed to save upload history:", err);
            }
          }

          uploadResult.failed.forEach((failure) => {
            const [videoPath, errorMessage] = failure;
            const trackIndex = batch.findIndex(item => item.videoPath === videoPath);
            if (trackIndex !== -1) {
              const originalIndex = batch[trackIndex].trackIndex;
              const progress = uploadProgresses.value.get(originalIndex);
              if (progress) {
                progress.status = "failed";
                progress.message = errorMessage;
              }
            }
          });
        } catch (uploadError) {
          console.error("Upload batch failed:", uploadError);
          // Mark all items in batch as failed
          batch.forEach((item) => {
            const progress = uploadProgresses.value.get(item.trackIndex);
            if (progress) {
              progress.status = "failed";
              progress.message = uploadError instanceof Error ? uploadError.message : String(uploadError);
            }
          });
        }
      }
    } finally {
      isUploadProcessorRunning.value = false;
      // Only set isUploading to false if queue is empty
      if (uploadQueue.value.length === 0) {
        isUploading.value = false;
      }
    }
  };

  // Main create and upload function
  const createAndUpload = async () => {
    if (!canCreateAndUpload.value) return;

    isProcessing.value = true;
    isCancelled.value = false;
    error.value = null;
    success.value = null;
    bulkResults.value = null;
    videoProgresses.value.clear();
    uploadProgresses.value.clear();
    uploadIndexToVideoPath.value.clear();
    itemDisplayProgress.value = new Map();
    tempVideoPaths.value = [];
    uploadQueue.value = [];

    currentStep.value = "processing";

    try {
      // Ensure image previews exist for all tracks so pipeline thumbnails show
      for (const track of audioTracks.value) {
        const pair = audioImagePairs.value.get(track.path);
        if (pair) generatePreviewForPair(pair);
      }

      // Step 1: Create videos (uploads will start automatically as videos complete)
      if (saveVideosLocally.value || uploadToYouTube.value) {
        isCreatingVideos.value = true;
        
        // Determine output path
        let outputPath: string;
        if (saveVideosLocally.value && outputFolder.value) {
          outputPath = outputFolder.value;
        } else {
          // Use temp directory
          const tempDir = await invoke<string>("get_temp_dir");
          outputPath = tempDir;
        }

        const imageInputs: ImageInput[] = [];
        let lastInput: ImageInput | null = null;

        for (const track of audioTracks.value) {
          const pair = audioImagePairs.value.get(track.path);
          if (!pair) {
            throw new Error(`No image pair for track: ${track.name}`);
          }

          const hasBackground =
            !!pair.backgroundPath ||
            !!pair.backgroundUrl ||
            !!pair.backgroundPredefinedId;
          const baseStyle: ImageStyle = pair.imageStyle ?? DEFAULT_IMAGE_STYLE;

          let input: ImageInput;

          if (pair.usePrevious && lastInput) {
            input = { ...lastInput };
          } else if (baseStyle === "background-only") {
            if (!hasBackground) {
              throw new Error(`Track "${track.name}" uses background-only style but has no background assigned`);
            }
            input = {
              image_path: undefined,
              image_url: undefined,
              image_style: "background-only",
              fit_mode: undefined,
              background_path: pair.backgroundPath ?? undefined,
              background_url: pair.backgroundUrl ?? undefined,
              background_predefined_id: pair.backgroundPredefinedId ?? undefined
            };
            lastInput = input;
          } else if (pair.imagePath || pair.imageUrl) {
            const style: ImageStyle =
              baseStyle === "custom-background" && !hasBackground
                ? DEFAULT_IMAGE_STYLE
                : baseStyle;
            input = {
              image_path: pair.imagePath ?? undefined,
              image_url: pair.imageUrl ?? undefined,
              image_style: style,
              fit_mode: pair.fitMode ?? DEFAULT_FIT_MODE,
              background_path: pair.backgroundPath ?? undefined,
              background_url: pair.backgroundUrl ?? undefined,
              background_predefined_id: pair.backgroundPredefinedId ?? undefined
            };
            lastInput = input;
          } else {
            throw new Error(`Track "${track.name}" has no image assigned`);
          }
          imageInputs.push(input);
        }

        const audioPaths = audioTracks.value.map(t => t.path);
        
        // Initialize video progress
        audioTracks.value.forEach((track, index) => {
          videoProgresses.value.set(index, {
            index,
            audioPath: track.path,
            audioName: track.metadata && (track.metadata.artist || track.metadata.title)
              ? (track.metadata.artist && track.metadata.title
                  ? `${track.metadata.artist} - ${track.metadata.title}`
                  : track.metadata.title || track.metadata.artist || track.name)
              : track.name,
            progress: 0,
            message: "Waiting...",
            status: "pending"
          });
        });

        const bulkRequest: BulkVideoRequest = {
          image_inputs: imageInputs,
          audio_paths: audioPaths,
          output_path: outputPath,
          config: {
            width: 1920,
            height: 1080,
            fps: 30
          },
          normalize_audio: normalizeAudio.value,
          high_pass_filter: normalizeAudio.value ? highPassFilter.value : undefined,
          export_normalized_audio: exportNormalizedAudio.value && normalizeAudio.value,
          audio_export_path: exportNormalizedAudio.value && normalizeAudio.value && audioExportFolder.outputFolder.value ? audioExportFolder.outputFolder.value : undefined
        };

        const result = await invoke<BulkVideoResult>("create_bulk_videos", { request: bulkRequest });
        bulkResults.value = result;

        if (result.failed.length > 0) {
          throw new Error(`Failed to create ${result.failed.length} video(s)`);
        }

        // Update video metadata with created paths
        result.successful.forEach((videoPath, index) => {
          const track = audioTracks.value[index];
          if (track) {
            const metadata = videoMetadata.value.get(track.path);
            if (metadata) {
              metadata.videoPath = videoPath;
              metadata.isTempFile = !saveVideosLocally.value;
              videoMetadata.value.set(track.path, metadata);
              
              if (!saveVideosLocally.value) {
                tempVideoPaths.value.push(videoPath);
              }
            }
          }
        });

        isCreatingVideos.value = false;
      }

      // Wait for all uploads to complete if uploading to YouTube
      if (uploadToYouTube.value && youtube.isAuthenticated.value) {
        // Wait for upload queue to be processed
        while (isUploadProcessorRunning.value || uploadQueue.value.length > 0) {
          await new Promise(resolve => setTimeout(resolve, 100));
        }
        
        // Set final success/error messages
        const totalUploads = Array.from(uploadProgresses.value.values());
        const successfulUploads = totalUploads.filter(p => p.status === "completed").length;
        const failedUploads = totalUploads.filter(p => p.status === "failed").length;
        
        if (failedUploads > 0) {
          error.value = `Uploaded ${successfulUploads} video(s), ${failedUploads} failed`;
        } else if (successfulUploads > 0) {
          success.value = `Successfully uploaded ${successfulUploads} video(s)!`;
        }
      } else if (bulkResults.value) {
        // For video creation only
        if (bulkResults.value.failed.length > 0) {
          error.value = `Created ${bulkResults.value.successful.length} video(s), ${bulkResults.value.failed.length} failed`;
        } else {
          success.value = `Successfully created ${bulkResults.value.successful.length} video(s)!`;
        }
      }

      // Step 3: Cleanup temp files if not saving locally
      if (!saveVideosLocally.value && tempVideoPaths.value.length > 0) {
        try {
          await invoke("cleanup_temp_videos", { paths: tempVideoPaths.value });
        } catch (err) {
          console.warn("Failed to cleanup temp files:", err);
          // Don't fail the operation if cleanup fails
        }
      }

      // Desktop notification
      const notifyEnabled = preferences.value.preferences?.notify_on_upload_complete ?? false;
      if (notifyEnabled && uploadToYouTube.value) {
        const title = uploadToYouTube.value ? "Upload complete" : "Videos created";
        const body = success.value || "Operation completed successfully.";
        await notify(title, body);
      }

    } catch (err) {
      const errMsg = String(err);
      if (errMsg.includes("cancelled") || errMsg.includes("Operation cancelled")) {
        isCancelled.value = true;
        error.value = "Operation cancelled";
        // Redirect back to options step when operation is cancelled
        currentStep.value = "options";
      } else if (errMsg.includes("connection has expired") || 
                 errMsg.includes("reconnect your YouTube account") ||
                 errMsg.includes("invalid_grant") ||
                 errMsg.includes("Token expired")) {
        // Authentication error - clear auth status and show re-auth prompt
        youtube.isAuthenticated.value = false;
        error.value = "Your YouTube connection has expired. Please reconnect your YouTube account in Settings.";
      } else {
        error.value = `Failed: ${err}`;
      }
    } finally {
      isProcessing.value = false;
      isCreatingVideos.value = false;
      isUploading.value = false;
    }
  };

  // Cleanup temp files on cancel
  const cleanupTempFiles = async () => {
    if (tempVideoPaths.value.length > 0) {
      try {
        await invoke("cleanup_temp_videos", { paths: tempVideoPaths.value });
        tempVideoPaths.value = [];
      } catch (err) {
        console.warn("Failed to cleanup temp files:", err);
      }
    }
  };

  const cancelOperation = async (phaseId: string) => {
    try {
      let operationType: string;
      if (phaseId === "normalize") {
        operationType = "audio_normalization";
      } else if (phaseId === "videos") {
        operationType = "bulk_video_creation";
      } else if (phaseId === "upload") {
        operationType = "youtube_upload";
      } else {
        return;
      }
      await invoke("cancel_operation", { operationType });
      await cleanupTempFiles();
      resetProcessingState();
      // Redirect back to options step after cancellation
      currentStep.value = "options";
    } catch (err) {
      error.value = `Failed to cancel operation: ${err}`;
      await cleanupTempFiles();
      resetProcessingState();
      // Even if cancellation fails, redirect back to options
      currentStep.value = "options";
    }
  };

  const cancelItem = async (phaseId: string, itemId: string | number) => {
    try {
      let operationType: string;
      if (phaseId === "normalize") {
        operationType = "audio_normalization";
      } else if (phaseId === "videos") {
        operationType = "bulk_video_creation";
      } else if (phaseId === "upload") {
        operationType = "youtube_upload";
      } else {
        return;
      }
      
      await invoke("cancel_item", { operationType, itemId: String(itemId) });
      
      // Mark the specific item as cancelled immediately based on phase
      if (phaseId === "upload") {
        const existing = uploadProgresses.value.get(Number(itemId));
        if (existing && existing.status === "pending") {
          existing.status = "failed";
          existing.message = "Cancelled";
          existing.progress = 0;
        }
      } else if (phaseId === "videos") {
        const existing = videoProgresses.value.get(Number(itemId));
        if (existing && existing.status !== "completed") {
          existing.status = "failed";
          existing.message = "Cancelled";
          existing.progress = 0;
        }
      }
      // Note: normalize phase doesn't have individual item progress tracking
    } catch (error) {
      console.error("Failed to cancel item:", error);
    }
  };

  const resetProcessingState = () => {
    isProcessing.value = false;
    isNormalizing.value = false;
    isCreatingVideos.value = false;
    isUploading.value = false;
    isCancelled.value = false;
    videoProgresses.value.clear();
    uploadProgresses.value.clear();
    itemDisplayProgress.value = new Map();
    normalizationProgress.value = { total: 0, completed: 0, message: "" };
    error.value = null;
    success.value = null;
    bulkResults.value = null;
    // Don't reset currentStep here as it should be handled by the caller
  };

  const clearSource = () => {
    audioTracks.value = [];
    audioImagePairs.value.clear();
    videoMetadata.value.clear();
    selectedAudioIndex.value = null;
    selectedFolder.value = null;
    folderContents.value = null;
    currentImagePath.value = null;
    currentImageUrl.value = "";
    usePreviousImage.value = false;
    currentImageStyle.value = DEFAULT_IMAGE_STYLE;
    currentFitMode.value = DEFAULT_FIT_MODE;
    currentBackgroundPath.value = null;
    currentBackgroundUrl.value = "";
    currentBackgroundPredefinedId.value = null;
    previewImage.value = null;
    outputFolder.value = null;
    currentStep.value = "source";
    isGeneratingPreview.value = false;
    isLoadingDiscogsImages.value = false;
    isLoadingFolder.value = false;
    isLoadingFiles.value = false;
    resetProcessingState();
    imageThumbnails.value.clear();
    imagePreviews.value.clear();
    discogsReleaseForImage.value.clear();
    tempVideoPaths.value = [];
  };

  const removeAudioTrack = (index: number) => {
    const track = audioTracks.value[index];
    if (track) {
      audioImagePairs.value.delete(track.path);
      videoMetadata.value.delete(track.path);
    }
    audioTracks.value.splice(index, 1);
    if (audioTracks.value.length === 0) {
      selectedAudioIndex.value = null;
      currentStep.value = "source";
      isGeneratingPreview.value = false;
      isLoadingDiscogsImages.value = false;
      isLoadingFolder.value = false;
      isLoadingFiles.value = false;
      resetProcessingState();
      imageThumbnails.value.clear();
      imagePreviews.value.clear();
      currentImagePath.value = null;
      currentImageUrl.value = "";
      usePreviousImage.value = false;
      currentImageStyle.value = DEFAULT_IMAGE_STYLE;
      currentFitMode.value = DEFAULT_FIT_MODE;
      currentBackgroundPath.value = null;
      currentBackgroundUrl.value = "";
      currentBackgroundPredefinedId.value = null;
      previewImage.value = null;
      discogsReleaseForImage.value.clear();
      return;
    }
    if (selectedAudioIndex.value === index) {
      selectAudioTrack(Math.min(index, audioTracks.value.length - 1));
    } else if (selectedAudioIndex.value !== null && selectedAudioIndex.value > index) {
      selectedAudioIndex.value--;
    }
  };

  const getPairStatus = (trackPath: string) => {
    const pair = audioImagePairs.value.get(trackPath);
    if (!pair) return "unassigned";
    if (pair.usePrevious) return "previous";
    if (pair.imagePath || pair.imageUrl) return "assigned";
    const isBackgroundOnly = (pair.imageStyle ?? DEFAULT_IMAGE_STYLE) === "background-only";
    const hasBackground =
      !!pair.backgroundPath || !!pair.backgroundUrl || !!pair.backgroundPredefinedId;
    if (isBackgroundOnly && hasBackground) return "assigned";
    return "unassigned";
  };

  // Event listeners setup
  const unlistens: Array<() => void> = [];
  const uploadIndexMap = ref<Map<number, number>>(new Map()); // upload index -> track index

  const setupEventListeners = async () => {
    // Video creation progress
    const unlisten1 = await listen("bulk-video-progress", () => {
      // Update progress handled by individual video progress events
    });
    unlistens.push(unlisten1);

    const unlisten2 = await listen("video-progress-start", (event) => {
      const data = event.payload as { index: number; audio_path: string; audio_name: string };
      const existing = videoProgresses.value.get(data.index);
      if (existing) {
        existing.status = "processing";
        existing.message = "Starting...";
        existing.progress = 0;
      }
    });
    unlistens.push(unlisten2);

    const unlisten3 = await listen("video-progress-update", (event) => {
      const data = event.payload as { index: number; progress: number; message: string };
      const rounded = Math.round(data.progress);
      const existing = videoProgresses.value.get(data.index);
      if (existing) {
        existing.progress = rounded;
        existing.message = data.message;
      }
      const next = new Map(itemDisplayProgress.value);
      next.set(data.index, rounded);
      itemDisplayProgress.value = next;
    });
    unlistens.push(unlisten3);

    const unlisten4 = await listen("video-progress-complete", (event) => {
      const data = event.payload as { index: number; success: boolean; error?: string; output_file?: string };
      const existing = videoProgresses.value.get(data.index);
      const finalProgress = data.success ? 100 : 0;
      if (existing) {
        if (data.error && (data.error.includes("Cancelled") || data.error.includes("cancelled"))) {
          existing.status = "failed";
          existing.message = "Cancelled";
          existing.error = "Cancelled";
        } else {
          existing.status = data.success ? "completed" : "failed";
          existing.message = data.success ? "Completed" : (data.error || "Failed");
          if (data.error) {
            existing.error = data.error;
          }
        }
        existing.progress = finalProgress;
      }
      const next = new Map(itemDisplayProgress.value);
      next.set(data.index, finalProgress);
      itemDisplayProgress.value = next;

      // Queue completed video for upload if uploading to YouTube
      if (data.success && data.output_file && uploadToYouTube.value && youtube.isAuthenticated.value) {
        const track = audioTracks.value[data.index];
        const metadata = videoMetadata.value.get(track.path);
        if (metadata) {
          // Update metadata with video path
          metadata.videoPath = data.output_file;
          metadata.isTempFile = !saveVideosLocally.value;
          videoMetadata.value.set(track.path, metadata);
          
          if (!saveVideosLocally.value) {
            tempVideoPaths.value.push(data.output_file);
          }

          // Add to upload queue (replace map so Vue reactivity triggers)
          uploadIndexToVideoPath.value = new Map(uploadIndexToVideoPath.value).set(data.index, data.output_file);
          if (!hasVideoThumbnail(data.output_file)) {
            generateVideoThumbnail(data.output_file);
          }
          uploadQueue.value.push({
            trackIndex: data.index,
            videoPath: data.output_file,
            metadata: metadata
          });

          // Process upload queue
          processUploadQueue();
        }
      }
    });
    unlistens.push(unlisten4);

    // Upload progress
    const unlisten5 = await listen("youtube-upload-start", (event) => {
      const data = event.payload as { index: number; video_path: string; title: string };
      // The backend index is the upload request index, map it to track index
      const trackIndex = uploadIndexMap.value.get(data.index);
      if (trackIndex !== undefined) {
        uploadProgresses.value.set(trackIndex, {
          index: trackIndex,
          progress: 0,
          message: "Starting upload...",
          status: "uploading"
        });
      }
    });
    unlistens.push(unlisten5);

    const unlisten6 = await listen("youtube-upload-progress", (event) => {
      const data = event.payload as { video_path: string; progress: number; message: string };
      // Find index by matching video path
      const trackIndex = audioTracks.value.findIndex(t => {
        const meta = videoMetadata.value.get(t.path);
        return meta?.videoPath === data.video_path;
      });
      if (trackIndex !== -1) {
        const existing = uploadProgresses.value.get(trackIndex);
        if (existing) {
          existing.progress = data.progress;
          existing.message = data.message;
        } else {
          uploadProgresses.value.set(trackIndex, {
            index: trackIndex,
            progress: data.progress,
            message: data.message,
            status: "uploading"
          });
        }
      }
    });
    unlistens.push(unlisten6);

    const unlisten7 = await listen("youtube-upload-complete", (event) => {
      const data = event.payload as { index: number; success: boolean; video_id?: string; error?: string };
      // The backend index is the upload request index, map it to track index
      const trackIndex = uploadIndexMap.value.get(data.index);
      
      // Check if this is an authentication error
      if (!data.success && data.error) {
        const errorMsg = data.error.toLowerCase();
        if (errorMsg.includes("connection has expired") || 
            errorMsg.includes("reconnect your youtube account") ||
            errorMsg.includes("invalid_grant") ||
            errorMsg.includes("token expired")) {
          // Authentication error - clear auth status
          youtube.isAuthenticated.value = false;
        }
      }
      
      if (trackIndex !== undefined) {
        const existing = uploadProgresses.value.get(trackIndex);
        if (existing) {
          existing.status = data.success ? "completed" : "failed";
          existing.progress = data.success ? 100 : 0;
          existing.message = data.success ? "Upload complete!" : (data.error || "Upload failed");
          if (data.error) {
            existing.error = data.error;
          }
        } else {
          uploadProgresses.value.set(trackIndex, {
            index: trackIndex,
            progress: data.success ? 100 : 0,
            message: data.success ? "Upload complete!" : (data.error || "Upload failed"),
            status: data.success ? "completed" : "failed",
            error: data.error
          });
        }
      } else {
        // Fallback: try to find by video_id if available
        // This shouldn't happen if mapping is correct, but handle gracefully
        console.warn(`Upload complete event received for unknown index ${data.index}`);
      }
    });
    unlistens.push(unlisten7);

    // Audio normalization
    const unlisten8 = await listen("audio-normalization-start", (event) => {
      const data = event.payload as { total: number; message: string };
      isNormalizing.value = true;
      normalizationProgress.value = {
        total: data.total,
        completed: 0,
        message: data.message
      };
    });
    unlistens.push(unlisten8);

    const unlisten9 = await listen("audio-normalization-complete", (event) => {
      const data = event.payload as { total: number; message: string };
      isNormalizing.value = false;
      normalizationProgress.value = {
        total: data.total,
        completed: data.total,
        message: data.message
      };
    });
    unlistens.push(unlisten9);
  };

  onScopeDispose(() => {
    unlistens.forEach((fn) => fn());
    unlistens.length = 0;
  });

  onMounted(async () => {
    await setupEventListeners();
    await loadPreferences();
    await youtube.checkAuthStatus();
    await discogs.checkAuthStatus();
    await youtube.loadPlaylists();
    
    const prefsSnapshot = preferences.value.preferences;
    const last = prefsSnapshot?.last_creation_output ?? null;
    const preferred = prefsSnapshot?.preferred_output_path ?? null;
    const effective = last || preferred;
    if (effective) {
      outputFolder.value = effective;
    }

    // Apply defaults from preferences
    const prefs = preferences.value.preferences;
    if (prefs) {
      normalizeAudio.value = prefs.default_normalize_for_creation ?? false;
      highPassFilter.value = prefs.default_high_pass_when_normalizing ?? false;
      saveVideosLocally.value = prefs.default_save_videos_locally ?? false;
      // uploadToYouTube is always true in this flow (it's the upload flow)
      uploadToYouTube.value = true;
    }

    // Load audio export folder from preferences
    await audioExportFolder.loadFromPreferences();
  });

  // Sync discogs search results from composable to local ref
  watch(() => discogs.searchResults.value, (results) => {
    discogsSearchResults.value = results;
  }, { immediate: true });

  // Watch for Discogs search prefilling
  watch(selectedAudio, (track) => {
    if (!track) {
      discogs.searchQuery.value = "";
      discogs.clearSearch();
      discogsImages.value = [];
      return;
    }
    if (!discogs.isAuthenticated.value) {
      discogs.searchQuery.value = "";
      discogs.clearSearch();
      discogsImages.value = [];
      return;
    }
    const baseTitle = track.name.replace(/\.[^/.]+$/, "").trim();
    if (baseTitle.length >= 3) {
      setDiscogsSearchQuery(baseTitle);
    }
  });

  const setNormalizeAudio = (v: boolean) => {
    normalizeAudio.value = v;
  };
  
  const setHighPassFilter = (v: boolean) => {
    highPassFilter.value = v;
  };
  
  const setOutputFolder = (v: string | null) => {
    outputFolder.value = v;
  };

  const dismissNoFilesModal = () => {
    noFilesFoundModalOpen.value = false;
    error.value = null;
  };

  const uploadIndexToThumbnailUrl = computed(() => {
    const m = new Map<number, string>();
    uploadIndexToVideoPath.value.forEach((path, index) => {
      const url = getVideoThumbnail(path);
      if (url) m.set(index, url);
    });
    return m;
  });

  /** Track index -> image preview data URL for pipeline thumbnails (from image assignment). */
  const trackIndexToImagePreviewUrl = computed(() => {
    const m = new Map<number, string>();
    const tracks = audioTracks.value;
    for (let i = 0; i < tracks.length; i++) {
      const track = tracks[i];
      const pair = audioImagePairs.value.get(track.path);
      if (!pair) continue;
      const style = pair.imageStyle ?? DEFAULT_IMAGE_STYLE;
      const fit = pair.fitMode ?? DEFAULT_FIT_MODE;
      const bgPath = pair.backgroundPath ?? null;
      const bgUrl = pair.backgroundUrl ?? null;
      const bgPredef = pair.backgroundPredefinedId ?? null;
      let main: string | null = pair.imagePath ?? pair.imageUrl ?? null;
      if (style === "background-only" && !main) {
        if (pair.backgroundPath) main = pair.backgroundPath;
        else if (pair.backgroundUrl) main = pair.backgroundUrl;
        else if (pair.backgroundPredefinedId) {
          const bg = predefinedBackgrounds.value.find((b) => b.id === pair.backgroundPredefinedId)
            || userBackgrounds.value.find((b) => b.id === pair.backgroundPredefinedId);
          if (bg) main = bg.imagePath;
        }
      }
      if (!main) continue;
      const key = previewCacheKey(main, style, fit, bgPath, bgUrl, bgPredef);
      const url = imagePreviews.value.get(key);
      if (url) m.set(i, url);
    }
    return m;
  });

  const { unifiedPipeline } = useUnifiedPipeline({
    normalizationProgress,
    videoProgresses,
    uploadProgresses,
    itemDisplayProgress,
    uploadQueue,
    videoMetadata,
    audioTracks,
    normalizeAudio,
    isNormalizing,
    isCancelled,
    saveVideosLocally,
    uploadToYouTube,
    uploadIndexToThumbnailUrl,
    trackIndexToImagePreviewUrl
  });

  return {
    // State
    audioTracks,
    selectedFolder,
    folderContents,
    selectedAudioIndex,
    selectedAudio,
    audioImagePairs,
    videoMetadata,
    selectedVideoMetadata,
    currentImagePath,
    currentImageUrl,
    usePreviousImage,
    currentImageStyle,
    currentFitMode,
    currentBackgroundPath,
    currentBackgroundUrl,
    currentBackgroundPredefinedId,
    previewImage,
    predefinedBackgrounds,
    isLoadingPredefinedBackgrounds,
    predefinedBackgroundsLoadError,
    userBackgrounds,
    isLoadingUserBackgrounds,
    userBackgroundsLoadError,
    imageThumbnails,
    imagePreviews,
    isGeneratingPreview,
    isDiscogsAuthenticated: computed(() => discogs.isAuthenticated.value),
    isDiscogsAuthenticating: computed(() => discogs.isAuthenticating.value),
    discogsError,
    discogsSearchQuery: computed({
      get: () => discogs.searchQuery.value,
      set: (val: string) => { discogs.searchQuery.value = val; }
    }),
    discogsSearchResults,
    isSearchingDiscogs: computed(() => discogs.isSearching.value),
    discogsImages,
    isLoadingDiscogsImages,
    noFilesFoundModalOpen,
    dismissNoFilesModal,
    saveVideosLocally,
    uploadToYouTube,
    outputFolder,
    currentStep,
    isProcessing,
    isCreatingVideos,
    isUploading,
    videoProgresses,
    uploadProgresses,
    unifiedPipeline,
    error,
    success,
    bulkResults,
    normalizeAudio,
    highPassFilter,
    isNormalizing,
    normalizationProgress,
    exportNormalizedAudio,
    audioExportFolder,
    isCancelled,
    isLoadingFolder,
    isLoadingFiles,
    favoriteBackgroundId,
    // Computed
    canProceedToImages,
    canProceedToMetadata,
    canProceedToOptions,
    canCreateAndUpload,
    getPairStatus,
    // Actions
    selectFolder,
    loadFolderContents,
    selectAudioFiles,
    selectAudioTrack,
    removeAudioTrack,
    selectImageFile,
    selectImageFromFolder,
    saveCurrentPair,
    // Image editor
    getCurrentImageSrc,
    hasSelectedImage,
    isImageEditorOpen,
    editingImageSrc,
    openImageEditor,
    closeImageEditor,
    handleEditedImage,
    generatePreview,
    toggleUsePrevious,
    previousGeneratedImages,
    selectPreviousGeneratedImage,
    applyCurrentImageToAllTracks,
    applyImageFromUrl,
    importBackground,
    selectBackgroundFile,
    setBackgroundFromUrl,
    setBackgroundPredefined,
    setFavoriteBackground,
    authenticateDiscogsForImages,
    setDiscogsSearchQuery,
    searchDiscogsReleasesForArt,
    selectDiscogsReleaseForImage,
    selectDiscogsImageForCurrentTrack,
    updateVideoMetadata,
    selectDiscogsReleaseForMetadata,
    applyDescriptionTemplate,
    createAndUpload,
    setNormalizeAudio,
    setHighPassFilter,
    setOutputFolder,
    clearSource,
    cancelOperation,
    cancelItem,
    // Concurrent upload state
    uploadQueue,
    isUploadProcessorRunning,
    // YouTube
    youtube,
    // Discogs - expose the composable directly
    discogs
  };
}
