<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { UploadHistoryEntry } from "../types";
import Button from "../components/ui/Button.vue";
import PageContainer from "../components/layout/PageContainer.vue";
import PageHeader from "../components/layout/PageHeader.vue";
import EmptyState from "../components/EmptyState.vue";
import ProcessingStatusBadge from "../components/ui/ProcessingStatusBadge.vue";
import { useVideoProcessingStatus } from "../composables/useVideoProcessingStatus";
import { DiscAlbum, Calendar, Loader2, Minus, Search, ChevronLeft, ChevronRight, RefreshCw } from "lucide-vue-next";
import { openUrl } from "@tauri-apps/plugin-opener";
import Input from "../components/ui/Input.vue";
import SectionHeader from "../components/ui/SectionHeader.vue";
import BottomNavigation from "../components/ui/BottomNavigation.vue";

const uploads = ref<UploadHistoryEntry[]>([]);
const isLoading = ref(false);
const error = ref<string | null>(null);
const autoCheckInterval = ref<number | null>(null);
const searchQuery = ref("");
const currentPage = ref(1);
const itemsPerPage = 50;
const isSyncing = ref(false);

const { checkStatus } = useVideoProcessingStatus();

const sortedUploads = computed(() => {
  return [...uploads.value].sort((a, b) => {
    return new Date(b.uploadedAt).getTime() - new Date(a.uploadedAt).getTime();
  });
});

const filteredUploads = computed(() => {
  if (!searchQuery.value.trim()) {
    return sortedUploads.value;
  }
  
  const query = searchQuery.value.toLowerCase().trim();
  return sortedUploads.value.filter((upload) => {
    const titleMatch = upload.title.toLowerCase().includes(query);
    const discogsMatch = upload.discogsReleaseTitle?.toLowerCase().includes(query) ?? false;
    return titleMatch || discogsMatch;
  });
});

const paginatedUploads = computed(() => {
  const start = (currentPage.value - 1) * itemsPerPage;
  const end = start + itemsPerPage;
  return filteredUploads.value.slice(start, end);
});

const totalPages = computed(() => {
  return Math.ceil(filteredUploads.value.length / itemsPerPage);
});

const paginationInfo = computed(() => {
  const start = (currentPage.value - 1) * itemsPerPage + 1;
  const end = Math.min(currentPage.value * itemsPerPage, filteredUploads.value.length);
  return { start, end, total: filteredUploads.value.length };
});

const goToPage = (page: number) => {
  if (page >= 1 && page <= totalPages.value) {
    currentPage.value = page;
  }
};

watch(searchQuery, () => {
  currentPage.value = 1; // Reset to first page when search changes
});

const emit = defineEmits<{
  (e: "back"): void;
}>();

const goBack = () => {
  emit("back");
};

const loadUploadHistory = async () => {
  try {
    isLoading.value = true;
    error.value = null;
    const history = await invoke<UploadHistoryEntry[]>("get_upload_history");
    uploads.value = history;
  } catch (err) {
    error.value = `Failed to load upload history: ${err}`;
  } finally {
    isLoading.value = false;
  }
};

const updateUploadStatus = (videoId: string, status: UploadHistoryEntry["processingStatus"], checkedAt?: string, failureReason?: string | null) => {
  const upload = uploads.value.find(u => u.youtubeVideoId === videoId);
  if (upload) {
    upload.processingStatus = status;
    if (checkedAt) {
      upload.processingStatusCheckedAt = checkedAt;
    }
    if (failureReason !== undefined) {
      upload.processingFailureReason = failureReason;
    }
    // Save updated history
    saveUploadHistory();
  }
};

const saveUploadHistory = async () => {
  try {
    await invoke("save_upload_history", {
      uploads: uploads.value
    });
  } catch (err) {
    console.error("Failed to save upload history:", err);
  }
};

const checkVideoStatus = async (videoId: string) => {
  try {
    const result = await checkStatus(videoId);
    updateUploadStatus(
      videoId,
      result.status,
      result.checked_at,
      result.failure_reason
    );
    return result;
  } catch (err) {
    console.error(`Failed to check status for video ${videoId}:`, err);
    throw err;
  }
};

