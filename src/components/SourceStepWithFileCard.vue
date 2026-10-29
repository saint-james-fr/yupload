<script setup lang="ts">
import Button from "./ui/Button.vue";
import Modal from "./ui/Modal.vue";
import SourceStepView from "./SourceStepView.vue";
import FileSelectionCard from "./FileSelectionCard.vue";
import { Music, FileVideo } from "lucide-vue-next";

const props = withDefaults(
  defineProps<{
    /** "audio" = folder/audio files + No Audio Found modal; "video" = folder/videos + No Videos Found modal */
    variant: "audio" | "video";
    selectedFolder: string | null;
    fileCount: number;
    isLoadingFolder: boolean;
    isLoadingFiles: boolean;
    isProcessing: boolean;
    noFilesFoundModalOpen: boolean;
    /** Override default step title (e.g. "Select source") */
    title?: string;
    /** Override default step subtitle */
    subtitle?: string;
  }>(),
  { title: "Select source" }
);

const emit = defineEmits<{
  (e: "back"): void;
  (e: "select-folder"): void;
  (e: "select-files"): void;
  (e: "dismiss-no-files"): void;
}>();

const defaultSubtitleByVariant = {
  audio: "Choose a folder or add audio files.",
  video: "Choose a folder or add videos to upload to YouTube.",
};

const modalTitleByVariant = {
  audio: "No Audio Found",
  video: "No Videos Found",
};

const modalMessageByVariant = {
  audio:
    "The selected folder doesn't contain any supported audio files (e.g. MP3, WAV, FLAC). Choose another folder or add files manually.",
  video:
    "The selected folder doesn't contain any supported video files (e.g. MP4, MOV, WebM). Choose another folder or add files manually.",
};

const stepSubtitle = props.subtitle ?? defaultSubtitleByVariant[props.variant];
const FileLegendIcon = props.variant === "audio" ? Music : FileVideo;
</script>

<template>
  <SourceStepView
    :title="title"
    :subtitle="stepSubtitle"
    @back="emit('back')"
  >
    <div class="w-full space-y-0">
      <FileSelectionCard
        :selected-folder="selectedFolder"
        :file-count="fileCount"
        :is-loading-folder="isLoadingFolder"
        :is-loading-files="isLoadingFiles"
        :is-processing="isProcessing"
        size="large"
        @select-folder="emit('select-folder')"
        @select-files="emit('select-files')"
      >
        <template #file-legend-icon>
          <FileLegendIcon class="h-3.5 w-3.5" />
        </template>
      </FileSelectionCard>
      <Modal :open="noFilesFoundModalOpen" @close="emit('dismiss-no-files')">
        <div class="space-y-4">
          <h3 class="text-lg font-semibold">{{ modalTitleByVariant[variant] }}</h3>
          <p class="text-sm text-muted-foreground">
            {{ modalMessageByVariant[variant] }}
          </p>
          <div class="flex justify-end">
            <Button @click="emit('dismiss-no-files')" variant="default">OK</Button>
          </div>
        </div>
      </Modal>
    </div>
  </SourceStepView>
</template>
