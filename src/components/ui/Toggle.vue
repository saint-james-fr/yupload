<script setup lang="ts">
import { computed } from "vue";
import { cn } from "../../lib/utils";

export interface ToggleProps {
  modelValue?: boolean;
  disabled?: boolean;
  /** Show "On" / "Off" text next to the switch */
  showLabel?: boolean;
  /** Accessible name when used without visible label */
  ariaLabel?: string;
  class?: string;
}

const props = withDefaults(defineProps<ToggleProps>(), {
  modelValue: false,
  disabled: false,
  showLabel: false,
});

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
}>();

const isOn = computed({
  get: () => props.modelValue,
  set: (val) => emit("update:modelValue", val),
});

const toggle = () => {
  if (props.disabled) return;
  isOn.value = !isOn.value;
};
</script>

<template>
  <div class="flex items-center gap-2" :class="props.class">
    <button
      type="button"
      role="switch"
      :aria-checked="isOn"
      :aria-label="ariaLabel"
      :disabled="disabled"
      :class="
        cn(
          'relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50',
          isOn ? 'bg-primary' : 'bg-input'
        )
      "
      @click="toggle"
    >
      <span
        :class="
          cn(
            'pointer-events-none inline-block h-5 w-5 transform rounded-full shadow ring-0 transition duration-200 ease-in-out',
            isOn ? 'translate-x-5 bg-primary-foreground' : 'translate-x-0.5 bg-muted-foreground'
          )
        "
      />
    </button>
    <span
      v-if="showLabel"
      class="text-sm text-foreground select-none"
      :class="{ 'opacity-50': disabled }"
    >
      {{ isOn ? "On" : "Off" }}
    </span>
  </div>
</template>
