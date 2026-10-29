import { ref, computed, watch } from "vue";
import type { Ref } from "vue";
import type { DiscogsRelease, DiscogsSearchResult } from "../types";
import { useDiscogs } from "./useDiscogs";
import { useDescriptionTemplate } from "./useDescriptionTemplate";

/**
 * Base metadata type that both UnifiedVideoMetadata and VideoUploadMetadata share
 */
export type BaseVideoMetadata = {
  title: string;
  description: string;
  tags: string[];
  categoryId: string;
  privacyStatus: "public" | "private" | "unlisted" | "draft";
  selfDeclaredMadeForKids: boolean;
  publishAt: string | null;
  playlistId: string | null;
  discogsReleaseId: number | null;
  discogsRelease: DiscogsRelease | null;
};

/**
 * Composable for managing video metadata editing
 * Handles tag management, Discogs search integration, and metadata updates
 */
export function useVideoMetadata<T extends BaseVideoMetadata>(
  metadata: Ref<T>,
  updateFn: (field: keyof T, value: any) => void
) {
  const discogs = useDiscogs();
  const { applyToVideo, buildDescriptionFromTemplate, cleanDiscogsName, preferences } = useDescriptionTemplate();

  // Tag management
  const tagInput = ref("");

  const addTag = (tag: string) => {
    if (metadata.value && tag.trim()) {
      const tags = metadata.value.tags;
      if (!tags.includes(tag.trim())) {
        updateFn("tags" as keyof T, [...tags, tag.trim()]);
      }
      tagInput.value = "";
    }
  };

  const removeTag = (tagToRemove: string) => {
    if (metadata.value) {
      updateFn(
        "tags" as keyof T,
        metadata.value.tags.filter((t) => t !== tagToRemove)
      );
    }
  };

  // Discogs search integration
  const discogsSearchQuery = computed({
    get: () => discogs.searchQuery.value,
    set: (val: string) => {
      discogs.searchQuery.value = val;
    },
  });

  const discogsSearchResults = computed(() => discogs.searchResults.value);
  const isSearchingDiscogs = computed(() => discogs.isSearching.value);
  const isDiscogsAuthenticated = computed(() => discogs.isAuthenticated.value);
  const isDiscogsAuthenticating = computed(() => discogs.isAuthenticating.value);
  const discogsError = computed(() => discogs.error.value);

  // Debounced search - watch the query and auto-search (like images section)
  let searchTimeout: ReturnType<typeof setTimeout> | null = null;
  
  watch(discogsSearchQuery, (query) => {
    if (searchTimeout) {
      clearTimeout(searchTimeout);
    }
    if (!query.trim() || query.trim().length < 3 || !isDiscogsAuthenticated.value) {
      discogs.clearSearch();
      return;
    }
    searchTimeout = setTimeout(() => {
      searchDiscogs();
    }, 500);
  });

  const onSearchInput = () => {
    // This is called on @input but we watch the query directly, so this is just for compatibility
  };

  const searchDiscogs = async () => {
    if (!discogsSearchQuery.value.trim() || !isDiscogsAuthenticated.value) return;
    await discogs.search(discogs.searchQuery.value);
  };

  const selectDiscogsRelease = async (result: DiscogsSearchResult) => {
    if (!metadata.value) return;

    const release = await discogs.selectRelease(result);
    if (!release) return;

    // Update metadata with release
    updateFn("discogsReleaseId" as keyof T, release.id);
    updateFn("discogsRelease" as keyof T, release);

    // Always autofill description using the current template when linking a release
    const template = preferences.value.preferences?.description_template_blocks ?? null;
    const fromTemplate = buildDescriptionFromTemplate(template as any, release, metadata.value as any);
    if (fromTemplate) {
      updateFn("description" as keyof T, fromTemplate);
    } else {
      // Fallback: create a basic description if no template is configured
      const lines: string[] = [];
      if (release.artists.length > 0) {
        const artistNames = release.artists.map(a => cleanDiscogsName(a.name)).join(", ");
        lines.push(`Artists: ${artistNames}`);
      }
      if (release.title) {
        lines.push(`Release: ${release.title}`);
      }
      if (release.year) {
        lines.push(`Year: ${release.year}`);
      }
      if (release.labels.length > 0) {
        const labelNames = release.labels.map(l => cleanDiscogsName(l.name)).join(", ");
        lines.push(`Label: ${labelNames}`);
      }
      if (release.genres.length > 0) {
        lines.push(`Genres: ${release.genres.join(", ")}`);
      }
      if (release.styles.length > 0) {
        lines.push(`Styles: ${release.styles.join(", ")}`);
      }
      const url = release.uri || `https://www.discogs.com/release/${release.id}`;
      lines.push(`Discogs: ${url}`);
      
      if (lines.length > 0) {
        updateFn("description" as keyof T, lines.join("\n"));
      }
    }
    
    // Always autofill tags from genres and styles when linking a release
    const tags: string[] = [];
    if (release.genres.length > 0) {
      tags.push(...release.genres);
    }
    if (release.styles.length > 0) {
      tags.push(...release.styles);
    }
    // Add artist names as tags (cleaned)
    if (release.artists.length > 0) {
      tags.push(...release.artists.map(a => cleanDiscogsName(a.name)));
    }
    // Add label names as tags (cleaned)
    if (release.labels.length > 0) {
      tags.push(...release.labels.map(l => cleanDiscogsName(l.name)));
    }
    const uniqueTags = [...new Set(tags)]; // Remove duplicates
    if (uniqueTags.length > 0) {
      updateFn("tags" as keyof T, uniqueTags);
    }
  };

  const clearDiscogsRelease = () => {
    if (metadata.value) {
      updateFn("discogsRelease" as keyof T, null);
      updateFn("discogsReleaseId" as keyof T, null);
    }
  };

  const authenticateDiscogs = async () => {
    await discogs.authenticate();
  };

  // Template application
  const applyDescriptionTemplate = () => {
    if (metadata.value) {
      applyToVideo(metadata.value as any);
    }
  };

  // Validation helpers
  const hasTitle = computed(() => {
    return (metadata.value?.title?.trim() ?? "").length > 0;
  });

  const hasDescription = computed(() => {
    return (metadata.value?.description?.trim() ?? "").length > 0;
  });

  const metadataStatus = computed<"success" | "warning" | "muted">(() => {
    if (!hasTitle.value) return "muted";
    if (!hasDescription.value) return "warning";
    return "success";
  });

  const statusCallout = computed(() => {
    if (!hasTitle.value)
      return {
        variant: "muted" as const,
        message: "No title set. Add a title above.",
      };
    if (!hasDescription.value)
      return {
        variant: "warning" as const,
        message: "Description missing. Add a description for better discoverability.",
      };
    return {
      variant: "success" as const,
      message: "Title and description set. Ready to upload.",
    };
  });

  return {
    // Tag management
    tagInput,
    addTag,
    removeTag,
    // Discogs integration
    discogsSearchQuery,
    discogsSearchResults,
    isSearchingDiscogs,
    isDiscogsAuthenticated,
    isDiscogsAuthenticating,
    discogsError,
    onSearchInput,
    searchDiscogs,
    selectDiscogsRelease,
    clearDiscogsRelease,
    authenticateDiscogs,
    checkDiscogsAuth: discogs.checkAuthStatus,
    // Template
    applyDescriptionTemplate,
    // Validation
    hasTitle,
    hasDescription,
    metadataStatus,
    statusCallout,
  };
}
