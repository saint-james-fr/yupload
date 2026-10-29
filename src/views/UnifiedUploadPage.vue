<script setup lang="ts">
import type { ImageStyle, FitMode } from "../types";
import { computed, watch, ref, nextTick, onMounted } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { useUnifiedUploadFlow } from "../composables/useUnifiedUploadFlow";
import { useSharedModals } from "../composables/useSharedModals";
import Button from "../components/ui/Button.vue";
import Card from "../components/ui/Card.vue";
import Modal from "../components/ui/Modal.vue";
import PageContainer from "../components/layout/PageContainer.vue";
import PageHeader from "../components/layout/PageHeader.vue";
import TwoPanelLayout from "../components/layout/TwoPanelLayout.vue";
import SelectableListItem from "../components/SelectableListItem.vue";
import EmptyState from "../components/EmptyState.vue";
import StatusAlert from "../components/StatusAlert.vue";
import AudioOptions from "../components/AudioOptions.vue";
import ImageAssignmentPanel from "../components/ImageAssignmentPanel.vue";
import AudioPreview from "../components/AudioPreview.vue";
import VideoMetadataEditor from "../components/VideoMetadataEditor.vue";
import Callout from "../components/ui/Callout.vue";
import SectionHeader from "../components/ui/SectionHeader.vue";
import BottomNavigation from "../components/ui/BottomNavigation.vue";
import SourceStepWithFileCard from "../components/SourceStepWithFileCard.vue";
import UnifiedUploadOptionsStep from "../components/unified-upload/UnifiedUploadOptionsStep.vue";
import UnifiedUploadProcessingStep from "../components/unified-upload/UnifiedUploadProcessingStep.vue";
import { 
  FileVideo, 
  ChevronLeft, 
  ChevronRight, 
  Music,
  Unlock,
  Copy,
  CheckCircle2
} from "lucide-vue-next";

const emit = defineEmits<{
  (e: "back"): void;
  (
    e: "navigate",
    view: "success",
    payload: { from: "unified-upload"; title: string; summary: string }
  ): void;
}>();

const {
  // Audio tracks
  audioTracks,
  selectedFolder,
  folderContents,
  selectedAudioIndex,
  selectedAudio,
  // Video metadata
  videoMetadata,
  selectedVideoMetadata,
  // Image assignment
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
  predefinedBackgroundsLoadError,
  userBackgrounds,
  isLoadingUserBackgrounds,
  userBackgroundsLoadError,
  favoriteBackgroundId,
  imageThumbnails,
  isGeneratingPreview,
  isDiscogsAuthenticated,
  isDiscogsAuthenticating,
  discogsError,
  discogsSearchQuery,
  discogsSearchResults,
  isSearchingDiscogs,
  discogsImages,
  isLoadingDiscogsImages,
  noFilesFoundModalOpen,
  dismissNoFilesModal,
  // Options
  saveVideosLocally,
  uploadToYouTube: _uploadToYouTube, // Always true in this flow, kept for composable internal logic
  outputFolder,
  // Processing
  currentStep,
  isProcessing,
  unifiedPipeline,
  error,
  success,
  bulkResults,
  normalizeAudio,
  highPassFilter,
  isNormalizing,
  exportNormalizedAudio,
  audioExportFolder,
  isCancelled,
  isLoadingFolder,
  isLoadingFiles,
  isLoadingPredefinedBackgrounds,
  // Computed
  canProceedToImages,
  canProceedToMetadata,
  canProceedToOptions,
  canCreateAndUpload,
  getPairStatus,
  previousGeneratedImages,
  // Actions
  selectFolder,
  selectAudioFiles,
  selectAudioTrack,
  removeAudioTrack,
  selectImageFile,
  selectImageFromFolder,
  saveCurrentPair,
  // Image editor
  hasSelectedImage,
  isImageEditorOpen,
  editingImageSrc,
  openImageEditor,
  closeImageEditor,
  handleEditedImage,
  generatePreview,
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
  selectPreviousGeneratedImage,
  applyCurrentImageToAllTracks,
  applyImageFromUrl,
  updateVideoMetadata,
  applyDescriptionTemplate,
  createAndUpload,
  setNormalizeAudio,
  setHighPassFilter,
  setOutputFolder,
  clearSource,
  cancelOperation,
  cancelItem,
  youtube,
  discogs
} = useUnifiedUploadFlow();

const showApplyDescriptionSuccess = ref(false);
const showApplyPrivacySuccess = ref(false);
const showApplyCategorySuccess = ref(false);
const showApplyTagsSuccess = ref(false);

