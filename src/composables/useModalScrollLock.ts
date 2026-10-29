/**
 * Global scroll lock for modals. When any modal is open, body/document scroll
 * is disabled so the user can't scroll the underlying page (e.g. past the
 * glass title bar on macOS).
 */
let openCount = 0;
const LOCK_CLASS = "modal-open";

/**
 * Force reset the scroll lock (useful for recovery from edge cases)
 */
export function resetModalScrollLock() {
  openCount = 0;
  document.documentElement.classList.remove(LOCK_CLASS);
  document.body.classList.remove(LOCK_CLASS);
}

export function useModalScrollLock() {
  const lock = () => {
    // Safety check: ensure DOM is available
    if (typeof document === "undefined") return;
    
    openCount += 1;
    document.documentElement.classList.add(LOCK_CLASS);
    document.body.classList.add(LOCK_CLASS);
  };

  const unlock = () => {
    // Safety check: ensure DOM is available
    if (typeof document === "undefined") return;
    
    openCount = Math.max(0, openCount - 1);
    if (openCount === 0) {
      document.documentElement.classList.remove(LOCK_CLASS);
      document.body.classList.remove(LOCK_CLASS);
    }
  };

  return { lock, unlock };
}
