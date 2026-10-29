<script setup lang="ts">
import { ref, watch, computed } from "vue";
import Alert from "./ui/Alert.vue";
import { cn } from "../lib/utils";
import { XCircle, CheckCircle2, Info, AlertCircle } from "lucide-vue-next";

export type AlertType = "error" | "success" | "info" | "warning";

export interface StatusAlertProps {
  type: AlertType;
  message: string;
  autoDismiss?: boolean;
  dismissDelay?: number;
  class?: string;
}

const props = withDefaults(defineProps<StatusAlertProps>(), {
  autoDismiss: false,
  dismissDelay: 5000,
});

const isVisible = ref(true);

watch(() => props.message, () => {
  isVisible.value = true;
  if (props.autoDismiss && props.type === "success") {
    setTimeout(() => {
      isVisible.value = false;
    }, props.dismissDelay);
  }
}, { immediate: true });

const iconComponent = computed(() => {
  switch (props.type) {
    case "error":
      return XCircle;
    case "success":
      return CheckCircle2;
    case "info":
      return Info;
    case "warning":
      return AlertCircle;
    default:
      return Info;
  }
});

const alertVariant = computed(() => {
  switch (props.type) {
    case "error":
      return "destructive";
    case "success":
      return "default";
    case "info":
    case "warning":
      return "default";
    default:
      return "default";
  }
});

const alertClass = computed(() => {
  if (props.type === "success") {
    return "bg-green-500/10 text-green-600 border-green-500/50";
  }
  if (props.type === "warning") {
    return "bg-yellow-500/10 text-yellow-600 border-yellow-500/50";
  }
  if (props.type === "info") {
    return "bg-blue-500/10 text-blue-600 border-blue-500/50";
  }
  return "";
});
</script>

<template>
  <Alert v-if="isVisible && message" :variant="alertVariant" :class="cn(alertClass, props.class)">
    <div class="flex items-center gap-2">
      <component :is="iconComponent" class="h-4 w-4" />
      <p class="whitespace-pre-line">{{ message }}</p>
    </div>
  </Alert>
</template>
