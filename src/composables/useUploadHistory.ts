import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { UploadHistoryEntry } from "../types";

export function useUploadHistory() {
  const uploads = ref<UploadHistoryEntry[]>([]);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  const load = async () => {
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

  const add = async (entry: UploadHistoryEntry) => {
    try {
      // Load existing history
      const existingHistory = await invoke<UploadHistoryEntry[]>("get_upload_history");
      
      // Combine with new entry and save
      const updatedHistory = [...existingHistory, entry];
      await invoke("save_upload_history", {
        uploads: updatedHistory
      });
      
      uploads.value = updatedHistory;
    } catch (err) {
      error.value = `Failed to save upload history: ${err}`;
    }
  };

  const addMultiple = async (entries: UploadHistoryEntry[]) => {
    try {
      // Load existing history
      const existingHistory = await invoke<UploadHistoryEntry[]>("get_upload_history");
      
      // Combine with new entries and save
      const updatedHistory = [...existingHistory, ...entries];
      await invoke("save_upload_history", {
        uploads: updatedHistory
      });
      
      uploads.value = updatedHistory;
    } catch (err) {
      error.value = `Failed to save upload history: ${err}`;
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

  return {
    uploads,
    isLoading,
    error,
    load,
    add,
    addMultiple,
    formatDate
  };
}
