<script setup lang="ts">
import { computed } from "vue";
import { cn } from "../lib/utils";
import { XCircle, CheckCircle2, AlertTriangle, Circle } from "lucide-vue-next";
import Button from "./ui/Button.vue";

/** One state per row: success, warning, or muted (grayed). */
export type SelectableListItemStatus =
  | "success"
  | "warning"
  | "muted"
  | "assigned"
  | "previous"
  | "unassigned"
  | "pending"
  | "processing"
  | "completed"
  | "failed";

export interface SelectableListItemProps {
  isSelected: boolean;
  /** Single state: success (check), warning (triangle), muted (gray circle). Creation flow can use assigned/previous/unassigned. */
  status?: SelectableListItemStatus;
  showRemove?: boolean;
  disabled?: boolean;
  class?: string;
}

const props = withDefaults(defineProps<SelectableListItemProps>(), {
  showRemove: true,
  disabled: false,
});

const emit = defineEmits<{
  (e: "select"): void;
  (e: "remove"): void;
}>();

const itemClass = computed(() =>
  cn(
    "w-full text-left py-2 px-3 rounded-md border-2 transition-all duration-200 flex items-center justify-between gap-2",
    props.isSelected
      ? "border-primary bg-primary/5 shadow-sm"
      : "border-border hover:border-primary/30",
    props.disabled ? "opacity-50 cursor-not-allowed" : "cursor-pointer",
    props.class
  )
);

/** Map status to one of success | warning | muted for icon. */
const iconState = computed<"success" | "warning" | "muted">(() => {
  const s = props.status;
  if (!s) return "muted";
  if (s === "success" || s === "assigned" || s === "previous" || s === "completed") return "success";
  if (s === "warning" || s === "unassigned" || s === "failed") return "warning";
  return "muted";
});
</script>

<template>
  <button :class="itemClass" @click="emit('select')" :disabled="disabled">
    <div class="flex flex-1 items-center gap-2 min-w-0 overflow-hidden">
      <div class="min-w-0 flex-1 truncate">
        <slot name="content" />
      </div>
      <div v-if="status !== undefined" class="flex shrink-0">
        <span
          class="inline-flex"
          :class="
            iconState === 'success'
              ? 'text-green-600 dark:text-green-400'
              : iconState === 'warning'
                ? 'text-amber-600 dark:text-amber-400'
                : 'text-muted-foreground opacity-70'
          "
        >
          <CheckCircle2 v-if="iconState === 'success'" class="h-4 w-4 shrink-0" aria-hidden />
          <AlertTriangle v-else-if="iconState === 'warning'" class="h-4 w-4 shrink-0" aria-hidden />
          <Circle v-else class="h-3.5 w-3.5 shrink-0" aria-hidden />
        </span>
      </div>
    </div>
    <Button
      v-if="showRemove"
      size="icon"
      variant="ghost"
      class="shrink-0 text-muted-foreground hover:text-destructive"
      :disabled="disabled"
      aria-label="Remove"
      @click.stop="emit('remove')"
    >
      <XCircle class="h-4 w-4" />
    </Button>
  </button>
</template>
