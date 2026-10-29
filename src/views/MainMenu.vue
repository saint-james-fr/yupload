<script setup lang="ts">
import { onMounted } from "vue";
import { Upload, Settings, Music, History, Sparkles, Loader2, HelpCircle, Shield } from "lucide-vue-next";
import { useYouTube } from "../composables/useYouTube";
import { useDiscogs } from "../composables/useDiscogs";
import Callout from "../components/ui/Callout.vue";
import StatusBadge from "../components/ui/StatusBadge.vue";
import Button from "../components/ui/Button.vue";

const emit = defineEmits<{
  (e: "select", view: "upload" | "unified-upload" | "settings" | "normalize" | "history" | "faq" | "privacy"): void;
}>();

const selectView = (view: "upload" | "unified-upload" | "settings" | "normalize" | "history" | "faq" | "privacy") => {
  // Prevent access to upload actions if YouTube is not connected
  if ((view === "unified-upload" || view === "upload") && !youtube.isAuthenticated.value) {
    // Show modal or alert - for now just return
    return;
  }
  emit("select", view);
};

// Connection status
const youtube = useYouTube();
const discogs = useDiscogs();

// Load data on mount
onMounted(async () => {
  await Promise.all([
    youtube.checkAuthStatus(),
    discogs.checkAuthStatus()
  ]);
});

// Handle badge clicks
const connectYouTube = async (e?: Event) => {
  if (e) {
    e.stopPropagation();
  }
  if (!youtube.isAuthenticated.value && !youtube.isAuthenticating.value) {
    await youtube.authenticate();
    await youtube.checkAuthStatus();
  }
};

const connectDiscogs = async (e: Event) => {
  e.stopPropagation();
  if (!discogs.isAuthenticated.value && !discogs.isAuthenticating.value) {
    await discogs.authenticate();
    await discogs.checkAuthStatus();
  }
};

</script>

