<script setup lang="ts">
import { computed } from "vue";
import { cn } from "../../lib/utils";
import { Info, AlertTriangle, XCircle, CheckCircle2, Circle, Zap } from "lucide-vue-next";

export type CalloutVariant = "info" | "warning" | "error" | "success" | "muted" | "action";

export interface CalloutProps {
  variant?: CalloutVariant;
  /** Optional title; body goes in default slot */
  title?: string;
  class?: string;
}

const props = withDefaults(defineProps<CalloutProps>(), {
  variant: "info",
});

const iconComponent = computed(() => {
  switch (props.variant) {
    case "error":
      return XCircle;
    case "success":
      return CheckCircle2;
    case "warning":
      return AlertTriangle;
    case "muted":
      return Circle;
    case "action":
      return Zap;
    case "info":
    default:
      return Info;
  }
});

const rootClass = computed(() => {
  const base = "relative w-full rounded-lg border p-4 flex gap-3";
  switch (props.variant) {
    case "success":
      return cn(base, "bg-green-500/10 text-green-700 border-green-500/50 dark:text-green-400 dark:border-green-500/50");
    case "warning":
      return cn(base, "bg-amber-500/10 text-amber-800 border-amber-500/50 dark:text-amber-200 dark:border-amber-500/50");
    case "error":
      return cn(base, "bg-destructive/10 text-destructive border-destructive/50");
    case "muted":
      return cn(base, "bg-muted/30 text-muted-foreground border-muted-foreground/30");
    case "action":
      return cn(base, "bg-primary/10 text-foreground border-primary/50 dark:border-primary/50");
    case "info":
    default:
      return cn(base, "bg-blue-500/10 text-blue-700 border-blue-500/50 dark:text-blue-300 dark:border-blue-500/50");
  }
});

const iconClass = computed(() => {
  switch (props.variant) {
    case "success":
      return "text-green-600 dark:text-green-400 shrink-0";
    case "warning":
      return "text-amber-600 dark:text-amber-400 shrink-0";
    case "error":
      return "text-destructive shrink-0";
    case "muted":
      return "text-muted-foreground shrink-0";
    case "action":
      return "text-primary shrink-0";
    case "info":
    default:
      return "text-blue-600 dark:text-blue-400 shrink-0";
  }
});
</script>

<template>
  <div :class="cn(rootClass, props.class)" role="status">
    <component :is="iconComponent" :class="cn('h-5 w-5 mt-0.5', iconClass)" aria-hidden />
    <div class="min-w-0 flex-1 space-y-1">
      <p v-if="title" class="font-semibold leading-tight">{{ title }}</p>
      <div class="text-sm leading-relaxed [&_a]:underline [&_a]:font-medium [&_a]:text-inherit">
        <slot />
      </div>
    </div>
  </div>
</template>
