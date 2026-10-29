<script setup lang="ts">
import { computed } from "vue";
import { convertFileSrc, isTauri } from "@tauri-apps/api/core";
import { Music } from "lucide-vue-next";

const props = withDefaults(
  defineProps<{
    /** Local file path (uses convertFileSrc when in Tauri) or data/remote URL as-is */
    filePath: string | null;
    label?: string;
    /** Use compact layout (reduced height) */
    compact?: boolean;
  }>(),
  { label: "Preview", compact: true }
);

const audioSrc = computed(() => {
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
    <div v-if="!audioSrc" class="flex items-center gap-3 text-muted-foreground">
      <div
        class="shrink-0 rounded bg-muted/50 flex items-center justify-center"
        :class="compact ? 'h-10 w-10' : 'h-12 w-12'"
      >
        <Music :class="compact ? 'h-5 w-5' : 'h-6 w-6'" />
      </div>
      <p class="text-sm">{{ label }} — select a file</p>
    </div>
    <div v-else class="space-y-1">
      <p v-if="label" class="text-xs font-medium uppercase tracking-wider text-muted-foreground">
        {{ label }}
      </p>
      <audio
        :src="audioSrc"
        controls
        preload="metadata"
        class="w-full max-w-full"
        :class="compact ? 'h-8' : 'h-10'"
      />
    </div>
  </div>
</template>