// Shared modals
const modals = useSharedModals();

const handleApplyDescriptionToAll = () => {
  if (!selectedVideoMetadata.value || selectedAudioIndex.value === null) return;
  const currentDescription = selectedVideoMetadata.value.description;
  
  audioTracks.value.forEach(track => {
    const metadata = videoMetadata.value.get(track.path);
    if (metadata) {
      videoMetadata.value.set(track.path, {
        ...metadata,
        description: currentDescription
      });
    }
  });
  
  showApplyDescriptionSuccess.value = true;
  setTimeout(() => {
    showApplyDescriptionSuccess.value = false;
  }, 2000);
};

const handleApplyPrivacyToAll = () => {
  if (!selectedVideoMetadata.value || selectedAudioIndex.value === null) return;
  const { privacyStatus, publishAt } = selectedVideoMetadata.value;
  
  audioTracks.value.forEach(track => {
    const metadata = videoMetadata.value.get(track.path);
    if (metadata) {
      videoMetadata.value.set(track.path, {
        ...metadata,
        privacyStatus,
        publishAt: publishAt ?? null
      });
    }
  });
  
  showApplyPrivacySuccess.value = true;
  setTimeout(() => {
    showApplyPrivacySuccess.value = false;
  }, 2000);
};

const handleApplyCategoryToAll = () => {
  if (!selectedVideoMetadata.value || selectedAudioIndex.value === null) return;
  const categoryId = selectedVideoMetadata.value.categoryId;
  
  audioTracks.value.forEach(track => {
    const metadata = videoMetadata.value.get(track.path);
    if (metadata) {
      videoMetadata.value.set(track.path, {
        ...metadata,
        categoryId
      });
    }
  });
  
  showApplyCategorySuccess.value = true;
  setTimeout(() => {
    showApplyCategorySuccess.value = false;
  }, 2000);
};

const handleApplyTagsToAll = () => {
  if (!selectedVideoMetadata.value || selectedAudioIndex.value === null) return;
  const tags = [...selectedVideoMetadata.value.tags];
  
  audioTracks.value.forEach(track => {
    const metadata = videoMetadata.value.get(track.path);
    if (metadata) {
      videoMetadata.value.set(track.path, {
        ...metadata,
        tags
      });
    }
  });
  
  showApplyTagsSuccess.value = true;
  setTimeout(() => {
    showApplyTagsSuccess.value = false;
  }, 2000);
};

// Redirect back if YouTube is not authenticated
onMounted(async () => {
  await youtube.checkAuthStatus();
  if (!youtube.isAuthenticated.value) {
    emit("back");
  }
});

const onUpdateCurrentImageStyle = async (v: ImageStyle) => {
  currentImageStyle.value = v;
  // Regenerate preview when style changes
  if (currentImagePath.value || currentImageUrl.value || currentBackgroundPath.value || currentBackgroundUrl.value || currentBackgroundPredefinedId.value) {
    previewImage.value = null;
    await generatePreview();
    saveCurrentPair();
  }
};

const onUpdateCurrentFitMode = async (v: FitMode) => {
  currentFitMode.value = v;
  // Regenerate preview when fit mode changes
  if (currentImagePath.value || currentImageUrl.value || currentBackgroundPath.value || currentBackgroundUrl.value || currentBackgroundPredefinedId.value) {
    previewImage.value = null;
    await generatePreview();
    saveCurrentPair();
  }
};

const onUpdateCurrentImageUrl = (v: string) => {
  currentImageUrl.value = v;
};

const onUpdateCurrentBackgroundUrl = (v: string) => {
  currentBackgroundUrl.value = v;
};

const onUpdateDiscogsSearchQuery = (v: string) => {
  setDiscogsSearchQuery(v);
};

const selectedTrackStatusCallout = computed(() => {
  if (selectedAudioIndex.value === null) return null;
  const track = audioTracks.value[selectedAudioIndex.value];
  if (!track) return null;
  const status = getPairStatus(track.path);
  if (status === "assigned") return { variant: "success" as const, message: "Image assigned." };
  if (status === "previous") return { variant: "success" as const, message: "Using previous track's image." };
  return { variant: "warning" as const, message: "No image assigned. Choose an image below." };
});

const selectedTrackMetadata = computed(() => {
  if (selectedAudioIndex.value === null) return null;
  const track = audioTracks.value[selectedAudioIndex.value];
  return track?.metadata ?? null;
});

const hasPreviousTrack = computed(
  () => selectedAudioIndex.value !== null && selectedAudioIndex.value > 0
);

