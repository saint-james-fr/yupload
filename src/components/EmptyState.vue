<script setup lang="ts">
import { computed } from "vue";
import { Music, FileVideo, Folder, Image } from "lucide-vue-next";

export type EmptyStateIcon = "music" | "video" | "folder" | "image" | "none";

export interface EmptyStateProps {
  icon?: EmptyStateIcon;
  message: string;
  hint?: string;
  class?: string;
}

const props = withDefaults(defineProps<EmptyStateProps>(), {
  icon: "none",
});

const iconComponent = computed(() => {
  switch (props.icon) {
    case "music":
      return Music;
    case "video":
      return FileVideo;
    case "folder":
      return Folder;
    case "image":
      return Image;
    default:
      return null;
  }
});
</script>

<template>
  <div :class="['text-center py-12 text-muted-foreground', props.class]">
    <component
      v-if="iconComponent"
      :is="iconComponent"
      class="h-12 w-12 mx-auto mb-4 opacity-50"
    />
    <p>{{ message }}</p>
    <p v-if="hint" class="text-xs mt-2">{{ hint }}</p>
    <slot />
  </div>
</template>
