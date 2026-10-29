import { ref, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

export type YouTubePlaylist = { id: string; title: string };

export function useYouTube() {
  const isAuthenticated = ref(false);
  const isAuthenticating = ref(false);
  const error = ref<string | null>(null);
  const playlists = ref<YouTubePlaylist[]>([]);
  const isLoadingPlaylists = ref(false);
  
  // Track timeout and focus handler for authentication cleanup
  let authTimeoutId: ReturnType<typeof setTimeout> | null = null;
  let focusHandler: (() => void) | null = null;

  const checkAuthStatus = async () => {
    try {
      const authenticated = await invoke<boolean>("check_youtube_auth");
      isAuthenticated.value = authenticated;
      // Clear error if we successfully checked status
      if (authenticated) {
        error.value = null;
      }
    } catch (err) {
      // Client not initialized or not authenticated
      isAuthenticated.value = false;
      // Don't set error here - this is expected when not authenticated
    }
  };

  const authenticate = async () => {
    // Clear any existing timeout/handler
    if (authTimeoutId) {
      clearTimeout(authTimeoutId);
      authTimeoutId = null;
    }
    if (focusHandler) {
      window.removeEventListener("focus", focusHandler);
      focusHandler = null;
    }

    // Set a timeout to reset loading state if authentication takes too long
    // This handles cases where user closes browser and backend is still waiting
    authTimeoutId = setTimeout(() => {
      if (isAuthenticating.value) {
        isAuthenticating.value = false;
        error.value = "Authentication timed out. Please try again.";
        if (focusHandler) {
          window.removeEventListener("focus", focusHandler);
          focusHandler = null;
        }
      }
    }, 4.5 * 60 * 1000); // 4.5 minutes timeout (slightly less than backend's 5 min)

    // Add window focus listener to detect when user returns to app
    // If they closed the browser, they'll likely come back to the app window
    // We check auth status when window regains focus to see if auth completed
    focusHandler = async () => {
      if (!isAuthenticating.value) {
        // Already done, remove listener
        if (focusHandler) {
          window.removeEventListener("focus", focusHandler);
          focusHandler = null;
        }
        return;
      }
      
      // Small delay to allow backend to process any pending callbacks
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      if (isAuthenticating.value) {
        // Still authenticating - check if it completed
        try {
          const authenticated = await invoke<boolean>("check_youtube_auth");
          if (authenticated) {
            // Successfully authenticated
            isAuthenticated.value = true;
            isAuthenticating.value = false;
            await loadPlaylists();
          } else {
            // Not authenticated - user likely cancelled by closing browser
            isAuthenticating.value = false;
            error.value = "Authentication was cancelled. Please try again.";
          }
        } catch (err) {
          // Error checking status - reset loading state
          isAuthenticating.value = false;
          error.value = "Authentication failed. Please try again.";
        }
        
        // Cleanup
        if (authTimeoutId) {
          clearTimeout(authTimeoutId);
          authTimeoutId = null;
        }
        if (focusHandler) {
          window.removeEventListener("focus", focusHandler);
          focusHandler = null;
        }
      }
    };
    window.addEventListener("focus", focusHandler);

    try {
      isAuthenticating.value = true;
      error.value = null;
      
      // The backend will handle opening browser and receiving callback automatically
      await invoke("authenticate_youtube");
      
      // Success - cleanup
      if (authTimeoutId) {
        clearTimeout(authTimeoutId);
        authTimeoutId = null;
      }
      if (focusHandler) {
        window.removeEventListener("focus", focusHandler);
        focusHandler = null;
      }
      
      isAuthenticated.value = true;
      await loadPlaylists();
    } catch (err) {
      // Error - cleanup
      if (authTimeoutId) {
        clearTimeout(authTimeoutId);
        authTimeoutId = null;
      }
      if (focusHandler) {
        window.removeEventListener("focus", focusHandler);
        focusHandler = null;
      }
      
      const errorMsg = String(err);
      // Provide clearer error messages
      if (errorMsg.includes("connection has expired") || 
          errorMsg.includes("reconnect your YouTube account")) {
        error.value = "Your YouTube connection has expired. Please reconnect your YouTube account.";
      } else {
        error.value = `Failed to authenticate: ${errorMsg}`;
      }
      isAuthenticated.value = false;
    } finally {
      isAuthenticating.value = false;
    }
  };

  const disconnect = async () => {
    try {
      isAuthenticating.value = true;
      error.value = null;
      await invoke("disconnect_youtube");
      isAuthenticated.value = false;
      playlists.value = [];
    } catch (err) {
      error.value = `Failed to disconnect: ${err}`;
    } finally {
      isAuthenticating.value = false;
    }
  };

  const loadPlaylists = async () => {
    if (!isAuthenticated.value) return;
    try {
      isLoadingPlaylists.value = true;
      error.value = null;
      const list = await invoke<YouTubePlaylist[]>("list_youtube_playlists");
      playlists.value = list ?? [];
    } catch (err) {
      const errorMsg = String(err);
      // Check if this is an authentication error that requires re-auth
      if (errorMsg.includes("connection has expired") || 
          errorMsg.includes("reconnect your YouTube account") ||
          errorMsg.includes("invalid_grant") ||
          errorMsg.includes("Token expired")) {
        isAuthenticated.value = false;
        error.value = "Your YouTube connection has expired. Please reconnect your YouTube account in Settings.";
      } else {
        error.value = `Failed to load playlists: ${errorMsg}`;
      }
      playlists.value = [];
    } finally {
      isLoadingPlaylists.value = false;
    }
  };

  // Cleanup on unmount
  onUnmounted(() => {
    if (authTimeoutId) {
      clearTimeout(authTimeoutId);
    }
    if (focusHandler) {
      window.removeEventListener("focus", focusHandler);
    }
  });

  return {
    isAuthenticated,
    isAuthenticating,
    error,
    playlists,
    isLoadingPlaylists,
    checkAuthStatus,
    authenticate,
    disconnect,
    loadPlaylists
  };
}