<template>
  <div class="main-menu-container">
    <!-- Header -->
    <header class="main-menu-header">
      <h1 class="main-menu-title">Yupload</h1>
      <p class="main-menu-subtitle">All-in-one tool for music ripping and video uploads</p>
    </header>

    <!-- YouTube Connection Warning (shown at top when not connected) -->
    <div v-if="!youtube.isAuthenticated.value" class="connection-warning">
      <Callout variant="warning">
        <div class="flex items-start justify-between gap-3">
          <div class="flex-1">
            <p class="font-medium mb-1">YouTube connection required</p>
            <p class="text-sm opacity-90">Connect your YouTube account to create and upload videos.</p>
          </div>
          <Button
            @click="connectYouTube"
            :disabled="youtube.isAuthenticating.value"
            variant="outline"
            size="sm"
            class="shrink-0 border-amber-500/50 text-amber-700 hover:bg-amber-500/10 hover:border-amber-500/70 dark:text-amber-200 dark:border-amber-500/50"
          >
            <Loader2 v-if="youtube.isAuthenticating.value" class="h-3.5 w-3.5 mr-1.5 animate-spin" />
            <svg v-else class="h-3.5 w-3.5 mr-1.5 shrink-0" viewBox="0 0 24 24">
              <path fill="currentColor" d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"/>
            </svg>
            Connect YouTube
          </Button>
        </div>
      </Callout>
    </div>

    <!-- Main Actions -->
    <nav class="main-menu-nav" aria-label="Main navigation">
      <!-- Hero Action: Create Videos & Upload to YouTube -->
      <div class="hero-section">
        <button 
          class="menu-item menu-item-hero"
          :class="{ 'menu-item-disabled': !youtube.isAuthenticated.value }"
          style="--delay: 0"
          @click="selectView('unified-upload')"
          aria-label="Create Videos & Upload to YouTube: Just bring your audio files. We'll create professional YouTube-ready videos with images, add metadata, and upload everything automatically."
        >
          <div class="menu-item-icon menu-item-icon-hero">
            <Sparkles class="icon icon-hero" />
          </div>
          <div class="menu-item-content">
            <h2 class="menu-item-title menu-item-title-hero">Create Videos & Upload to YouTube</h2>
            <p class="menu-item-description menu-item-description-hero">
              Just bring your audio files. We'll create professional YouTube-ready videos with images, add metadata, and upload everything automatically.
            </p>
          </div>
        </button>
      </div>

      <!-- Secondary Actions -->
      <div class="secondary-section">
        <div class="menu-grid-secondary">
          <!-- Upload Existing Videos -->
          <button 
            class="menu-item menu-item-secondary"
            :class="{ 'menu-item-disabled': !youtube.isAuthenticated.value }"
            style="--delay: 1"
            @click="selectView('upload')"
            aria-label="Upload Existing Videos: upload existing videos to YouTube with Discogs metadata"
          >
            <div class="menu-item-icon">
              <Upload class="icon" />
            </div>
            <div class="menu-item-content">
              <h2 class="menu-item-title">Upload Existing Videos</h2>
              <p class="menu-item-description">
                Already have videos ready? Upload them to YouTube with automatic metadata from Discogs—titles, descriptions, and tags included.
              </p>
            </div>
          </button>

          <!-- Audio Tools -->
          <button 
            class="menu-item menu-item-secondary"
            style="--delay: 1.5"
            @click="selectView('normalize')"
            aria-label="Audio Tools: Perfect your audio files. Normalize volume levels, remove low-frequency rumble, and ensure consistent quality across all tracks."
          >
            <div class="menu-item-icon">
              <Music class="icon" />
            </div>
            <div class="menu-item-content">
              <h2 class="menu-item-title">Audio Tools</h2>
              <p class="menu-item-description">
                Perfect your audio files. Normalize volume levels, remove low-frequency rumble, and ensure consistent quality across all tracks.
              </p>
            </div>
          </button>
        </div>
      </div>

      <!-- Utility Bar -->
      <div class="utility-section">
        <div class="utility-buttons">
          <button 
            class="utility-item"
            style="--delay: 2"
            @click="selectView('history')"
            aria-label="Upload History: videos you uploaded to YouTube and linked Discogs releases"
          >
            <History class="utility-icon" />
            <span class="utility-text">History</span>
          </button>
          <button 
            class="utility-item"
            style="--delay: 2.25"
            @click="selectView('faq')"
            aria-label="FAQ: frequently asked questions about security and data handling"
          >
            <HelpCircle class="utility-icon" />
            <span class="utility-text">FAQ</span>
          </button>
          <button 
            class="utility-item"
            style="--delay: 2.5"
            @click="selectView('privacy')"
            aria-label="Privacy Policy: how we handle your data and protect your privacy"
          >
            <Shield class="utility-icon" />
            <span class="utility-text">Privacy</span>
          </button>
          <button 
            class="utility-item"
            style="--delay: 2.75"
            @click="selectView('settings')"
            aria-label="Settings: manage your app preferences and your YouTube and Discogs connections"
          >
            <Settings class="utility-icon" />
            <span class="utility-text">Settings</span>
          </button>
        </div>
        
        <!-- Connection Status & Stats -->
        <div class="utility-status">
          <div class="status-badges">
            <button
              v-if="!youtube.isAuthenticated.value" 
              class="status-badge-clickable"
              @click="connectYouTube"
              title="Click to connect YouTube"
            >
              <StatusBadge :connected="false" connected-label="YouTube" disconnected-label="YouTube" />
            </button>
            <StatusBadge 
              v-else
              :connected="true" 
              connected-label="YouTube"
              disconnected-label="YouTube"
              title="YouTube connected"
            />
            <button
              v-if="!discogs.isAuthenticated.value" 
              class="status-badge-clickable"
              @click="connectDiscogs"
              title="Click to connect Discogs"
            >
              <StatusBadge :connected="false" connected-label="Discogs" disconnected-label="Discogs" />
            </button>
            <StatusBadge 
              v-else
              :connected="true" 
              connected-label="Discogs"
              disconnected-label="Discogs"
              title="Discogs connected. Metadata will be auto-filled."
            />
          </div>
        </div>
      </div>
    </nav>
  </div>
</template>

<style scoped>
/* Container */
.main-menu-container {
  height: 100vh;
  width: 100vw;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: space-between;
  padding: 2rem 1.5rem;
  background: var(--color-background);
  overflow: hidden;
  box-sizing: border-box;
  position: relative;
}

.main-menu-container > * {
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.main-menu-container::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: radial-gradient(circle at 50% 0%, hsl(var(--primary) / 0.03) 0%, transparent 50%);
  pointer-events: none;
  z-index: 0;
}

