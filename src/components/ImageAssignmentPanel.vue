<script setup lang="ts">
import type { ImageStyle, FitMode, FolderContents, AudioImagePair, AudioMetadata } from "../types";
import { convertFileSrc } from "@tauri-apps/api/core";
import { ref, watch, nextTick, computed } from "vue";
import Button from "./ui/Button.vue";
import Input from "./ui/Input.vue";
import Label from "./ui/Label.vue";
import SectionHeader from "./ui/SectionHeader.vue";
import SectionCard from "./ui/SectionCard.vue";
import ImageThumbnailCard from "./ui/ImageThumbnailCard.vue";
import Toggle from "./ui/Toggle.vue";
import Callout from "./ui/Callout.vue";
import SelectableRadioCard from "./ui/SelectableRadioCard.vue";
import SegmentedControl from "./ui/SegmentedControl.vue";
import { Image, Loader2, Sparkles, Maximize2, Layers, Link2, Library, Folder, DiscAlbum, Lock, Star, Music, Copy, CheckCircle2, Plus, Crop } from "lucide-vue-next";
import type { DiscogsSearchResult, DiscogsImage } from "../types";
import DiscogsSearchResults from "./DiscogsSearchResults.vue";
import ImageEditingModal from "./ImageEditingModal.vue";

const previewSectionRef = ref<HTMLElement | null>(null);
const showApplyToAllSuccess = ref(false);

const props = defineProps<{
  selectedAudioIndex: number | null;
  audioTracks: Array<{ path: string; name: string; metadata: AudioMetadata | null }>;
  currentImagePath: string | null;
  currentImageUrl: string;
  usePreviousImage: boolean;
  currentImageStyle: ImageStyle;
  currentFitMode: FitMode;
  currentBackgroundPath: string | null;
  currentBackgroundUrl: string;
  currentBackgroundPredefinedId: string | null;
  previewImage: string | null;
  predefinedBackgrounds: Array<{ id: string; label: string; imagePath: string }>;
  predefinedBackgroundsLoadError?: string | null;
  userBackgrounds: Array<{ id: string; label: string; imagePath: string }>;
  isLoadingUserBackgrounds: boolean;
  userBackgroundsLoadError?: string | null;
  favoriteBackgroundId: string | null;
  imageThumbnails: Map<string, string>;
  isGeneratingPreview: boolean;
  isLoadingPredefinedBackgrounds: boolean;
  folderContents: FolderContents | null;
  isProcessing: boolean;
  previousGeneratedImages: Array<{ trackName: string; pair: AudioImagePair; previewDataUrl: string | null }>;
  isDiscogsAuthenticated: boolean;
  isDiscogsAuthenticating: boolean;
  discogsError: string | null;
  discogsSearchQuery: string;
  discogsSearchResults: DiscogsSearchResult[];
  isSearchingDiscogs: boolean;
  discogsImages: DiscogsImage[];
  isLoadingDiscogsImages: boolean;
  saveImageForFutureUse: boolean;
  saveBackgroundForFutureUse: boolean;
  setSavedBackgroundAsDefault: boolean;
  tracksWithoutImagesCount: number;
  // Image editor props
  hasSelectedImage: boolean;
  isImageEditorOpen: boolean;
  editingImageSrc: string;
}>();

const canApplyToAll = computed(() => {
  if (props.usePreviousImage || props.audioTracks.length <= 1) return false;
  
  // For background-only style, check if there's a background
  if (props.currentImageStyle === 'background-only') {
    return !!(props.currentBackgroundPath || props.currentBackgroundUrl || props.currentBackgroundPredefinedId);
  }
  
  // For other styles, check if there's a main image
  return !!(props.currentImagePath || props.currentImageUrl);
});

const previewUrl = (path: string) => convertFileSrc(path);

const selectedTrackMetadata = computed(() => {
  if (props.selectedAudioIndex === null) return null;
  return props.audioTracks[props.selectedAudioIndex]?.metadata ?? null;
});

const hasAlbumArt = computed(() => {
  const metadata = selectedTrackMetadata.value;
  if (!metadata?.image_data_url) return false;
  // Only show album art if we have proper metadata (artist or title)
  // This prevents showing placeholder/random images when metadata is missing
  return !!(metadata.artist || metadata.title);
});

const imageSourceOptions = computed(() => {
  const options: Array<{ value: string; label: string; icon?: any; disabled?: boolean }> = [];
  
  if (hasAlbumArt.value) {
    options.push({ value: "album-art", label: "Album Art", icon: Music });
  }
  
  if (props.folderContents && props.folderContents.images.length > 0) {
    options.push({ value: "folder", label: "Folder", icon: Folder });
  }
  
  options.push({ value: "discogs", label: "Discogs", icon: DiscAlbum });
  options.push({ value: "url", label: "URL", icon: Link2 });
  options.push({ value: "file", label: "File", icon: Image });
  
  return options;
});

