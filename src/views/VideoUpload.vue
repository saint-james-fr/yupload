<script setup lang="ts">
import { useVideoUploadFlow } from "../composables/useVideoUploadFlow";
import Button from "../components/ui/Button.vue";
import Modal from "../components/ui/Modal.vue";
import Alert from "../components/ui/Alert.vue";
import Card from "../components/ui/Card.vue";
import Callout from "../components/ui/Callout.vue";
import PageContainer from "../components/layout/PageContainer.vue";
import PageHeader from "../components/layout/PageHeader.vue";
import SourceStepView from "../components/SourceStepView.vue";
import FileSelectionCard from "../components/FileSelectionCard.vue";
import TwoPanelLayout from "../components/layout/TwoPanelLayout.vue";
import SelectableListItem from "../components/SelectableListItem.vue";
import EmptyState from "../components/EmptyState.vue";
import PipelineProgress from "../components/PipelineProgress.vue";
import StatusAlert from "../components/StatusAlert.vue";
import SectionHeader from "../components/ui/SectionHeader.vue";
import VideoPreview from "../components/VideoPreview.vue";
import VideoMetadataEditor from "../components/VideoMetadataEditor.vue";
import BottomNavigation from "../components/ui/BottomNavigation.vue";
import {
  Upload,
  FileVideo,
  Loader2,
  Lock,
  ChevronLeft,
  ChevronRight,
  CheckCircle2,
  Copy,
} from "lucide-vue-next";

const emit = defineEmits<{
  (e: "back"): void;
  (
    e: "navigate",
    view: "upload" | "settings" | "normalize" | "history" | "success",
    payload?: { from: "upload" | "normalize"; title: string; summary: string },
  ): void;
}>();

const flow = useVideoUploadFlow();

const handleBack = () => {
  if (flow.goBack()) emit("back");
};

const handleCancelConfirm = () => {
  flow.modals.confirmCancel(() => emit("back"));
};

const handleUploadClick = async () => {
  const result = await flow.uploadVideos();
  if (result) emit("navigate", "success", result);
};

// Re-export flow refs/computed/actions for template (flow.xxx)
const {
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
} = flow;
</script>