@media (prefers-color-scheme: dark) {
  .main-menu-container {
    background: hsl(0 0% 8%);
  }
  
  .main-menu-container::before {
    background: radial-gradient(circle at 50% 0%, hsl(var(--primary) / 0.05) 0%, transparent 50%);
  }
}

/* Header */
.main-menu-header {
  text-align: center;
  margin-bottom: 2rem;
  animation: fadeInUp 0.6s ease forwards;
  flex-shrink: 0;
  width: 100%;
  max-width: 60rem;
  position: relative;
  z-index: 1;
}

/* Connection Warning */
.connection-warning {
  width: 100%;
  max-width: 60rem;
  margin-bottom: 1.5rem;
  animation: fadeInUp 0.6s ease forwards;
  animation-delay: 50ms;
  opacity: 0;
  position: relative;
  z-index: 1;
}

.main-menu-title {
  font-size: clamp(2rem, 4vw, 2.75rem);
  color: var(--color-primary);
  font-weight: 700;
  margin-bottom: 0.5rem;
  line-height: 1.2;
  letter-spacing: -0.02em;
}

.main-menu-subtitle {
  font-size: clamp(0.875rem, 1.5vw, 1rem);
  color: hsl(var(--muted-foreground));
  margin-bottom: 0;
  font-weight: 400;
  opacity: 0.9;
}

.main-menu-intro {
  font-size: 0.75rem;
  color: hsl(var(--muted-foreground));
  margin-top: 0;
  font-weight: 500;
}

/* Navigation */
.main-menu-nav {
  width: 100%;
  max-width: 60rem;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  flex: 1;
  justify-content: center;
  min-height: 0;
  box-sizing: border-box;
  overflow: visible;
  position: relative;
  z-index: 1;
  margin: 0 auto;
}

/* Hero Section */
.hero-section {
  width: 100%;
  flex-shrink: 0;
}

/* Secondary Section */
.secondary-section {
  width: 100%;
  flex-shrink: 0;
}

.menu-grid-secondary {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 1rem;
  width: 100%;
  max-width: 100%;
  box-sizing: border-box;
}

/* Utility Section */
.utility-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding-top: 1rem;
  border-top: 1px solid hsl(var(--border) / 0.5);
  flex-shrink: 0;
  width: 100%;
  max-width: 60rem;
  box-sizing: border-box;
  margin-top: auto;
  overflow: visible;
  min-height: auto;
}

.utility-buttons {
  display: flex;
  justify-content: center;
  gap: 1rem;
  width: 100%;
}

.utility-status {
  display: flex;
  justify-content: center;
  align-items: center;
  width: 100%;
  padding-top: 0.5rem;
  min-height: auto;
  overflow: visible;
}

/* Menu Item */
.menu-item {
  all: unset;
  display: flex;
  flex-direction: row;
  align-items: center;
  text-align: left;
  padding: 1.25rem 1.5rem;
  border-radius: 0.75rem;
  background: hsl(var(--card));
  border: 2px solid hsl(var(--border) / 0.6);
  box-shadow: 
    0 1px 3px 0 rgb(0 0 0 / 0.1), 
    0 1px 2px -1px rgb(0 0 0 / 0.1),
    0 0 0 0 hsl(var(--primary) / 0);
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  opacity: 0;
  animation: fadeInUp 0.6s ease forwards;
  animation-delay: calc(var(--delay) * 100ms);
  position: relative;
  overflow: hidden;
  min-height: auto;
  width: 100%;
  gap: 1.25rem;
  box-sizing: border-box;
}

/* Hero Item - Extra Large */
.menu-item-hero {
  padding: 1.75rem 2rem;
  border-width: 2px;
  background: linear-gradient(135deg, hsl(var(--card)), hsl(var(--card) / 0.98));
  border-color: hsl(var(--primary) / 0.2);
}

.menu-item-hero::before {
  background: rgba(234, 179, 8, 0.1);
}

.menu-item-hero:hover {
  transform: translateY(-4px) scale(1.01);
  box-shadow: 
    0 20px 40px -10px hsl(var(--primary) / 0.25),
    0 0 40px hsl(var(--primary) / 0.15),
    0 0 0 1px hsl(var(--primary) / 0.3);
  border-color: hsl(var(--primary) / 0.4);
}

