<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { withTimeout, FOLDER_LIST_TIMEOUT_MS } from "../lib/tauri-timeout";
import { listen } from "@tauri-apps/api/event";
import type { FolderContents } from "../types";
import Button from "../components/ui/Button.vue";
import SectionHeader from "../components/ui/SectionHeader.vue";
import SectionCard from "../components/ui/SectionCard.vue";
import Collapsible from "../components/ui/Collapsible.vue";
import PathDisplay from "../components/ui/PathDisplay.vue";
import PageContainer from "../components/layout/PageContainer.vue";
import PageHeader from "../components/layout/PageHeader.vue";
import SourceStepView from "../components/SourceStepView.vue";
import FileSelectionCard from "../components/FileSelectionCard.vue";
import EmptyState from "../components/EmptyState.vue";
import ProgressCard from "../components/ProgressCard.vue";
import StatusAlert from "../components/StatusAlert.vue";
import AudioPreview from "../components/AudioPreview.vue";
import Modal from "../components/ui/Modal.vue";
import Toggle from "../components/ui/Toggle.vue";
import SettingsRow from "../components/ui/SettingsRow.vue";
import Callout from "../components/ui/Callout.vue";
import FolderSelectionZone from "../components/ui/FolderSelectionZone.vue";
import { Music, Loader2, XCircle, Filter, ChevronRight } from "lucide-vue-next";
import { usePreferences } from "../composables/usePreferences";
import { useOutputFolder } from "../composables/useOutputFolder";

const emit = defineEmits<{
  (e: "back"): void;
  (
    e: "navigate",
    view: "success",
    payload: { from: "normalize"; title: string; summary: string }
  ): void;
}>();

// Audio files list
const audioFiles = ref<Array<{ path: string; name: string }>>([]);
const selectedFolder = ref<string | null>(null);
const folderContents = ref<FolderContents | null>(null);

// Output and processing
const isProcessing = ref(false);
const progress = ref(0);
const progressMessage = ref("");
const error = ref<string | null>(null);
const success = ref<string | null>(null);
const normalizationResults = ref<{ successful: string[]; failed: Array<[string, string]> } | null>(null);
const highPassFilter = ref(false);
const overwriteOriginals = ref(false);

// Normalization progress
const normalizationProgress = ref({ total: 0, completed: 0, message: "" });

// File selection loading states
const isLoadingFolder = ref(false);
const isLoadingFiles = ref(false);

// Modal when folder has no audio
const noFilesFoundModalOpen = ref(false);

const { preferences, loadPreferences, updatePreferences } = usePreferences();
const {
  outputFolder,
  isLoading: isLoadingOutput,
  error: outputFolderError,
  loadFromPreferences,
  selectFolder: selectOutputFolderComposable,
  clear: clearOutputFolderComposable
} = useOutputFolder({
  preferenceKey: "last_normalize_output",
  defaultPreferenceKey: "preferred_output_path"
});

// Load preferred output path and last-used on mount
onMounted(async () => {
  await loadPreferences();
  await loadFromPreferences();
  overwriteOriginals.value = preferences.value.preferences?.normalize_audio_add_suffix === false;
  // Set up event listeners
  await setupEventListeners();
});

// Merge output folder errors into main error
watch(outputFolderError, (err) => {
  if (err) {
    error.value = err;
  }
});

const canNormalize = computed(() => {
  return audioFiles.value.length > 0 && outputFolder.value !== null && !isProcessing.value;
});

// Store unlisten functions for all event listeners
const unlistens: Array<() => void> = [];

