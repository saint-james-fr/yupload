import { ref, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { DiscogsSearchResult, DiscogsRelease } from "../types";

export function useDiscogs() {
  const isAuthenticated = ref(false);
  const isAuthenticating = ref(false);
  const error = ref<string | null>(null);
  const searchQuery = ref("");
  const searchResults = ref<DiscogsSearchResult[]>([]);
  const isSearching = ref(false);
  const selectedRelease = ref<DiscogsRelease | null>(null);
  const isLoadingRelease = ref(false);
  
  // Track timeout and focus handler for authentication cleanup
  let authTimeoutId: ReturnType<typeof setTimeout> | null = null;
  let focusHandler: (() => void) | null = null;

  const checkAuthStatus = async () => {
    try {
      const authenticated = await invoke<boolean>("check_discogs_auth");
      isAuthenticated.value = authenticated;
    } catch (err) {
      // Client not initialized or not authenticated
      isAuthenticated.value = false;
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
          const authenticated = await invoke<boolean>("check_discogs_auth");
          if (authenticated) {
            // Successfully authenticated
            isAuthenticated.value = true;
            isAuthenticating.value = false;
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
      await invoke("authenticate_discogs");
      
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
      
      error.value = `Failed to authenticate: ${err}`;
      isAuthenticated.value = false;
    } finally {
      isAuthenticating.value = false;
    }
  };

  const disconnect = async () => {
    try {
      isAuthenticating.value = true;
      error.value = null;
      await invoke("disconnect_discogs");
      isAuthenticated.value = false;
      searchQuery.value = "";
      searchResults.value = [];
      selectedRelease.value = null;
    } catch (err) {
      error.value = `Failed to disconnect: ${err}`;
    } finally {
      isAuthenticating.value = false;
    }
  };

  const search = async (query: string) => {
    if (!query.trim() || !isAuthenticated.value) return;
    
    try {
      isSearching.value = true;
      error.value = null;
      const response = await invoke<{ results: DiscogsSearchResult[] }>("search_discogs_releases", {
        query: query.trim()
      });
      searchResults.value = response.results;
    } catch (err) {
      error.value = `Search failed: ${err}`;
      searchResults.value = [];
    } finally {
      isSearching.value = false;
    }
  };

  const selectRelease = async (result: DiscogsSearchResult) => {
    if (!isAuthenticated.value) return;
    
    try {
      isLoadingRelease.value = true;
      error.value = null;
      
      // Get full release data
      const release = await invoke<DiscogsRelease>("get_discogs_release", {
        releaseId: result.id
      });
      
      selectedRelease.value = release;
      searchResults.value = [];
      searchQuery.value = "";
      
      return release;
    } catch (err) {
      error.value = `Failed to load release: ${err}`;
      selectedRelease.value = null;
      return null;
    } finally {
      isLoadingRelease.value = false;
    }
  };

  const clearSearch = () => {
    searchQuery.value = "";
    searchResults.value = [];
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
    searchQuery,
    searchResults,
    isSearching,
    selectedRelease,
    isLoadingRelease,
    checkAuthStatus,
    authenticate,
    disconnect,
    search,
    selectRelease,
    clearSearch
  };
}