.menu-item-hero:active {
  transform: translateY(-2px) scale(1);
}

/* Secondary Item */
.menu-item-secondary {
  padding: 1.25rem 1.5rem;
}

.menu-item::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(234, 179, 8, 0.1);
  opacity: 0;
  transition: opacity 0.3s ease;
  z-index: 0;
}

.menu-item:hover::before {
  opacity: 1;
}

.menu-item:hover {
  transform: translateY(-3px) scale(1.01);
  box-shadow: 
    0 12px 28px -8px hsl(var(--primary) / 0.2),
    0 0 24px hsl(var(--primary) / 0.1);
  border-color: hsl(var(--primary) / 0.25);
}

.menu-item:hover .menu-item-title {
  color: #eab308;
}

.menu-item:active {
  transform: translateY(-1px) scale(1);
}

.menu-item-disabled {
  opacity: 0.6;
  cursor: not-allowed;
  pointer-events: none;
}

/* Allow pointer events on interactive children even when parent is disabled */
.menu-item-disabled .menu-item-content {
  pointer-events: auto;
}

.menu-item-disabled:hover {
  transform: none;
  box-shadow: none;
  border-color: hsl(var(--border) / 0.6);
}


.menu-item:focus-visible {
  outline: 2px solid hsl(var(--ring));
  outline-offset: 3px;
  border-color: hsl(var(--primary) / 0.4);
}

/* Icon */
.menu-item-icon {
  width: 3.5rem;
  height: 3.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 0.625rem;
  background: linear-gradient(135deg, hsl(var(--primary) / 0.15), hsl(var(--primary) / 0.1));
  border: 2px solid hsl(var(--primary) / 0.3);
  box-shadow: 
    0 4px 12px hsl(var(--primary) / 0.2),
    inset 0 1px 0 hsl(var(--primary) / 0.1);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  flex-shrink: 0;
  position: relative;
  z-index: 1;
}

.menu-item-icon-hero {
  width: 4rem;
  height: 4rem;
  border-width: 2px;
  background: linear-gradient(135deg, hsl(var(--primary) / 0.2), hsl(var(--primary) / 0.15));
  border-color: hsl(var(--primary) / 0.4);
  box-shadow: 
    0 6px 16px hsl(var(--primary) / 0.3),
    inset 0 1px 0 hsl(var(--primary) / 0.15);
}

.menu-item:hover .menu-item-icon {
  transform: scale(1.08) rotate(3deg);
  background: linear-gradient(135deg, hsl(var(--primary) / 0.2), hsl(var(--primary) / 0.15));
  border-color: hsl(var(--primary) / 0.4);
  box-shadow: 
    0 6px 16px hsl(var(--primary) / 0.3),
    inset 0 1px 0 hsl(var(--primary) / 0.15);
}

.menu-item-hero:hover .menu-item-icon-hero {
  transform: scale(1.1) rotate(5deg);
  background: linear-gradient(135deg, hsl(var(--primary) / 0.25), hsl(var(--primary) / 0.2));
  border-color: hsl(var(--primary) / 0.5);
  box-shadow: 
    0 8px 20px hsl(var(--primary) / 0.4),
    inset 0 1px 0 hsl(var(--primary) / 0.2);
}

.icon {
  width: 1.75rem;
  height: 1.75rem;
  color: var(--color-primary);
  transition: all 0.3s ease;
}

.icon-hero {
  width: 2rem;
  height: 2rem;
}

.menu-item:hover .icon {
  color: var(--color-primary-accent);
}

/* Content */
.menu-item-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  min-width: 0;
  overflow: hidden;
  position: relative;
  z-index: 1;
}

.menu-item-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: hsl(var(--foreground));
  margin-bottom: 0.5rem;
  line-height: 1.3;
  letter-spacing: -0.01em;
  transition: color 0.3s ease;
}

.menu-item-title-hero {
  font-size: 1.5rem;
  font-weight: 700;
  margin-bottom: 0.625rem;
  letter-spacing: -0.02em;
}

.status-badges {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-shrink: 0;
}

.status-badge {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.25rem 0.5rem;
  border-radius: 0.375rem;
  font-size: 0.6875rem;
  font-weight: 500;
  transition: all 0.2s ease;
}