// Set up event listeners for normalization progress
const setupEventListeners = async () => {
  const unlisten1 = await listen("audio-normalization-start", (event) => {
    const data = event.payload as { total: number; message: string };
    normalizationProgress.value = {
      total: data.total,
      completed: 0,
      message: data.message
    };
    progressMessage.value = data.message;
  });
  unlistens.push(unlisten1);

  const unlisten2 = await listen("audio-normalization-complete", (event) => {
    const data = event.payload as { total: number; message: string };
    normalizationProgress.value = {
      total: data.total,
      completed: data.total,
      message: data.message
    };
    progressMessage.value = data.message;
  });
  unlistens.push(unlisten2);
};

// Clean up all listeners when component is unmounted
onUnmounted(() => {
  unlistens.forEach((fn) => fn());
  unlistens.length = 0;
});

const selectFolder = async () => {
  try {
    isLoadingFolder.value = true;
    error.value = null;
    const path = await open({
      directory: true,
      multiple: false,
      defaultPath: preferences.value.preferences?.last_normalize_output ?? undefined
    });
    if (path && typeof path === "string") {
      selectedFolder.value = path;
      await loadFolderContents(path);
      updatePreferences({ last_normalize_output: path });
    }
  } catch (err) {
    error.value = `Failed to select folder: ${err}`;
  } finally {
    isLoadingFolder.value = false;
  }
};

const handleSelectFolder = () => {
  selectFolder();
};

const handleSelectFiles = () => {
  selectAudioFiles();
};

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
      error.value = "No audio files found in the selected folder";
      noFilesFoundModalOpen.value = true;
      return;
    }
    audioFiles.value = contents.audios.map(audio => ({
      path: audio.path,
      name: audio.name
    }));
  } catch (err) {
    error.value = `Failed to load folder contents: ${err}`;
  } finally {
    isLoadingFolder.value = false;
  }
};

const dismissNoFilesModal = () => {
  noFilesFoundModalOpen.value = false;
  error.value = null;
};

const selectAudioFiles = async () => {
  try {
    isLoadingFiles.value = true;
    error.value = null;
    const paths = await open({
      multiple: true,
      filters: [{
        name: "Audio",
        extensions: ["mp3", "wav", "flac", "aac", "m4a", "ogg", "aif", "aiff"]
      }]
    });
    if (paths && Array.isArray(paths)) {
      const newFiles = (paths as string[]).map(path => ({
        path,
        name: path.split("/").pop() || path
      }));
      audioFiles.value = [...audioFiles.value, ...newFiles];
    }
  } catch (err) {
    error.value = `Failed to select audio files: ${err}`;
  } finally {
    isLoadingFiles.value = false;
  }
};

const removeAudioFile = (index: number) => {
  audioFiles.value.splice(index, 1);
};

const selectOutputFolder = async () => {
  await selectOutputFolderComposable();
};

const normalizeAudio = async () => {
  if (!canNormalize.value) return;

  isProcessing.value = true;
  error.value = null;
  success.value = null;
  normalizationResults.value = null;
  progress.value = 0;
  progressMessage.value = "Starting audio normalization...";
  normalizationProgress.value = { total: 0, completed: 0, message: "" };

  try {
    const audioPaths = audioFiles.value.map(f => f.path);
    
    const result = await invoke<{ successful: string[]; failed: Array<[string, string]> }>("normalize_audio_files_standalone", {
      audioPaths,
      outputPath: outputFolder.value!,
      highPassFilter: highPassFilter.value,
      addSuffix: !overwriteOriginals.value
    });

    normalizationResults.value = result;
    
    progress.value = 100;
    progressMessage.value = `Normalization complete: ${result.successful.length} successful, ${result.failed.length} failed`;

    if (result.failed.length === 0) {
      const message = `Successfully normalized ${result.successful.length} audio file(s)!`;
      success.value = message;
      emit("navigate", "success", {
        from: "normalize",
        title: "Normalization complete",
        summary: message,
      });
    } else {
      success.value = `Normalized ${result.successful.length} audio file(s), ${result.failed.length} failed`;
      error.value = `Failed files:\n${result.failed.map(([path, err]) => `${path.split("/").pop()}: ${err}`).join("\n")}`;
      setTimeout(() => {
        audioFiles.value = [];
        selectedFolder.value = null;
        folderContents.value = null;
        outputFolder.value = null;
        highPassFilter.value = false;
        progress.value = 0;
        progressMessage.value = "";
        success.value = null;
        normalizationResults.value = null;
        normalizationProgress.value = { total: 0, completed: 0, message: "" };
      }, 5000);
    }
  } catch (err) {
    error.value = `Failed to normalize audio files: ${err}`;
    progress.value = 0;
    progressMessage.value = "";
  } finally {
    isProcessing.value = false;
  }
};

