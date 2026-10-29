<script setup lang="ts">
import type { MaybeRef } from "vue";
import { computed, unref } from "vue";
import Button from "../ui/Button.vue";
import Card from "../ui/Card.vue";
import FolderSelectionZone from "../ui/FolderSelectionZone.vue";
import SettingsRow from "../ui/SettingsRow.vue";
import Toggle from "../ui/Toggle.vue";
import Alert from "../ui/Alert.vue";
import { Folder, Loader2, Music, Upload } from "lucide-vue-next";

const props = withDefaults(
  defineProps<{
    saveVideosLocally: boolean;
    outputFolder: string | null;
    isProcessing: boolean;
    isOutputFolderLoading?: MaybeRef<boolean>;
    exportNormalizedAudio: boolean;
    normalizeAudio: boolean;
    audioExportFolderValue: string | null;
    isAudioExportFolderLoading?: MaybeRef<boolean>;
    isYouTubeAuthenticated: boolean;
    canCreateAndUpload: boolean;
    audioTracksCount: number;
  }>(),
  { isOutputFolderLoading: false, isAudioExportFolderLoading: false }
);

const isOutputFolderLoadingValue = computed(() => unref(props.isOutputFolderLoading));
const isAudioExportFolderLoadingValue = computed(() => unref(props.isAudioExportFolderLoading));

const emit = defineEmits<{
  (e: "update:saveVideosLocally", value: boolean): void;
  (e: "update:exportNormalizedAudio", value: boolean): void;
  (e: "select-output-folder"): void;
  (e: "clear-output-folder"): void;
  (e: "select-audio-export-folder"): void;
  (e: "clear-audio-export-folder"): void;
  (e: "create-and-upload"): void;
}>();
</script>

<template>
  <div class="space-y-6">
    <Card class="p-6 border-2">
      <h3 class="text-lg font-semibold mb-4">Options</h3>
      <div class="space-y-6">
        <SettingsRow
          :icon="Folder"
          title="Save videos locally"
          description="Save created videos to your computer. If disabled, videos will be created temporarily and deleted after upload."
        >
          <Toggle
            :model-value="saveVideosLocally"
            :disabled="isProcessing"
            aria-label="Save videos locally"
            @update:model-value="(v: boolean) => emit('update:saveVideosLocally', v)"
          />
        </SettingsRow>

        <FolderSelectionZone
          v-if="saveVideosLocally"
          :value="outputFolder"
          label="Output folder"
          description="Created videos will be saved here."
          empty-message="Not selected"
          :is-loading="isOutputFolderLoadingValue"
          :disabled="isProcessing"
          @select="emit('select-output-folder')"
          @clear="emit('clear-output-folder')"
        />

        <Alert v-if="saveVideosLocally && !outputFolder" variant="destructive">
          <div class="flex items-center gap-2">
            <p>Please select an output folder to save videos locally.</p>
          </div>
        </Alert>

        <SettingsRow
          :icon="Music"
          title="Export normalized audio files"
          description="Save normalized audio files alongside videos (requires normalization to be enabled)"
        >
          <Toggle
            :model-value="exportNormalizedAudio"
            :disabled="isProcessing || !normalizeAudio"
            aria-label="Export normalized audio files"
            @update:model-value="(v: boolean) => emit('update:exportNormalizedAudio', v)"
          />
        </SettingsRow>

        <FolderSelectionZone
          v-if="exportNormalizedAudio && normalizeAudio"
          :value="audioExportFolderValue"
          label="Audio output folder"
          description="Normalized audio files will be saved here."
          empty-message="Not selected"
          :is-loading="isAudioExportFolderLoadingValue"
          :disabled="isProcessing"
          @select="emit('select-audio-export-folder')"
          @clear="emit('clear-audio-export-folder')"
        />

        <Alert v-if="exportNormalizedAudio && normalizeAudio && !audioExportFolderValue" variant="destructive">
          <div class="flex items-center gap-2">
            <p>Please select an output folder to export normalized audio files.</p>
          </div>
        </Alert>

        <Alert v-if="!isYouTubeAuthenticated" variant="destructive">
          <div class="flex items-center gap-2">
            <p>Please connect your YouTube account to upload videos.</p>
          </div>
        </Alert>
      </div>
    </Card>

    <Card class="p-6 border-2">
      <Button
        @click="emit('create-and-upload')"
        :disabled="!canCreateAndUpload"
        class="w-full text-lg py-6 px-8 font-semibold"
        size="lg"
        variant="default"
      >
        <Upload v-if="!isProcessing" class="h-5 w-5 mr-2" />
        <Loader2 v-else class="h-5 w-5 mr-2 animate-spin" />
        <span v-if="isProcessing">Processing...</span>
        <span v-else>
          Create & Upload ({{ audioTracksCount }})
        </span>
      </Button>
    </Card>
  </div>
</template>