.status-badge-clickable {
  all: unset;
  cursor: pointer;
  user-select: none;
}

.status-badge-clickable:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 2px 4px hsl(var(--primary) / 0.2);
}

.status-badge-clickable:active:not(:disabled) {
  transform: translateY(0);
}

.status-badge-clickable:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.status-badge-success {
  background: hsl(142 76% 36% / 0.15);
  color: hsl(142 76% 36%);
  border: 1px solid hsl(142 76% 36% / 0.3);
}

.status-badge-warning {
  background: hsl(38 92% 50% / 0.15);
  color: hsl(38 92% 50%);
  border: 1px solid hsl(38 92% 50% / 0.3);
}

.status-badge-icon {
  width: 0.875rem;
  height: 0.875rem;
  flex-shrink: 0;
}

.status-badge-text {
  white-space: nowrap;
  transition: opacity 0.2s ease;
}

.status-badge-clickable:hover:not(:disabled) .status-badge-text {
  opacity: 0.7;
}


.menu-item-description {
  font-size: 0.875rem;
  color: hsl(var(--muted-foreground));
  line-height: 1.5;
  margin: 0;
  opacity: 0.85;
}

.menu-item-description-hero {
  font-size: 0.9375rem;
  line-height: 1.5;
  opacity: 0.9;
}

/* Arrow indicator for hero item */
.menu-item-arrow {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  border-radius: 0.5rem;
  background: hsl(var(--primary) / 0.1);
  color: hsl(var(--primary));
  opacity: 0;
  transform: translateX(-0.5rem);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  flex-shrink: 0;
  position: relative;
  z-index: 1;
}

.menu-item-hero:hover .menu-item-arrow {
  opacity: 1;
  transform: translateX(0);
  background: hsl(var(--primary) / 0.15);
}

/* Utility Items */
.utility-item {
  all: unset;
  display: flex;
  align-items: center;
  gap: 0.625rem;
  padding: 0.875rem 1.75rem;
  border-radius: 0.625rem;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border) / 0.5);
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  opacity: 0;
  animation: fadeInUp 0.6s ease forwards;
  animation-delay: calc(var(--delay) * 100ms);
  color: hsl(var(--muted-foreground));
  font-weight: 500;
}

.utility-item-content {
  display: flex;
  align-items: center;
  gap: 0.625rem;
}

.utility-text-group {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.125rem;
}

.utility-hint {
  font-size: 0.6875rem;
  color: hsl(var(--muted-foreground));
  opacity: 0.7;
  font-weight: 400;
  line-height: 1.2;
}

.utility-item:hover {
  background: rgba(234, 179, 8, 0.1);
  color: hsl(var(--foreground));
  transform: translateY(-2px);
  border-color: hsl(var(--border));
  box-shadow: 0 4px 12px hsl(var(--primary) / 0.1);
}

.utility-item:active {
  transform: translateY(0);
}

.utility-icon {
  width: 1.25rem;
  height: 1.25rem;
  color: var(--color-primary) !important;
  transition: color 0.25s ease;
}

.utility-item:hover .utility-icon {
  color: var(--color-primary-accent) !important;
}

.utility-text {
  font-size: 0.875rem;
  font-weight: 500;
  transition: color 0.25s ease;
}

.utility-item:hover .utility-text {
  color: #eab308;
}

/* Responsive Design */
@media (max-width: 1024px) {
  .main-menu-container {
    padding: 1.5rem 1rem;
  }
  
  .main-menu-nav {
    max-width: 100%;
    gap: 1.25rem;
  }
  
  .menu-grid-secondary {
    gap: 0.875rem;
  }
  
  .menu-item-hero {
    padding: 1.5rem 1.5rem;
  }
  
  .menu-item-secondary {
    padding: 1.125rem 1.25rem;
  }
}

