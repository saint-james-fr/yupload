import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { DiscogsRelease } from "../types";

export type OperationType = "youtube_upload" | "bulk_video_creation";

export const useSharedModals = () => {
  // Cancel confirmation modal
  const showCancelConfirmModal = ref(false);
  const pendingOperationType = ref<OperationType | null>(null);
  
  // YouTube connection modal
  const showYouTubeConnectModal = ref(false);
  
  // Discogs release linking modal
  const showLinkToOtherVideosModal = ref(false);
  const pendingRelease = ref<DiscogsRelease | null>(null);
  const selectedItemsForLinking = ref<Set<number>>(new Set());
  
  // Cancel confirmation
  const showCancelConfirmation = (operationType: OperationType) => {
    pendingOperationType.value = operationType;
    showCancelConfirmModal.value = true;
  };
  
  const confirmCancel = async (onCancel?: () => void) => {
    showCancelConfirmModal.value = false;
    
    if (pendingOperationType.value) {
      try {
        await invoke("cancel_operation", { operationType: pendingOperationType.value });
      } catch (err) {
        console.error("Failed to cancel operation:", err);
      }
    }
    
    pendingOperationType.value = null;
    
    if (onCancel) {
      onCancel();
    }
  };
  
  const cancelCancel = () => {
    showCancelConfirmModal.value = false;
    pendingOperationType.value = null;
  };
  
  // YouTube connection
  const showYouTubeConnection = () => {
    showYouTubeConnectModal.value = true;
  };
  
  const hideYouTubeConnection = () => {
    showYouTubeConnectModal.value = false;
  };
  
  // Discogs release linking
  const showReleaseLinker = (release: DiscogsRelease, totalItems: number) => {
    if (totalItems <= 1) return; // No point showing modal for single item
    
    pendingRelease.value = release;
    selectedItemsForLinking.value = new Set();
    showLinkToOtherVideosModal.value = true;
  };
  
  const toggleItemForLinking = (index: number, currentIndex: number | null) => {
    if (index === currentIndex) return; // Can't select current item
    
    if (selectedItemsForLinking.value.has(index)) {
      selectedItemsForLinking.value.delete(index);
    } else {
      selectedItemsForLinking.value.add(index);
    }
  };
  
  const applyReleaseToSelectedItems = (
    onApply: (release: DiscogsRelease, selectedIndices: number[]) => void
  ) => {
    if (!pendingRelease.value) return;
    
    const selectedIndices = Array.from(selectedItemsForLinking.value);
    onApply(pendingRelease.value, selectedIndices);
    
    // Close modal
    showLinkToOtherVideosModal.value = false;
    pendingRelease.value = null;
    selectedItemsForLinking.value = new Set();
  };
  
  const cancelReleaseLinker = () => {
    showLinkToOtherVideosModal.value = false;
    pendingRelease.value = null;
    selectedItemsForLinking.value = new Set();
  };
  
  return {
    // Cancel confirmation
    showCancelConfirmModal,
    showCancelConfirmation,
    confirmCancel,
    cancelCancel,
    
    // YouTube connection
    showYouTubeConnectModal,
    showYouTubeConnection,
    hideYouTubeConnection,
    
    // Discogs release linking
    showLinkToOtherVideosModal,
    pendingRelease,
    selectedItemsForLinking,
    showReleaseLinker,
    toggleItemForLinking,
    applyReleaseToSelectedItems,
    cancelReleaseLinker,
  };
};