const hasNextTrack = computed(
  () =>
    selectedAudioIndex.value !== null &&
    audioTracks.value.length > 1 &&
    selectedAudioIndex.value < audioTracks.value.length - 1
);

const videoStatus = (trackPath: string): "success" | "warning" | "muted" => {
  const meta = videoMetadata.value.get(trackPath);
  if (!meta) return "muted";
  const hasTitle = (meta.title?.trim() ?? "").length > 0;
  const hasDescription = (meta.description?.trim() ?? "").length > 0;
  if (!hasTitle) return "muted";
  if (!hasDescription) return "warning";
  return "success";
};

const rightPanelStartRef = ref<HTMLElement | null>(null);
const progressSectionRef = ref<HTMLElement | null>(null);

watch(selectedAudioIndex, () => {
  nextTick(() => {
    rightPanelStartRef.value?.scrollIntoView({ behavior: "smooth", block: "start" });
  });
});

watch(isProcessing, (processing) => {
  if (processing) {
    nextTick(() => {
      progressSectionRef.value?.scrollIntoView({ behavior: "smooth", block: "start" });
    });
  }
});

watch(currentStep, (newStep) => {
  if (newStep === 'metadata' || newStep === 'options') {
    nextTick(() => {
      window.scrollTo({ top: 0, behavior: 'smooth' });
    });
  }
});

watch(success, (msg) => {
  if (msg && bulkResults.value && bulkResults.value.failed.length === 0 && !isCancelled.value) {
    emit("navigate", "success", {
      from: "unified-upload",
      title: "Upload complete",
      summary: msg,
    });
  }
});

const handleBack = async () => {
  if (isProcessing.value || isNormalizing.value) {
    modals.showCancelConfirmation("bulk_video_creation");
  } else {
    emit("back");
  }
};

const handleCreateAndUpload = async () => {
  // Check if YouTube is not authenticated (upload is always enabled in this flow)
  if (!youtube.isAuthenticated.value) {
    modals.showYouTubeConnection();
    return;
  }
  
  // Check if save is enabled but no output folder
  if (saveVideosLocally.value && !outputFolder.value) {
    error.value = "Please select an output folder to save videos locally.";
    return;
  }
  
  await createAndUpload();
};

const connectYouTubeAndProceed = async () => {
  modals.hideYouTubeConnection();
  await youtube.authenticate();
  if (youtube.isAuthenticated.value) {
    await createAndUpload();
  }
};

const handleCancelConfirm = async () => {
  await modals.confirmCancel();
  // After cancellation, redirect back to the previous step (options)
  currentStep.value = "options";
};

const setSaveVideosLocally = (v: boolean) => {
  saveVideosLocally.value = v;
};
const setExportNormalizedAudio = (v: boolean) => {
  exportNormalizedAudio.value = v;
};
const isSelectingOutputFolder = ref(false);
const handleSelectOutputFolder = async () => {
  isSelectingOutputFolder.value = true;
  try {
    const path = await open({ directory: true, multiple: false });
    if (path && typeof path === "string") setOutputFolder(path);
  } finally {
    isSelectingOutputFolder.value = false;
  }
};
const handleSelectAudioExportFolder = () => {
  audioExportFolder.selectFolder();
};
const handleClearOutputFolder = () => {
  setOutputFolder(null);
};
const handleClearAudioExportFolder = () => {
  audioExportFolder.clear();
};

// Watch for video selection to prefill Discogs search
// Note: The VideoMetadataEditor component handles Discogs search internally,
// but we can still prefill the search query when a track is selected
watch(selectedAudio, (newTrack) => {
  if (newTrack && discogs.isAuthenticated.value && selectedVideoMetadata.value && !selectedVideoMetadata.value.discogsRelease) {
    const title = selectedVideoMetadata.value.title.trim();
    if (title && title.length >= 3) {
      // Prefill search query - the component will handle the actual search
      discogs.searchQuery.value = title;
    }
  } else if (newTrack && !discogs.isAuthenticated.value) {
    discogs.clearSearch();
  }
});

// Watch for image style changes - backgrounds are loaded automatically by the composable
watch(currentImageStyle, async () => {
  // Backgrounds are loaded automatically when needed
}, { immediate: true });
</script>

