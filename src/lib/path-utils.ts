/**
 * Path utilities for smart truncation and display
 */

// Average character width in pixels for path display (approximate)
const CHAR_WIDTH_ESTIMATE = 8;

// Minimum characters to show for meaningful truncation
const MIN_CHARS_BEFORE_TRUNCATION = 20;

// Separator characters for different platforms (kept for future use)
// const PATH_SEPARATORS = ['/', '\\'];

/**
 * Normalizes path separators to forward slashes for consistent handling
 */
export function normalizePath(path: string): string {
  return path.replace(/\\/g, '/');
}

/**
 * Gets the appropriate path separator for the current platform
 */
export function getPathSeparator(path: string): string {
  return path.includes('\\') ? '\\' : '/';
}

/**
 * Splits a path into segments, handling both Unix and Windows separators
 */
export function splitPath(path: string): string[] {
  const normalized = normalizePath(path);
  return normalized.split('/').filter(segment => segment.length > 0);
}

/**
 * Gets the filename from a path
 */
export function getFileName(path: string): string {
  const segments = splitPath(path);
  return segments[segments.length - 1] || path;
}

/**
 * Gets the parent directory name from a path
 */
export function getParentDir(path: string): string | null {
  const segments = splitPath(path);
  return segments.length > 1 ? segments[segments.length - 2] : null;
}

/**
 * Smart path truncation that preserves meaningful information
 * Shows important path segments with ellipsis when needed
 * 
 * Examples:
 * - Short path: "/Users/john/Documents/project"
 * - Medium truncation: "/Users/.../Documents/project" 
 * - Heavy truncation: ".../Documents/project"
 * - Extreme truncation: ".../project"
 */
export function truncatePath(path: string, maxChars: number): string {
  if (!path || path.length <= maxChars) {
    return path;
  }

  if (maxChars < MIN_CHARS_BEFORE_TRUNCATION) {
    // For very small spaces, just show the filename
    return getFileName(path);
  }

  const separator = getPathSeparator(path);
  const segments = splitPath(path);
  
  if (segments.length === 0) {
    return path;
  }

  const fileName = segments[segments.length - 1];
  const parentDir = segments.length > 1 ? segments[segments.length - 2] : null;
  
  // If just filename fits, show it
  if (fileName.length <= maxChars) {
    // Try to include parent directory
    if (parentDir && `${parentDir}${separator}${fileName}`.length <= maxChars) {
      return `${parentDir}${separator}${fileName}`;
    }
    
    // Try to include parent with ellipsis
    const withParentEllipsis = `...${separator}${parentDir}${separator}${fileName}`;
    if (parentDir && withParentEllipsis.length <= maxChars) {
      return withParentEllipsis;
    }
    
    // Just filename with ellipsis
    return `...${separator}${fileName}`;
  }
  
  // If even filename is too long, truncate it
  if (fileName.length > maxChars - 4) { // Reserve space for ".../"
    const truncatedFileName = fileName.substring(0, maxChars - 4) + '...';
    return `...${separator}${truncatedFileName}`;
  }
  
  // Try different truncation strategies
  const ellipsis = '...';
  const ellipsisWithSep = `${ellipsis}${separator}`;
  
  // Strategy 1: Show beginning + end with ellipsis in middle
  if (segments.length >= 3) {
    const firstSegment = segments[0];
    const lastTwoSegments = `${parentDir}${separator}${fileName}`;
    const candidate = `${firstSegment}${separator}${ellipsisWithSep}${lastTwoSegments}`;
    
    if (candidate.length <= maxChars) {
      return candidate;
    }
  }
  
  // Strategy 2: Show just parent + filename with leading ellipsis
  if (parentDir) {
    const candidate = `${ellipsisWithSep}${parentDir}${separator}${fileName}`;
    if (candidate.length <= maxChars) {
      return candidate;
    }
  }
  
  // Strategy 3: Just filename with leading ellipsis
  return `${ellipsisWithSep}${fileName}`;
}

/**
 * Calculates maximum characters based on container width
 */
export function calculateMaxChars(containerWidth: number): number {
  if (containerWidth <= 0) {
    return MIN_CHARS_BEFORE_TRUNCATION;
  }
  
  const maxChars = Math.floor(containerWidth / CHAR_WIDTH_ESTIMATE);
  return Math.max(maxChars, MIN_CHARS_BEFORE_TRUNCATION);
}

/**
 * Truncates a path based on container width
 */
export function truncatePathByWidth(path: string, containerWidth: number): string {
  const maxChars = calculateMaxChars(containerWidth);
  return truncatePath(path, maxChars);
}

/**
 * Gets a user-friendly display name for a path
 * Prioritizes showing the most relevant information
 */
export function getPathDisplayName(path: string, maxChars?: number): string {
  if (!path) return '';
  
  if (maxChars) {
    return truncatePath(path, maxChars);
  }
  
  // Default to showing just the folder/file name
  return getFileName(path);
}

/**
 * Checks if a path needs truncation based on character limit
 */
export function pathNeedsTruncation(path: string, maxChars: number): boolean {
  return path.length > maxChars;
}