<script setup lang="ts">
import { computed } from "vue";
import { Loader2 } from "lucide-vue-next";
import ImageHoverPreview from "./ImageHoverPreview.vue";

const props = defineProps<{
  src: string | null;
  /** Optional higher‑quality source for hover preview (falls back to src when omitted). */
  hoverSrc?: string | null;
  label?: string;
  selected?: boolean;
  disabled?: boolean;
  loading?: boolean;
  /** Visual aspect ratio of the thumbnail. Defaults to \"video\" (16:9). */
  aspect?: "video" | "square";
  /** Extra classes applied to the outer button (width, shrink, etc.). */
  containerClass?: string;
  /** Where to show the hover preview. Defaults to \"bottom-left\". */
  hoverPlacement?: "top-right" | "bottom-right" | "top-left" | "bottom-left";
}>();

const emit = defineEmits<{
  (e: "click"): void;
}>();

const aspectClass = computed(() => (props.aspect === "square" ? "aspect-square" : "aspect-video"));

const onClick = () => {
  if (props.disabled) return;
  emit("click");
};
</script>

<template>
  <button
    type="button"
    :disabled="disabled"
    @click="onClick"
    :class="[
      'flex flex-col gap-2 p-3 rounded-lg border-2 transition-all text-left cursor-pointer focus:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2',
      selected ? 'border-primary bg-primary/5 shadow-sm ring-2 ring-primary/20' : 'border-border hover:border-muted-foreground/40 hover:bg-muted/30',
      containerClass,
    ]"
  >
    <ImageHoverPreview
      v-if="hoverSrc || src"
      :src="hoverSrc || src"
      :alt="label"
      :placement="hoverPlacement || 'bottom-left'"
    >
      <div
        class="w-full rounded-md overflow-hidden bg-muted border border-border"
        :class="aspectClass"
      >
        <img
          v-if="src && !loading"
          :src="src"
          :alt="label"
          class="w-full h-full object-cover"
        />
        <div
          v-else
          class="w-full h-full flex items-center justify-center"
        >
          <Loader2 class="h-4 w-4 animate-spin text-muted-foreground" />
        </div>
      </div>
    </ImageHoverPreview>

    <p
      v-if="label"
      class="text-xs text-muted-foreground truncate capitalize font-medium"
    >
      {{ label }}
    </p>
  </button>
</template>

