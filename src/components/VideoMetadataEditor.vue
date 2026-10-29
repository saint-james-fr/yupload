<script setup lang="ts">
import { computed, onMounted, watch } from "vue";
import type { Ref } from "vue";
import type { BaseVideoMetadata } from "../composables/useVideoMetadata";
import { useVideoMetadata } from "../composables/useVideoMetadata";
import { useYouTube } from "../composables/useYouTube";
import Button from "./ui/Button.vue";
import Input from "./ui/Input.vue";
import Label from "./ui/Label.vue";
import Callout from "./ui/Callout.vue";
import SectionHeader from "./ui/SectionHeader.vue";
import SettingsRow from "./ui/SettingsRow.vue";
import DiscogsSearchResults from "./DiscogsSearchResults.vue";
import SegmentedControl from "./ui/SegmentedControl.vue";
import VideoPreview from "./VideoPreview.vue";
import { YOUTUBE_CATEGORIES, YOUTUBE_PRIVACY_OPTIONS } from "../lib/constants";
import {
  Music,
  XCircle,
  Plus,
  Loader2,
  Lock,
  Unlock,
  Globe,
  EyeOff,
  ListVideo,
} from "lucide-vue-next";

const props = defineProps<{
  metadata: BaseVideoMetadata;
  isProcessing?: boolean;
  videoPath?: string | null;
  showVideoPreview?: boolean;
  forcePrivateUploads?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:metadata", field: keyof BaseVideoMetadata, value: any): void;
  (e: "apply-template"): void;
  (e: "select-discogs-release", release: any): void;
}>();

const youtube = useYouTube();

// Create a ref wrapper for metadata to work with composable
const metadataRef = computed(() => props.metadata);

const updateMetadata = (field: keyof BaseVideoMetadata, value: any) => {
  emit("update:metadata", field, value);
};

const {
  tagInput,
  addTag,
  removeTag,
  discogsSearchQuery,
  discogsSearchResults,
  isSearchingDiscogs,
  isDiscogsAuthenticated,
  onSearchInput,
  searchDiscogs,
  selectDiscogsRelease,
  clearDiscogsRelease,
  authenticateDiscogs,
  checkDiscogsAuth,
  applyDescriptionTemplate,
  statusCallout,
} = useVideoMetadata(metadataRef as Ref<BaseVideoMetadata>, updateMetadata);

// Helper to prefill search query from title
const prefillSearchFromTitle = () => {
  if (props.metadata.title && props.metadata.title.trim().length >= 3 && isDiscogsAuthenticated.value && !props.metadata.discogsRelease) {
    discogsSearchQuery.value = props.metadata.title.trim();
  }
};

// Check Discogs auth status on mount to sync with any existing authentication
onMounted(async () => {
  await checkDiscogsAuth();
  // Prefill search query with title after auth check
  prefillSearchFromTitle();
});

// Prefill Discogs search query from metadata title when it changes (like images section does)
watch(() => props.metadata.title, () => {
  prefillSearchFromTitle();
}, { immediate: true });

// Also watch for auth status changes to prefill when auth becomes available
watch(isDiscogsAuthenticated, () => {
  prefillSearchFromTitle();
});

// Watch for when discogsRelease is cleared to allow prefilling again
watch(() => props.metadata.discogsRelease, () => {
  prefillSearchFromTitle();
});

const handleSelectDiscogsRelease = async (result: any) => {
  await selectDiscogsRelease(result);
  // Emit the release after it's been selected and metadata updated
  if (props.metadata.discogsRelease) {
    emit("select-discogs-release", props.metadata.discogsRelease);
  }
};

const handleApplyTemplate = () => {
  applyDescriptionTemplate();
  emit("apply-template");
};

const PRIVACY_ICONS = { public: Globe, unlisted: EyeOff, private: Lock, draft: Lock } as const;
const privacyOptions = computed(() =>
  YOUTUBE_PRIVACY_OPTIONS.map((o) => ({
    ...o,
    icon: PRIVACY_ICONS[o.value],
    disabled:
      props.isProcessing ||
      (o.value !== "private" && props.forcePrivateUploads),
  }))
);
</script>

