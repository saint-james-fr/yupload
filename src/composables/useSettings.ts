import { ref, computed, type Ref, type ComputedRef } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useNotifications } from "./useNotifications";
import { usePreferences } from "./usePreferences";
import { useYouTube } from "./useYouTube";
import { useDiscogs } from "./useDiscogs";
import { useOutputFolder } from "./useOutputFolder";
import type { ImageStyle, FitMode } from "../types";

export type DescriptionTemplateTagKey =
  | "artist"
  | "release"
  | "year"
  | "label"
  | "catalogNumber"
  | "genres"
  | "styles"
  | "discogsUrl";

export type DescriptionTemplateBlock = {
  id: string;
  type: "tag" | "text";
  enabled: boolean;
  tagKey?: DescriptionTemplateTagKey;
  content?: string;
};

export type SettingsTab = "general" | "creation" | "upload";

export type UseSettingsReturn = {
  // Notifications
  notifyOnUploadComplete: Ref<boolean>;
  isSavingNotifyPref: Ref<boolean>;
  permissionState: Ref<string>;
  loadNotifyOnUploadComplete: () => Promise<void>;
  saveNotifyOnUploadComplete: () => Promise<void>;
  isRequestingPermission: Ref<boolean>;
  requestNotificationPermission: () => Promise<void>;
  // YouTube / Discogs
  youtube: ReturnType<typeof useYouTube>;
  discogs: ReturnType<typeof useDiscogs>;
  isYouTubeAuthenticated: Ref<boolean>;
  youtubeError: Ref<string | null>;
  isYouTubeDisconnecting: Ref<boolean>;
  isDiscogsAuthenticated: Ref<boolean>;
  discogsError: Ref<string | null>;
  isDiscogsDisconnecting: Ref<boolean>;
  authenticateYouTube: () => Promise<void>;
  authenticateDiscogs: () => Promise<void>;
  disconnectYouTube: () => Promise<void>;
  disconnectDiscogs: () => Promise<void>;
  // Output paths
  preferredAudioOutputPath: Ref<string | null>;
  isSelectingAudioOutputPath: Ref<boolean>;
  preferredVideoOutputPath: Ref<string | null>;
  isSelectingVideoOutputPath: Ref<boolean>;
  preferredAudioInputPath: Ref<string | null>;
  isSelectingAudioInputPath: Ref<boolean>;
  preferredImageInputPath: Ref<string | null>;
  isSelectingImageInputPath: Ref<boolean>;
  selectPreferredAudioOutputPath: () => Promise<void>;
  clearPreferredAudioOutputPath: () => Promise<void>;
  selectPreferredVideoOutputPath: () => Promise<void>;
  clearPreferredVideoOutputPath: () => Promise<void>;
  selectPreferredAudioInputPath: () => Promise<void>;
  clearPreferredAudioInputPath: () => Promise<void>;
  selectPreferredImageInputPath: () => Promise<void>;
  clearPreferredImageInputPath: () => Promise<void>;
  // Logs & data
  isExportingLogs: Ref<boolean>;
  exportLogs: () => Promise<void>;
  clearDataModalOpen: Ref<boolean>;
  isClearingData: Ref<boolean>;
  clearDataError: Ref<string | null>;
  clearDataSuccess: Ref<string | null>;
  openClearDataModal: () => void;
  closeClearDataModal: () => void;
  runClearAllData: () => Promise<void>;
  // Preferences & init
  preferences: ReturnType<typeof usePreferences>["preferences"];
  loadPreferences: () => Promise<void>;
  // Creation defaults (computed)
  creationDefaultNormalize: ComputedRef<boolean>;
  creationDefaultHighPass: ComputedRef<boolean>;
  creationDefaultImageStyle: ComputedRef<ImageStyle>;
  creationDefaultFitMode: ComputedRef<FitMode>;
  creationDefaultBackgroundPredefinedId: ComputedRef<string | null>;
  // Upload defaults (computed)
  uploadDefaultPrivacy: ComputedRef<"public" | "private" | "unlisted" | "draft">;
  uploadDefaultCategoryId: ComputedRef<string>;
  uploadDefaultPlaylistId: ComputedRef<string | null>;
  // Backgrounds (creation tab)
  creationBackgrounds: Ref<Array<{ id: string; label: string; imagePath: string }>>;
  isLoadingCreationBackgrounds: Ref<boolean>;
  creationBackgroundsError: Ref<string | null>;
  userBackgrounds: Ref<Array<{ id: string; label: string; imagePath: string }>>;
  isLoadingUserBackgrounds: Ref<boolean>;
  userBackgroundsError: Ref<string | null>;
  loadCreationBackgrounds: () => Promise<void>;
  loadUserBackgrounds: () => Promise<void>;
  importBackground: () => Promise<void>;
  // Description template
  addTemplateBlock: (type: "tag" | "text") => void;
  removeTemplateBlock: (index: number) => void;
  moveTemplateBlock: (from: number, to: number) => void;
  onTemplateBlocksChange: () => void;
  resetTemplateToDefault: () => void;
  descriptionTemplatePreview: ComputedRef<string>;
  // Tab & init
  activeTab: Ref<SettingsTab>;
  initializeSettings: () => Promise<void>;
};

