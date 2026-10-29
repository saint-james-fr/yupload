import { usePreferences } from "./usePreferences";
import type {
  DescriptionTemplate,
  DescriptionTagKey,
  DiscogsRelease
} from "../types";

export type VideoUploadMetadata = {
  videoPath: string;
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

export function useDescriptionTemplate() {
  const { preferences } = usePreferences();

  const cleanDiscogsName = (name: string): string => {
    return name.replace(/\s*\(\d+\)\s*$/, "");
  };

  const valueForTagKey = (
    tagKey: DescriptionTagKey | undefined,
    release: DiscogsRelease | null,
    video: VideoUploadMetadata
  ): string | null => {
    if (!tagKey) return null;
    if (!release) {
      if (tagKey === "release") return video.title || null;
      return null;
    }

    switch (tagKey) {
      case "artist": {
        if (release.artists.length === 0) return null;
        const cleaned = release.artists.map(a => cleanDiscogsName(a.name));
        return `Artists: ${cleaned.join(", ")}`;
      }
      case "release":
        return `Release: ${release.title}`;
      case "year":
        return release.year ? `Year: ${release.year}` : null;
      case "label": {
        if (release.labels.length === 0) return null;
        const cleaned = release.labels.map(l => cleanDiscogsName(l.name));
        return `Label: ${cleaned.join(", ")}`;
      }
      case "catalogNumber": {
        if (release.labels.length === 0) return null;
        const catnos = release.labels.map(l => l.catno).filter((c): c is string => c !== null);
        return catnos.length > 0 ? `Catalogue Number: ${catnos.join(", ")}` : null;
      }
      case "genres":
        return release.genres.length > 0 ? `Genres: ${release.genres.join(", ")}` : null;
      case "styles":
        return release.styles.length > 0 ? `Styles: ${release.styles.join(", ")}` : null;
      case "discogsUrl": {
        const url = release.uri || `https://www.discogs.com/release/${release.id}`;
        return `Discogs: ${url}`;
      }
      default:
        return null;
    }
  };

  const buildDescriptionFromTemplate = (
    template: DescriptionTemplate | null | undefined,
    release: DiscogsRelease | null,
    video: VideoUploadMetadata
  ): string | null => {
    if (!template || template.length === 0) return null;
    const activeBlocks = template.filter(b => b.enabled);
    if (activeBlocks.length === 0) return null;

    const lines: string[] = [];
    for (const block of activeBlocks) {
      if (block.type === "tag") {
        const value = valueForTagKey(block.tagKey as DescriptionTagKey | undefined, release, video);
        if (value) lines.push(value);
      } else if (block.type === "text" && block.content) {
        lines.push(block.content);
      }
    }
    return lines.length > 0 ? lines.join("\n") : null;
  };

  const autofillMetadataFromRelease = (
    release: DiscogsRelease,
    video: VideoUploadMetadata
  ) => {
    // Always autofill description using the current template when linking a release
    const template = preferences.value.preferences?.description_template_blocks ?? null;
    const fromTemplate = buildDescriptionFromTemplate(template as DescriptionTemplate | null, release, video);
    if (fromTemplate) {
      video.description = fromTemplate;
    }
    
    // Autofill tags from genres and styles
    if (video.tags.length === 0) {
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
      video.tags = [...new Set(tags)]; // Remove duplicates
    }
  };

  const applyToVideo = (video: VideoUploadMetadata) => {
    const template = preferences.value.preferences?.description_template_blocks ?? null;
    const release = video.discogsRelease;
    const description = buildDescriptionFromTemplate(template as DescriptionTemplate | null, release, video);
    if (description) {
      video.description = description;
    }
  };

  return {
    cleanDiscogsName,
    valueForTagKey,
    buildDescriptionFromTemplate,
    autofillMetadataFromRelease,
    applyToVideo,
    preferences
  };
}