// Confirmation modal for canceling operations
const showCancelConfirmModal = ref(false);

const goBack = async () => {
  // Check if operation is in progress
  if (isProcessing.value || normalizationProgress.value.total > 0) {
    showCancelConfirmModal.value = true;
  } else {
    emit("back");
  }
};

const confirmCancel = async () => {
  showCancelConfirmModal.value = false;
  try {
    // Cancel the operation
    await invoke("cancel_operation", { operationType: "audio_normalization" });
  } catch (err) {
    console.error("Failed to cancel operation:", err);
  }
  emit("back");
};

const cancelCancel = () => {
  showCancelConfirmModal.value = false;
};

const clearSource = () => {
  audioFiles.value = [];
  selectedFolder.value = null;
  folderContents.value = null;
  error.value = null;
  success.value = null;
  normalizationResults.value = null;
  progress.value = 0;
  progressMessage.value = "";
  normalizationProgress.value = { total: 0, completed: 0, message: "" };
};
</script>

<template>
  <!-- Step 1: Select source only (centered, full viewport) -->
  <SourceStepView
    v-if="audioFiles.length === 0"
    title="Select source"
    subtitle="Choose a folder or add audio files to normalize."
    @back="goBack"
  >
    <div class="w-full space-y-0">
      <FileSelectionCard
        :selected-folder="selectedFolder"
        :file-count="audioFiles.length"
        :is-loading-folder="isLoadingFolder"
        :is-loading-files="isLoadingFiles"
        :is-processing="isProcessing"
        size="large"
        @select-folder="handleSelectFolder"
        @select-files="handleSelectFiles"
      >
        <template #file-legend-icon>
          <Music class="h-3.5 w-3.5" />
        </template>
      </FileSelectionCard>
      <Modal :open="noFilesFoundModalOpen" @close="dismissNoFilesModal">
        <div class="space-y-4">
          <h3 class="text-lg font-semibold">No Audio Found</h3>
          <p class="text-sm text-muted-foreground">
            The selected folder doesn’t contain any supported audio files (e.g. MP3, WAV, FLAC). Choose another folder or add files manually.
          </p>
          <div class="flex justify-end">
            <Button @click="dismissNoFilesModal" variant="default">OK</Button>
          </div>
        </div>
      </Modal>
    </div>
  </SourceStepView>

  <!-- Step 2: Rest of flow (source already selected) -->
  <PageContainer v-else>
    <PageHeader
      title="Audio Normalization"
      description="Get consistent volume across your tracks so they sound even in playlists or videos. Optionally tame vinyl rumble, then export the batch in one go."
      @back="goBack"
    />

    <!-- Step Navigation (like unified flow) -->
    <div v-if="!isProcessing" class="mb-6 flex items-center gap-2 text-sm">
      <button
        type="button"
        @click="clearSource"
        class="text-muted-foreground hover:text-foreground cursor-pointer"
      >
        1. Source
      </button>
      <ChevronRight class="h-4 w-4 text-muted-foreground shrink-0" />
      <span class="font-semibold text-primary">2. Normalize</span>
    </div>

    <!-- Technical details about normalization -->
    <div class="mt-6">
      <Collapsible label="Technical details" body-id="normalize-loudness">
        <p><strong>Loudness.</strong> EBU R128 loudnorm: integrated loudness −16 LUFS, true peak −0.5 dBTP, LRA 10 LU.</p>
      </Collapsible>
    </div>

    <!-- Section: Audio files -->
    <section class="space-y-2 mt-8">
      <SectionHeader label="Audio files" description="Files to normalize. Remove any you don't want in the batch." />
      <SectionCard padding="compact">
        <div class="space-y-2">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-sm font-semibold flex items-center gap-2 text-foreground">
            <Music class="h-5 w-5 text-muted-foreground" />
            List
          </h2>
          <span class="text-xs text-muted-foreground">{{ audioFiles.length }} file(s)</span>
        </div>

        <AudioPreview
          v-if="audioFiles.length > 0"
          :file-path="audioFiles[0].path"
          label="Preview"
          :compact="true"
          class="mb-4"
        />
        
        <EmptyState
          v-if="audioFiles.length === 0"
          icon="music"
          message="No audio files selected"
          hint="Select a folder or add audio files"
        />

        <div v-else class="space-y-1 max-h-[400px] overflow-y-auto">
          <div
            v-for="(file, index) in audioFiles"
            :key="file.path"
            class="w-full text-left p-3 rounded-lg border-2 border-border bg-muted/20 hover:bg-muted/30 flex items-center justify-between gap-2"
          >
            <div class="flex-1 min-w-0">
              <div class="font-medium truncate">{{ file.name }}</div>
              <PathDisplay :path="file.path" class="text-xs text-muted-foreground" />
            </div>
            <button
              @click="removeAudioFile(index)"
              class="cursor-pointer text-muted-foreground hover:text-destructive p-1"
              :disabled="isProcessing"
            >
              <XCircle class="h-4 w-4" />
            </button>
          </div>
        </div>

        <Button
          variant="ghost"
          size="sm"
          @click="clearSource"
          class="mt-3 w-full justify-center"
        >
          Change source
        </Button>
      </div>
      </SectionCard>
    </section>

    <!-- Section: Options -->
    <section class="space-y-2 mt-8">
      <SectionHeader label="Options" description="Optional high-pass for vinyl." />
      <SectionCard class="space-y-4">
        <SettingsRow
          :icon="Filter"
          title="30Hz high-pass filter"
          description="Removes subsonic noise that can affect loudness."
        >
          <Toggle
            v-model="highPassFilter"
            :disabled="isProcessing"
            aria-label="30Hz high-pass filter"
          />
        </SettingsRow>
        <Collapsible label="Technical details" body-id="normalize-options" class="mt-2">
          <p><strong>High-pass.</strong> 30 Hz filter removes subsonic and vinyl rumble before loudness measurement.</p>
        </Collapsible>
      </SectionCard>
    </section>

    <!-- Section: Output & normalize -->
    <section class="space-y-2 mt-8">
      <SectionHeader label="Output & normalize" description="Run the batch once the output folder is set." />
      <SectionCard class="space-y-6">
        <Collapsible label="Technical details" body-id="normalize-output" class="mb-2">
          <p>
            <strong>Output format.</strong>
            Input format is preserved: MP3 → MP3, WAV → WAV, FLAC → FLAC, M4A/AAC → M4A, OGG → OGG, AIFF → AIFF.
            All output is 44.1 kHz, stereo (WAV 16-bit, MP3 320 kbps, FLAC lossless, etc.).
            By default, files are written as the original name stem plus
            <code class="rounded bg-muted px-1">-normalized.{same extension}</code>
            (e.g. <code class="rounded bg-muted px-1">song.mp3</code> → <code class="rounded bg-muted px-1">song-normalized.mp3</code>) in the chosen folder.
            When you choose to overwrite originals, the normalized audio replaces the existing files.
          </p>
        </Collapsible>
        <div class="space-y-3">
          <div class="flex items-center justify-between gap-3">
            <div class="flex items-center gap-2">
              <Filter class="h-4 w-4 text-muted-foreground" />
              <span class="text-sm font-medium text-foreground">Overwrite original files instead of adding “-normalized”</span>
            </div>
            <Toggle
              v-model="overwriteOriginals"
              :disabled="isProcessing"
              aria-label="Overwrite original files instead of adding -normalized suffix"
              @update:model-value="(val: boolean) => updatePreferences({ normalize_audio_add_suffix: !val })"
            />
          </div>
          <Callout v-if="overwriteOriginals" variant="warning">
            Overwriting will permanently replace the original audio files in the output folder. Make sure you have backups if you might need the originals later.
          </Callout>
        </div>
        <FolderSelectionZone
          :value="outputFolder"
          label="Output folder"
          description="Normalized files will be written here. Uses your preferred path if set in Settings."
          empty-message="Not selected"
          :is-loading="isLoadingOutput"
          :disabled="isProcessing"
          @select="selectOutputFolder"
          @clear="clearOutputFolderComposable"
        />
        <div class="pt-4 border-t border-border">
          <Button
            @click="normalizeAudio"
            :disabled="!canNormalize"
            class="w-full text-lg py-6 font-semibold"
            size="lg"
            variant="default"
          >
            <Music v-if="!isProcessing" class="h-5 w-5 mr-2" />
            <Loader2 v-else class="h-5 w-5 mr-2 animate-spin" />
            <span v-if="isProcessing">Normalizing...</span>
            <span v-else>Normalize audio ({{ audioFiles.length }})</span>
          </Button>
        </div>
      </SectionCard>
    </section>

    <!-- Progress -->
    <div v-if="isProcessing || normalizationProgress.total > 0" class="space-y-4 mt-8">
      <ProgressCard
        title="Normalization progress"
        :overall-progress="normalizationProgress.total > 0 ? (normalizationProgress.completed / normalizationProgress.total) * 100 : progress"
        :preparing-message="progressMessage || 'Preparing...'"
      >
        <template v-if="normalizationProgress.total > 0" #overall-summary>
          {{ normalizationProgress.completed }} / {{ normalizationProgress.total }} files normalized
        </template>
      </ProgressCard>

      <SectionCard v-if="normalizationResults">
        <SectionHeader label="Results" class="mb-2" />
        <div class="text-sm space-y-2">
          <div class="text-green-600 dark:text-green-400">✓ Successful: {{ normalizationResults.successful.length }}</div>
          <div v-if="normalizationResults.failed.length > 0" class="text-destructive">✗ Failed: {{ normalizationResults.failed.length }}</div>
          <div v-if="normalizationResults.successful.length > 0" class="mt-4">
            <SectionHeader label="Normalized files" />
            <div class="mt-2 space-y-1 max-h-48 overflow-y-auto text-xs text-muted-foreground">
              <div v-for="file in normalizationResults.successful" :key="file">
                <PathDisplay :path="file" filename-only />
              </div>
            </div>
          </div>
        </div>
      </SectionCard>
    </div>

    <!-- Status Messages -->
    <StatusAlert v-if="error" type="error" :message="error" />
    <StatusAlert v-if="success" type="success" :message="success" :auto-dismiss="true" />

    <!-- Modal: Cancel Operation Confirmation -->
    <Modal :open="showCancelConfirmModal" @close="cancelCancel">
      <div class="space-y-4">
        <h3 class="text-lg font-semibold">Cancel Operation?</h3>
        <p class="text-sm text-muted-foreground">
          Audio normalization is in progress. Are you sure you want to cancel? This will stop the current operation.
        </p>
        <div class="flex justify-end gap-2">
          <Button @click="cancelCancel" variant="outline">No, Continue</Button>
          <Button @click="confirmCancel" variant="destructive">Yes, Cancel</Button>
        </div>
      </div>
    </Modal>
  </PageContainer>
</template>