const activeImageSourceTab = ref<"file" | "folder" | "url" | "discogs" | "album-art">("file");

// Initialize active tab to first available option when no image is selected
watch(
  imageSourceOptions,
  (options) => {
    // Only set if no image is currently selected and tab is still default "file"
    if (!props.currentImagePath && !props.currentImageUrl && activeImageSourceTab.value === "file" && options.length > 0) {
      activeImageSourceTab.value = options[0].value as typeof activeImageSourceTab.value;
    }
  },
  { immediate: true }
);

const canSaveImageAsBackground = computed(() => {
  return !!(props.currentImagePath && (props.currentImageStyle === 'custom-background' || props.currentImageStyle === 'background-only'));
});

const isBackgroundOnlyStyle = computed(() => {
  return props.currentImageStyle === 'background-only';
});

const emit = defineEmits<{
  (e: "update:currentImageStyle", value: ImageStyle): void;
  (e: "update:currentFitMode", value: FitMode): void;
  (e: "update:currentImageUrl", value: string): void;
  (e: "applyImageFromUrl"): void;
  (e: "update:currentBackgroundUrl", value: string): void;
  (e: "selectImageFile"): void;
  (e: "selectImageFromFolder", path: string): void;
  (e: "selectPreviousGeneratedImage", pair: AudioImagePair): void;
  (e: "selectBackgroundFile"): void;
  (e: "setBackgroundFromUrl"): void;
  (e: "setBackgroundPredefined", id: string): void;
  (e: "setFavoriteBackground", id: string | null): void;
  (e: "importBackground"): void;
  (e: "authenticateDiscogs"): void;
  (e: "update:discogsSearchQuery", value: string): void;
  (e: "discogsSearchNow"): void;
  (e: "selectDiscogsReleaseForImage", result: DiscogsSearchResult): void;
  (e: "selectDiscogsImage", url: string): void;
  (e: "update:saveImageForFutureUse", value: boolean): void;
  (e: "update:saveBackgroundForFutureUse", value: boolean): void;
  (e: "update:setSavedBackgroundAsDefault", value: boolean): void;
  // Image editor events
  (e: "openImageEditor"): void;
  (e: "closeImageEditor"): void;
  (e: "handleEditedImage", editedDataUrl: string): void;
  (e: "applyCurrentImageToAllTracks"): void;
  (e: "applyPreviousImageToAllRemainingTracks", pair: AudioImagePair): void;
}>();

const handleApplyToAll = () => {
  emit("applyCurrentImageToAllTracks");
  showApplyToAllSuccess.value = true;
  setTimeout(() => {
    showApplyToAllSuccess.value = false;
  }, 2000);
};

const handleImageStyleUpdate = (value: string | number) => {
  emit("update:currentImageStyle", value as ImageStyle);
};

const handleFitModeUpdate = (value: string | number) => {
  emit("update:currentFitMode", value as FitMode);
};

/** After paste, apply URL once the input has the pasted value (defer so v-model updates first). */
const onUrlPaste = () => {
  setTimeout(() => emit("applyImageFromUrl"), 0);
};

// Ensure the folder tab is only active when folder images are available
watch(
  () => !!(props.folderContents && props.folderContents.images.length > 0),
  (hasFolderImages, _oldVal, onCleanup) => {
    // Prefer folder tab when images are available and nothing else is explicitly chosen
    if (!props.currentImagePath && !props.currentImageUrl) {
      if (hasFolderImages && activeImageSourceTab.value === "file") {
        activeImageSourceTab.value = "folder";
      }
    }

    if (!hasFolderImages && activeImageSourceTab.value === "folder") {
      activeImageSourceTab.value = props.currentImageUrl && !props.currentImagePath 
        ? "url" 
        : (hasAlbumArt.value ? "album-art" : "file");
    }

    // Initialize based on props on first run
    if (hasFolderImages && props.currentImagePath && !props.currentImageUrl) {
      activeImageSourceTab.value = "folder";
    } else if (!hasFolderImages && props.currentImageUrl && !props.currentImagePath) {
      activeImageSourceTab.value = "url";
    }

    onCleanup?.(() => {
      // no-op, needed only for proper type inference in some tooling
    });
  },
  { immediate: true }
);

// When preview starts generating, scroll so the preview section is in view
watch(
  () => props.isGeneratingPreview,
  (val, oldVal) => {
    if (val && !oldVal) {
      nextTick(() => {
        previewSectionRef.value?.scrollIntoView({ behavior: "smooth", block: "start" });
      });
    }
  }
);
</script>

