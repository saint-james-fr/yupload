<script setup lang="ts">
import { computed } from "vue";
import { cn } from "../../lib/utils";

export interface InputProps {
  type?: string;
  placeholder?: string;
  disabled?: boolean;
  modelValue?: string;
}

const props = withDefaults(defineProps<InputProps>(), {
  type: "text",
  disabled: false,
});

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const inputClass = computed(() =>
  cn(
    "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
  )
);

const value = computed({
  get: () => props.modelValue || "",
  set: (val) => emit("update:modelValue", val),
});
</script>

<template>
  <input
    :type="type"
    :class="inputClass"
    :placeholder="placeholder"
    :disabled="disabled"
    v-model="value"
  />
</template>
