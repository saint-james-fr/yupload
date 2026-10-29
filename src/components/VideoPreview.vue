<script setup lang="ts">
import { computed } from "vue";
import { convertFileSrc, isTauri } from "@tauri-apps/api/core";
import { FileVideo } from "lucide-vue-next";

const props = withDefaults(
  defineProps<{
    /** Local file path (uses convertFileSrc when in Tauri) or data/remote URL as-is */
    filePath: string | null;
    label?: string;
    /** Use compact layout (smaller player) */
    compact?: boolean;
    /** Thumbnail image URL (data URL) to display as poster */
    thumbnail?: string | null;
  }>(),
  { label: "Preview", compact: true, thumbnail: null }
);

const videoSrc = computed(() => {
  if (!props.filePath) return null;
  if (props.filePath.startsWith("data:") || props.filePath.startsWith("http")) {
    return props.filePath;
  }
  if (!isTauri()) return null;
  try {
    return convertFileSrc(props.filePath);
  } catch {
    return null;
  }
});
</script>

<template>
  <div
    class="rounded-lg border-2 border-border bg-muted/20 overflow-hidden"
    :class="compact ? 'p-2' : 'p-3'"
  >
    <div v-if="!videoSrc" class="flex items-center gap-3 text-muted-foreground">
      <div
        class="shrink-0 rounded bg-muted/50 flex items-center justify-center aspect-video"
        :class="compact ? 'w-24' : 'w-32'"
      >
        <FileVideo :class="compact ? 'h-6 w-6' : 'h-8 w-8'" />
      </div>
      <p class="text-sm">{{ label }} — select a file</p>
    </div>
    <div v-else class="space-y-1">
      <p v-if="label" class="text-xs font-medium uppercase tracking-wider text-muted-foreground">
        {{ label }}
      </p>
      <video
        :src="videoSrc"
        :poster="thumbnail || undefined"
        controls
        playsinline
        preload="metadata"
        class="w-full rounded border border-border bg-black/5 aspect-video"
      />
    </div>
  </div>
</template>