<template>
  <div class="space-y-6">
    <!-- Video Preview (optional) -->
    <div v-if="showVideoPreview && videoPath" class="space-y-2">
      <SectionHeader label="Video preview" />
      <VideoPreview :file-path="videoPath" label="" :compact="true" />
    </div>

    <!-- Status Callout -->
    <Callout v-if="statusCallout" :variant="statusCallout.variant">
      {{ statusCallout.message }}
    </Callout>

    <!-- Title -->
    <div>
      <Label>Title *</Label>
      <Input
        :model-value="metadata.title"
        @update:model-value="updateMetadata('title', $event)"
        placeholder="Enter video title (max 100 characters)"
        :maxlength="100"
        class="mt-2"
        :disabled="isProcessing"
      />
      <p class="text-xs text-muted-foreground mt-1">
        {{ metadata.title.length }}/100 characters
      </p>
    </div>

    <!-- Description -->
    <div>
      <Label>Description</Label>
      <Callout variant="info" class="mt-2">
        Write your own below, or link a Discogs release underneath to auto-fill
        this, the title, and tags from the catalog. Go to <strong>Settings</strong> to create custom description templates that combine your text with Discogs information in any order.
      </Callout>
      <textarea
        :value="metadata.description"
        @input="updateMetadata('description', ($event.target as HTMLTextAreaElement).value)"
        placeholder="Enter video description (max 5000 characters)"
        :maxlength="5000"
        rows="6"
        class="w-full mt-2 px-3 py-2 rounded-md border border-input bg-background text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50"
        :disabled="isProcessing"
      />
      <p class="text-xs text-muted-foreground mt-1">
        {{ metadata.description.length }}/5000 characters
      </p>
      <div class="mt-2 flex flex-wrap items-center gap-2">
        <Button
          type="button"
          variant="outline"
          size="sm"
          @click="handleApplyTemplate"
          :disabled="isProcessing"
        >
          <ListVideo class="h-4 w-4 mr-2" />
          Apply template
        </Button>
      </div>
      <!-- Slot for additional content after description (e.g., "Apply to all" button) -->
      <slot name="after-description"></slot>
    </div>

    <!-- Discogs Integration -->
    <div v-if="isDiscogsAuthenticated" class="space-y-4 p-4 rounded-lg border bg-muted/50">
      <div class="space-y-1">
        <Label class="text-base font-semibold">Or fill from Discogs</Label>
        <p class="text-xs text-muted-foreground">
          Search and link a release to auto-fill title, description, and tags
          from the catalog.
        </p>
      </div>

      <div v-if="metadata.discogsRelease" class="p-3 rounded-md bg-background border">
        <div class="flex items-start gap-3">
          <div
            class="w-16 h-16 rounded bg-muted shrink-0 flex items-center justify-center"
          >
            <Music class="h-6 w-6 text-muted-foreground" />
          </div>
          <div class="flex-1 min-w-0">
            <p class="font-medium truncate">
              {{ metadata.discogsRelease.title }}
            </p>
            <p
              v-if="metadata.discogsRelease.artists.length > 0"
              class="text-sm text-muted-foreground"
            >
              {{ metadata.discogsRelease.artists.map((a: any) => a.name).join(", ") }}
            </p>
            <p
              v-if="metadata.discogsRelease.year"
              class="text-xs text-muted-foreground"
            >
              {{ metadata.discogsRelease.year }}
            </p>
            <p
              v-if="metadata.discogsRelease.labels.length > 0"
              class="text-xs text-muted-foreground"
            >
              {{ metadata.discogsRelease.labels.map((l: any) => l.name).join(", ") }}
            </p>
          </div>
          <button
            @click="clearDiscogsRelease"
            class="cursor-pointer text-muted-foreground hover:text-destructive"
            :disabled="isProcessing"
          >
            <XCircle class="h-4 w-4" />
          </button>
        </div>
      </div>

      <div v-else class="space-y-2">
        <div class="relative">
          <Input
            v-model="discogsSearchQuery"
            placeholder="Search by artist, title... (autocomplete)"
            class="flex-1"
            @input="onSearchInput"
            @keyup.enter="searchDiscogs"
            :disabled="isProcessing || isSearchingDiscogs"
          />
          <Loader2
            v-if="isSearchingDiscogs"
            class="absolute right-3 top-1/2 -translate-y-1/2 h-4 w-4 animate-spin text-muted-foreground"
          />
        </div>
        <DiscogsSearchResults
          :results="discogsSearchResults"
          :is-disabled="isProcessing"
          :is-loading="isSearchingDiscogs"
          @select="handleSelectDiscogsRelease"
        />
      </div>
    </div>

    <div v-else class="p-4 rounded-lg border bg-muted/30">
      <SettingsRow
        :icon="Lock"
        title="Fill description and metadata from Discogs"
        description="Connect Discogs to search and link a release, then auto-fill title, description, and tags from the catalog."
      >
        <Button
          @click="authenticateDiscogs"
          variant="outline"
          size="sm"
          class="shrink-0"
        >
          <Unlock class="h-4 w-4 mr-2" />
          Connect
        </Button>
      </SettingsRow>
    </div>

    <!-- Tags -->
    <div>
      <Label>Tags</Label>
      <div class="flex gap-2 mt-2">
        <Input
          v-model="tagInput"
          placeholder="Add tag (press Enter)"
          class="flex-1"
          @keyup.enter="addTag(tagInput)"
          :disabled="isProcessing"
        />
        <Button
          @click="addTag(tagInput)"
          :disabled="isProcessing || !tagInput.trim()"
        >
          <Plus class="h-4 w-4" />
        </Button>
      </div>
      <div v-if="metadata.tags.length > 0" class="flex flex-wrap gap-2 mt-2">
        <span
          v-for="tag in metadata.tags"
          :key="tag"
          class="inline-flex items-center gap-1 px-2 py-1 rounded-md bg-secondary text-secondary-foreground text-sm"
        >
          {{ tag }}
          <button
            @click="removeTag(tag)"
            class="cursor-pointer hover:text-destructive"
            :disabled="isProcessing"
          >
            <XCircle class="h-3 w-3" />
          </button>
        </span>
      </div>
      <p class="text-xs text-muted-foreground mt-1">
        {{ metadata.tags.length }}/500 tags
      </p>
      <slot name="after-tags" />
    </div>

    <!-- Category -->
    <div>
      <Label>Category</Label>
      <select
        :value="metadata.categoryId"
        @change="updateMetadata('categoryId', ($event.target as HTMLSelectElement).value)"
        class="w-full mt-2 px-3 py-2 rounded-md border border-input bg-background text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50"
        :disabled="isProcessing"
      >
        <option
          v-for="cat in YOUTUBE_CATEGORIES"
          :key="cat.value"
          :value="cat.value"
        >
          {{ cat.label }}
        </option>
      </select>
      <slot name="after-category" />
    </div>

    <!-- Privacy Status -->
    <div>
      <Callout
        v-if="forcePrivateUploads"
        variant="warning"
        title="Limited to private uploads"
        class="mb-4"
      >
        <p class="mb-2">
          Uploads are limited to <strong>private</strong> until our app is
          verified by YouTube.
        </p>
        <p>
          We are working on verification. Public, unlisted, and draft are
          disabled for now.
        </p>
      </Callout>
      <Label>Privacy &amp; scheduling</Label>
      <div class="mt-2">
        <SegmentedControl
          :model-value="metadata.privacyStatus"
          @update:model-value="updateMetadata('privacyStatus', $event)"
          :options="privacyOptions"
          aria-label="Privacy status"
        >
          <template #option="{ option }">
            <component v-if="option.icon" :is="option.icon" class="h-4 w-4" />
            <span>{{ option.label }}</span>
          </template>
        </SegmentedControl>
      </div>
      <p class="text-xs text-muted-foreground mt-2">
        Select <strong>Private</strong> to enable scheduling options. Private videos can be scheduled to go public at a specific date and time.
      </p>
      <div v-if="metadata.privacyStatus === 'private'" class="mt-3">
        <Label class="text-muted-foreground font-normal"
          >Schedule for (optional)</Label
        >
        <div class="relative">
          <input
            type="datetime-local"
            :value="metadata.publishAt ? metadata.publishAt.slice(0, 16) : ''"
            @input="updateMetadata('publishAt', ($event.target as HTMLInputElement).value || null)"
            :disabled="isProcessing"
            class="w-full mt-2 px-3 py-2 rounded-md border border-input bg-background text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 cursor-pointer"
            :class="metadata.publishAt ? 'pr-10' : 'text-transparent'"
          />
          <div
            v-if="!metadata.publishAt"
            class="absolute left-3 top-1/2 -translate-y-1/2 mt-1 pointer-events-none text-sm text-muted-foreground"
          >
            Click to schedule publication time
          </div>
          <button
            v-if="metadata.publishAt"
            @click="updateMetadata('publishAt', null)"
            :disabled="isProcessing"
            class="absolute right-2 top-1/2 -translate-y-1/2 mt-1 p-1 rounded-sm text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors cursor-pointer"
            aria-label="Clear schedule"
            type="button"
          >
            <XCircle class="h-4 w-4" />
          </button>
        </div>
        <p class="text-xs text-muted-foreground mt-1">
          Video will go public at this time. Leave empty to keep private.
        </p>
      </div>
      <slot name="after-privacy" />
    </div>

    <!-- Playlist -->
    <div v-if="youtube.isAuthenticated.value">
      <Label class="flex items-center gap-2">
        <ListVideo class="h-4 w-4" />
        Add to playlist (optional)
      </Label>
      <select
        :value="metadata.playlistId ?? ''"
        @change="updateMetadata('playlistId', ($event.target as HTMLSelectElement).value || null)"
        :disabled="isProcessing || youtube.isLoadingPlaylists.value"
        class="w-full mt-2 px-3 py-2 rounded-md border border-input bg-background text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50"
      >
        <option value="">None</option>
        <option
          v-for="p in youtube.playlists.value"
          :key="p.id"
          :value="p.id"
        >
          {{ p.title }}
        </option>
      </select>
    </div>

    <!-- Made for Kids -->
    <div class="flex items-center gap-2 p-4 rounded-lg border bg-muted/50">
      <input
        type="checkbox"
        id="madeForKids"
        :checked="metadata.selfDeclaredMadeForKids"
        @change="updateMetadata('selfDeclaredMadeForKids', ($event.target as HTMLInputElement).checked)"
        class="w-4 h-4 cursor-pointer"
        :disabled="isProcessing"
      />
      <label for="madeForKids" class="cursor-pointer flex-1">
        <span class="font-medium">Made for Kids</span>
        <p class="text-xs text-muted-foreground mt-1">
          Indicates whether the video is intended for children
        </p>
      </label>
    </div>
  </div>
</template>
