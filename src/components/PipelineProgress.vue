<script setup lang="ts">
import Card from "./ui/Card.vue";
import {
  CheckCircle2,
  XCircle,
  Loader2,
  Circle,
  Music2,
  Film,
  Upload,
  X,
  type LucideIcon,
} from "lucide-vue-next";
import { cn } from "../lib/utils";
import Button from "./ui/Button.vue";
import type { StepStatus, PipelinePhase, PipelineItem } from "../types";

const phaseIconById: Record<string, LucideIcon> = {
  normalize: Music2,
  videos: Film,
  upload: Upload,
};

export type { StepStatus, PipelinePhase, PipelineItem };

type PipelineProgressProps = {
  title: string;
  phases: PipelinePhase[];
  /** Per-phase items (e.g. videos). Key = phase id, value = items for that phase. */
  itemsByPhase?: Record<string, PipelineItem[]>;
  /** When true, show per-item rows (icon + name + step label + message). */
  showItemProgress?: boolean;
  /** Callback when cancel is clicked for a phase. Receives phase id. */
  onCancelPhase?: (phaseId: string) => void;
  /** Callback when cancel is clicked for a specific item. Receives phase id and item id. */
  onCancelItem?: (phaseId: string, itemId: string | number) => void;
};

const props = withDefaults(defineProps<PipelineProgressProps>(), {
  itemsByPhase: () => ({}),
  showItemProgress: true,
  onCancelPhase: undefined,
  onCancelItem: undefined,
});

const statusIcon = (status: StepStatus) => {
  switch (status) {
    case "done":
      return CheckCircle2;
    case "failed":
      return XCircle;
    case "cancelled":
      return XCircle;
    case "active":
      return Loader2;
    default:
      return Circle;
  }
};

const statusColor = (status: StepStatus) => {
  switch (status) {
    case "done":
      return "text-green-600 dark:text-green-500";
    case "failed":
      return "text-destructive";
    case "cancelled":
      return "text-muted-foreground";
    case "active":
      return "text-primary";
    default:
      return "text-muted-foreground";
  }
};

const canCancelPhase = (phase: PipelinePhase) => {
  return phase.status === "active" && props.onCancelPhase !== undefined;
};

const canCancelItem = (phase: PipelinePhase, item: PipelineItem) => {
  // Item must be pending or active to be cancellable
  if (item.status !== "pending" && item.status !== "active") {
    return false;
  }
  
  // For YouTube uploads, don't allow cancellation once uploading starts (status becomes "active")
  // because HTTP PUT requests cannot be cancelled mid-upload
  if (phase.id === "upload" && item.status === "active") {
    return false;
  }
  
  // For other phases (video creation, normalization), allow cancellation of active items
  // because they have periodic cancellation checks
  return props.onCancelItem !== undefined || canCancelPhase(phase);
};

const phaseItems = (phaseId: string) => props.itemsByPhase[phaseId] ?? [];
</script>

<template>
  <Card class="overflow-hidden">
    <div class="p-6 space-y-6">
      <div class="flex items-center justify-between">
        <h3 class="text-base font-semibold text-foreground">{{ title }}</h3>
      </div>

      <!-- Phase timeline -->
      <div class="space-y-6">
        <div
          v-for="(phase, idx) in phases"
          :key="phase.id"
          class="relative"
        >
          <!-- Connector line (except last) -->
          <div
            v-if="idx < phases.length - 1"
            class="absolute left-[11px] top-8 bottom-0 w-px bg-border -mb-6"
            aria-hidden="true"
          />

          <div class="flex gap-4">
            <!-- Phase icon -->
            <div
              class="relative z-10 flex h-8 w-8 shrink-0 items-center justify-center rounded-full border-2"
              :class="
                phase.status === 'active'
                  ? 'border-primary text-primary'
                  : phase.status === 'done'
                    ? 'border-green-500 text-green-600 dark:text-green-500'
                    : phase.status === 'failed'
                      ? 'border-destructive text-destructive'
                      : 'border-muted-foreground/40 text-muted-foreground'
              "
            >
              <component
                :is="phase.icon ?? phaseIconById[phase.id] ?? statusIcon(phase.status)"
                v-if="phase.status !== 'active'"
                class="h-4 w-4"
              />
              <Loader2
                v-else
                class="h-4 w-4 animate-spin"
              />
            </div>

            <!-- Phase content -->
            <div class="flex-1 min-w-0 space-y-2">
              <div class="flex flex-wrap items-center justify-between gap-2">
                <div class="flex-1 min-w-0">
                  <p class="font-medium text-foreground">{{ phase.label }}</p>
                  <p
                    v-if="phase.description || phase.message"
                    class="text-sm text-muted-foreground"
                  >
                    {{ phase.message ?? phase.description }}
                  </p>
                </div>
                <div class="flex items-center gap-2">
                  <div
                    v-if="phase.total != null && phase.total > 0"
                    class="text-sm font-medium tabular-nums text-muted-foreground"
                  >
                    {{ phase.completed ?? 0 }} / {{ phase.total }}
                  </div>
                  <Button
                    v-if="canCancelPhase(phase)"
                    variant="ghost"
                    size="icon"
                    class="h-8 w-8 text-muted-foreground hover:text-destructive"
                    @click="props.onCancelPhase?.(phase.id)"
                  >
                    <X class="h-4 w-4" />
                  </Button>
                </div>
              </div>

              <!-- Per-item rows (videos / uploads) – status-only, no progress bars -->
              <div
                v-if="showItemProgress && phaseItems(phase.id).length > 0"
                class="rounded-lg border space-y-3 p-3 max-h-[400px] overflow-y-auto" style="background: rgba(234, 179, 8, 0.08);"
              >
                <div
                  v-for="item in phaseItems(phase.id)"
                  :key="String(item.id)"
                  class="space-y-1.5"
                >
                  <div class="flex items-center gap-2">
                    <img
                      v-if="item.thumbnailUrl"
                      :src="item.thumbnailUrl"
                      :alt="''"
                      class="h-9 w-16 shrink-0 rounded object-cover border border-border"
                    />
                    <component
                      :is="item.status === 'active' ? Film : statusIcon(item.status)"
                      :class="cn('h-4 w-4 shrink-0', item.status === 'active' ? 'text-primary' : statusColor(item.status))"
                    />
                    <span class="text-sm font-medium truncate flex-1">{{ item.name }}</span>
                    <span
                      v-if="item.message || item.stepLabel"
                      class="text-xs text-muted-foreground shrink-0"
                    >
                      {{ item.message || item.stepLabel }}
                    </span>
                    <Button
                      v-if="canCancelItem(phase, item)"
                      variant="ghost"
                      size="icon"
                      class="h-7 w-7 text-muted-foreground hover:text-destructive shrink-0"
                      @click="props.onCancelItem ? props.onCancelItem(phase.id, item.id) : props.onCancelPhase?.(phase.id)"
                    >
                      <X class="h-3.5 w-3.5" />
                    </Button>
                  </div>
                  <p
                    v-if="item.error"
                    class="text-xs text-destructive pl-6"
                  >
                    {{ item.error }}
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Card>
</template>
