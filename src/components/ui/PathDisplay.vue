<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from "vue";
import { truncatePathByWidth, getPathDisplayName } from "../../lib/path-utils";
import { cn } from "../../lib/utils";

export interface PathDisplayProps {
  /** The full path to display */
  path: string | null;
  /** Maximum characters before truncation (overrides dynamic width calculation) */
  maxChars?: number;
  /** CSS classes to apply to the container */
  class?: string;
  /** Show only the filename instead of smart truncation */
  filenameOnly?: boolean;
  /** Custom empty state text */
  emptyText?: string;
  /** Disable tooltip on hover */
  noTooltip?: boolean;
}

const props = withDefaults(defineProps<PathDisplayProps>(), {
  emptyText: "No path selected",
  filenameOnly: false,
  noTooltip: false,
});

const containerRef = ref<HTMLElement>();
const containerWidth = ref(0);

// Reactive display text based on container width or maxChars
const displayText = computed(() => {
  if (!props.path) {
    return props.emptyText;
  }

  if (props.filenameOnly) {
    return getPathDisplayName(props.path);
  }

  if (props.maxChars) {
    return getPathDisplayName(props.path, props.maxChars);
  }

  // Use dynamic width calculation
  if (containerWidth.value > 0) {
    return truncatePathByWidth(props.path, containerWidth.value);
  }

  // Fallback to filename only if width not calculated yet
  return getPathDisplayName(props.path);
});

// Check if path is truncated for tooltip display
const isTruncated = computed(() => {
  if (!props.path || props.noTooltip) return false;
  
  if (props.filenameOnly) {
    return props.path !== getPathDisplayName(props.path);
  }

  return displayText.value !== props.path;
});

// Tooltip text (full path when truncated)
const tooltipText = computed(() => {
  return isTruncated.value ? (props.path ?? undefined) : undefined;
});

// Resize observer to track container width changes
let resizeObserver: ResizeObserver | null = null;

const updateContainerWidth = () => {
  if (containerRef.value) {
    containerWidth.value = containerRef.value.getBoundingClientRect().width;
  }
};

onMounted(() => {
  updateContainerWidth();
  
  if (containerRef.value && window.ResizeObserver) {
    resizeObserver = new ResizeObserver(() => {
      updateContainerWidth();
    });
    resizeObserver.observe(containerRef.value);
  }
  
  // Fallback for browsers without ResizeObserver
  window.addEventListener('resize', updateContainerWidth);
});

onUnmounted(() => {
  if (resizeObserver) {
    resizeObserver.disconnect();
  }
  window.removeEventListener('resize', updateContainerWidth);
});

// CSS classes for the display text
const textClass = computed(() =>
  cn(
    "text-sm",
    props.path ? "text-foreground" : "text-muted-foreground",
    props.class
  )
);
</script>

<template>
  <span
    ref="containerRef"
    :class="textClass"
    :title="tooltipText"
    class="inline-block min-w-0 max-w-full"
  >
    {{ displayText }}
  </span>
</template>