<script setup lang="ts">
import { computed, onMounted } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useSettings } from "../composables/useSettings";
import Button from "../components/ui/Button.vue";
import SectionHeader from "../components/ui/SectionHeader.vue";
import SectionCard from "../components/ui/SectionCard.vue";
import StatusBadge from "../components/ui/StatusBadge.vue";
import Toggle from "../components/ui/Toggle.vue";
import SegmentedControl from "../components/ui/SegmentedControl.vue";
import Tabs from "../components/ui/Tabs.vue";
import PageContainer from "../components/layout/PageContainer.vue";
import PageHeader from "../components/layout/PageHeader.vue";
import StatusAlert from "../components/StatusAlert.vue";
import SelectableRadioCard from "../components/ui/SelectableRadioCard.vue";
import SettingsRow from "../components/ui/SettingsRow.vue";
import FolderSelectionZone from "../components/ui/FolderSelectionZone.vue";
import { YOUTUBE_CATEGORIES, YOUTUBE_PRIVACY_OPTIONS } from "../lib/constants";
import {
  XCircle,
  Loader2,
  DiscAlbum,
  Bell,
  Trash2,
  Sparkles,
  Maximize2,
  Layers,
  Globe,
  EyeOff,
  Lock,
  ListVideo,
  Music,
  Plus,
  Settings as SettingsIcon,
  Video,
  Upload,
} from "lucide-vue-next";
import Callout from "../components/ui/Callout.vue";
import Modal from "../components/ui/Modal.vue";
import ImageThumbnailCard from "../components/ui/ImageThumbnailCard.vue";
import BottomNavigation from "../components/ui/BottomNavigation.vue";
import type { ImageStyle, FitMode } from "../types";

const PRIVACY_ICONS = { public: Globe, unlisted: EyeOff, private: Lock, draft: Lock } as const;
const uploadDefaultPrivacyOptions = computed(() =>
  YOUTUBE_PRIVACY_OPTIONS.map((o) => ({ ...o, icon: PRIVACY_ICONS[o.value] }))
);

const emit = defineEmits<{
  (e: "back"): void;
}>();

const {
  notifyOnUploadComplete,
  isSavingNotifyPref,
  permissionState,
  saveNotifyOnUploadComplete,
  isRequestingPermission,
  requestNotificationPermission,
  isYouTubeAuthenticated,
  youtubeError,
  isYouTubeDisconnecting,
  isDiscogsAuthenticated,
  discogsError,
  isDiscogsDisconnecting,
  authenticateYouTube,
  authenticateDiscogs,
  disconnectYouTube,
  disconnectDiscogs,
  preferredAudioOutputPath,
  isSelectingAudioOutputPath,
  selectPreferredAudioOutputPath,
  clearPreferredAudioOutputPath,
  preferredVideoOutputPath,
  isSelectingVideoOutputPath,
  selectPreferredVideoOutputPath,
  clearPreferredVideoOutputPath,
  preferredAudioInputPath,
  isSelectingAudioInputPath,
  selectPreferredAudioInputPath,
  clearPreferredAudioInputPath,
  preferredImageInputPath,
  isSelectingImageInputPath,
  selectPreferredImageInputPath,
  clearPreferredImageInputPath,
  isExportingLogs,
  exportLogs,
  clearDataModalOpen,
  isClearingData,
  clearDataError,
  clearDataSuccess,
  openClearDataModal,
  closeClearDataModal,
  runClearAllData,
  preferences,
  creationDefaultNormalize,
  creationDefaultHighPass,
  creationDefaultImageStyle,
  creationDefaultFitMode,
  creationDefaultBackgroundPredefinedId,
  uploadDefaultPrivacy,
  uploadDefaultCategoryId,
  uploadDefaultPlaylistId,
  creationBackgrounds,
  isLoadingCreationBackgrounds,
  creationBackgroundsError,
  userBackgrounds,
  isLoadingUserBackgrounds,
  userBackgroundsError,
  importBackground,
  addTemplateBlock,
  removeTemplateBlock,
  moveTemplateBlock,
  onTemplateBlocksChange,
  resetTemplateToDefault,
  descriptionTemplatePreview,
  activeTab,
  initializeSettings,
} = useSettings();