<template>
  <!-- Step 1: Source (global step component) -->
  <SourceStepWithFileCard
    v-if="currentStep === 'source' && audioTracks.length === 0"
    variant="audio"
    title="Select source"
    subtitle="Choose a folder or add audio files to create videos and upload to YouTube."
    :selected-folder="selectedFolder"
    :file-count="audioTracks.length"
    :is-loading-folder="isLoadingFolder"
    :is-loading-files="isLoadingFiles"
    :is-processing="isProcessing"
    :no-files-found-modal-open="noFilesFoundModalOpen"
    @back="handleBack"
    @select-folder="selectFolder"
    @select-files="selectAudioFiles"
    @dismiss-no-files="dismissNoFilesModal"
  />

  <!-- Step 2-4: Main Flow -->
  <PageContainer v-else>
    <PageHeader
      title="Upload to YouTube"
      @back="handleBack"
    />

    <!-- Step Navigation -->
    <div v-if="currentStep !== 'processing'" class="mb-6 flex items-center gap-2 text-sm">
      <button
        @click="clearSource"
        :class="currentStep === 'source' ? 'font-semibold text-primary cursor-pointer' : 'text-muted-foreground hover:text-foreground cursor-pointer'"
      >
        1. Source
      </button>
      <ChevronRight class="h-4 w-4 text-muted-foreground" />
      <button
        @click="canProceedToImages && (currentStep = 'images')"
        :disabled="!canProceedToImages"
        :class="[
          currentStep === 'images' ? 'font-semibold text-primary cursor-pointer' : 'text-muted-foreground',
          canProceedToImages ? 'hover:text-foreground cursor-pointer' : 'opacity-50 cursor-not-allowed'
        ]"
      >
        2. Images
      </button>
      <ChevronRight class="h-4 w-4 text-muted-foreground" />
      <button
        @click="canProceedToMetadata && (currentStep = 'metadata')"
        :disabled="!canProceedToMetadata"
        :class="[
          currentStep === 'metadata' ? 'font-semibold text-primary cursor-pointer' : 'text-muted-foreground',
          canProceedToMetadata ? 'hover:text-foreground cursor-pointer' : 'opacity-50 cursor-not-allowed'
        ]"
      >
        3. Metadata
      </button>
      <ChevronRight class="h-4 w-4 text-muted-foreground" />
      <button
        @click="canProceedToOptions && (currentStep = 'options')"
        :disabled="!canProceedToOptions"
        :class="[
          currentStep === 'options' ? 'font-semibold text-primary cursor-pointer' : 'text-muted-foreground',
          canProceedToOptions ? 'hover:text-foreground cursor-pointer' : 'opacity-50 cursor-not-allowed'
        ]"
      >
        4. Options
      </button>
    </div>

    <!-- Step 2: Image Assignment -->
    <div v-if="currentStep === 'images'" class="space-y-6">
      <TwoPanelLayout
        left-title="Audio Tracks"
        right-title="Image Assignment"
        :items-count="audioTracks.length"
        :selected-index="selectedAudioIndex"
      >
        <template #left-icon>
          <Music class="h-5 w-5" />
        </template>

        <template #left-panel>
          <div class="flex flex-col h-full min-h-0">
            <div v-if="audioTracks.length === 0" class="flex-1 flex items-center justify-center">
              <EmptyState
                icon="music"
                message="No audio tracks selected"
                hint="Select a folder or add audio files"
              />
            </div>

            <div v-else class="space-y-1 max-h-[600px] overflow-y-auto flex-1 min-h-0">
              <SelectableListItem
                v-for="(track, index) in audioTracks"
                :key="track.path"
                :is-selected="selectedAudioIndex === index"
                :status="getPairStatus(track.path)"
                :disabled="isProcessing"
                @select="selectAudioTrack(index)"
                @remove="removeAudioTrack(index)"
              >
                <template #content>
                  <div class="flex flex-col min-w-0">
                    <span class="text-sm font-medium truncate block">
                      {{ track.metadata && (track.metadata.artist || track.metadata.title) 
                        ? (track.metadata.artist && track.metadata.title 
                            ? `${track.metadata.artist} - ${track.metadata.title}`
                            : track.metadata.title || track.metadata.artist || track.name)
                        : track.name }}
                    </span>
                    <span v-if="track.metadata && (track.metadata.artist || track.metadata.title)" class="text-xs text-muted-foreground truncate block">
                      {{ track.name }}
                    </span>
                  </div>
                </template>
              </SelectableListItem>
            </div>

            <Button
              variant="ghost"
              size="sm"
              @click="clearSource"
              class="mt-3 w-full justify-center shrink-0"
            >
              Change source
            </Button>
          </div>
        </template>

        <template #right-panel>
          <div ref="rightPanelStartRef" class="space-y-6">
            <section v-if="selectedAudioIndex !== null" class="space-y-2">
              <SectionHeader label="Track preview" />
              <AudioPreview
                :file-path="audioTracks[selectedAudioIndex]?.path ?? null"
                label=""
                :compact="true"
              />
              <div v-if="selectedTrackMetadata && (selectedTrackMetadata.artist || selectedTrackMetadata.title || selectedTrackMetadata.album)" class="text-xs text-muted-foreground space-y-0.5 pt-1">
                <div v-if="selectedTrackMetadata.artist" class="truncate">
                  <span class="font-medium">Artist:</span> {{ selectedTrackMetadata.artist }}
                </div>
                <div v-if="selectedTrackMetadata.title" class="truncate">
                  <span class="font-medium">Title:</span> {{ selectedTrackMetadata.title }}
                </div>
                <div v-if="selectedTrackMetadata.album" class="truncate">
                  <span class="font-medium">Album:</span> {{ selectedTrackMetadata.album }}
                </div>
              </div>
              <Callout
                v-if="selectedTrackStatusCallout"
                :variant="selectedTrackStatusCallout.variant"
                class="mt-3"
              >
                {{ selectedTrackStatusCallout.message }}
              </Callout>
            </section>
            <ImageAssignmentPanel
              :selected-audio-index="selectedAudioIndex"
              :audio-tracks="audioTracks"
              :current-image-path="currentImagePath"
              :current-image-url="currentImageUrl"
              :use-previous-image="usePreviousImage"
              :current-image-style="currentImageStyle"
              :current-fit-mode="currentFitMode"
              :current-background-path="currentBackgroundPath"
              :current-background-url="currentBackgroundUrl"
              :current-background-predefined-id="currentBackgroundPredefinedId"
              :preview-image="previewImage"
              :predefined-backgrounds="predefinedBackgrounds"
              :predefined-backgrounds-load-error="predefinedBackgroundsLoadError"
              :user-backgrounds="userBackgrounds"
              :is-loading-user-backgrounds="isLoadingUserBackgrounds"
              :user-backgrounds-load-error="userBackgroundsLoadError"
              :favorite-background-id="favoriteBackgroundId"
              :image-thumbnails="imageThumbnails"
              :is-generating-preview="isGeneratingPreview"
              :is-loading-predefined-backgrounds="isLoadingPredefinedBackgrounds"
              :folder-contents="folderContents"
              :is-processing="isProcessing"
              :previous-generated-images="previousGeneratedImages"
              :is-discogs-authenticated="isDiscogsAuthenticated"
              :is-discogs-authenticating="isDiscogsAuthenticating"
              :discogs-error="discogsError"
              :discogs-search-query="discogsSearchQuery"
              :discogs-search-results="discogsSearchResults"
              :is-searching-discogs="isSearchingDiscogs"
              :discogs-images="discogsImages"
              :is-loading-discogs-images="isLoadingDiscogsImages"
              :save-image-for-future-use="false"
              :save-background-for-future-use="false"
              :set-saved-background-as-default="false"
              :tracks-without-images-count="0"
              :has-selected-image="hasSelectedImage"
              :is-image-editor-open="isImageEditorOpen"
              :editing-image-src="editingImageSrc"
              @update:current-image-style="onUpdateCurrentImageStyle"
              @update:current-fit-mode="onUpdateCurrentFitMode"
              @update:current-image-url="onUpdateCurrentImageUrl"
              @update:current-background-url="onUpdateCurrentBackgroundUrl"
              @select-image-file="selectImageFile"
              @select-image-from-folder="selectImageFromFolder"
              @select-previous-generated-image="selectPreviousGeneratedImage"
              @select-background-file="selectBackgroundFile"
              @set-background-from-url="setBackgroundFromUrl"
              @set-background-predefined="setBackgroundPredefined"
              @set-favorite-background="setFavoriteBackground"
              @import-background="importBackground"
              @authenticate-discogs="authenticateDiscogsForImages"
              @update:discogs-search-query="onUpdateDiscogsSearchQuery"
              @discogs-search-now="searchDiscogsReleasesForArt"
              @select-discogs-release-for-image="selectDiscogsReleaseForImage"
              @select-discogs-image="selectDiscogsImageForCurrentTrack"
              @update:save-image-for-future-use="() => {}"
              @update:save-background-for-future-use="() => {}"
              @update:set-saved-background-as-default="() => {}"
              @apply-current-image-to-all-tracks="applyCurrentImageToAllTracks"
              @apply-image-from-url="applyImageFromUrl"
              @apply-previous-image-to-all-remaining-tracks="() => {}"
              @open-image-editor="openImageEditor"
              @close-image-editor="closeImageEditor"
              @handle-edited-image="handleEditedImage"
            />
            <div
              v-if="(hasPreviousTrack || hasNextTrack) && selectedAudioIndex !== null"
              class="flex items-center justify-between gap-4 pt-4 border-t border-border"
            >
              <Button
                v-if="hasPreviousTrack"
                variant="outline"
                size="sm"
                :disabled="isProcessing"
                @click="selectAudioTrack(selectedAudioIndex - 1)"
              >
                <ChevronLeft class="h-4 w-4 mr-1" />
                Previous track
              </Button>
              <span v-else />
              <Button
                v-if="hasNextTrack"
                variant="outline"
                size="sm"
                :disabled="isProcessing"
                @click="selectAudioTrack(selectedAudioIndex + 1)"
              >
                Next track
                <ChevronRight class="h-4 w-4 ml-1" />
              </Button>
            </div>
          </div>
        </template>

        <template #empty-state>
          <EmptyState
            icon="music"
            message="Select an audio track to assign an image"
          />
        </template>
      </TwoPanelLayout>

      <!-- Audio Options -->
      <Card class="p-6 border-2">
        <h3 class="text-lg font-semibold mb-4">Audio Options</h3>
        <AudioOptions
          :normalize-audio="normalizeAudio"
          :high-pass-filter="highPassFilter"
          :disabled="isProcessing"
          @update:normalize-audio="setNormalizeAudio"
          @update:high-pass-filter="setHighPassFilter"
        />
      </Card>

      <!-- Continue Button -->
      <div class="flex justify-end">
        <Button
          @click="currentStep = 'metadata'"
          :disabled="!canProceedToMetadata"
          size="lg"
        >
          Continue to Metadata
          <ChevronRight class="h-4 w-4 ml-2" />
        </Button>
      </div>
    </div>

    <!-- Step 3: Metadata Editing -->
    <div v-if="currentStep === 'metadata'" class="space-y-6">
      <TwoPanelLayout
        left-title="Videos"
        right-title="Video Metadata"
        :items-count="audioTracks.length"
        :selected-index="selectedAudioIndex"
      >
        <template #left-icon>
          <FileVideo class="h-5 w-5" />
        </template>

        <template #left-panel>
          <div class="flex flex-col h-full min-h-0">
            <div v-if="audioTracks.length === 0" class="flex-1 flex items-center justify-center">
              <EmptyState
                icon="video"
                message="No videos"
                hint="Select audio files first"
              />
            </div>

            <div v-else class="space-y-1 max-h-[600px] overflow-y-auto flex-1 min-h-0">
              <SelectableListItem
                v-for="(track, index) in audioTracks"
                :key="track.path"
                :is-selected="selectedAudioIndex === index"
                :status="videoStatus(track.path)"
                :disabled="isProcessing"
                @select="selectAudioTrack(index)"
                @remove="removeAudioTrack(index)"
              >
                <template #content>
                  <span class="text-sm font-medium truncate block">
                    {{ videoMetadata.get(track.path)?.title || "Untitled" }}
                  </span>
                </template>
              </SelectableListItem>
            </div>
          </div>
        </template>

        <template #right-panel>
          <div v-if="selectedVideoMetadata" ref="rightPanelStartRef" class="space-y-6">
            <VideoMetadataEditor
              :metadata="selectedVideoMetadata"
              :is-processing="isProcessing"
              :video-path="selectedVideoMetadata.videoPath"
              :show-video-preview="!!selectedVideoMetadata.videoPath"
              @update:metadata="updateVideoMetadata"
              @apply-template="applyDescriptionTemplate"
            >
              <template #after-description>
                <!-- Apply description to all videos -->
                <div
                  v-if="audioTracks.length > 1 && selectedVideoMetadata && selectedVideoMetadata.description.trim().length > 0"
                  class="mt-3"
                >
                  <Callout variant="action">
                    <div class="space-y-2">
                      <p class="text-sm leading-relaxed">
                        Apply the current description to all videos. You can still change individual videos later.
                      </p>
                      <button
                        @click="handleApplyDescriptionToAll"
                        :disabled="isProcessing"
                        :class="[
                          'inline-flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium transition-all duration-200 cursor-pointer',
                          'hover:bg-primary/20 hover:text-primary',
                          'disabled:opacity-50 disabled:cursor-not-allowed',
                          showApplyDescriptionSuccess 
                            ? 'text-green-700 dark:text-green-400 bg-green-500/20' 
                            : 'text-primary underline decoration-2 underline-offset-2 decoration-primary/50 hover:decoration-primary'
                        ]"
                      >
                        <CheckCircle2 v-if="showApplyDescriptionSuccess" class="h-4 w-4 shrink-0" />
                        <Copy v-else class="h-4 w-4 shrink-0" />
                        <span>{{ showApplyDescriptionSuccess ? 'Applied to all videos' : `Apply to all ${audioTracks.length} videos` }}</span>
                      </button>
                    </div>
                  </Callout>
                </div>
              </template>
              <template #after-tags>
                <!-- Apply tags to all videos -->
                <div
                  v-if="audioTracks.length > 1 && selectedVideoMetadata"
                  class="mt-3"
                >
                  <Callout variant="action">
                    <div class="space-y-2">
                      <p class="text-sm leading-relaxed">
                        Apply the current tags to all videos. You can still change individual videos later.
                      </p>
                      <button
                        @click="handleApplyTagsToAll"
                        :disabled="isProcessing"
                        :class="[
                          'inline-flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium transition-all duration-200 cursor-pointer',
                          'hover:bg-primary/20 hover:text-primary',
                          'disabled:opacity-50 disabled:cursor-not-allowed',
                          showApplyTagsSuccess
                            ? 'text-green-700 dark:text-green-400 bg-green-500/20'
                            : 'text-primary underline decoration-2 underline-offset-2 decoration-primary/50 hover:decoration-primary'
                        ]"
                      >
                        <CheckCircle2 v-if="showApplyTagsSuccess" class="h-4 w-4 shrink-0" />
                        <Copy v-else class="h-4 w-4 shrink-0" />
                        <span>{{ showApplyTagsSuccess ? 'Applied to all videos' : `Apply to all ${audioTracks.length} videos` }}</span>
                      </button>
                    </div>
                  </Callout>
                </div>
              </template>
              <template #after-category>
                <!-- Apply category to all videos -->
                <div
                  v-if="audioTracks.length > 1 && selectedVideoMetadata"
                  class="mt-3"
                >
                  <Callout variant="action">
                    <div class="space-y-2">
                      <p class="text-sm leading-relaxed">
                        Apply the current category to all videos. You can still change individual videos later.
                      </p>
                      <button
                        @click="handleApplyCategoryToAll"
                        :disabled="isProcessing"
                        :class="[
                          'inline-flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium transition-all duration-200 cursor-pointer',
                          'hover:bg-primary/20 hover:text-primary',
                          'disabled:opacity-50 disabled:cursor-not-allowed',
                          showApplyCategorySuccess
                            ? 'text-green-700 dark:text-green-400 bg-green-500/20'
                            : 'text-primary underline decoration-2 underline-offset-2 decoration-primary/50 hover:decoration-primary'
                        ]"
                      >
                        <CheckCircle2 v-if="showApplyCategorySuccess" class="h-4 w-4 shrink-0" />
                        <Copy v-else class="h-4 w-4 shrink-0" />
                        <span>{{ showApplyCategorySuccess ? 'Applied to all videos' : `Apply to all ${audioTracks.length} videos` }}</span>
                      </button>
                    </div>
                  </Callout>
                </div>
              </template>
              <template #after-privacy>
                <!-- Apply privacy & scheduling to all videos -->
                <div
                  v-if="audioTracks.length > 1 && selectedVideoMetadata"
                  class="mt-3"
                >
                  <Callout variant="action">
                    <div class="space-y-2">
                      <p class="text-sm leading-relaxed">
                        Apply the current privacy status and schedule to all videos. You can still change individual videos later.
                      </p>
                      <button
                        @click="handleApplyPrivacyToAll"
                        :disabled="isProcessing"
                        :class="[
                          'inline-flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium transition-all duration-200 cursor-pointer',
                          'hover:bg-primary/20 hover:text-primary',
                          'disabled:opacity-50 disabled:cursor-not-allowed',
                          showApplyPrivacySuccess
                            ? 'text-green-700 dark:text-green-400 bg-green-500/20'
                            : 'text-primary underline decoration-2 underline-offset-2 decoration-primary/50 hover:decoration-primary'
                        ]"
                      >
                        <CheckCircle2 v-if="showApplyPrivacySuccess" class="h-4 w-4 shrink-0" />
                        <Copy v-else class="h-4 w-4 shrink-0" />
                        <span>{{ showApplyPrivacySuccess ? 'Applied to all videos' : `Apply to all ${audioTracks.length} videos` }}</span>
                      </button>
                    </div>
                  </Callout>
                </div>
              </template>
            </VideoMetadataEditor>

            <!-- Navigation -->
            <div
              v-if="(hasPreviousTrack || hasNextTrack) && selectedAudioIndex !== null"
              class="flex items-center justify-between gap-4 pt-4 border-t border-border"
            >
              <Button
                v-if="hasPreviousTrack"
                variant="outline"
                size="sm"
                :disabled="isProcessing"
                @click="selectAudioTrack(selectedAudioIndex - 1)"
              >
                <ChevronLeft class="h-4 w-4 mr-1" />
                Previous video
              </Button>
              <span v-else />
              <Button
                v-if="hasNextTrack"
                variant="outline"
                size="sm"
                :disabled="isProcessing"
                @click="selectAudioTrack(selectedAudioIndex + 1)"
              >
                Next video
                <ChevronRight class="h-4 w-4 ml-1" />
              </Button>
            </div>
          </div>
        </template>

        <template #empty-state>
          <EmptyState
            icon="video"
            message="Select a video to edit metadata"
          />
        </template>
      </TwoPanelLayout>

      <!-- Continue Button -->
      <div class="flex justify-end">
        <Button
          @click="currentStep = 'options'"
          :disabled="!canProceedToOptions"
          size="lg"
        >
          Continue to Options
          <ChevronRight class="h-4 w-4 ml-2" />
        </Button>
      </div>
    </div>

    <!-- Step 4: Options -->
    <div v-if="currentStep === 'options'" class="space-y-6">
      <UnifiedUploadOptionsStep
        :save-videos-locally="saveVideosLocally"
        :output-folder="outputFolder"
        :is-processing="isProcessing"
        :is-output-folder-loading="isSelectingOutputFolder"
        :export-normalized-audio="exportNormalizedAudio"
        :normalize-audio="normalizeAudio"
        :audio-export-folder-value="audioExportFolder.outputFolder.value"
        :is-audio-export-folder-loading="audioExportFolder.isLoading"
        :is-you-tube-authenticated="youtube.isAuthenticated.value"
        :can-create-and-upload="canCreateAndUpload"
        :audio-tracks-count="audioTracks.length"
        @update:save-videos-locally="setSaveVideosLocally"
        @update:export-normalized-audio="setExportNormalizedAudio"
        @select-output-folder="handleSelectOutputFolder"
        @clear-output-folder="handleClearOutputFolder"
        @select-audio-export-folder="handleSelectAudioExportFolder"
        @clear-audio-export-folder="handleClearAudioExportFolder"
        @create-and-upload="handleCreateAndUpload"
      />
    </div>

    <!-- Step 5: Progress -->
    <div
      ref="progressSectionRef"
      v-if="currentStep === 'processing' && (isProcessing || unifiedPipeline.phases.length > 0)"
      class="space-y-4"
    >
      <UnifiedUploadProcessingStep
        :phases="unifiedPipeline.phases"
        :items-by-phase="unifiedPipeline.itemsByPhase"
        @cancel-phase="cancelOperation"
        @cancel-item="cancelItem"
      />
    </div>

    <!-- Status Messages -->
    <StatusAlert v-if="error" type="error" :message="error" />
    <StatusAlert v-if="success" type="success" :message="success" :auto-dismiss="true" />

    <!-- Modals -->
    <Modal :open="modals.showCancelConfirmModal.value" @close="modals.cancelCancel">
      <div class="space-y-4">
        <h3 class="text-lg font-semibold">Cancel Operation?</h3>
        <p class="text-sm text-muted-foreground">
          Processing is in progress. Are you sure you want to cancel? This will stop the current operation.
        </p>
        <div class="flex justify-end gap-2">
          <Button @click="modals.cancelCancel" variant="outline">No, Continue</Button>
          <Button @click="handleCancelConfirm" variant="destructive">Yes, Cancel</Button>
        </div>
      </div>
    </Modal>

    <Modal :open="modals.showYouTubeConnectModal.value" @close="modals.hideYouTubeConnection">
      <div class="space-y-4">
        <h3 class="text-lg font-semibold">YouTube Connection Required</h3>
        <p class="text-sm text-muted-foreground">
          You've enabled "Upload to YouTube" but you're not connected to YouTube. Please connect your YouTube account to proceed.
        </p>
        <div class="flex justify-end gap-2">
          <Button @click="modals.hideYouTubeConnection" variant="outline">Cancel</Button>
          <Button @click="connectYouTubeAndProceed" variant="default">
            <Unlock class="h-4 w-4 mr-2" />
            Connect & Continue
          </Button>
        </div>
      </div>
    </Modal>

    <!-- Bottom Navigation -->
    <BottomNavigation @back-to-menu="emit('back')" />
  </PageContainer>
</template>
