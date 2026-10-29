<script setup lang="ts">
import { computed } from "vue";
import { cn } from "../../lib/utils";

type MaxWidth = "sm" | "md" | "lg" | "xl" | "2xl" | "3xl" | "4xl" | "5xl" | "6xl" | "7xl" | "full";

export interface PageContainerProps {
  maxWidth?: MaxWidth;
  spacing?: "4" | "6" | "8";
  class?: string;
}

const props = withDefaults(defineProps<PageContainerProps>(), {
  maxWidth: "7xl",
  spacing: "6",
});

const containerClass = computed(() =>
  cn(
    "min-h-screen p-4 md:p-8 page-container-bg animate-fadeInUp",
    props.class
  )
);

const innerClass = computed(() => {
  const maxWidthClasses: Record<MaxWidth, string> = {
    sm: "max-w-sm",
    md: "max-w-md",
    lg: "max-w-lg",
    xl: "max-w-xl",
    "2xl": "max-w-2xl",
    "3xl": "max-w-3xl",
    "4xl": "max-w-4xl",
    "5xl": "max-w-5xl",
    "6xl": "max-w-6xl",
    "7xl": "max-w-7xl",
    full: "max-w-full",
  };
  
  const spacingClasses: Record<string, string> = {
    "4": "space-y-4",
    "6": "space-y-6",
    "8": "space-y-8",
  };
  
  return cn(
    maxWidthClasses[props.maxWidth],
    "mx-auto",
    spacingClasses[props.spacing]
  );
});
</script>

<template>
  <div :class="containerClass">
    <div :class="innerClass">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.page-container-bg {
  background: var(--color-background);
}

@media (prefers-color-scheme: dark) {
  .page-container-bg {
    background: hsl(0 0% 8%);
  }
}

@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

</style>
