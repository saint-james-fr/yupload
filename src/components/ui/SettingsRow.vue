<script setup lang="ts">
import { type Component } from "vue";
import { cn } from "../../lib/utils";

export interface SettingsRowProps {
  icon?: Component;
  title: string;
  description?: string;
  disabled?: boolean;
  /** Show icon in a rounded container (like Settings.vue) */
  iconInContainer?: boolean;
  class?: string;
}

const props = withDefaults(defineProps<SettingsRowProps>(), {
  disabled: false,
  iconInContainer: false,
});
</script>

<template>
  <div :class="cn('flex items-center gap-3', props.class)">
    <div
      v-if="icon"
      :class="
        iconInContainer
          ? 'flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-muted'
          : 'shrink-0'
      "
    >
      <component
        :is="icon"
        :class="iconInContainer ? 'h-5 w-5 text-muted-foreground' : 'h-5 w-5 text-muted-foreground'"
      />
    </div>
    <div class="flex-1 min-w-0">
      <h3 class="text-sm font-medium text-foreground">{{ title }}</h3>
      <p v-if="description" class="text-xs text-muted-foreground mt-0.5">
        {{ description }}
      </p>
    </div>
    <slot />
  </div>
</template>