<template>
  <div class="space-y-6">
    <!-- Section: Preview (16:9, full width) -->
    <section ref="previewSectionRef" class="space-y-2">
      <SectionHeader
        label="Image Preview"
        description="Your image as it will appear in the final 16:9 video."
      />
      <div class="w-full rounded-lg overflow-hidden border bg-muted/30 shadow-sm aspect-video">
        <div v-if="isGeneratingPreview" class="w-full h-full bg-muted flex items-center justify-center">
          <div class="text-center space-y-2">
            <Loader2 class="h-8 w-8 animate-spin text-muted-foreground mx-auto" />
            <p class="text-sm text-muted-foreground">
              {{ currentImageUrl && !currentImagePath ? 'Loading image from URL…' : 'Assembling preview…' }}
            </p>
          </div>
        </div>
        <img v-else-if="previewImage" :src="previewImage" alt="Preview" class="w-full h-full object-contain" />
        <div v-else class="w-full h-full bg-muted flex items-center justify-center border-dashed border-2 border-muted-foreground/20">
          <div class="text-center space-y-2 px-4">
            <Image class="h-10 w-10 text-muted-foreground/40 mx-auto" />
            <p class="text-sm text-muted-foreground">No image selected</p>
            <p class="text-xs text-muted-foreground/60">Choose an image below to see a live preview</p>
          </div>
        </div>
      </div>
    </section>

    <!-- Section: Apply to all tracks (when multiple tracks and can apply) -->
    <section
      v-if="!usePreviousImage && audioTracks.length > 1 && canApplyToAll"
      class="space-y-2"
    >
      <Callout variant="action">
        <div class="space-y-2">
          <p class="text-sm leading-relaxed">
            Apply the current track's image, style, and background settings to all tracks. You can still change individual tracks later.
          </p>
          <button
            @click="handleApplyToAll"
            :disabled="isProcessing"
            :class="[
              'inline-flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium transition-all duration-200 cursor-pointer',
              'hover:bg-primary/20 hover:text-primary',
              'disabled:opacity-50 disabled:cursor-not-allowed',
              showApplyToAllSuccess 
                ? 'text-green-700 dark:text-green-400 bg-green-500/20' 
                : 'text-primary underline decoration-2 underline-offset-2 decoration-primary/50 hover:decoration-primary'
            ]"
          >
            <CheckCircle2 v-if="showApplyToAllSuccess" class="h-4 w-4 shrink-0" />
            <Copy v-else class="h-4 w-4 shrink-0" />
            <span>{{ showApplyToAllSuccess ? 'Applied to all tracks' : `Apply to all ${audioTracks.length} tracks` }}</span>
          </button>
        </div>
      </Callout>
    </section>

    <!-- Section: Use previous generated images (visible once any track has an image, but not on first track) -->
    <section
      v-if="!usePreviousImage && selectedAudioIndex !== 0 && (previousGeneratedImages.length > 0 || currentImagePath || currentImageUrl)"
      class="space-y-2"
    >
      <SectionHeader
        label="Use previous generated images"
        description="Reuse an image already assigned to another track."
      />
      <SectionCard padding="compact">
        <div v-if="previousGeneratedImages.length > 0">
          <p class="text-xs text-muted-foreground mb-2">Pick an image already assigned to another track.</p>
          <div class="flex gap-2 overflow-x-auto overflow-y-hidden pb-1 -mx-0.5 px-0.5">
            <div
              v-for="(item, i) in previousGeneratedImages"
              :key="i"
              class="shrink-0 w-48 flex flex-col gap-1.5"
            >
              <ImageThumbnailCard
                :src="item.previewDataUrl"
                :hover-src="item.previewDataUrl"
                :label="item.trackName"
                :selected="currentImagePath === (item.pair.imagePath ?? null) && currentImageUrl === (item.pair.imageUrl ?? '')"
                :disabled="isProcessing"
                :loading="!item.previewDataUrl"
                aspect="video"
                container-class="w-full"
                @click="emit('selectPreviousGeneratedImage', item.pair)"
              />
              <Button
                v-if="tracksWithoutImagesCount > 0"
                @click.stop="emit('applyPreviousImageToAllRemainingTracks', item.pair)"
                :disabled="isProcessing"
                variant="ghost"
                size="sm"
                class="w-full text-xs h-auto py-1.5"
              >
                <Copy class="h-3 w-3 mr-1.5" />
                Assign to all {{ tracksWithoutImagesCount }} track{{ tracksWithoutImagesCount === 1 ? '' : 's' }} left
              </Button>
            </div>
          </div>
        </div>
        <p v-else class="text-xs text-muted-foreground">
          Assign an image to another track — it will appear here when you select that track.
        </p>
      </SectionCard>
    </section>

    <!-- Section: Image source (when not using previous and not background-only) -->
    <section v-if="!usePreviousImage && currentImageStyle !== 'background-only'" class="space-y-2">
      <SectionHeader
        label="Image source"
        description="Main image for this track (file, folder, URL, or Discogs release art)."
      />
      <SectionCard padding="compact" class="space-y-4">
        <!-- Image source tabs -->
        <div class="border-b border-border pb-2 mb-2">
          <SegmentedControl
            :model-value="activeImageSourceTab"
            :options="imageSourceOptions"
            :disabled="isProcessing"
            aria-label="Image source"
            @update:model-value="activeImageSourceTab = ($event ?? 'file') as typeof activeImageSourceTab"
          />
        </div>

        <!-- Album Art tab -->
        <div v-if="activeImageSourceTab === 'album-art' && hasAlbumArt">
          <Label class="text-sm font-medium text-foreground flex items-center gap-2">
            <Music class="h-4 w-4 text-muted-foreground" />
            From audio metadata
          </Label>
          <div class="mt-1.5 space-y-3">
            <div class="flex items-center justify-center bg-background rounded-lg overflow-hidden border p-4">
              <img 
                v-if="selectedTrackMetadata?.image_data_url"
                :src="selectedTrackMetadata.image_data_url" 
                alt="Album art" 
                class="max-w-full max-h-48 object-contain"
              />
            </div>
            <p class="text-xs text-muted-foreground">
              Album art is automatically used when available. Extracted from the audio file's metadata tags.
            </p>
          </div>
        </div>

        <!-- File tab -->
        <div v-if="activeImageSourceTab === 'file'">
          <Label class="text-sm font-medium text-foreground flex items-center gap-2">
            <Image class="h-4 w-4 text-muted-foreground" />
            From your computer
          </Label>
          
          <!-- Image file selection zone -->
          <div
            @click="emit('selectImageFile')"
            :class="[
              'group flex items-center gap-3 p-4 rounded-lg border-2 transition-all cursor-pointer mt-1.5',
              currentImagePath
                ? 'border-border bg-muted/20 hover:border-primary/40 hover:bg-muted/30'
                : 'border-dashed border-muted-foreground/30 bg-muted/10 hover:border-muted-foreground/50 hover:bg-muted/20',
              isProcessing && 'opacity-50 pointer-events-none'
            ]"
            role="button"
            tabindex="0"
            :aria-label="currentImagePath ? 'Change image file' : 'Choose image file'"
            @keydown.enter.prevent="emit('selectImageFile')"
            @keydown.space.prevent="emit('selectImageFile')"
          >
            <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-muted">
              <Image class="h-5 w-5 text-muted-foreground" />
            </div>
            <div class="flex-1 min-w-0">
              <template v-if="currentImagePath">
                <p class="text-sm font-medium text-foreground truncate">
                  {{ currentImagePath.split("/").pop() }}
                </p>
                <p class="text-xs text-muted-foreground">
                  Click to change image file
                </p>
              </template>
              <template v-else>
                <p class="text-sm font-medium text-foreground">
                  Choose image file
                </p>
                <p class="text-xs text-muted-foreground">
                  Click to select an image from your computer
                </p>
              </template>
            </div>
          </div>
        </div>

        <!-- Folder tab -->
        <div v-if="activeImageSourceTab === 'folder' && folderContents && folderContents.images.length > 0">
          <Label class="text-sm font-medium text-foreground flex items-center gap-2">
            <Folder class="h-4 w-4 text-muted-foreground" />
            From selected folder
          </Label>
          <p class="text-xs text-muted-foreground mt-0.5">Images found in the folder you picked.</p>
          <div class="mt-2 grid grid-cols-2 gap-2 max-h-64 overflow-y-auto pr-0.5">
            <ImageThumbnailCard
              v-for="img in folderContents.images"
              :key="img.path"
              :src="imageThumbnails.get(img.path) || null"
              :hover-src="imageThumbnails.get(img.path) || null"
              :label="img.name"
              :selected="currentImagePath === img.path"
              :disabled="isProcessing"
              :loading="!imageThumbnails.has(img.path)"
              aspect="video"
              @click="emit('selectImageFromFolder', img.path)"
            />
          </div>
        </div>

        <!-- URL tab -->
        <div v-if="activeImageSourceTab === 'url'">
          <Label class="text-sm font-medium text-foreground flex items-center gap-2">
            <Link2 class="h-4 w-4 text-muted-foreground" />
            Or paste an image URL
          </Label>
          <p class="text-xs text-muted-foreground mt-1">Use a direct link to an image file (e.g. .jpg, .png).</p>
          <Input
            :model-value="currentImageUrl"
            @update:model-value="emit('update:currentImageUrl', $event)"
            placeholder="https://example.com/image.jpg"
            :disabled="isProcessing"
            class="mt-1.5"
            @blur="emit('applyImageFromUrl')"
            @paste="onUrlPaste"
          />
        </div>

        <!-- Discogs tab -->
        <div v-if="activeImageSourceTab === 'discogs'">
          <div v-if="!isDiscogsAuthenticated" class="p-3 rounded-lg border bg-muted/30 flex items-start gap-3">
            <Lock class="h-4 w-4 text-muted-foreground mt-0.5 shrink-0" />
            <div class="flex-1 min-w-0">
              <p class="text-sm font-medium">Use artwork from Discogs releases</p>
              <p class="text-xs text-muted-foreground mt-1">
                Connect Discogs to search releases by track title and pick cover art as your video image.
              </p>
            </div>
            <Button
              size="sm"
              variant="outline"
              :disabled="isProcessing"
              @click="emit('authenticateDiscogs')"
            >
              <DiscAlbum class="h-4 w-4 mr-2" stroke-width="2" />
              Connect
            </Button>
          </div>

          <div v-else class="space-y-3">
            <div>
              <Label class="text-sm font-medium text-foreground flex items-center gap-2">
                <DiscAlbum class="h-4 w-4 text-muted-foreground" />
                Search Discogs release
              </Label>
              <p class="text-xs text-muted-foreground mt-0.5">
                Prefilled from the audio title. Pick a release, then choose one of its images below.
              </p>
              <div class="relative mt-2">
                <Input
                  :model-value="discogsSearchQuery"
                  @update:model-value="emit('update:discogsSearchQuery', $event)"
                  placeholder="Search by artist, title…"
                  class="flex-1"
                  @keyup.enter="emit('discogsSearchNow')"
                  :disabled="isProcessing || isSearchingDiscogs"
                />
                <Loader2
                  v-if="isSearchingDiscogs"
                  class="absolute right-3 top-1/2 -translate-y-1/2 h-4 w-4 animate-spin text-muted-foreground"
                />
              </div>
            </div>

            <!-- Discogs search results -->
            <DiscogsSearchResults
              :results="discogsSearchResults"
              :is-disabled="isProcessing || isLoadingDiscogsImages"
              :is-loading="isSearchingDiscogs || isLoadingDiscogsImages"
              @select="emit('selectDiscogsReleaseForImage', $event)"
            />

            <!-- Images from selected release -->
            <div class="space-y-2">
              <div class="flex items-center justify-between gap-2">
                <Label class="text-sm font-medium text-foreground">Images from release</Label>
                <span v-if="isLoadingDiscogsImages" class="inline-flex items-center gap-1 text-xs text-muted-foreground">
                  <Loader2 class="h-3 w-3 animate-spin" />
                  Loading images…
                </span>
              </div>
              <p v-if="!discogsImages.length && !isLoadingDiscogsImages" class="text-xs text-muted-foreground">
                Select a release above to see its artwork.
              </p>
              <div
                v-else
                class="mt-1 grid grid-cols-2 gap-3 max-h-[500px] overflow-y-auto pr-0.5"
              >
                <ImageThumbnailCard
                  v-for="img in discogsImages"
                  :key="img.uri"
                  :src="img.uri150 || img.uri"
                  :hover-src="img.uri"
                  :label="img.type || undefined"
                  :selected="currentImageUrl === img.uri"
                  :disabled="isProcessing"
                  :loading="false"
                  aspect="square"
                  @click="emit('selectDiscogsImage', img.uri)"
                />
              </div>
            </div>
          </div>
        </div>
      </SectionCard>
    </section>

    <!-- Section: Image Editing (when image is selected) -->
    <section v-if="hasSelectedImage && !usePreviousImage && currentImageStyle !== 'background-only'" class="space-y-2">
      <SectionCard padding="compact">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div class="flex items-center justify-center w-8 h-8 rounded-md bg-muted">
              <Crop class="h-4 w-4 text-muted-foreground" />
            </div>
            <div>
              <p class="text-sm font-medium text-foreground">Edit Image</p>
              <p class="text-xs text-muted-foreground">Crop and adjust your image before processing</p>
            </div>
          </div>
          <Button
            variant="outline"
            size="sm"
            @click="emit('openImageEditor')"
            :disabled="isProcessing"
            class="flex items-center gap-2"
          >
            <Crop class="h-4 w-4" />
            Edit
          </Button>
        </div>
      </SectionCard>
    </section>

    <!-- Section: Frame & layout (when not using previous) -->
    <section v-if="!usePreviousImage" class="space-y-2">
      <SectionHeader
        label="Frame & layout"
        description="How your image is turned into a 16:9 frame."
      />
      <SectionCard padding="compact" class="space-y-4">
        <div>
          <Label class="text-sm font-medium text-foreground">Frame style</Label>
          <div class="mt-3 grid gap-2">
            <SelectableRadioCard
              value="automatic"
              :model-value="currentImageStyle"
              :icon="Sparkles"
              title="Automatic"
              description="Subject centered, same image blurred behind. Best for portraits and product shots."
              :disabled="isProcessing"
              @update:model-value="handleImageStyleUpdate($event)"
            />
            <SelectableRadioCard
              value="fit16x9"
              :model-value="currentImageStyle"
              :icon="Maximize2"
              title="Fit to 16:9"
              description="Only your image, scaled to fit the frame. Choose letterbox or crop below."
              :disabled="isProcessing"
              @update:model-value="handleImageStyleUpdate($event)"
            />
            <SelectableRadioCard
              value="custom-background"
              :model-value="currentImageStyle"
              :icon="Layers"
              title="Custom Background + Centered Image"
              description="Subject centered, with a second image or preset behind. Pick a background below."
              :disabled="isProcessing"
              @update:model-value="handleImageStyleUpdate($event)"
            />
            <SelectableRadioCard
              value="background-only"
              :model-value="currentImageStyle"
              :icon="Layers"
              title="Background only"
              description="Use only a predefined background with no centered image."
              :disabled="isProcessing"
              @update:model-value="handleImageStyleUpdate($event)"
            />
          </div>
        </div>

        <!-- Nested: Fit mode (when fit16x9) -->
        <div v-if="currentImageStyle === 'fit16x9'" class="pt-3 pl-4 border-l-2 border-primary/50 rounded-r-md space-y-2 bg-muted/20 -mx-1 px-4 py-3">
          <Label class="text-sm font-medium text-muted-foreground">Fit mode</Label>
          <p class="text-xs text-muted-foreground -mt-1">Letterbox, pillarbox, or crop to fill.</p>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2 pt-1">
            <SelectableRadioCard
              value="contain"
              :model-value="currentFitMode"
              title="Contain"
              description="Letterbox or pillarbox; full image visible"
              :disabled="isProcessing"
              class="flex-col gap-1.5 px-4 py-2.5"
              @update:model-value="handleFitModeUpdate($event)"
            >
              <template #content>
                <!-- Contain: 16:9 frame with image fitted inside (letterbox/pillarbox) -->
                <svg viewBox="0 0 64 36" class="w-full h-14 shrink-0 text-muted-foreground/70" aria-hidden="true">
                  <rect x="1" y="1" width="62" height="34" rx="1.5" fill="none" stroke="currentColor" stroke-width="1.5" />
                  <rect x="14" y="1" width="36" height="34" fill="currentColor" opacity="0.4" />
                </svg>
                <span class="text-sm font-semibold text-foreground block text-center">Contain</span>
                <span class="text-xs text-muted-foreground/80 mt-0.5 block leading-relaxed text-center">Letterbox or pillarbox; full image visible</span>
              </template>
            </SelectableRadioCard>
            <SelectableRadioCard
              value="cover"
              :model-value="currentFitMode"
              title="Cover"
              description="Crop to fill; no empty bars"
              :disabled="isProcessing"
              class="flex-col gap-1.5 px-4 py-2.5"
              @update:model-value="handleFitModeUpdate($event)"
            >
              <template #content>
                <!-- Cover: 16:9 frame fully filled (image cropped to fill) -->
                <svg viewBox="0 0 64 36" class="w-full h-14 shrink-0 text-muted-foreground/70" aria-hidden="true">
                  <rect x="1" y="1" width="62" height="34" rx="1.5" fill="none" stroke="currentColor" stroke-width="1.5" />
                  <rect x="-4" y="1" width="72" height="34" fill="currentColor" opacity="0.4" />
                </svg>
                <span class="text-sm font-semibold text-foreground block text-center">Cover</span>
                <span class="text-xs text-muted-foreground/80 mt-0.5 block leading-relaxed text-center">Crop to fill; no empty bars</span>
              </template>
            </SelectableRadioCard>
          </div>
        </div>

        <!-- Nested: Custom/background-only (when custom-background or background-only) -->
        <div
          v-if="currentImageStyle === 'custom-background' || currentImageStyle === 'background-only'"
          class="pt-3 pl-4 border-l-2 border-primary/50 rounded-r-md space-y-3 bg-muted/20 -mx-1 px-4 py-3"
        >
          <div>
            <Label class="text-sm font-medium text-muted-foreground">Background source</Label>
            <p class="text-xs text-muted-foreground -mt-0.5">
              {{ currentImageStyle === 'background-only' ? 'Choose from the background library.' : 'File, URL, or library.' }}
            </p>
          </div>
          <div class="space-y-3">
            <div v-if="currentImageStyle === 'custom-background'">
              <!-- Background file selection zone -->
              <div
                @click="emit('selectBackgroundFile')"
                :class="[
                  'group flex items-center gap-2.5 p-3 rounded-lg border-2 transition-all cursor-pointer',
                  currentBackgroundPath
                    ? 'border-border bg-muted/20 hover:border-primary/40 hover:bg-muted/30'
                    : 'border-dashed border-muted-foreground/30 bg-muted/10 hover:border-muted-foreground/50 hover:bg-muted/20',
                  isProcessing && 'opacity-50 pointer-events-none'
                ]"
                role="button"
                tabindex="0"
                :aria-label="currentBackgroundPath ? 'Change background image' : 'Upload background image'"
                @keydown.enter.prevent="emit('selectBackgroundFile')"
                @keydown.space.prevent="emit('selectBackgroundFile')"
              >
                <div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-muted">
                  <Image class="h-4 w-4 text-muted-foreground" />
                </div>
                <div class="flex-1 min-w-0">
                  <template v-if="currentBackgroundPath">
                    <p class="text-xs font-medium text-foreground truncate">
                      {{ currentBackgroundPath.split("/").pop() }}
                    </p>
                    <p class="text-xs text-muted-foreground">
                      Click to change background
                    </p>
                  </template>
                  <template v-else>
                    <p class="text-xs font-medium text-foreground">
                      Upload background image
                    </p>
                    <p class="text-xs text-muted-foreground">
                      Click to select from your computer
                    </p>
                  </template>
                </div>
              </div>
            </div>
            <div v-if="currentImageStyle === 'custom-background'">
              <Label class="text-xs text-muted-foreground flex items-center gap-1.5">
                <Link2 class="h-3 w-3" />
                Or paste a URL
              </Label>
              <Input
                :model-value="currentBackgroundUrl"
                @update:model-value="emit('update:currentBackgroundUrl', $event)"
                placeholder="https://example.com/background.jpg"
                :disabled="isProcessing"
                class="mt-1.5 h-9 text-sm"
                @blur="emit('setBackgroundFromUrl')"
              />
            </div>
            <!-- Background library (predefined + user backgrounds) -->
            <div>
              <Label class="text-xs text-muted-foreground flex items-center gap-1.5">
                <Library class="h-3 w-3" />
                Background library
              </Label>
              <p class="text-xs text-muted-foreground mt-0.5">Built-in templates and your imported backgrounds.</p>
              
              <!-- Predefined backgrounds -->
              <div class="mt-3">
                <p class="text-xs font-medium text-muted-foreground mb-2">Predefined backgrounds</p>
                <div v-if="isLoadingPredefinedBackgrounds" class="mt-2 flex items-center gap-2">
                  <Loader2 class="h-4 w-4 animate-spin text-muted-foreground shrink-0" />
                  <span class="text-xs text-muted-foreground">Loading templates…</span>
                </div>
                <p v-else-if="predefinedBackgroundsLoadError" class="text-xs text-destructive mt-2">
                  {{ predefinedBackgroundsLoadError }}
                </p>
                <div
                  v-else-if="predefinedBackgrounds.length > 0"
                  class="mt-2 flex gap-2 overflow-x-auto overflow-y-hidden pb-1 -mx-0.5 px-0.5"
                >
                  <div
                    v-for="bg in predefinedBackgrounds"
                    :key="bg.id"
                    class="shrink-0 w-48"
                  >
                    <ImageThumbnailCard
                      :src="previewUrl(bg.imagePath)"
                      :hover-src="previewUrl(bg.imagePath)"
                      :label="undefined"
                      :selected="currentBackgroundPredefinedId === bg.id"
                      :disabled="isProcessing"
                      aspect="video"
                      container-class="w-full"
                      @click="emit('setBackgroundPredefined', bg.id)"
                    />
                  </div>
                </div>
                <p v-else class="text-xs text-muted-foreground mt-2">
                  No predefined backgrounds available.
                </p>
              </div>

              <!-- User-imported backgrounds -->
              <div class="mt-4">
                <p class="text-xs font-medium text-muted-foreground mb-2">Your backgrounds</p>
                <div v-if="isLoadingUserBackgrounds" class="mt-2 flex items-center gap-2">
                  <Loader2 class="h-4 w-4 animate-spin text-muted-foreground shrink-0" />
                  <span class="text-xs text-muted-foreground">Loading your backgrounds…</span>
                </div>
                <p v-else-if="userBackgroundsLoadError" class="text-xs text-destructive mt-2">
                  {{ userBackgroundsLoadError }}
                </p>
                <div
                  v-else-if="userBackgrounds.length > 0"
                  class="mt-2 flex gap-2 overflow-x-auto overflow-y-hidden pb-1 -mx-0.5 px-0.5"
                >
                  <div
                    v-for="bg in userBackgrounds"
                    :key="bg.id"
                    class="shrink-0 w-48"
                  >
                    <ImageThumbnailCard
                      :src="previewUrl(bg.imagePath)"
                      :hover-src="previewUrl(bg.imagePath)"
                      :label="undefined"
                      :selected="currentBackgroundPredefinedId === bg.id"
                      :disabled="isProcessing"
                      aspect="video"
                      container-class="w-full"
                      @click="emit('setBackgroundPredefined', bg.id)"
                    />
                  </div>
                  <!-- Add button card -->
                  <Button
                    variant="outline"
                    @click="emit('importBackground')"
                    :disabled="isProcessing || isLoadingUserBackgrounds"
                    class="shrink-0 w-48 h-28 border-dashed flex items-center justify-center"
                    title="Import background"
                  >
                    <Plus class="h-5 w-5 text-muted-foreground" />
                  </Button>
                </div>
                <!-- Empty state with import button -->
                <div v-else class="mt-2">
                  <Button
                    variant="outline"
                    @click="emit('importBackground')"
                    :disabled="isProcessing || isLoadingUserBackgrounds"
                    class="w-full h-20 border-dashed flex flex-col items-center justify-center gap-2"
                  >
                    <Plus class="h-6 w-6 text-muted-foreground" />
                    <span class="text-xs text-muted-foreground font-medium">Import background</span>
                  </Button>
                </div>
              </div>
            </div>
            <!-- Favorite toggle - show when a background is selected -->
            <div v-if="currentBackgroundPredefinedId && (currentImageStyle === 'background-only' || currentImageStyle === 'custom-background')" class="mt-3 flex items-center justify-between p-3 rounded-lg border bg-muted/30">
              <div class="flex items-center gap-2">
                <Star class="h-4 w-4 text-muted-foreground" />
                <Label class="text-sm font-medium">Set as favorite background</Label>
              </div>
              <Toggle
                :model-value="!!(favoriteBackgroundId && favoriteBackgroundId === currentBackgroundPredefinedId)"
                @update:model-value="emit('setFavoriteBackground', favoriteBackgroundId === currentBackgroundPredefinedId ? null : currentBackgroundPredefinedId)"
                :disabled="isProcessing"
                aria-label="Set as favorite background"
              />
            </div>
          </div>
        </div>
      </SectionCard>
    </section>

    <!-- Section: Save for future use -->
    <section v-if="(currentImagePath && canSaveImageAsBackground) || currentBackgroundPath" class="space-y-2">
      <SectionHeader
        label="Save for future use"
        description="Add this image to your background library for easy reuse."
      />
      <Callout variant="info" class="mt-2">
        <div class="space-y-3">
          <!-- Save main image as background -->
          <div v-if="currentImagePath && canSaveImageAsBackground" class="space-y-2">
            <div class="flex items-center justify-between">
              <Label class="text-sm font-medium">Save as background for future use</Label>
              <Toggle
                :model-value="saveImageForFutureUse"
                @update:model-value="emit('update:saveImageForFutureUse', $event)"
                :disabled="isProcessing"
              />
            </div>
            <div v-if="saveImageForFutureUse && isBackgroundOnlyStyle" class="flex items-center justify-between pl-4">
              <Label class="text-sm">Set as default preset</Label>
              <Toggle
                :model-value="setSavedBackgroundAsDefault"
                @update:model-value="emit('update:setSavedBackgroundAsDefault', $event)"
                :disabled="isProcessing"
              />
            </div>
          </div>
          
          <!-- Save background image -->
          <div v-if="currentBackgroundPath" class="space-y-2">
            <div class="flex items-center justify-between">
              <Label class="text-sm font-medium">Save background for future use</Label>
              <Toggle
                :model-value="saveBackgroundForFutureUse"
                @update:model-value="emit('update:saveBackgroundForFutureUse', $event)"
                :disabled="isProcessing"
              />
            </div>
            <div v-if="saveBackgroundForFutureUse" class="flex items-center justify-between pl-4">
              <Label class="text-sm">Set as default preset</Label>
              <Toggle
                :model-value="setSavedBackgroundAsDefault"
                @update:model-value="emit('update:setSavedBackgroundAsDefault', $event)"
                :disabled="isProcessing"
              />
            </div>
          </div>
        </div>
      </Callout>
    </section>
  </div>

  <!-- Image Editing Modal -->
  <ImageEditingModal
    :open="isImageEditorOpen"
    :image-src="editingImageSrc"
    @close="emit('closeImageEditor')"
    @save="emit('handleEditedImage', $event)"
  />
</template>
