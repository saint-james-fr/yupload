<script setup lang="ts">
// Remove unused computed import
import { Folder, Loader2, Files } from "lucide-vue-next";
import { cn } from "../lib/utils";
import PathDisplay from "./ui/PathDisplay.vue";

export interface FileSelectionCardProps {
  folderLabel?: string;
  fileLabel?: string;
  selectedFolder: string | null;
  fileCount?: number;
  isLoadingFolder?: boolean;
  isLoadingFiles?: boolean;
  isProcessing?: boolean;
  showFolderButton?: boolean;
  showFileButton?: boolean;
  title?: string;
  description?: string;
  /** "large" for standalone step (centered, bigger icons/padding) */
  size?: "default" | "large";
}

withDefaults(defineProps<FileSelectionCardProps>(), {
  folderLabel: "Select Folder (recursive, max 10 levels)",
  fileLabel: "Add Files",
  fileCount: 0,
  isLoadingFolder: false,
  isLoadingFiles: false,
  isProcessing: false,
  showFolderButton: true,
  showFileButton: true,
  title: "Select Source",
  size: "default",
});

const emit = defineEmits<{
  (e: "select-folder"): void;
  (e: "select-files"): void;
}>();

// Remove folderName computed since PathDisplay handles this
</script>

<template>
  <div
    :class="cn(
      'flex items-center justify-center',
      size === 'large' ? 'gap-6 md:gap-8' : 'gap-4'
    )"
  >
    <!-- Folder Card -->
    <button
      v-if="showFolderButton"
      @click="emit('select-folder')"
      :disabled="isProcessing || isLoadingFolder"
      :class="cn(
        'file-selection-card group rounded-xl border-2 border-border shadow-sm relative',
        'flex flex-col items-center justify-center transition-all duration-200',
        'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2',
        'disabled:opacity-50 disabled:cursor-not-allowed',
        'cursor-pointer',
        size === 'large' ? 'p-10 md:p-12 gap-4' : 'p-6 gap-3'
      )"
    >
      <div :class="cn('relative transition-transform duration-200 ease-out group-hover:rotate-[5deg] group-hover:-translate-y-1 group-active:translate-y-0', size === 'large' ? 'h-16 w-16 md:h-20 md:w-20' : 'h-16 w-16')">
        <Loader2
          v-if="isLoadingFolder"
          :class="size === 'large' ? 'h-16 w-16 md:h-20 md:w-20' : 'h-16 w-16'"
          class="text-primary animate-spin"
        />
        <Folder
          v-else
          :class="size === 'large' ? 'h-16 w-16 md:h-20 md:w-20' : 'h-16 w-16'"
          class="text-primary transition-colors group-hover:text-primary-accent"
          stroke-width="1.5"
        />
      </div>
      <div :class="cn('flex items-center gap-1.5 text-muted-foreground group-hover:text-foreground transition-colors', size === 'large' ? 'text-sm' : 'text-xs')">
        <span>Folder</span>
      </div>
    </button>

    <!-- File Card -->
    <button
      v-if="showFileButton"
      @click="emit('select-files')"
      :disabled="isProcessing || isLoadingFiles"
      :class="cn(
        'file-selection-card group rounded-xl border-2 border-border shadow-sm relative',
        'flex flex-col items-center justify-center transition-all duration-200',
        'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2',
        'disabled:opacity-50 disabled:cursor-not-allowed',
        'cursor-pointer',
        size === 'large' ? 'p-10 md:p-12 gap-4' : 'p-6 gap-3'
      )"
    >
      <div :class="cn('relative transition-transform duration-200 ease-out group-hover:rotate-[5deg] group-hover:-translate-y-1 group-active:translate-y-0', size === 'large' ? 'h-16 w-16 md:h-20 md:w-20' : 'h-16 w-16')">
        <Loader2
          v-if="isLoadingFiles"
          :class="size === 'large' ? 'h-16 w-16 md:h-20 md:w-20' : 'h-16 w-16'"
          class="text-primary animate-spin"
        />
        <slot v-else name="file-icon">
          <Files
            :class="size === 'large' ? 'h-16 w-16 md:h-20 md:w-20' : 'h-16 w-16'"
            class="text-primary transition-colors group-hover:text-primary-accent"
            stroke-width="1.5"
          />
        </slot>
      </div>
      <div :class="cn('flex items-center gap-1.5 text-muted-foreground group-hover:text-foreground transition-colors', size === 'large' ? 'text-sm' : 'text-xs')">
        <span>Files</span>
      </div>
    </button>
  </div>

  <!-- Bundle Explanation -->
  <p v-if="size === 'large' && showFolderButton" class="mt-4 text-xs text-muted-foreground/70 text-center max-w-2xl mx-auto leading-relaxed">
    Selecting a folder bundles audio and images together for video content. Automatically loads all audio files recursively from subdirectories.
  </p>

  <!-- Status Info -->
  <div v-if="selectedFolder || fileCount > 0" :class="cn('mt-6 flex flex-col gap-2', size === 'large' ? 'text-sm' : 'text-xs')">
    <div v-if="selectedFolder" class="flex items-center gap-2 text-muted-foreground">
      <Folder class="h-3.5 w-3.5 shrink-0" />
      <span><span class="font-medium text-foreground">Folder:</span> <PathDisplay :path="selectedFolder" filename-only /></span>
    </div>
    <div v-if="fileCount > 0" class="text-muted-foreground">
      <span class="font-medium text-foreground">{{ fileCount }}</span> file(s) selected
    </div>
  </div>
</template>

<style scoped>
.file-selection-card > * {
  position: relative;
}
</style>
