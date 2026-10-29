<script setup lang="ts">
import { computed, ref } from "vue";

const props = defineProps<{
  src: string | null;
  alt?: string;
  /** Where to show the enlarged preview relative to the viewport */
  placement?: "top-right" | "bottom-right" | "top-left" | "bottom-left";
  /** Optional max height in pixels for the enlarged preview image */
  maxHeight?: number;
}>();

const isHovered = ref(false);

const positionClass = computed(() => {
  if (props.placement === "top-right") {
    return "top-4 right-4";
  }
  if (props.placement === "top-left") {
    return "top-4 left-4";
  }
  if (props.placement === "bottom-left") {
    return "bottom-4 left-4";
  }
  // Default: bottom-right of the viewport
  return "bottom-4 right-4";
});

const maxHeightStyle = computed(() => {
  const h = props.maxHeight ?? 360;
  return { maxHeight: `${h}px` };
});

const onEnter = () => {
  if (!props.src) return;
  isHovered.value = true;
};

const onLeave = () => {
  isHovered.value = false;
};
</script>

<template>
  <div
    class="relative"
    @mouseenter="onEnter"
    @mouseleave="onLeave"
  >
    <!-- Thumbnail / trigger -->
    <slot />

    <!-- Teleported enlarged preview, fixed so it isn't clipped by scroll containers -->
    <Teleport to="body">
      <div
        v-if="src && isHovered"
        class="pointer-events-none fixed z-[60] hidden md:block"
        :class="positionClass"
      >
        <div class="rounded-lg border border-border bg-background/95 shadow-2xl overflow-hidden max-w-xl">
          <img
            :src="src"
            :alt="alt"
            class="w-full object-contain bg-black"
            :style="maxHeightStyle"
          />
        </div>
      </div>
    </Teleport>
  </div>
</template>

