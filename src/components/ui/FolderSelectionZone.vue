<script setup lang="ts">
import { computed } from "vue";
import { Folder, Loader2, XCircle, CheckCircle2 } from "lucide-vue-next";
import { cn } from "../../lib/utils";
import Button from "./Button.vue";
import PathDisplay from "./PathDisplay.vue";

export interface FolderSelectionZoneProps {
  value: string | null;
  label?: string;
  description?: string;
  isLoading?: boolean;
  disabled?: boolean;
  /** Show success checkmark when value is set */
  showSuccess?: boolean;
  /** Custom empty state message */
  emptyMessage?: string;
  class?: string;
}

const props = withDefaults(defineProps<FolderSelectionZoneProps>(), {
  isLoading: false,
  disabled: false,
  showSuccess: false,
  emptyMessage: "No default path set",
});

const emit = defineEmits<{
  (e: "select"): void;
  (e: "clear"): void;
}>();

const hasValue = computed(() => !!props.value);

// Remove the old folderName computed since PathDisplay handles this

const handleClick = () => {
  if (props.disabled || props.isLoading) return;
  emit("select");
};

const handleClear = (e: Event) => {
  e.stopPropagation();
  if (props.disabled || props.isLoading) return;
  emit("clear");
};

const zoneClass = computed(() =>
  cn(
    "group flex items-center gap-3 p-4 rounded-lg border-2 transition-all cursor-pointer",
    hasValue.value
      ? "border-border bg-muted/20 hover:border-primary/40 hover:bg-muted/30"
      : "border-dashed border-muted-foreground/30 bg-muted/10 hover:border-muted-foreground/50 hover:bg-muted/20",
    (props.disabled || props.isLoading) && "opacity-50 pointer-events-none",
    props.class
  )
);
</script>

<template>
  <div
    :class="zoneClass"
    role="button"
    tabindex="0"
    :aria-label="label || 'Choose folder'"
    @click="handleClick"
    @keydown.enter.prevent="handleClick"
    @keydown.space.prevent="handleClick"
  >
    <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-muted">
      <Folder class="h-5 w-5 text-muted-foreground" />
    </div>
    <div class="flex-1 min-w-0">
      <template v-if="hasValue">
        <p v-if="label" class="text-xs font-medium uppercase tracking-wider text-muted-foreground mb-0.5">
          {{ label }}
        </p>
        <PathDisplay 
          :path="value" 
          class="font-medium break-all"
          :empty-text="emptyMessage"
          :no-tooltip="false"
        />
      </template>
      <template v-else>
        <PathDisplay 
          :path="null" 
          class="font-medium"
          :empty-text="emptyMessage"
        />
        <p v-if="description" class="text-xs text-muted-foreground/70 mt-0.5">
          {{ description }}
        </p>
      </template>
    </div>
    <CheckCircle2
      v-if="showSuccess && hasValue && !isLoading"
      class="h-5 w-5 text-green-600 dark:text-green-400 shrink-0"
    />
    <Loader2
      v-else-if="isLoading"
      class="h-5 w-5 text-muted-foreground animate-spin shrink-0"
    />
    <Button
      v-else-if="hasValue"
      size="icon"
      variant="ghost"
      class="shrink-0 text-muted-foreground hover:text-destructive"
      :aria-label="`Clear ${label || 'folder'}`"
      @click.stop="handleClear"
    >
      <XCircle class="h-5 w-5" />
    </Button>
  </div>
</template>