function defaultDescriptionTemplate(): DescriptionTemplateBlock[] {
  return [
    { id: "artists", type: "tag", enabled: true, tagKey: "artist" },
    { id: "release", type: "tag", enabled: true, tagKey: "release" },
    { id: "label", type: "tag", enabled: true, tagKey: "label" },
    { id: "catalogNumber", type: "tag", enabled: true, tagKey: "catalogNumber" },
    { id: "genres", type: "tag", enabled: true, tagKey: "genres" },
    { id: "styles", type: "tag", enabled: true, tagKey: "styles" },
    { id: "year", type: "tag", enabled: true, tagKey: "year" },
    { id: "divider", type: "text", enabled: true },
    { id: "discogs", type: "tag", enabled: true, tagKey: "discogsUrl" },
  ];
}

export function useSettings(): UseSettingsReturn {
  const { permissionState, initializePermission, requestPermissionExplicit, notify } = useNotifications();
  const notifyOnUploadComplete = ref(false);
  const isSavingNotifyPref = ref(false);

  const youtube = useYouTube();
  const isYouTubeDisconnecting = ref(false);
  const isYouTubeAuthenticated = youtube.isAuthenticated;
  const youtubeError = youtube.error;

  const discogs = useDiscogs();
  const isDiscogsDisconnecting = ref(false);
  const isDiscogsAuthenticated = discogs.isAuthenticated;
  const discogsError = discogs.error;

  const {
    outputFolder: preferredAudioOutputPath,
    isLoading: isSelectingAudioOutputPath,
    loadFromPreferences: loadPreferredAudioOutputPath,
    selectFolder: selectPreferredAudioOutputPathComposable,
  } = useOutputFolder({
    defaultPreferenceKey: "preferred_audio_output_path",
    outputType: "audio",
  });

  const {
    outputFolder: preferredVideoOutputPath,
    isLoading: isSelectingVideoOutputPath,
    loadFromPreferences: loadPreferredVideoOutputPath,
    selectFolder: selectPreferredVideoOutputPathComposable,
  } = useOutputFolder({
    defaultPreferenceKey: "preferred_video_output_path",
    outputType: "video",
  });

  const {
    outputFolder: preferredAudioInputPath,
    isLoading: isSelectingAudioInputPath,
    loadFromPreferences: loadPreferredAudioInputPath,
    selectFolder: selectPreferredAudioInputPathComposable,
  } = useOutputFolder({
    defaultPreferenceKey: "preferred_audio_input_path",
  });

  const {
    outputFolder: preferredImageInputPath,
    isLoading: isSelectingImageInputPath,
    loadFromPreferences: loadPreferredImageInputPath,
    selectFolder: selectPreferredImageInputPathComposable,
  } = useOutputFolder({
    defaultPreferenceKey: "preferred_image_input_path",
  });

  const isExportingLogs = ref(false);
  const clearDataModalOpen = ref(false);
  const isClearingData = ref(false);
  const clearDataError = ref<string | null>(null);
  const clearDataSuccess = ref<string | null>(null);

  const { preferences, loadPreferences, updatePreferences } = usePreferences();

  const creationDefaultNormalize = computed({
    get: () => preferences.value.preferences?.default_normalize_for_creation ?? false,
    set: (val: boolean) => {
      updatePreferences({ default_normalize_for_creation: val });
    },
  });

  const creationDefaultHighPass = computed({
    get: () => preferences.value.preferences?.default_high_pass_when_normalizing ?? false,
    set: (val: boolean) => {
      updatePreferences({ default_high_pass_when_normalizing: val });
    },
  });

  const creationBackgrounds = ref<Array<{ id: string; label: string; imagePath: string }>>([]);
  const isLoadingCreationBackgrounds = ref(false);
  const creationBackgroundsError = ref<string | null>(null);

  const userBackgrounds = ref<Array<{ id: string; label: string; imagePath: string }>>([]);
  const isLoadingUserBackgrounds = ref(false);
  const userBackgroundsError = ref<string | null>(null);

  const creationDefaultImageStyle = computed({
    get: () => (preferences.value.preferences?.default_image_style as ImageStyle) ?? "automatic",
    set: (val: ImageStyle) => {
      updatePreferences({ default_image_style: val });
      if (val === "custom-background" || val === "background-only") {
        loadCreationBackgrounds();
      }
    },
  });

  const creationDefaultFitMode = computed({
    get: () => (preferences.value.preferences?.default_fit_mode as FitMode) ?? "contain",
    set: (val: FitMode) => {
      updatePreferences({ default_fit_mode: val });
    },
  });

  const uploadDefaultPrivacy = computed({
    get: () => (preferences.value.preferences?.default_privacy_status as "public" | "private" | "unlisted" | "draft") ?? "private",
    set: (val: "public" | "private" | "unlisted" | "draft") => {
      updatePreferences({ default_privacy_status: val });
    },
  });

  const uploadDefaultCategoryId = computed({
    get: () => preferences.value.preferences?.default_category_id ?? "10",
    set: (val: string) => {
      updatePreferences({ default_category_id: val });
    },
  });

  const creationDefaultBackgroundPredefinedId = computed({
    get: () => preferences.value.preferences?.default_background_predefined_id ?? null,
    set: (val: string | null) => {
      updatePreferences({ default_background_predefined_id: val });
    },
  });

  const uploadDefaultPlaylistId = computed({
    get: () => preferences.value.preferences?.default_playlist_id ?? null,
    set: (val: string | null) => {
      updatePreferences({ default_playlist_id: val });
    },
  });

  const addTemplateBlock = (type: "tag" | "text") => {
    const current = (preferences.value.preferences?.description_template_blocks as DescriptionTemplateBlock[] | undefined) ?? [];
    const newBlock: DescriptionTemplateBlock = {
      id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      type,
      enabled: true,
      tagKey: type === "tag" ? "artist" : undefined,
      content: type === "text" ? "" : undefined,
    };
    const updated = [...current, newBlock];
    const normalized = updated.map((block) => {
      if (block.type === "tag" && !block.tagKey) {
        return { ...block, tagKey: "artist" as const };
      }
      return block;
    });
    updatePreferences({ description_template_blocks: normalized });
  };

  const removeTemplateBlock = (index: number) => {
    const current = (preferences.value.preferences?.description_template_blocks as DescriptionTemplateBlock[] | undefined) ?? [];
    updatePreferences({
      description_template_blocks: current.filter((_, i) => i !== index),
    });
  };

  const moveTemplateBlock = (from: number, to: number) => {
    const current = (preferences.value.preferences?.description_template_blocks as DescriptionTemplateBlock[] | undefined) ?? [];
    if (from < 0 || to < 0 || from >= current.length || to >= current.length) return;
    const copy = [...current];
    const [item] = copy.splice(from, 1);
    copy.splice(to, 0, item);
    updatePreferences({ description_template_blocks: copy });
  };

  const onTemplateBlocksChange = () => {
    const current = (preferences.value.preferences?.description_template_blocks as DescriptionTemplateBlock[] | undefined) ?? [];
    const normalized = current.map((block) => {
      if (block.type === "tag" && !block.tagKey) {
        return { ...block, tagKey: "artist" as const };
      }
      return block;
    });
    const needsUpdate = normalized.some((block, index) => {
      const original = current[index];
      return block.type === "tag" && !original?.tagKey && block.tagKey;
    });
    if (needsUpdate) {
      updatePreferences({ description_template_blocks: normalized });
    }
  };

  const resetTemplateToDefault = () => {
    const base = defaultDescriptionTemplate().map((b) => ({
      id: `${b.id}-${Date.now()}`,
      type: b.type,
      enabled: b.enabled,
      tagKey: b.tagKey,
      content: b.content,
    }));
    updatePreferences({ description_template_blocks: base });
  };

  const descriptionTemplatePreview = computed(() => {
    const template = (preferences.value.preferences?.description_template_blocks as DescriptionTemplateBlock[] | undefined) ?? [];
    if (!template || template.length === 0) {
      return "No blocks yet. Start with a tag or a text block below.";
    }
    const active = template.filter((b) => b.enabled);
    if (active.length === 0) {
      return "All blocks are disabled. Enable at least one block to see a preview.";
    }
    const lines: string[] = [];
    for (const block of active) {
      if (block.type === "tag") {
        switch (block.tagKey) {
          case "artist":
            lines.push("Artists: Example Artist");
            break;
          case "release":
            lines.push("Release: Example Release Title");
            break;
          case "year":
            lines.push("Year: 2024");
            break;
          case "label":
            lines.push("Label: Example Label");
            break;
          case "catalogNumber":
            lines.push("Catalogue Number: EX-123");
            break;
          case "genres":
            lines.push("Genres: Electronic, Ambient");
            break;
          case "styles":
            lines.push("Styles: Downtempo, Experimental");
            break;
          case "discogsUrl":
            lines.push("Discogs: https://www.discogs.com/release/123456");
            break;
          default:
            break;
        }
      } else if (block.type === "text" && block.content) {
        lines.push(block.content);
      }
    }
    return lines.length > 0 ? lines.join("\n") : "Template would currently produce an empty description.";
  });

  const loadCreationBackgrounds = async () => {
    if (creationBackgrounds.value.length > 0) return;
    try {
      isLoadingCreationBackgrounds.value = true;
      creationBackgroundsError.value = null;
      const list = await invoke<Array<[string, string, string]>>("list_predefined_backgrounds");
      creationBackgrounds.value = list.map(([id, label, imagePath]) => ({ id, label, imagePath }));
      const defaultBgId = creationDefaultBackgroundPredefinedId.value;
      if (defaultBgId) {
        const exists = creationBackgrounds.value.some((bg) => bg.id === defaultBgId);
        if (!exists) {
          const userExists = userBackgrounds.value.some((bg) => bg.id === defaultBgId);
          if (!userExists) {
            updatePreferences({ default_background_predefined_id: null });
          }
        }
      }
    } catch (err) {
      console.error("Failed to load creation backgrounds:", err);
      creationBackgroundsError.value = "Could not load background library.";
      creationBackgrounds.value = [];
    } finally {
      isLoadingCreationBackgrounds.value = false;
    }
  };

  const loadUserBackgrounds = async () => {
    try {
      isLoadingUserBackgrounds.value = true;
      userBackgroundsError.value = null;
      const list = await invoke<Array<[string, string, string]>>("list_user_backgrounds");
      userBackgrounds.value = list.map(([id, label, imagePath]) => ({ id, label, imagePath }));
      const defaultBgId = creationDefaultBackgroundPredefinedId.value;
      if (defaultBgId) {
        const exists = userBackgrounds.value.some((bg) => bg.id === defaultBgId);
        if (exists) return;
        const predefinedExists = creationBackgrounds.value.some((bg) => bg.id === defaultBgId);
        if (!predefinedExists) {
          updatePreferences({ default_background_predefined_id: null });
        }
      }
    } catch (err) {
      console.error("Failed to load user backgrounds:", err);
      userBackgroundsError.value = "Could not load user backgrounds.";
      userBackgrounds.value = [];
    } finally {
      isLoadingUserBackgrounds.value = false;
    }
  };

  const importBackground = async () => {
    try {
      isLoadingUserBackgrounds.value = true;
      userBackgroundsError.value = null;
      const path = await open({
        multiple: false,
        filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "gif", "bmp", "webp"] }],
      });
      if (path && typeof path === "string") {
        await invoke<string>("save_predefined_background", { imagePath: path });
        await loadUserBackgrounds();
      }
    } catch (err) {
      console.error("Failed to import background:", err);
      userBackgroundsError.value = err instanceof Error ? err.message : "Could not import background.";
    } finally {
      isLoadingUserBackgrounds.value = false;
    }
  };

  const loadNotifyOnUploadComplete = async () => {
    try {
      const enabled = preferences.value.preferences?.notify_on_upload_complete ?? false;
      notifyOnUploadComplete.value = enabled;
      await initializePermission();
    } catch (err) {
      console.error("Failed to load notification preference:", err);
    }
  };

  const saveNotifyOnUploadComplete = async () => {
    try {
      isSavingNotifyPref.value = true;
      if (notifyOnUploadComplete.value) {
        await requestPermissionExplicit();
      }
      await updatePreferences({ notify_on_upload_complete: notifyOnUploadComplete.value });
    } catch (err) {
      console.error("Failed to save notification preference:", err);
    } finally {
      isSavingNotifyPref.value = false;
    }
  };

  const isRequestingPermission = ref(false);
  const requestNotificationPermission = async () => {
    try {
      isRequestingPermission.value = true;
      await requestPermissionExplicit();
    } catch (err) {
      console.error("Failed to request notification permission:", err);
    } finally {
      isRequestingPermission.value = false;
    }
  };

  const selectPreferredAudioOutputPath = async () => {
    await selectPreferredAudioOutputPathComposable();
    if (preferredAudioOutputPath.value) {
      await updatePreferences({ preferred_audio_output_path: preferredAudioOutputPath.value ?? null });
    }
  };

  const clearPreferredAudioOutputPath = async () => {
    preferredAudioOutputPath.value = null;
    await updatePreferences({ preferred_audio_output_path: null });
  };

  const selectPreferredVideoOutputPath = async () => {
    await selectPreferredVideoOutputPathComposable();
    if (preferredVideoOutputPath.value) {
      await updatePreferences({ preferred_video_output_path: preferredVideoOutputPath.value ?? null });
    }
  };

  const clearPreferredVideoOutputPath = async () => {
    preferredVideoOutputPath.value = null;
    await updatePreferences({ preferred_video_output_path: null });
  };

  const selectPreferredAudioInputPath = async () => {
    await selectPreferredAudioInputPathComposable();
    if (preferredAudioInputPath.value) {
      await updatePreferences({ preferred_audio_input_path: preferredAudioInputPath.value ?? null });
    }
  };

  const clearPreferredAudioInputPath = async () => {
    preferredAudioInputPath.value = null;
    await updatePreferences({ preferred_audio_input_path: null });
  };

  const selectPreferredImageInputPath = async () => {
    await selectPreferredImageInputPathComposable();
    if (preferredImageInputPath.value) {
      await updatePreferences({ preferred_image_input_path: preferredImageInputPath.value ?? null });
    }
  };

  const clearPreferredImageInputPath = async () => {
    preferredImageInputPath.value = null;
    await updatePreferences({ preferred_image_input_path: null });
  };

  const disconnectYouTube = async () => {
    try {
      isYouTubeDisconnecting.value = true;
      await youtube.disconnect();
    } catch {
      // Error handled by composable
    } finally {
      isYouTubeDisconnecting.value = false;
    }
  };

  const disconnectDiscogs = async () => {
    try {
      isDiscogsDisconnecting.value = true;
      await discogs.disconnect();
    } catch {
      // Error handled by composable
    } finally {
      isDiscogsDisconnecting.value = false;
    }
  };

  const authenticateYouTube = async () => {
    await youtube.authenticate();
  };

  const authenticateDiscogs = async () => {
    await discogs.authenticate();
  };

  const activeTab = ref<SettingsTab>("general");

  const exportLogs = async () => {
    try {
      isExportingLogs.value = true;
      const exportPath = await invoke<string>("export_logs");
      notify("Logs exported", `Logs saved to: ${exportPath}`);
    } catch (err) {
      console.error("Failed to export logs:", err);
      notify("Export failed", `Failed to export logs: ${err}`);
    } finally {
      isExportingLogs.value = false;
    }
  };

  const openClearDataModal = () => {
    clearDataError.value = null;
    clearDataSuccess.value = null;
    clearDataModalOpen.value = true;
  };

  const closeClearDataModal = () => {
    clearDataModalOpen.value = false;
    clearDataError.value = null;
  };

  const runClearAllData = async () => {
    try {
      isClearingData.value = true;
      clearDataError.value = null;
      await invoke("clear_all_data");
      await youtube.disconnect();
      await discogs.disconnect();
      preferredAudioOutputPath.value = null;
      preferredVideoOutputPath.value = null;
      preferredAudioInputPath.value = null;
      preferredImageInputPath.value = null;
      notifyOnUploadComplete.value = false;
      clearDataSuccess.value = "All data cleared. You can close the app and uninstall it.";
      closeClearDataModal();
      setTimeout(() => {
        clearDataSuccess.value = null;
      }, 4000);
    } catch (err) {
      clearDataError.value = `Failed to clear data: ${err}`;
    } finally {
      isClearingData.value = false;
    }
  };

  const initializeSettings = async () => {
    await loadPreferences();
    const blocks = preferences.value.preferences?.description_template_blocks as DescriptionTemplateBlock[] | undefined;
    if (!blocks || blocks.length === 0) {
      resetTemplateToDefault();
    } else {
      onTemplateBlocksChange();
    }
    await youtube.checkAuthStatus();
    await discogs.checkAuthStatus();
    await loadPreferredAudioOutputPath();
    await loadPreferredVideoOutputPath();
    await loadPreferredAudioInputPath();
    await loadPreferredImageInputPath();
    await loadNotifyOnUploadComplete();
    await loadCreationBackgrounds();
    await loadUserBackgrounds();
  };

  return {
    notifyOnUploadComplete,
    isSavingNotifyPref,
    permissionState,
    loadNotifyOnUploadComplete,
    saveNotifyOnUploadComplete,
    isRequestingPermission,
    requestNotificationPermission,
    youtube,
    discogs,
    isYouTubeAuthenticated,
    youtubeError,
    isYouTubeDisconnecting,
    isDiscogsAuthenticated,
    discogsError,
    isDiscogsDisconnecting,
    authenticateYouTube,
    authenticateDiscogs,
    disconnectYouTube,
    disconnectDiscogs,
    preferredAudioOutputPath,
    isSelectingAudioOutputPath,
    preferredVideoOutputPath,
    isSelectingVideoOutputPath,
    preferredAudioInputPath,
    isSelectingAudioInputPath,
    preferredImageInputPath,
    isSelectingImageInputPath,
    selectPreferredAudioOutputPath,
    clearPreferredAudioOutputPath,
    selectPreferredVideoOutputPath,
    clearPreferredVideoOutputPath,
    selectPreferredAudioInputPath,
    clearPreferredAudioInputPath,
    selectPreferredImageInputPath,
    clearPreferredImageInputPath,
    isExportingLogs,
    exportLogs,
    clearDataModalOpen,
    isClearingData,
    clearDataError,
    clearDataSuccess,
    openClearDataModal,
    closeClearDataModal,
    runClearAllData,
    preferences,
    loadPreferences,
    creationDefaultNormalize,
    creationDefaultHighPass,
    creationDefaultImageStyle,
    creationDefaultFitMode,
    creationDefaultBackgroundPredefinedId,
    uploadDefaultPrivacy,
    uploadDefaultCategoryId,
    uploadDefaultPlaylistId,
    creationBackgrounds,
    isLoadingCreationBackgrounds,
    creationBackgroundsError,
    userBackgrounds,
    isLoadingUserBackgrounds,
    userBackgroundsError,
    loadCreationBackgrounds,
    loadUserBackgrounds,
    importBackground,
    addTemplateBlock,
    removeTemplateBlock,
    moveTemplateBlock,
    onTemplateBlocksChange,
    resetTemplateToDefault,
    descriptionTemplatePreview,
    activeTab,
    initializeSettings,
  };
}
