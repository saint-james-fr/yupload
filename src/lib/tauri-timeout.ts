/**
 * Timeouts for Tauri invoke calls that may hang (e.g. folder scan, thumbnail gen).
 * Prevents the UI from spinning indefinitely when the backend is slow or stuck.
 */
export const FOLDER_LIST_TIMEOUT_MS = 30_000;
export const THUMBNAIL_TIMEOUT_MS = 30_000;

/**
 * Race a promise against a timer. Rejects with a clear error if the promise
 * doesn't settle within `ms` milliseconds.
 */
export function withTimeout<T>(
  promise: Promise<T>,
  ms: number,
  message?: string
): Promise<T> {
  const timeout = new Promise<never>((_, reject) =>
    setTimeout(
      () => reject(new Error(message ?? `Operation timed out after ${ms / 1000}s`)),
      ms
    )
  );
  return Promise.race([promise, timeout]);
}