// Only check videos that are in progress (uploaded or processing status)
// Skip videos that are already processed, failed, or deleted to save API quota
const getVideosToCheck = (): UploadHistoryEntry[] => {
  return uploads.value.filter((upload) => {
    const status = upload.processingStatus;
    // Only check videos that are in progress or don't have a status yet (backward compatibility)
    return !status || status === "uploaded" || status === "processing";
  });
};

const hasInProgressVideos = computed(() => {
  return getVideosToCheck().length > 0;
});

// Manual sync: check all videos that need checking
const syncAllVideos = async () => {
  try {
    isSyncing.value = true;
    error.value = null;
    
    const videosToCheck = getVideosToCheck();
    
    if (videosToCheck.length === 0) {
      // No videos to check, but still show loading state for feedback
      await new Promise(resolve => setTimeout(resolve, 300));
      return;
    }
    
    // Ensure loading state is visible for at least 300ms for better UX feedback
    await Promise.all([
      Promise.allSettled(
        videosToCheck.map((upload) => checkVideoStatus(upload.youtubeVideoId))
      ),
      new Promise(resolve => setTimeout(resolve, 300))
    ]);
  } catch (err) {
    error.value = `Failed to sync videos: ${err}`;
  } finally {
    isSyncing.value = false;
  }
};

// Auto-check: only check videos in progress
const checkInProgressVideos = async () => {
  const videosToCheck = getVideosToCheck();
  if (videosToCheck.length === 0) {
    // No videos to check, stop polling
    stopAutoCheck();
    return;
  }

  // Check only videos that are in progress
  await Promise.allSettled(
    videosToCheck.map((upload) => checkVideoStatus(upload.youtubeVideoId))
  );
};

const startAutoCheck = () => {
  stopAutoCheck(); // Clear any existing interval

  // Only start polling if there are videos in progress
  if (!hasInProgressVideos.value) {
    return;
  }

  // Poll every 10 seconds when there are videos in progress
  autoCheckInterval.value = window.setInterval(() => {
    checkInProgressVideos();
  }, 10000);
};

const stopAutoCheck = () => {
  if (autoCheckInterval.value !== null) {
    clearInterval(autoCheckInterval.value);
    autoCheckInterval.value = null;
  }
};

const openUrlHandler = async (url: string) => {
  try {
    await openUrl(url);
  } catch (err) {
    error.value = `Failed to open URL: ${err}`;
  }
};

const formatDate = (dateString: string) => {
  const date = new Date(dateString);
  return date.toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit"
  });
};

onMounted(async () => {
  await loadUploadHistory();
  
  // Initial check on mount: check all videos that need checking
  await syncAllVideos();
  
  // Start auto-checking if there are videos in progress
  startAutoCheck();
});

onUnmounted(() => {
  stopAutoCheck();
});

// Watch for changes in upload status to start/stop polling
watch(
  hasInProgressVideos,
  (hasInProgress) => {
    if (hasInProgress) {
      // Start polling if there are videos in progress
      startAutoCheck();
    } else {
      // Stop polling if no videos are in progress
      stopAutoCheck();
    }
  }
);
</script>