<template>
  <!-- Step 1: Select source only (centered, full viewport) -->
  <SourceStepView
    v-if="videos.length === 0"
    title="Select source"
    subtitle="Choose a folder or add videos to upload to YouTube."
    @back="handleBack"
  >
    <div class="w-full space-y-0">
      <FileSelectionCard
        :selected-folder="selectedFolder"
        :file-count="videos.length"
        :is-loading-folder="isLoadingFolder"
        :is-loading-files="isLoadingVideos"
        :is-processing="isUploading"
        size="large"
        @select-folder="selectFolder"
        @select-files="selectVideos"
      >
        <template #file-legend-icon>
          <FileVideo class="h-3.5 w-3.5" />
        </template>
      </FileSelectionCard>
      <Modal :open="noVideosFoundModalOpen" @close="dismissNoVideosModal">
        <div class="space-y-4">
          <h3 class="text-lg font-semibold">No Videos Found</h3>
          <p class="text-sm text-muted-foreground">
            The selected folder doesn’t contain any supported video files (e.g. MP4, MOV, WebM). Choose another folder
            or add files manually.
          </p>
          <div class="flex justify-end">
            <Button @click="dismissNoVideosModal" variant="default">OK</Button>
          </div>
        </div>
      </Modal>
    </div>
  </SourceStepView>

  <!-- Step 2: Rest of flow (source already selected) -->
  <PageContainer v-else>
    <!-- Header -->
    <PageHeader
      title="Video Upload"
      @back="handleBack"
    />

    <!-- Step Navigation (like unified flow) -->
    <div v-if="!isUploading" class="mb-6 flex items-center gap-2 text-sm">
      <button
        type="button"
        @click="clearSource"
        class="text-muted-foreground hover:text-foreground cursor-pointer"
      >
        1. Source
      </button>
      <ChevronRight class="h-4 w-4 text-muted-foreground shrink-0" />
      <span class="font-semibold text-primary">2. Metadata</span>
    </div>

    <!-- OAuth Error Alert -->
    <StatusAlert v-if="oauthError" type="error" :message="oauthError" />
    <StatusAlert v-if="discogsError" type="error" :message="discogsError" />
    <!-- Main Content: Two Panel Layout -->
    <TwoPanelLayout
      left-title="Videos"
      right-title="Video Metadata"
      :items-count="videos.length"
      :selected-index="selectedVideoIndex"
    >
      <template #left-icon>
        <FileVideo class="h-5 w-5" />
      </template>

      <template #left-panel>
        <div class="flex flex-col h-full min-h-0">
          <EmptyState
            v-if="videos.length === 0"
            icon="video"
            message="No videos selected"
            hint="Click 'Select Videos' to add videos"
          />

          <div v-else class="space-y-1 max-h-[600px] overflow-y-auto flex-1 min-h-0">
            <SelectableListItem
              v-for="(video, index) in videos"
              :key="video.videoPath"
              :is-selected="selectedVideoIndex === index"
              :status="videoStatus(video)"
              :disabled="isUploading"
              @select="selectVideo(index)"
              @remove="removeVideo(index)"
            >
              <template #content>
                <span class="text-sm font-medium truncate block">{{ video.title || "Untitled" }}</span>
              </template>
            </SelectableListItem>
          </div>

          <Button variant="ghost" size="sm" @click="clearSource" class="mt-3 w-full justify-center shrink-0">
            Change source
          </Button>
        </div>
      </template>

      <template #right-panel>
        <div v-if="selectedVideo" ref="rightPanelStartRef" class="space-y-6">
          <!-- Video Preview -->
          <div class="space-y-2">
            <SectionHeader label="Video preview" />
            <VideoPreview
              :file-path="selectedVideo.videoPath"
              :thumbnail="getVideoThumbnail(selectedVideo.videoPath)"
              label=""
              :compact="true"
            />
          </div>

          <VideoMetadataEditor
            :metadata="selectedVideo"
            :is-processing="isUploading"
            :force-private-uploads="forcePrivateUploads"
            @update:metadata="updateVideo"
            @apply-template="applyDescriptionTemplate"
            @select-discogs-release="handleDiscogsReleaseSelected"
          >
            <template #after-description>
              <!-- Apply description to all videos -->
              <div
                v-if="videos.length > 1 && selectedVideo && selectedVideo.description.trim().length > 0"
                class="mt-3"
              >
                <Callout variant="action">
                  <div class="space-y-2">
                    <p class="text-sm leading-relaxed">
                      Apply the current description to all videos. You can still change individual videos later.
                    </p>
                    <button
                      @click="handleApplyDescriptionToAll"
                      :disabled="isUploading"
                      :class="[
                        'inline-flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium transition-all duration-200 cursor-pointer',
                        'hover:bg-primary/20 hover:text-primary',
                        'disabled:opacity-50 disabled:cursor-not-allowed',
                        showApplyDescriptionSuccess
                          ? 'text-green-700 dark:text-green-400 bg-green-500/20'
                          : 'text-primary underline decoration-2 underline-offset-2 decoration-primary/50 hover:decoration-primary',
                      ]"
                    >
                      <CheckCircle2 v-if="showApplyDescriptionSuccess" class="h-4 w-4 shrink-0" />
                      <Copy v-else class="h-4 w-4 shrink-0" />
                      <span>{{
                        showApplyDescriptionSuccess ? "Applied to all videos" : `Apply to all ${videos.length} videos`
                      }}</span>
                    </button>
                  </div>
                </Callout>
              </div>
            </template>
            <template #after-tags>
              <!-- Apply tags to all videos -->
              <div v-if="videos.length > 1 && selectedVideo" class="mt-3">
                <Callout variant="action">
                  <div class="space-y-2">
                    <p class="text-sm leading-relaxed">
                      Apply the current tags to all videos. You can still change individual videos later.
                    </p>
                    <button
                      @click="handleApplyTagsToAll"
                      :disabled="isUploading"
                      :class="[
                        'inline-flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium transition-all duration-200 cursor-pointer',
                        'hover:bg-primary/20 hover:text-primary',
                        'disabled:opacity-50 disabled:cursor-not-allowed',
                        showApplyTagsSuccess
                          ? 'text-green-700 dark:text-green-400 bg-green-500/20'
                          : 'text-primary underline decoration-2 underline-offset-2 decoration-primary/50 hover:decoration-primary',
                      ]"
                    >
                      <CheckCircle2 v-if="showApplyTagsSuccess" class="h-4 w-4 shrink-0" />
                      <Copy v-else class="h-4 w-4 shrink-0" />
                      <span>{{
                        showApplyTagsSuccess ? "Applied to all videos" : `Apply to all ${videos.length} videos`
                      }}</span>
                    </button>
                  </div>
                </Callout>
              </div>
            </template>
            <template #after-category>
              <!-- Apply category to all videos -->
              <div v-if="videos.length > 1 && selectedVideo" class="mt-3">
                <Callout variant="action">
                  <div class="space-y-2">
                    <p class="text-sm leading-relaxed">
                      Apply the current category to all videos. You can still change individual videos later.
                    </p>
                    <button
                      @click="handleApplyCategoryToAll"
                      :disabled="isUploading"
                      :class="[
                        'inline-flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium transition-all duration-200 cursor-pointer',
                        'hover:bg-primary/20 hover:text-primary',
                        'disabled:opacity-50 disabled:cursor-not-allowed',
                        showApplyCategorySuccess
                          ? 'text-green-700 dark:text-green-400 bg-green-500/20'
                          : 'text-primary underline decoration-2 underline-offset-2 decoration-primary/50 hover:decoration-primary',
                      ]"
                    >
                      <CheckCircle2 v-if="showApplyCategorySuccess" class="h-4 w-4 shrink-0" />
                      <Copy v-else class="h-4 w-4 shrink-0" />
                      <span>{{
                        showApplyCategorySuccess ? "Applied to all videos" : `Apply to all ${videos.length} videos`
                      }}</span>
                    </button>
                  </div>
                </Callout>
              </div>
            </template>
            <template #after-privacy>
              <!-- Apply privacy & scheduling to all videos -->
              <div v-if="videos.length > 1 && selectedVideo" class="mt-3">
                <Callout variant="action">
                  <div class="space-y-2">
                    <p class="text-sm leading-relaxed">
                      Apply the current privacy status and schedule to all videos. You can still change individual
                      videos later.
                    </p>
                    <button
                      @click="handleApplyPrivacyToAll"
                      :disabled="isUploading"
                      :class="[
                        'inline-flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium transition-all duration-200 cursor-pointer',
                        'hover:bg-primary/20 hover:text-primary',
                        'disabled:opacity-50 disabled:cursor-not-allowed',
                        showApplyPrivacySuccess
                          ? 'text-green-700 dark:text-green-400 bg-green-500/20'
                          : 'text-primary underline decoration-2 underline-offset-2 decoration-primary/50 hover:decoration-primary',
                      ]"
                    >
                      <CheckCircle2 v-if="showApplyPrivacySuccess" class="h-4 w-4 shrink-0" />
                      <Copy v-else class="h-4 w-4 shrink-0" />
                      <span>{{
                        showApplyPrivacySuccess ? "Applied to all videos" : `Apply to all ${videos.length} videos`
                      }}</span>
                    </button>
                  </div>
                </Callout>
              </div>
            </template>
          </VideoMetadataEditor>

          <!-- Video navigation -->
          <div
            v-if="(hasPreviousVideo || hasNextVideo) && selectedVideoIndex !== null"
            class="flex items-center justify-between gap-4 pt-4 border-t border-border"
          >
            <Button
              v-if="hasPreviousVideo"
              variant="outline"
              size="sm"
              :disabled="isUploading"
              @click="selectVideo(selectedVideoIndex - 1)"
            >
              <ChevronLeft class="h-4 w-4 mr-1" />
              Previous video
            </Button>
            <span v-else />
            <Button
              v-if="hasNextVideo"
              variant="outline"
              size="sm"
              :disabled="isUploading"
              @click="selectVideo(selectedVideoIndex + 1)"
            >
              Next video
              <ChevronRight class="h-4 w-4 ml-1" />
            </Button>
          </div>
        </div>
      </template>

      <template #right-subtitle>
        {{ selectedVideo?.videoPath?.split("/").pop() }}
      </template>

      <template #empty-state>
        <EmptyState icon="video" message="Select a video to edit metadata" />
      </template>
    </TwoPanelLayout>

    <!-- Upload CTA: always full-width -->
    <Card class="p-6 border-2 w-full">
      <div class="space-y-4">
        <Alert v-if="!isAuthenticated" variant="destructive" class="w-full">
          <div class="flex items-center gap-2">
            <Lock class="h-4 w-4" />
            <p>Please connect YouTube before uploading videos</p>
          </div>
        </Alert>
        <Button
          v-else
          @click="handleUploadClick"
          :disabled="!canUpload"
          class="w-full text-lg py-6 px-8 font-semibold"
          size="lg"
          variant="default"
        >
          <Upload v-if="!isUploading" class="h-5 w-5 mr-2" />
          <Loader2 v-else class="h-5 w-5 mr-2 animate-spin" />
          <span v-if="isUploading">Uploading...</span>
          <span v-else>Upload Videos ({{ videos.length }})</span>
        </Button>
      </div>
    </Card>

    <!-- Upload Progress -->
    <div ref="progressSectionRef" v-if="isUploading || uploadProgress.size > 0">
      <PipelineProgress
        title="Upload pipeline"
        :phases="uploadPipeline.phases"
        :items-by-phase="uploadPipeline.itemsByPhase"
        :show-item-progress="true"
        :on-cancel-phase="cancelOperation"
        :on-cancel-item="cancelItem"
      />
    </div>

    <!-- Status Messages -->
    <StatusAlert v-if="error" type="error" :message="error" />
    <StatusAlert v-if="success" type="success" :message="success" :auto-dismiss="true" />

    <!-- Modal: Link Release to Other Videos -->
    <Modal :open="showLinkToOtherVideosModal && !!pendingRelease" @close="cancelLinkToOtherVideos" class="max-w-2xl">
      <div class="space-y-6">
        <div>
          <h2 class="text-2xl font-semibold mb-2">Link Release to Other Videos</h2>
          <p class="text-sm text-muted-foreground">
            You've linked <strong>{{ pendingRelease?.title }}</strong> to the current video. Would you like to link it
            to other videos as well?
          </p>
        </div>

        <div v-if="otherVideos.length > 0" class="space-y-3 max-h-96 overflow-y-auto border rounded-lg p-4 bg-muted/30">
          <div
            v-for="{ video, index } in otherVideos"
            :key="index"
            class="flex items-center gap-3 p-3 rounded-lg border bg-background hover:bg-accent transition-colors cursor-pointer"
            @click="toggleVideoForLinking(index)"
          >
            <input
              type="checkbox"
              :checked="selectedVideosForLinking.has(index)"
              @click.stop="toggleVideoForLinking(index)"
              class="w-4 h-4 rounded border-gray-300 text-primary focus:ring-primary"
            />
            <div class="flex-1 min-w-0">
              <p class="font-medium truncate">{{ video.title || video.videoPath.split("/").pop() }}</p>
              <p class="text-xs text-muted-foreground truncate">{{ video.videoPath.split("/").pop() }}</p>
            </div>
            <div v-if="video.discogsRelease" class="text-xs text-muted-foreground">Already linked</div>
          </div>
        </div>

        <div v-else class="text-center py-8 text-muted-foreground">
          <p>No other videos available</p>
        </div>

        <div class="flex items-center justify-end gap-3 pt-4 border-t">
          <Button @click="cancelLinkToOtherVideos" variant="outline"> Cancel </Button>
          <Button @click="applyReleaseToSelectedVideos" :disabled="selectedVideosForLinking.size === 0">
            Link to Selected Videos ({{ selectedVideosForLinking.size }})
          </Button>
        </div>
      </div>
    </Modal>

    <!-- Modal: Cancel Operation Confirmation -->
    <Modal :open="modals.showCancelConfirmModal.value" @close="modals.cancelCancel">
      <div class="space-y-4">
        <h3 class="text-lg font-semibold">Cancel Operation?</h3>
        <p class="text-sm text-muted-foreground">
          Video upload is in progress. Are you sure you want to cancel? This will stop the current operation.
        </p>
        <div class="flex justify-end gap-2">
          <Button @click="modals.cancelCancel" variant="outline">No, Continue</Button>
          <Button @click="handleCancelConfirm" variant="destructive">Yes, Cancel</Button>
        </div>
      </div>
    </Modal>

    <!-- Bottom Navigation -->
    <BottomNavigation @back-to-menu="emit('back')" />
  </PageContainer>
</template>
