<script setup lang="ts">
import { Shield, Eye, Database, Lock, Wifi, Trash2, Settings as SettingsIcon, Calendar, Users, AlertTriangle } from "lucide-vue-next";
import PageContainer from "../components/layout/PageContainer.vue";
import PageHeader from "../components/layout/PageHeader.vue";
import SectionHeader from "../components/ui/SectionHeader.vue";
import SectionCard from "../components/ui/SectionCard.vue";
import Callout from "../components/ui/Callout.vue";

const emit = defineEmits<{
  (e: "back"): void;
}>();

const lastUpdated = "January 29, 2026";
</script>

<template>
  <PageContainer>
    <PageHeader
      title="Privacy Policy"
      :description="`How we protect your privacy and handle your data (last updated ${lastUpdated})`"
      @back="emit('back')"
    />

    <div class="space-y-8">
      <!-- Overview -->
      <section class="space-y-2">
        <SectionHeader 
          label="Our Commitment"
          description="Privacy-first design with local data processing"
        />
        <SectionCard>
          <Callout variant="success" title="Your data stays private">
            <p class="text-sm">
              This application is designed with privacy as a core principle. All your data is processed locally on your device, 
              and we never collect, store, or transmit your personal information to our servers.
            </p>
          </Callout>
        </SectionCard>
      </section>

      <!-- Data Collection -->
      <section class="space-y-2">
        <SectionHeader 
          label="Data Collection"
          description="What information we access and why"
        />
        <SectionCard>
          <div class="space-y-6">
            <div class="flex items-start gap-3">
              <div class="flex items-center justify-center w-8 h-8 rounded-lg bg-green-500/10 border border-green-500/20 mt-0.5">
                <Eye class="h-4 w-4 text-green-600 dark:text-green-500" />
              </div>
              <div class="flex-1">
                <h3 class="text-sm font-semibold text-foreground mb-2">Files You Select</h3>
                <p class="text-sm text-muted-foreground leading-relaxed mb-3">
                  We only access files and folders you explicitly choose through the file selection dialogs. 
                  This includes audio files for processing, images for video creation, and output folders for saving results.
                </p>
                <ul class="text-xs text-muted-foreground space-y-1 ml-4">
                  <li>• Audio metadata (artist, title, album, duration, format) for video creation</li>
                  <li>• Embedded album artwork from audio files</li>
                  <li>• Image files for video backgrounds and thumbnails</li>
                  <li>• Video files for upload and metadata editing</li>
                  <li>• Output folder locations for saving processed files</li>
                </ul>
              </div>
            </div>

            <div class="flex items-start gap-3">
              <div class="flex items-center justify-center w-8 h-8 rounded-lg bg-blue-500/10 border border-blue-500/20 mt-0.5">
                <SettingsIcon class="h-4 w-4 text-blue-600 dark:text-blue-500" />
              </div>
              <div class="flex-1">
                <h3 class="text-sm font-semibold text-foreground mb-2">App Preferences</h3>
                <p class="text-sm text-muted-foreground leading-relaxed mb-3">
                  Your settings and preferences are stored locally to improve your experience across sessions.
                </p>
                <ul class="text-xs text-muted-foreground space-y-1 ml-4">
                  <li>• Default image styles and processing options</li>
                  <li>• Preferred output folders and file naming conventions</li>
                  <li>• YouTube and Discogs connection status and tokens</li>
                  <li>• Upload history with video metadata and links</li>
                  <li>• Audio normalization preferences</li>
                  <li>• Description templates and custom text</li>
                  <li>• Notification preferences</li>
                </ul>
              </div>
            </div>
          </div>
        </SectionCard>
      </section>

      <!-- Data Storage -->
      <section class="space-y-2">
        <SectionHeader 
          label="Data Storage"
          description="Where your information is kept"
        />
        <SectionCard>
          <div class="space-y-6">
            <div class="flex items-start gap-3">
              <div class="flex items-center justify-center w-8 h-8 rounded-lg bg-purple-500/10 border border-purple-500/20 mt-0.5">
                <Database class="h-4 w-4 text-purple-600 dark:text-purple-500" />
              </div>
              <div class="flex-1">
                <h3 class="text-sm font-semibold text-foreground mb-2">Local Storage Only</h3>
                <p class="text-sm text-muted-foreground leading-relaxed">
                  All your data is stored on your device using Tauri's secure storage system. This includes:
                </p>
                <ul class="text-xs text-muted-foreground space-y-1 ml-4 mt-2">
                  <li>• App preferences and settings</li>
                  <li>• Upload history and processing status</li>
                  <li>• OAuth tokens for YouTube and Discogs (encrypted)</li>
                  <li>• Temporary processing files in your chosen output folders</li>
                </ul>
              </div>
            </div>

            <div class="flex items-start gap-3">
              <div class="flex items-center justify-center w-8 h-8 rounded-lg bg-red-500/10 border border-red-500/20 mt-0.5">
                <Shield class="h-4 w-4 text-red-600 dark:text-red-500" />
              </div>
              <div class="flex-1">
                <h3 class="text-sm font-semibold text-foreground mb-2">No Remote Storage</h3>
                <p class="text-sm text-muted-foreground leading-relaxed">
                  We do not operate any servers or databases that store your personal information. 
                  Your data never leaves your device except when you explicitly upload to YouTube or search Discogs.
                </p>
              </div>
            </div>
          </div>
        </SectionCard>
      </section>

      <!-- Third-Party Services -->
      <section class="space-y-2">
        <SectionHeader 
          label="Third-Party Services"
          description="External services the app connects to"
        />
        <SectionCard>
          <div class="space-y-6">
            <div class="flex items-start gap-3">
              <div class="flex items-center justify-center w-8 h-8 rounded-lg bg-red-500/10 border border-red-500/20 mt-0.5">
                <Wifi class="h-4 w-4 text-red-600 dark:text-red-500" />
              </div>
              <div class="flex-1">
                <h3 class="text-sm font-semibold text-foreground mb-2">YouTube API Services</h3>
                <p class="text-sm text-muted-foreground leading-relaxed mb-3">
                  When you connect your YouTube account, the app uses Google's OAuth 2.0 system for secure authentication. 
                  Video uploads go directly from your device to YouTube's servers.
                </p>
                <div class="space-y-2 mb-3">
                  <p class="text-xs font-medium text-foreground">The following YouTube data is collected:</p>
                  <ul class="text-xs text-muted-foreground space-y-1 ml-4">
                    <li>• YouTube Channel ID and Channel Name</li>
                    <li>• YouTube Video IDs (after upload)</li>
                    <li>• OAuth Access Token (encrypted, stored locally)</li>
                  </ul>
                </div>
                <div class="space-y-2 mb-3">
                  <p class="text-xs font-medium text-foreground">YouTube API scope allows us to:</p>
                  <ul class="text-xs text-muted-foreground space-y-1 ml-4">
                    <li>• Retrieve your YouTube channel information</li>
                    <li>• Upload videos to your YouTube account</li>
                    <li>• View and manage your videos and playlists</li>
                    <li>• Check video processing status</li>
                  </ul>
                </div>
                <Callout variant="info" class="mb-3">
                  <p class="text-xs">
                    <strong>Important:</strong> While the API scope technically allows broader access to your YouTube activity, 
                    this app only uses these permissions for video uploads and status checking. We never modify existing videos, 
                    post comments, or access other YouTube data.
                  </p>
                </Callout>
                <p class="text-xs text-muted-foreground">
                  You can revoke access anytime via your <a href="https://myaccount.google.com/permissions" target="_blank" class="text-primary hover:underline">Google Account permissions</a> or 
                  <a href="https://policies.google.com/privacy" target="_blank" class="text-primary hover:underline">Google's Privacy Policy</a>
                </p>
              </div>
            </div>

            <div class="flex items-start gap-3">
              <div class="flex items-center justify-center w-8 h-8 rounded-lg bg-orange-500/10 border border-orange-500/20 mt-0.5">
                <Database class="h-4 w-4 text-orange-600 dark:text-orange-500" />
              </div>
              <div class="flex-1">
                <h3 class="text-sm font-semibold text-foreground mb-2">Discogs API</h3>
                <p class="text-sm text-muted-foreground leading-relaxed mb-3">
                  When you connect your Discogs account, the app searches for release information and artwork. 
                  All communication happens directly between your device and Discogs.
                </p>
                <div class="space-y-2 mb-3">
                  <p class="text-xs font-medium text-foreground">Discogs data accessed:</p>
                  <ul class="text-xs text-muted-foreground space-y-1 ml-4">
                    <li>• Release information (artist, title, year, label, catalog number)</li>
                    <li>• Release artwork and images</li>
                    <li>• Genre and style information</li>
                    <li>• OAuth Access Token (encrypted, stored locally)</li>
                  </ul>
                </div>
                <p class="text-xs text-muted-foreground">
                  Governed by <a href="https://www.discogs.com/privacy" target="_blank" class="text-primary hover:underline">Discogs Privacy Policy</a>
                </p>
              </div>
            </div>
          </div>
        </SectionCard>
      </section>

      <!-- Data Security -->
      <section class="space-y-2">
        <SectionHeader 
          label="Data Security"
          description="How we protect your information"
        />
        <SectionCard>
          <div class="space-y-6">
            <div class="flex items-start gap-3">
              <div class="flex items-center justify-center w-8 h-8 rounded-lg bg-green-500/10 border border-green-500/20 mt-0.5">
                <Lock class="h-4 w-4 text-green-600 dark:text-green-500" />
              </div>
              <div class="flex-1">
                <h3 class="text-sm font-semibold text-foreground mb-2">Secure Local Storage</h3>
                <p class="text-sm text-muted-foreground leading-relaxed mb-3">
                  All sensitive data is encrypted and stored securely on your device using Tauri's built-in security features.
                </p>
                <ul class="text-xs text-muted-foreground space-y-1 ml-4">
                  <li>• OAuth tokens are encrypted before storage</li>
                  <li>• App preferences use secure key-value storage</li>
                  <li>• No sensitive data is transmitted to external servers</li>
                  <li>• Temporary processing files are cleaned up automatically</li>
                </ul>
              </div>
            </div>

            <div class="flex items-start gap-3">
              <div class="flex items-center justify-center w-8 h-8 rounded-lg bg-blue-500/10 border border-blue-500/20 mt-0.5">
                <Shield class="h-4 w-4 text-blue-600 dark:text-blue-500" />
              </div>
              <div class="flex-1">
                <h3 class="text-sm font-semibold text-foreground mb-2">No Network Tracking</h3>
                <p class="text-sm text-muted-foreground leading-relaxed">
                  This application does not use cookies, analytics, or tracking mechanisms. 
                  All processing happens locally on your device without sending usage data to remote servers.
                </p>
              </div>
            </div>

            <div class="flex items-start gap-3">
              <div class="flex items-center justify-center w-8 h-8 rounded-lg bg-purple-500/10 border border-purple-500/20 mt-0.5">
                <AlertTriangle class="h-4 w-4 text-purple-600 dark:text-purple-500" />
              </div>
              <div class="flex-1">
                <h3 class="text-sm font-semibold text-foreground mb-2">Data Breach Protection</h3>
                <p class="text-sm text-muted-foreground leading-relaxed">
                  Since all data is stored locally and we operate no servers, your information is not vulnerable 
                  to data breaches or unauthorized access from external parties. Only you have access to your data.
                </p>
              </div>
            </div>
          </div>
        </SectionCard>
      </section>

      <!-- Data Usage -->
      <section class="space-y-2">
        <SectionHeader 
          label="How We Use Your Data"
          description="What we do with the information we access"
        />
        <SectionCard>
          <div class="space-y-6">
            <div class="flex items-start gap-3">
              <div class="flex items-center justify-center w-8 h-8 rounded-lg bg-green-500/10 border border-green-500/20 mt-0.5">
                <SettingsIcon class="h-4 w-4 text-green-600 dark:text-green-500" />
              </div>
              <div class="flex-1">
                <h3 class="text-sm font-semibold text-foreground mb-2">Application Functionality</h3>
                <p class="text-sm text-muted-foreground leading-relaxed mb-3">
                  Your data is used exclusively to provide the core functionality of the application:
                </p>
                <ul class="text-xs text-muted-foreground space-y-1 ml-4">
                  <li>• Process audio files for video creation and normalization</li>
                  <li>• Generate videos with your selected images and audio</li>
                  <li>• Upload videos to your YouTube channel</li>
                  <li>• Search and link Discogs releases for metadata</li>
                  <li>• Remember your preferences for improved user experience</li>
                  <li>• Track upload history for your reference</li>
                </ul>
              </div>
            </div>

            <div class="flex items-start gap-3">
              <div class="flex items-center justify-center w-8 h-8 rounded-lg bg-red-500/10 border border-red-500/20 mt-0.5">
                <Users class="h-4 w-4 text-red-600 dark:text-red-500" />
              </div>
              <div class="flex-1">
                <h3 class="text-sm font-semibold text-foreground mb-2">No Data Sharing</h3>
                <p class="text-sm text-muted-foreground leading-relaxed">
                  We do not sell, distribute, lease, or share your personal information with third parties. 
                  Your data is never used for marketing, analytics, or any purpose other than application functionality.
                </p>
              </div>
            </div>
          </div>
        </SectionCard>
      </section>

      <!-- Your Rights -->
      <section class="space-y-2">
        <SectionHeader 
          label="Your Rights"
          description="Control over your data and privacy"
        />
        <SectionCard>
          <div class="space-y-6">
            <div class="flex items-start gap-3">
              <div class="flex items-center justify-center w-8 h-8 rounded-lg bg-green-500/10 border border-green-500/20 mt-0.5">
                <Lock class="h-4 w-4 text-green-600 dark:text-green-500" />
              </div>
              <div class="flex-1">
                <h3 class="text-sm font-semibold text-foreground mb-2">Full Control</h3>
                <p class="text-sm text-muted-foreground leading-relaxed">
                  You have complete control over your data at all times:
                </p>
                <ul class="text-xs text-muted-foreground space-y-1 ml-4 mt-2">
                  <li>• Disconnect YouTube and Discogs accounts anytime from Settings</li>
                  <li>• Clear all app data from the Settings page (preferences, history, tokens)</li>
                  <li>• Delete processed files from your chosen output folders</li>
                  <li>• Revoke API access from your <a href="https://myaccount.google.com/permissions" target="_blank" class="text-primary hover:underline">Google Account</a> or <a href="https://www.discogs.com/settings/applications" target="_blank" class="text-primary hover:underline">Discogs account settings</a></li>
                  <li>• Export your upload history data (available in Settings)</li>
                </ul>
              </div>
            </div>

            <div class="flex items-start gap-3">
              <div class="flex items-center justify-center w-8 h-8 rounded-lg bg-gray-500/10 border border-gray-500/20 mt-0.5">
                <Trash2 class="h-4 w-4 text-gray-600 dark:text-gray-500" />
              </div>
              <div class="flex-1">
                <h3 class="text-sm font-semibold text-foreground mb-2">Data Deletion</h3>
                <p class="text-sm text-muted-foreground leading-relaxed">
                  Since all data is stored locally, uninstalling the app will remove all associated data. 
                  You can also use the "Clear All Data" option in Settings for a complete reset.
                </p>
              </div>
            </div>
          </div>
        </SectionCard>
      </section>

      <!-- Data Retention -->
      <section class="space-y-2">
        <SectionHeader 
          label="Data Retention"
          description="How long we keep your information"
        />
        <SectionCard>
          <div class="space-y-6">
            <div class="flex items-start gap-3">
              <div class="flex items-center justify-center w-8 h-8 rounded-lg bg-blue-500/10 border border-blue-500/20 mt-0.5">
                <Calendar class="h-4 w-4 text-blue-600 dark:text-blue-500" />
              </div>
              <div class="flex-1">
                <h3 class="text-sm font-semibold text-foreground mb-2">Local Storage Duration</h3>
                <p class="text-sm text-muted-foreground leading-relaxed mb-3">
                  Your data is stored locally on your device for as long as you keep the application installed:
                </p>
                <ul class="text-xs text-muted-foreground space-y-1 ml-4">
                  <li>• App preferences: Stored until you clear them or uninstall the app</li>
                  <li>• Upload history: Stored until you clear it or uninstall the app</li>
                  <li>• OAuth tokens: Stored until you disconnect accounts or uninstall the app</li>
                  <li>• Temporary files: Automatically cleaned up after processing</li>
                </ul>
              </div>
            </div>

            <div class="flex items-start gap-3">
              <div class="flex items-center justify-center w-8 h-8 rounded-lg bg-green-500/10 border border-green-500/20 mt-0.5">
                <Trash2 class="h-4 w-4 text-green-600 dark:text-green-500" />
              </div>
              <div class="flex-1">
                <h3 class="text-sm font-semibold text-foreground mb-2">Automatic Cleanup</h3>
                <p class="text-sm text-muted-foreground leading-relaxed">
                  The application automatically manages temporary files and processing data. 
                  No data persists beyond what's necessary for the application to function properly.
                </p>
              </div>
            </div>
          </div>
        </SectionCard>
      </section>

      <!-- Updates -->
      <section class="space-y-2">
        <SectionHeader 
          label="Policy Updates"
          description="How we handle changes to this policy"
        />
        <SectionCard>
          <div class="flex items-start gap-3">
            <div class="flex items-center justify-center w-8 h-8 rounded-lg bg-blue-500/10 border border-blue-500/20 mt-0.5">
              <Shield class="h-4 w-4 text-blue-600 dark:text-blue-500" />
            </div>
            <div class="flex-1">
              <h3 class="text-sm font-semibold text-foreground mb-2">Transparency</h3>
              <p class="text-sm text-muted-foreground leading-relaxed mb-3">
                Any changes to this privacy policy will be clearly communicated through app updates. 
                We will never change our core commitment to local data processing and privacy protection.
              </p>
              <p class="text-xs text-muted-foreground">
                You can check this page at any time to review the current policy. 
                Significant changes will be highlighted in app release notes.
              </p>
            </div>
          </div>
        </SectionCard>
      </section>

      <!-- Contact & Legal -->
      <section class="space-y-2">
        <SectionHeader 
          label="Contact & Legal"
          description="Get in touch or request information"
        />
        <SectionCard>
          <div class="space-y-4">
            <Callout variant="info" title="Questions about privacy?">
              <p class="text-sm">
                This app is designed to work entirely on your device. If you have questions about how your data is handled, 
                please contact us at <a href="mailto:contact@maxencerobinet.fr" class="text-primary hover:underline">contact@maxencerobinet.fr</a>.
              </p>
            </Callout>
            
            <div class="pt-2 border-t border-border">
              <h3 class="text-sm font-semibold text-foreground mb-2">Your Rights Under Data Protection Laws</h3>
              <p class="text-sm text-muted-foreground leading-relaxed mb-3">
                While this application processes data entirely locally, you still have rights regarding your personal information:
              </p>
              <ul class="text-xs text-muted-foreground space-y-1 ml-4">
                <li>• <strong>Right to access:</strong> You can view all your stored data in the app's Settings</li>
                <li>• <strong>Right to rectification:</strong> You can modify any stored preferences or data</li>
                <li>• <strong>Right to erasure:</strong> You can delete all data using "Clear All Data" in Settings</li>
                <li>• <strong>Right to data portability:</strong> Contact us if you need data in a specific format</li>
                <li>• <strong>Right to object:</strong> You can disconnect services or stop using the app at any time</li>
              </ul>
              <p class="text-xs text-muted-foreground mt-3">
                For any data protection requests or concerns, contact <a href="mailto:contact@maxencerobinet.fr" class="text-primary hover:underline">contact@maxencerobinet.fr</a>.
              </p>
            </div>
          </div>
        </SectionCard>
      </section>
    </div>
  </PageContainer>
</template>