@media (max-width: 768px) {
  .main-menu-container {
    padding: 1.25rem 1rem;
  }
  
  .menu-grid-secondary {
    grid-template-columns: 1fr;
    gap: 1rem;
  }
  
  .main-menu-header {
    margin-bottom: 1.5rem;
  }
  
  .connection-warning {
    margin-bottom: 1.25rem;
  }
  
  .main-menu-nav {
    gap: 1.25rem;
  }
  
  .menu-item {
    flex-direction: column;
    text-align: center;
    padding: 1.5rem 1.25rem;
  }
  
  .menu-item-hero {
    padding: 2rem 1.5rem;
  }
  
  .menu-item-secondary {
    padding: 1.5rem 1.25rem;
  }
  
  .menu-item-icon-hero {
    width: 4rem;
    height: 4rem;
    margin-bottom: 0.75rem;
  }
  
  .icon-hero {
    width: 2rem;
    height: 2rem;
  }
  
  .menu-item-title-hero {
    font-size: 1.375rem;
    margin-bottom: 0.5rem;
  }
  
  .utility-section {
    gap: 0.875rem;
  }
  
  .utility-status {
    padding-top: 0.375rem;
  }
  
  .menu-item-description-hero {
    font-size: 0.875rem;
  }
  
  .menu-item-icon {
    width: 3.5rem;
    height: 3.5rem;
    margin-bottom: 0.75rem;
  }
  
  .icon {
    width: 1.75rem;
    height: 1.75rem;
  }
  
  .menu-item-title {
    font-size: 1.0625rem;
    margin-bottom: 0.375rem;
  }
  
  .menu-item-description {
    font-size: 0.8125rem;
  }
  
  .menu-item-arrow {
    display: none;
  }
  
  .utility-section {
    padding-top: 1.5rem;
    gap: 0.875rem;
  }
}

@media (max-width: 480px) {
  .main-menu-container {
    padding: 1rem 0.75rem;
  }
  
  .main-menu-header {
    margin-bottom: 1rem;
  }
  
  .connection-warning {
    margin-bottom: 1rem;
  }
  
  .main-menu-title {
    font-size: 1.75rem;
  }
  
  .main-menu-subtitle {
    font-size: 0.8125rem;
  }
  
  .main-menu-nav {
    gap: 1rem;
  }
  
  .menu-item {
    padding: 1.25rem 1rem;
  }
  
  .menu-item-hero {
    padding: 1.75rem 1.25rem;
  }
  
  .menu-item-secondary {
    padding: 1.25rem 1rem;
  }
  
  .menu-item-icon-hero {
    width: 3.5rem;
    height: 3.5rem;
    margin-bottom: 0.625rem;
  }
  
  .icon-hero {
    width: 1.75rem;
    height: 1.75rem;
  }
  
  .menu-item-title-hero {
    font-size: 1.25rem;
  }
  
  .menu-item-description-hero {
    font-size: 0.8125rem;
  }
  
  .menu-item-icon {
    width: 3rem;
    height: 3rem;
    margin-bottom: 0.625rem;
  }
  
  .icon {
    width: 1.5rem;
    height: 1.5rem;
  }
  
  .menu-item-title {
    font-size: 1rem;
    margin-bottom: 0.375rem;
  }
  
  .menu-item-title-hero {
    font-size: 1.25rem;
    margin-bottom: 0.5rem;
  }
  
  
  .utility-buttons {
    flex-direction: column;
    gap: 0.75rem;
  }
  
  .utility-item-content {
    flex-direction: column;
    align-items: center;
    gap: 0.375rem;
  }
  
  .utility-text-group {
    align-items: center;
    text-align: center;
  }
  
  .utility-hint {
    font-size: 0.625rem;
  }
  
  .utility-status {
    padding-top: 0.5rem;
  }
  
  .status-badges {
    gap: 0.5rem;
  }
  
  .menu-item-description {
    font-size: 0.75rem;
  }
  
  .utility-section {
    flex-direction: row;
    gap: 0.75rem;
    padding-top: 1.25rem;
  }
  
  .utility-item {
    padding: 0.75rem 1.25rem;
    font-size: 0.8125rem;
  }
  
  .utility-icon {
    width: 1.125rem;
    height: 1.125rem;
  }
}

/* Animations */
@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}


/* Dark mode adjustments */
@media (prefers-color-scheme: dark) {
  .menu-item:hover {
    box-shadow: 
      0 12px 28px -8px hsl(var(--primary) / 0.3),
      0 0 32px hsl(var(--primary) / 0.2);
  }
  
  .menu-item-hero:hover {
    box-shadow: 
      0 20px 40px -10px hsl(var(--primary) / 0.3),
      0 0 45px hsl(var(--primary) / 0.2),
      0 0 0 1px hsl(var(--primary) / 0.35);
  }
}
</style>