onMounted(() => initializeSettings());

const goBack = () => emit("back");
</script>

<template>
  <PageContainer>
    <PageHeader title="Settings" description="Manage your account connections and preferences." @back="goBack" />

    <!-- Error Alerts -->
    <StatusAlert v-if="youtubeError" type="error" :message="youtubeError" />
    <StatusAlert v-if="discogsError" type="error" :message="discogsError" />
    <!-- Path errors are handled by individual composables -->
    <!-- Path success feedback removed for cleaner UX -->
    <StatusAlert v-if="clearDataError" type="error" :message="clearDataError" />
    <StatusAlert v-if="clearDataSuccess" type="success" :message="clearDataSuccess" />

    <!-- Tabs -->
    <Tabs
      v-model="activeTab"
      :options="[
        { value: 'general', label: 'General', icon: SettingsIcon },
        { value: 'creation', label: 'Video Creation', icon: Video },
        { value: 'upload', label: 'Upload', icon: Upload },
      ]"
      aria-label="Settings sections"
    />

    <!-- General Tab -->
    <div v-if="activeTab === 'general'" class="space-y-8">
      <!-- Section: Accounts -->
      <section class="space-y-2">
        <SectionHeader label="Accounts" description="YouTube and Discogs: upload videos and attach release metadata." />

        <!-- YouTube -->
        <SectionCard class="space-y-4">
          <div class="flex items-start justify-between gap-4 flex-wrap">
            <div class="flex items-center gap-3 min-w-0">
              <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-muted">
                <svg class="h-5 w-5" viewBox="0 0 24 24">
                  <path
                    fill="#FF0000"
                    d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"
                  />
                </svg>
              </div>
              <div>
                <h3 class="text-sm font-semibold text-foreground">YouTube</h3>
                <p class="text-xs text-muted-foreground mt-0.5">
                  Required for uploading videos. Connect with OAuth to authorize the app.
                </p>
              </div>
            </div>
            <StatusBadge :connected="isYouTubeAuthenticated" />
          </div>
          <div class="pt-2 border-t border-border">
            <Button
              v-if="!isYouTubeAuthenticated"
              @click="authenticateYouTube"
              variant="outline"
              size="sm"
              class="border-red-500/50 text-red-600 hover:bg-red-500/10 hover:border-red-500/70 dark:text-red-400"
            >
              <svg class="h-4 w-4 mr-2 shrink-0" viewBox="0 0 24 24">
                <path
                  fill="currentColor"
                  d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"
                />
              </svg>
              Connect YouTube
            </Button>
            <Button
              v-else
              @click="disconnectYouTube"
              variant="outline"
              size="sm"
              :disabled="isYouTubeDisconnecting"
              class="text-muted-foreground hover:text-destructive hover:border-destructive"
            >
              <Loader2 v-if="isYouTubeDisconnecting" class="h-4 w-4 mr-2 animate-spin" />
              <XCircle v-else class="h-4 w-4 mr-2" />
              Disconnect
            </Button>
          </div>
        </SectionCard>

        <!-- Discogs -->
        <SectionCard class="mt-4 space-y-4">
          <div class="flex items-start justify-between gap-4 flex-wrap">
            <div class="flex items-center gap-3 min-w-0">
              <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-muted">
                <DiscAlbum class="h-5 w-5 text-muted-foreground" stroke-width="2" />
              </div>
              <div>
                <h3 class="text-sm font-semibold text-foreground">Discogs</h3>
                <p class="text-xs text-muted-foreground mt-0.5">
                  Optional. Link releases to uploads for metadata and catalog links.
                </p>
              </div>
            </div>
            <StatusBadge :connected="isDiscogsAuthenticated" />
          </div>
          <div class="pt-2 border-t border-border">
            <Button v-if="!isDiscogsAuthenticated" @click="authenticateDiscogs" variant="outline" size="sm">
              <DiscAlbum class="h-4 w-4 mr-2" stroke-width="2" />
              Connect Discogs
            </Button>
            <Button
              v-else
              @click="disconnectDiscogs"
              variant="outline"
              size="sm"
              :disabled="isDiscogsDisconnecting"
              class="text-muted-foreground hover:text-destructive hover:border-destructive"
            >
              <Loader2 v-if="isDiscogsDisconnecting" class="h-4 w-4 mr-2 animate-spin" />
              <XCircle v-else class="h-4 w-4 mr-2" />
              Disconnect
            </Button>
          </div>
        </SectionCard>
      </section>

      <!-- Section: Output Paths -->
      <section class="space-y-2">
        <SectionHeader label="Output Paths" description="Default destinations for audio and video exports." />
        <SectionCard>
          <div class="space-y-6">
            <!-- Audio Output Path -->
            <div class="space-y-4">
              <div class="flex items-center gap-2">
                <Music class="h-4 w-4 text-muted-foreground" />
                <h3 class="text-sm font-medium text-foreground">Audio Output Path</h3>
              </div>
              <p class="text-xs text-muted-foreground -mt-2">
                Default folder for normalized audio files. Used when no specific folder is chosen.
              </p>
              <div class="pt-2">
                <FolderSelectionZone
                  :value="preferredAudioOutputPath"
                  label="Current audio output path"
                  empty-message="No audio output path set"
                  :is-loading="isSelectingAudioOutputPath"
                  @select="selectPreferredAudioOutputPath"
                  @clear="clearPreferredAudioOutputPath"
                />
              </div>
            </div>

            <!-- Video Output Path -->
            <div class="space-y-4">
              <div class="flex items-center gap-2">
                <Video class="h-4 w-4 text-muted-foreground" />
                <h3 class="text-sm font-medium text-foreground">Video Output Path</h3>
              </div>
              <p class="text-xs text-muted-foreground -mt-2">
                Default folder for created videos. Used when no specific folder is chosen.
              </p>
              <div class="pt-2">
                <FolderSelectionZone
                  :value="preferredVideoOutputPath"
                  label="Current video output path"
                  empty-message="No video output path set"
                  :is-loading="isSelectingVideoOutputPath"
                  @select="selectPreferredVideoOutputPath"
                  @clear="clearPreferredVideoOutputPath"
                />
              </div>
            </div>
          </div>
        </SectionCard>
      </section>

      <!-- Section: Input Paths -->
      <section class="space-y-2">
        <SectionHeader label="Input Paths" description="Default locations for browsing audio and image files." />
        <SectionCard>
          <div class="space-y-6">
            <!-- Audio Input Path -->
            <div class="space-y-4">
              <div class="flex items-center gap-2">
                <Music class="h-4 w-4 text-muted-foreground" />
                <h3 class="text-sm font-medium text-foreground">Audio Input Path</h3>
              </div>
              <p class="text-xs text-muted-foreground -mt-2">
                Default folder when browsing for audio files. Saves time navigating to frequently used locations.
              </p>
              <div class="pt-2">
                <FolderSelectionZone
                  :value="preferredAudioInputPath"
                  label="Current audio input path"
                  empty-message="No audio input path set"
                  :is-loading="isSelectingAudioInputPath"
                  @select="selectPreferredAudioInputPath"
                  @clear="clearPreferredAudioInputPath"
                />
              </div>
            </div>

            <!-- Image Input Path -->
            <div class="space-y-4">
              <div class="flex items-center gap-2">
                <Layers class="h-4 w-4 text-muted-foreground" />
                <h3 class="text-sm font-medium text-foreground">Image Input Path</h3>
              </div>
              <p class="text-xs text-muted-foreground -mt-2">
                Default folder when browsing for image files. Useful for artwork and background collections.
              </p>
              <div class="pt-2">
                <FolderSelectionZone
                  :value="preferredImageInputPath"
                  label="Current image input path"
                  empty-message="No image input path set"
                  :is-loading="isSelectingImageInputPath"
                  @select="selectPreferredImageInputPath"
                  @clear="clearPreferredImageInputPath"
                />
              </div>
            </div>
          </div>
        </SectionCard>
      </section>
      
      <!-- Section: Data -->
      <section class="space-y-2">
        <SectionHeader
          label="Data"
          description="Clear stored preferences, account tokens, upload history, and temporary files."
        />
        <SectionCard>
          <div class="space-y-4">
            <div class="flex items-start gap-3">
              <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-muted">
                <SettingsIcon class="h-5 w-5 text-muted-foreground" />
              </div>
              <div class="min-w-0 flex-1 space-y-3">
                <h3 class="text-sm font-medium text-foreground">Export logs</h3>
                <p class="text-sm text-muted-foreground">
                  Download application logs for debugging. Includes system information and operation history with sensitive data redacted.
                </p>
                <div class="pt-1">
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    :disabled="isExportingLogs"
                    @click="exportLogs"
                  >
                    <Loader2 v-if="isExportingLogs" class="h-4 w-4 mr-2 animate-spin" />
                    <Upload v-else class="h-4 w-4 mr-2" />
                    Export logs
                  </Button>
                </div>
              </div>
            </div>
            
            <div class="flex items-start gap-3">
              <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-muted">
                <Trash2 class="h-5 w-5 text-muted-foreground" />
              </div>
              <div class="min-w-0 flex-1 space-y-3">
                <h3 class="text-sm font-medium text-foreground">Clear all data</h3>
                <Callout variant="warning" class="mt-2">
                  Run this before uninstalling the app. It removes stored preferences, YouTube and Discogs tokens,
                  upload history, and temporary files. Then quit and move the app to Trash (or use your system’s
                  uninstall).
                </Callout>
                <div class="pt-1">
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    class="text-muted-foreground hover:text-destructive hover:border-destructive"
                    :disabled="isClearingData"
                    @click="openClearDataModal"
                  >
                    <Loader2 v-if="isClearingData" class="h-4 w-4 mr-2 animate-spin" />
                    <Trash2 v-else class="h-4 w-4 mr-2" />
                    Clear all data
                  </Button>
                </div>
              </div>
            </div>
          </div>
        </SectionCard>
      </section>
    </div>

    <!-- Video Creation Tab -->
    <div v-if="activeTab === 'creation'" class="space-y-8">
      <!-- Section: Creation defaults -->
      <section class="space-y-2">
        <SectionHeader
          label="Creation defaults"
          description="How new video projects start out for image style and normalization."
        />
        <SectionCard class="space-y-4">
          <div class="space-y-3">
            <SectionHeader label="Audio" />

            <div class="space-y-4">
              <SettingsRow
                :icon="Music"
                title="Normalize audio by default"
                description="Turn on loudness normalization when starting a new creation batch."
                icon-in-container
              >
                <Toggle v-model="creationDefaultNormalize" aria-label="Normalize audio by default in Creation" />
              </SettingsRow>

              <SettingsRow
                :icon="Music"
                title="Enable 30Hz high-pass with normalization"
                description="Tames subsonic rumble when normalization is enabled in Creation."
                icon-in-container
              >
                <Toggle
                  v-model="creationDefaultHighPass"
                  aria-label="Enable 30Hz high-pass when normalizing in Creation"
                />
              </SettingsRow>
            </div>
          </div>

          <div class="space-y-3 pt-2 border-t border-border">
            <SectionHeader label="Default image frame" />
            <div class="grid gap-2">
              <SelectableRadioCard
                value="automatic"
                :model-value="creationDefaultImageStyle"
                :icon="Sparkles"
                title="Automatic"
                description="Subject centered with a blurred copy behind. Good general-purpose default."
                @update:model-value="creationDefaultImageStyle = $event as ImageStyle"
              />
              <SelectableRadioCard
                value="fit16x9"
                :model-value="creationDefaultImageStyle"
                :icon="Maximize2"
                title="Fit to 16:9"
                description="Use only the main image, scaled into the frame."
                @update:model-value="creationDefaultImageStyle = $event as ImageStyle"
              />
              <SelectableRadioCard
                value="custom-background"
                :model-value="creationDefaultImageStyle"
                :icon="Layers"
                title="Custom Background + Centered Image"
                description="Use a separate background from the library or a file behind the main image."
                @update:model-value="creationDefaultImageStyle = $event as ImageStyle"
              />
              <SelectableRadioCard
                value="background-only"
                :model-value="creationDefaultImageStyle"
                :icon="Layers"
                title="Background only"
                description="Hide the centered image and use only a background template."
                @update:model-value="creationDefaultImageStyle = $event as ImageStyle"
              />
            </div>

            <div
              v-if="creationDefaultImageStyle === 'fit16x9'"
              class="pt-3 pl-4 border-l-2 border-primary/50 rounded-r-md space-y-2 bg-muted/20 -mx-1 px-4 py-3"
            >
              <SectionHeader label="Default fit mode" />
              <div class="space-y-2 pt-1">
                <SelectableRadioCard
                  value="contain"
                  :model-value="creationDefaultFitMode"
                  title="Contain"
                  description="Letterbox or pillarbox; full image visible."
                  class="flex-col gap-1.5 px-4 py-2.5"
                  @update:model-value="creationDefaultFitMode = $event as FitMode"
                />
                <SelectableRadioCard
                  value="cover"
                  :model-value="creationDefaultFitMode"
                  title="Cover"
                  description="Crop to fill; no empty bars."
                  class="flex-col gap-1.5 px-4 py-2.5"
                  @update:model-value="creationDefaultFitMode = $event as FitMode"
                />
              </div>
            </div>

            <!-- Default background selection - always visible -->
            <div class="pt-3 pl-4 border-l-2 border-primary/50 rounded-r-md space-y-3 bg-muted/20 -mx-1 px-4 py-3 mt-3">
              <SectionHeader label="Default background" />
              <p class="text-xs text-muted-foreground -mt-1">
                Choose a background to use when starting new creation batches.
              </p>

              <!-- Predefined backgrounds -->
              <div>
                <p class="text-xs font-medium text-muted-foreground mb-2">Predefined backgrounds</p>
                <div v-if="isLoadingCreationBackgrounds" class="mt-2 flex items-center gap-2">
                  <Loader2 class="h-4 w-4 animate-spin text-muted-foreground shrink-0" />
                  <span class="text-xs text-muted-foreground">Loading templates…</span>
                </div>
                <p v-else-if="creationBackgroundsError" class="text-xs text-destructive mt-2">
                  {{ creationBackgroundsError }}
                </p>
                <div
                  v-else-if="creationBackgrounds.length > 0"
                  class="mt-2 flex gap-2 overflow-x-auto overflow-y-hidden pb-1 -mx-0.5 px-0.5"
                >
                  <div v-for="bg in creationBackgrounds" :key="bg.id" class="shrink-0 w-32">
                    <ImageThumbnailCard
                      :src="convertFileSrc(bg.imagePath)"
                      :hover-src="convertFileSrc(bg.imagePath)"
                      :label="undefined"
                      :selected="creationDefaultBackgroundPredefinedId === bg.id"
                      aspect="video"
                      container-class="w-full"
                      @click="creationDefaultBackgroundPredefinedId = bg.id"
                    />
                  </div>
                </div>
                <p v-else class="text-xs text-muted-foreground mt-2">No predefined backgrounds available.</p>
              </div>

              <!-- User-imported backgrounds -->
              <div>
                <p class="text-xs font-medium text-muted-foreground mb-2">Your backgrounds</p>
                <div v-if="isLoadingUserBackgrounds" class="mt-2 flex items-center gap-2">
                  <Loader2 class="h-4 w-4 animate-spin text-muted-foreground shrink-0" />
                  <span class="text-xs text-muted-foreground">Loading your backgrounds…</span>
                </div>
                <p v-else-if="userBackgroundsError" class="text-xs text-destructive mt-2">
                  {{ userBackgroundsError }}
                </p>
                <div
                  v-else-if="userBackgrounds.length > 0"
                  class="mt-2 flex gap-2 overflow-x-auto overflow-y-hidden pb-1 -mx-0.5 px-0.5"
                >
                  <div v-for="bg in userBackgrounds" :key="bg.id" class="shrink-0 w-32">
                    <ImageThumbnailCard
                      :src="convertFileSrc(bg.imagePath)"
                      :hover-src="convertFileSrc(bg.imagePath)"
                      :label="undefined"
                      :selected="creationDefaultBackgroundPredefinedId === bg.id"
                      aspect="video"
                      container-class="w-full"
                      @click="creationDefaultBackgroundPredefinedId = bg.id"
                    />
                  </div>
                  <!-- Add button card -->
                  <Button
                    variant="outline"
                    @click="importBackground"
                    :disabled="isLoadingUserBackgrounds"
                    class="shrink-0 w-32 h-20 border-dashed flex items-center justify-center"
                    title="Import background"
                  >
                    <Plus class="h-5 w-5 text-muted-foreground" />
                  </Button>
                </div>
                <!-- Empty state with import button -->
                <div v-else class="mt-2">
                  <Button
                    variant="outline"
                    @click="importBackground"
                    :disabled="isLoadingUserBackgrounds"
                    class="w-full h-20 border-dashed flex flex-col items-center justify-center gap-2"
                  >
                    <Plus class="h-6 w-6 text-muted-foreground" />
                    <span class="text-xs text-muted-foreground font-medium">Import background</span>
                  </Button>
                </div>
              </div>
            </div>
          </div>
        </SectionCard>
      </section>
    </div>

    <!-- Upload Tab -->
    <div v-if="activeTab === 'upload'" class="space-y-8">
      <!-- Section: Upload defaults -->
      <section class="space-y-2">
        <SectionHeader
          label="Upload defaults"
          description="How new YouTube uploads are pre-filled before you edit metadata."
        />
        <SectionCard class="space-y-4">
          <div class="space-y-2">
            <p class="text-sm font-medium text-foreground flex items-center gap-2">
              <Globe class="h-4 w-4 text-muted-foreground" />
              Default privacy
            </p>
            <p class="text-xs text-muted-foreground -mt-1">
              Initial privacy when you add videos in the Upload screen. Verification limits may still force uploads to
              private.
            </p>
            <SegmentedControl
              class="mt-2"
              v-model="uploadDefaultPrivacy"
              :options="uploadDefaultPrivacyOptions"
              aria-label="Default upload privacy"
            >
              <template #option="{ option }">
                <component v-if="option.icon" :is="option.icon" class="h-4 w-4" />
                <span>{{ option.label }}</span>
              </template>
            </SegmentedControl>
          </div>

          <div class="space-y-2">
            <p class="text-sm font-medium text-foreground flex items-center gap-2">
              <ListVideo class="h-4 w-4 text-muted-foreground" />
              Default category
            </p>
            <p class="text-xs text-muted-foreground -mt-1">
              Category pre-selected for new uploads. You can still change it per video.
            </p>
            <select
              class="w-full mt-1 px-3 py-2 rounded-md border border-input bg-background text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
              :value="uploadDefaultCategoryId"
              @change="uploadDefaultCategoryId = ($event.target as HTMLSelectElement).value"
            >
              <option
                v-for="cat in YOUTUBE_CATEGORIES"
                :key="cat.value"
                :value="cat.value"
              >
                {{ cat.label }}
              </option>
            </select>
          </div>

          <div class="space-y-2">
            <p class="text-sm font-medium text-foreground flex items-center gap-2">
              <ListVideo class="h-4 w-4 text-muted-foreground" />
              Default playlist
            </p>
            <p class="text-xs text-muted-foreground -mt-1">
              Playlist to pre-select for new uploads when playlists are available.
            </p>
            <select
              class="w-full mt-1 px-3 py-2 rounded-md border border-input bg-background text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
              :value="uploadDefaultPlaylistId ?? ''"
              @change="uploadDefaultPlaylistId = ($event.target as HTMLSelectElement).value || null"
            >
              <option value="">None</option>
              <!-- Options are populated in Upload flow; Settings only stores the id -->
            </select>
          </div>
        </SectionCard>
      </section>

      <!-- Section: Description template -->
      <section class="space-y-2">
        <SectionHeader
          label="Description template"
          description="Build a reusable pattern for YouTube descriptions using tags and free text."
        />
        <SectionCard class="space-y-4">
          <p class="text-xs text-muted-foreground">
            Blocks are applied in order when you click <span class="font-medium">Apply template</span> in the Upload
            screen. Tag blocks pull from Discogs when a release is linked; text blocks stay exactly as written.
          </p>

          <!-- Template blocks list -->
          <div class="space-y-3">
            <div
              v-for="(block, index) in preferences.preferences?.description_template_blocks ?? []"
              :key="block.id"
              class="flex items-start gap-3 p-3 rounded-lg border bg-muted/30"
            >
              <div class="flex flex-col items-center gap-2 pt-1">
                <button
                  type="button"
                  class="text-xs text-muted-foreground hover:text-foreground disabled:opacity-30 cursor-pointer"
                  :disabled="index === 0"
                  @click="moveTemplateBlock(index, index - 1)"
                >
                  ↑
                </button>
                <button
                  type="button"
                  class="text-xs text-muted-foreground hover:text-foreground disabled:opacity-30 cursor-pointer"
                  :disabled="index === (preferences.preferences?.description_template_blocks?.length ?? 0) - 1"
                  @click="moveTemplateBlock(index, index + 1)"
                >
                  ↓
                </button>
              </div>
              <div class="flex-1 min-w-0 space-y-2">
                <div class="flex items-center gap-2">
                  <Toggle
                    v-model="block.enabled"
                    aria-label="Enable block"
                    @update:model-value="onTemplateBlocksChange()"
                  />
                  <span class="text-xs font-medium uppercase tracking-wider text-muted-foreground">
                    {{ block.type === "tag" ? "Tag" : "Text" }} block
                  </span>
                  <button
                    type="button"
                    class="ml-auto text-xs text-muted-foreground hover:text-destructive cursor-pointer"
                    @click="removeTemplateBlock(index)"
                  >
                    Remove
                  </button>
                </div>

                <div v-if="block.type === 'tag'" class="space-y-1">
                  <label class="text-xs text-muted-foreground">Tag</label>
                  <select
                    v-model="block.tagKey"
                    class="w-full mt-1 px-3 py-2 rounded-md border border-input bg-background text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
                    @change="onTemplateBlocksChange()"
                  >
                    <option value="artist">Artist(s)</option>
                    <option value="release">Release title</option>
                    <option value="year">Year</option>
                    <option value="label">Label(s)</option>
                    <option value="catalogNumber">Catalogue number</option>
                    <option value="genres">Genres</option>
                    <option value="styles">Styles</option>
                    <option value="discogsUrl">Discogs URL</option>
                  </select>
                </div>

                <div v-else class="space-y-1">
                  <label class="text-xs text-muted-foreground">Text</label>
                  <textarea
                    v-model="block.content"
                    rows="2"
                    class="w-full mt-1 px-3 py-2 rounded-md border border-input bg-background text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
                    placeholder="Custom lines of text. Use multiple lines if needed."
                    @input="onTemplateBlocksChange()"
                  />
                </div>
              </div>
            </div>

            <p
              v-if="
                !preferences.preferences?.description_template_blocks ||
                preferences.preferences.description_template_blocks.length === 0
              "
              class="text-xs text-muted-foreground"
            >
              No blocks yet. Start with a tag or a text block below.
            </p>
          </div>

          <!-- Preview -->
          <div class="mt-3 p-3 rounded-md border bg-muted/40">
            <p class="text-xs font-medium text-foreground">Preview</p>
            <p class="text-[11px] text-muted-foreground mt-1 whitespace-pre-wrap">
              {{ descriptionTemplatePreview }}
            </p>
          </div>

          <!-- Actions -->
          <div class="flex flex-wrap items-center gap-2 pt-2 border-t border-border">
            <Button type="button" variant="outline" size="sm" @click="addTemplateBlock('tag')">
              <Music class="h-4 w-4 mr-2" />
              Add tag block
            </Button>
            <Button type="button" variant="outline" size="sm" @click="addTemplateBlock('text')">
              <Loader2 class="h-4 w-4 mr-2" />
              Add text block
            </Button>
            <Button
              type="button"
              variant="ghost"
              size="sm"
              class="ml-auto text-xs text-muted-foreground"
              @click="resetTemplateToDefault"
            >
              Reset to default template
            </Button>
          </div>
        </SectionCard>
      </section>

      <!-- Section: Notifications -->
      <section class="space-y-2">
        <SectionHeader label="Notifications" description="Desktop notifications when long-running tasks finish." />
        <SectionCard>
          <div class="space-y-4">
            <div class="flex items-center gap-3">
              <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-muted">
                <Bell class="h-5 w-5 text-muted-foreground" />
              </div>
              <div class="flex-1 min-w-0">
                <h3 class="text-sm font-medium text-foreground">Notify when uploads finish</h3>
                <p class="text-xs text-muted-foreground mt-0.5">
                  Show a system notification when a YouTube upload batch completes.
                </p>
              </div>
              <Toggle
                v-model="notifyOnUploadComplete"
                :disabled="isSavingNotifyPref"
                aria-label="Notify when uploads finish"
                @update:model-value="saveNotifyOnUploadComplete"
              />
            </div>
            <div v-if="notifyOnUploadComplete && permissionState === 'denied'" class="pt-2 border-t border-border">
              <p class="text-xs text-muted-foreground mb-2">
                macOS may not have shown a prompt. Click below to request notification permission.
              </p>
              <Button
                type="button"
                variant="outline"
                size="sm"
                :disabled="isRequestingPermission"
                @click="requestNotificationPermission"
              >
                <Loader2 v-if="isRequestingPermission" class="h-4 w-4 mr-2 animate-spin" />
                Request notification permission
              </Button>
            </div>
          </div>
        </SectionCard>
      </section>
    </div>

    <!-- Modal: Confirm Clear all data -->
    <Modal :open="clearDataModalOpen" @close="closeClearDataModal">
      <div class="space-y-4">
        <h3 class="text-base font-semibold text-foreground">Clear all data?</h3>
        <p class="text-sm text-muted-foreground">
          This will remove stored preferences, account tokens, upload history, and temporary files. You’ll need to
          reconnect YouTube and Discogs and set preferences again.
        </p>
        <StatusAlert v-if="clearDataError" type="error" :message="clearDataError" />
        <div class="flex gap-2 justify-end pt-2">
          <Button type="button" variant="outline" size="sm" :disabled="isClearingData" @click="closeClearDataModal">
            Cancel
          </Button>
          <Button type="button" variant="destructive" size="sm" :disabled="isClearingData" @click="runClearAllData">
            <Loader2 v-if="isClearingData" class="h-4 w-4 mr-2 animate-spin" />
            Clear all data
          </Button>
        </div>
      </div>
    </Modal>

    <!-- Bottom Navigation -->
    <BottomNavigation @back-to-menu="emit('back')" />
  </PageContainer>
</template>