<template>
  <PageContainer>
    <PageHeader
      title="Upload History"
      description="Videos you've uploaded to YouTube and linked Discogs releases. Sorted newest first, stored locally."
      @back="goBack"
    />

    <div v-if="isLoading" class="flex items-center justify-center py-12">
      <Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
    </div>

    <div v-else-if="error" class="p-4 rounded-lg border border-destructive bg-destructive/10">
      <p class="text-sm text-destructive">{{ error }}</p>
    </div>

    <EmptyState
      v-else-if="sortedUploads.length === 0"
      icon="video"
      message="No uploads yet"
      hint="Upload videos to see them here"
    />

    <div v-else class="space-y-4">
      <section class="space-y-2">
        <SectionHeader
          label="Your uploads"
          description="Open YouTube or Discogs links to view your uploaded videos."
        />
        <div class="flex items-center gap-3">
          <div class="relative flex-1 max-w-md">
            <Search class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
            <Input
              v-model="searchQuery"
              type="text"
              placeholder="Search by title or Discogs release..."
              class="pl-9"
            />
          </div>
          <Button
            @click="syncAllVideos"
            variant="outline"
            size="sm"
            :disabled="isSyncing || uploads.length === 0"
            class="gap-2 whitespace-nowrap"
            title="Manually check status of videos in progress"
          >
            <RefreshCw :class="['h-4 w-4', { 'animate-spin': isSyncing }]" />
            {{ isSyncing ? 'Syncing...' : 'Sync Status' }}
          </Button>
          <div v-if="filteredUploads.length > 0" class="text-xs text-muted-foreground whitespace-nowrap">
            Showing {{ paginationInfo.start }}-{{ paginationInfo.end }} of {{ paginationInfo.total }}
          </div>
        </div>
        <div class="rounded-lg border-2 border-border bg-card overflow-hidden">
          <div class="overflow-x-auto">
            <table class="w-full">
              <thead>
                <tr class="border-b border-border bg-muted/30">
                  <th class="text-left px-4 py-3 text-xs font-medium uppercase tracking-wider text-muted-foreground">Title</th>
                  <th class="text-left px-4 py-3 text-xs font-medium uppercase tracking-wider text-muted-foreground">Date</th>
                  <th class="text-left px-4 py-3 text-xs font-medium uppercase tracking-wider text-muted-foreground">Status</th>
                  <th class="text-center px-4 py-3 text-xs font-medium uppercase tracking-wider text-muted-foreground w-20">YouTube</th>
                  <th class="text-center px-4 py-3 text-xs font-medium uppercase tracking-wider text-muted-foreground w-20">Discogs</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-border">
                <tr
                  v-for="upload in paginatedUploads"
                  :key="upload.id"
                  class="hover:bg-muted/20 transition-colors"
                >
                  <td class="px-4 py-3">
                    <div class="text-sm font-medium text-foreground truncate max-w-xs" :title="upload.title">
                      {{ upload.title }}
                    </div>
                  </td>
                  <td class="px-4 py-3">
                    <div class="flex items-center gap-1.5 text-xs text-muted-foreground">
                      <Calendar class="h-3.5 w-3.5 shrink-0" />
                      <span class="whitespace-nowrap">{{ formatDate(upload.uploadedAt) }}</span>
                    </div>
                  </td>
                  <td class="px-4 py-3">
                    <ProcessingStatusBadge
                      :status="upload.processingStatus"
                      :checked-at="upload.processingStatusCheckedAt"
                      :failure-reason="upload.processingFailureReason"
                    />
                  </td>
                  <td class="px-4 py-3 text-center">
                    <Button
                      @click="openUrlHandler(upload.youtubeUrl)"
                      variant="ghost"
                      size="icon"
                      class="h-8 w-8 text-red-600 hover:bg-red-500/10 hover:text-red-700 dark:text-red-400 dark:hover:text-red-300"
                      aria-label="View on YouTube"
                    >
                      <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"/>
                      </svg>
                    </Button>
                  </td>
                  <td class="px-4 py-3 text-center">
                    <Button
                      v-if="upload.discogsReleaseUrl"
                      @click="openUrlHandler(upload.discogsReleaseUrl)"
                      variant="ghost"
                      size="icon"
                      class="h-8 w-8"
                      :aria-label="upload.discogsReleaseTitle ? `View ${upload.discogsReleaseTitle} on Discogs` : 'View on Discogs'"
                    >
                      <DiscAlbum class="h-4 w-4" stroke-width="2" />
                    </Button>
                    <Minus v-else class="h-4 w-4 mx-auto text-muted-foreground/50" />
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
        
        <div v-if="totalPages > 1" class="flex items-center justify-between pt-2">
          <Button
            @click="goToPage(currentPage - 1)"
            variant="outline"
            size="sm"
            :disabled="currentPage === 1"
            class="gap-2"
          >
            <ChevronLeft class="h-4 w-4" />
            Previous
          </Button>
          
          <div class="flex items-center gap-2">
            <span class="text-xs text-muted-foreground">
              Page {{ currentPage }} of {{ totalPages }}
            </span>
          </div>
          
          <Button
            @click="goToPage(currentPage + 1)"
            variant="outline"
            size="sm"
            :disabled="currentPage === totalPages"
            class="gap-2"
          >
            Next
            <ChevronRight class="h-4 w-4" />
          </Button>
        </div>
        
        <div v-if="filteredUploads.length === 0 && searchQuery.trim()" class="py-8 text-center">
          <p class="text-sm text-muted-foreground">No uploads found matching "{{ searchQuery }}"</p>
        </div>
      </section>
    </div>

    <!-- Bottom Navigation -->
    <BottomNavigation @back-to-menu="emit('back')" />
  </PageContainer>
</template>
