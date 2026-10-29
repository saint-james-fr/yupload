<script setup lang="ts">
import { computed } from "vue";
import type { VideoProcessingStatus } from "../../types";
import { Clock, CheckCircle2, XCircle, Upload, Trash2 } from "lucide-vue-next";

const props = defineProps<{
  status?: VideoProcessingStatus;
  checkedAt?: string;
  failureReason?: string | null;
}>();

const statusConfig = {
  uploaded: {
    label: "Uploaded",
    color: "text-muted-foreground",
    bgColor: "bg-muted",
    icon: Upload,
  },
  processing: {
    label: "Processing",
    color: "text-yellow-600 dark:text-yellow-500",
    bgColor: "bg-yellow-50 dark:bg-yellow-950/30",
    icon: Clock,
  },
  processed: {
    label: "Ready",
    color: "text-green-600 dark:text-green-500",
    bgColor: "bg-green-50 dark:bg-green-950/30",
    icon: CheckCircle2,
  },
  failed: {
    label: "Failed",
    color: "text-red-600 dark:text-red-500",
    bgColor: "bg-red-50 dark:bg-red-950/30",
    icon: XCircle,
  },
  deleted: {
    label: "Deleted",
    color: "text-gray-600 dark:text-gray-400",
    bgColor: "bg-gray-50 dark:bg-gray-950/30",
    icon: Trash2,
  },
};

const config = computed(() => {
  if (!props.status) {
    return {
      label: "Unknown",
      color: "text-muted-foreground",
      bgColor: "bg-muted",
      icon: Clock,
    };
  }
  return statusConfig[props.status];
});

const formatCheckedAt = (dateString?: string) => {
  if (!dateString) return null;
  try {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    
    if (diffMins < 1) return "Just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) return `${diffHours}h ago`;
    const diffDays = Math.floor(diffHours / 24);
    return `${diffDays}d ago`;
  } catch {
    return null;
  }
};
</script>

<template>
  <div
    v-if="status"
    :class="[
      'inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md text-xs font-medium',
      config.color,
      config.bgColor,
    ]"
  >
    <component :is="config.icon" class="h-3.5 w-3.5" />
    <span>{{ config.label }}</span>
    <span
      v-if="checkedAt && status !== 'deleted'"
      class="text-muted-foreground/70 text-[10px]"
      :title="new Date(checkedAt).toLocaleString()"
    >
      {{ formatCheckedAt(checkedAt) }}
    </span>
  </div>
  <div
    v-else
    class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md text-xs font-medium text-muted-foreground bg-muted"
  >
    <Clock class="h-3.5 w-3.5" />
    <span>Not checked</span>
  </div>
</template>
