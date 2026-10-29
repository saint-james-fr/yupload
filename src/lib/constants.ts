/**
 * Shared frontend constants (YouTube API, privacy, UI options).
 * Single source of truth for option lists and enum-like values.
 */

// ---------------------------------------------------------------------------
// YouTube video categories (standard API values)
// @see https://developers.google.com/youtube/v3/docs/videoCategories/list
// ---------------------------------------------------------------------------

export const YOUTUBE_CATEGORIES = [
  { value: "1", label: "Film & Animation" },
  { value: "2", label: "Autos & Vehicles" },
  { value: "10", label: "Music" },
  { value: "15", label: "Pets & Animals" },
  { value: "17", label: "Sports" },
  { value: "19", label: "Travel & Events" },
  { value: "20", label: "Gaming" },
  { value: "22", label: "People & Blogs" },
  { value: "23", label: "Comedy" },
  { value: "24", label: "Entertainment" },
  { value: "25", label: "News & Politics" },
  { value: "26", label: "Howto & Style" },
  { value: "27", label: "Education" },
  { value: "28", label: "Science & Technology" },
  { value: "29", label: "Nonprofits & Activism" },
] as const;

export type YouTubeCategoryId = (typeof YOUTUBE_CATEGORIES)[number]["value"];

// ---------------------------------------------------------------------------
// YouTube privacy status options (value + label; components add icons)
// @see https://developers.google.com/youtube/v3/docs/videos#status.privacyStatus
// ---------------------------------------------------------------------------

export const YOUTUBE_PRIVACY_OPTIONS = [
  { value: "public", label: "Public" },
  { value: "unlisted", label: "Unlisted" },
  { value: "private", label: "Private" },
  { value: "draft", label: "Save as draft" },
] as const;

export type YouTubePrivacyStatus = (typeof YOUTUBE_PRIVACY_OPTIONS)[number]["value"];

// ---------------------------------------------------------------------------
// Image crop aspect ratio options (SegmentedControl; components add icon if needed)
// ---------------------------------------------------------------------------

export const IMAGE_CROP_RATIO_OPTIONS = [
  { value: "free", label: "Free" },
  { value: "1:1", label: "1:1" },
] as const;

export type ImageCropRatio = (typeof IMAGE_CROP_RATIO_OPTIONS)[number]["value"];
