<script setup lang="ts">
import { Shield, Database, Wifi, Play, Settings as SettingsIcon } from "lucide-vue-next";
import PageContainer from "../components/layout/PageContainer.vue";
import PageHeader from "../components/layout/PageHeader.vue";
import SectionHeader from "../components/ui/SectionHeader.vue";
import SectionCard from "../components/ui/SectionCard.vue";
import Collapsible from "../components/ui/Collapsible.vue";

const emit = defineEmits<{
  (e: "back"): void;
}>();

const usageFAQs = [
  {
    question: "How do I create and upload videos to YouTube?",
    answer: "Use the main 'Create Videos & Upload to YouTube' option. First connect your YouTube account, then select your audio files, assign images (from files, URLs, or Discogs), choose your settings, and the app will create videos and upload them automatically with metadata."
  },
  {
    question: "What's the difference between the upload options?",
    answer: "The main option creates videos from audio files and uploads them. 'Upload Existing Videos' is for when you already have video files ready and just want to upload them to YouTube with Discogs metadata."
  },
  {
    question: "How do I normalize my audio files?",
    answer: "You can do it during the upload process or under the 'Audio Tools' option to normalize volume levels using professional EBU R128 standards."
  },
  {
    question: "How does Discogs integration work?",
    answer: "Connect your Discogs account to search for release information. You can link releases to your videos to automatically fill titles, descriptions, and tags. You can also use Discogs artwork as video images during creation."
  },
  {
    question: "Where do my processed files go?",
    answer: "You choose the output folder for each operation. The app remembers your last used folders and you can set a preferred default path in Settings. Your original files are never modified."
  },
  {
    question: "Can I track my uploads?",
    answer: "Yes! The Upload History shows all your YouTube uploads with their processing status, links to the videos, and any linked Discogs releases. You can search and filter your history."
  },
  {
    question: "Why are my links truncated and not clickable?",
    answer: "To add clickable links in your YouTube video descriptions, you need to verify your YouTube account first. Once verified, you can include links by editing the video details in YouTube Studio and ensuring the links are formatted correctly with \"http://\" or \"https://\"."
  }
];

const settingsFAQs = [
  {
    question: "What video creation settings can I configure?",
    answer: "You can set default image styles (Automatic, Fit 16:9, Custom background, Background-only), fit modes (contain/cover), and choose your favorite predefined backgrounds. You can also enable audio normalization by default and set high-pass filtering preferences."
  },
  {
    question: "What YouTube upload defaults can I set?",
    answer: "Configure default privacy settings (public, private, unlisted, draft), video categories, playlists for auto-assignment, and enable desktop notifications when uploads complete. You can also set whether to schedule private videos to go public."
  },
  {
    question: "How do I manage output folders?",
    answer: "Set a preferred default output path that will be used across all operations. The app remembers your last used folders for each feature, and you can always override the default when needed."
  },
  {
    question: "Can I customize description templates?",
    answer: "Yes! Build reusable description templates using Discogs tags like artist, release year, label, catalog number, genres, and styles. Mix these with your own custom text to create consistent video descriptions."
  },
  {
    question: "What audio processing defaults are available?",
    answer: "Enable audio normalization by default during video creation, set the high-pass filter (30Hz) to be on by default, and choose whether to overwrite original files or add suffixes when normalizing audio separately."
  },
  {
    question: "How do I manage my connected accounts?",
    answer: "View and disconnect your YouTube and Discogs accounts anytime from Settings. You can see connection status, re-authenticate if needed, and completely clear all app data to reset everything to defaults."
  }
];

const securityFAQs = [
  {
    question: "How secure is my data?",
    answer: "All your data is stored locally on your device. We don't collect, store, or transmit any personal information to our servers. Your audio files, videos, and metadata never leave your computer unless you explicitly upload them to YouTube or Discogs through their official APIs."
  },
  {
    question: "What data does the app access?",
    answer: "The app only accesses files and folders you explicitly select. It reads audio metadata from your files, processes images you choose, and connects to YouTube and Discogs APIs only when you authenticate and authorize these connections."
  },
  {
    question: "Are my YouTube and Discogs credentials safe?",
    answer: "Yes. We use OAuth 2.0 authentication, which means your passwords are never stored in the app. Authentication tokens are stored securely on your device and can be revoked at any time from your YouTube or Discogs account settings."
  },
  {
    question: "Can you see my uploaded videos?",
    answer: "No. The app uploads directly to YouTube using your authenticated connection. We have no access to your YouTube account, videos, or any data you upload. All communication happens directly between your device and YouTube's servers."
  }
];

const dataPracticesFAQs = [
  {
    question: "What happens to my files?",
    answer: "Your original files remain untouched. The app creates copies for processing (video creation, audio normalization) in your chosen output folders. You can delete these processed files at any time."
  },
  {
    question: "Is my upload history private?",
    answer: "Yes. Your upload history is stored locally on your device only. It helps you track your uploads and their processing status, but this data never leaves your computer."
  },
  {
    question: "Do you track my usage?",
    answer: "No. We don't collect analytics, usage statistics, or any telemetry data. The app works entirely offline except for the YouTube and Discogs API connections you explicitly authorize."
  },
  {
    question: "Can I delete all my data?",
    answer: "Yes. You can clear all app data (preferences, history, tokens) from the Settings page. This will reset the app to its initial state and remove all stored information."
  }
];

const technicalFAQs = [
  {
    question: "Where are my settings stored?",
    answer: "All settings and preferences are stored locally using Tauri's secure storage system. This data is encrypted and tied to your user account on your device."
  },
  {
    question: "What network connections does the app make?",
    answer: "The app only connects to YouTube and Discogs APIs when you're authenticated and performing actions like uploading videos or searching releases. No other network connections are made."
  },
  {
    question: "How does audio processing work?",
    answer: "Audio normalization is performed locally using FFmpeg. Your audio files are processed entirely on your device - no audio data is sent to external servers."
  }
];
</script>

<template>
  <PageContainer>
    <PageHeader
      title="Frequently Asked Questions"
      description="Common questions about security, privacy, and how your data is handled"
      @back="emit('back')"
    />

    <div class="space-y-8">
      <!-- Usage Section -->
      <section class="space-y-2">
        <SectionHeader 
          label="How to Use"
          description="Getting started and using the app's main features"
        />
        <SectionCard>
          <div class="space-y-4">
            <div class="flex items-center gap-3 mb-6">
              <div class="flex items-center justify-center w-10 h-10 rounded-lg bg-primary/10 border border-primary/20">
                <Play class="h-5 w-5 text-primary" />
              </div>
              <div>
                <h3 class="text-sm font-semibold text-foreground">Quick start guide</h3>
                <p class="text-xs text-muted-foreground mt-0.5">Learn how to use the main features effectively</p>
              </div>
            </div>
            
            <div class="space-y-3">
              <Collapsible
                v-for="faq in usageFAQs"
                :key="faq.question"
                :label="faq.question"
                class="border border-border rounded-lg"
              >
                <p class="text-sm text-foreground leading-relaxed">{{ faq.answer }}</p>
              </Collapsible>
            </div>
          </div>
        </SectionCard>
      </section>

      <!-- Settings Section -->
      <section class="space-y-2">
        <SectionHeader 
          label="Settings & Configuration"
          description="Customizing defaults for video creation and uploads"
        />
        <SectionCard>
          <div class="space-y-4">
            <div class="flex items-center gap-3 mb-6">
              <div class="flex items-center justify-center w-10 h-10 rounded-lg bg-orange-500/10 border border-orange-500/20">
                <SettingsIcon class="h-5 w-5 text-orange-600 dark:text-orange-500" />
              </div>
              <div>
                <h3 class="text-sm font-semibold text-foreground">Personalize your workflow</h3>
                <p class="text-xs text-muted-foreground mt-0.5">Set defaults to speed up video creation and uploads</p>
              </div>
            </div>
            
            <div class="space-y-3">
              <Collapsible
                v-for="faq in settingsFAQs"
                :key="faq.question"
                :label="faq.question"
                class="border border-border rounded-lg"
              >
                <p class="text-sm text-foreground leading-relaxed">{{ faq.answer }}</p>
              </Collapsible>
            </div>
          </div>
        </SectionCard>
      </section>

      <!-- Security Section -->
      <section class="space-y-2">
        <SectionHeader 
          label="Security & Privacy"
          description="How we protect your data and ensure your privacy"
        />
        <SectionCard>
          <div class="space-y-4">
            <div class="flex items-center gap-3 mb-6">
              <div class="flex items-center justify-center w-10 h-10 rounded-lg bg-green-500/10 border border-green-500/20">
                <Shield class="h-5 w-5 text-green-600 dark:text-green-500" />
              </div>
              <div>
                <h3 class="text-sm font-semibold text-foreground">Your data stays on your device</h3>
                <p class="text-xs text-muted-foreground mt-0.5">Local processing, no external data collection</p>
              </div>
            </div>
            
            <div class="space-y-3">
              <Collapsible
                v-for="faq in securityFAQs"
                :key="faq.question"
                :label="faq.question"
                class="border border-border rounded-lg"
              >
                <p class="text-sm text-foreground leading-relaxed">{{ faq.answer }}</p>
              </Collapsible>
            </div>
          </div>
        </SectionCard>
      </section>

      <!-- Data Practices Section -->
      <section class="space-y-2">
        <SectionHeader 
          label="Data Practices"
          description="How your files and information are handled"
        />
        <SectionCard>
          <div class="space-y-4">
            <div class="flex items-center gap-3 mb-6">
              <div class="flex items-center justify-center w-10 h-10 rounded-lg bg-blue-500/10 border border-blue-500/20">
                <Database class="h-5 w-5 text-blue-600 dark:text-blue-500" />
              </div>
              <div>
                <h3 class="text-sm font-semibold text-foreground">Local storage only</h3>
                <p class="text-xs text-muted-foreground mt-0.5">All data remains under your control</p>
              </div>
            </div>
            
            <div class="space-y-3">
              <Collapsible
                v-for="faq in dataPracticesFAQs"
                :key="faq.question"
                :label="faq.question"
                class="border border-border rounded-lg"
              >
                <p class="text-sm text-foreground leading-relaxed">{{ faq.answer }}</p>
              </Collapsible>
            </div>
          </div>
        </SectionCard>
      </section>

      <!-- Technical Section -->
      <section class="space-y-2">
        <SectionHeader 
          label="Technical Details"
          description="How the app works under the hood"
        />
        <SectionCard>
          <div class="space-y-4">
            <div class="flex items-center gap-3 mb-6">
              <div class="flex items-center justify-center w-10 h-10 rounded-lg bg-purple-500/10 border border-purple-500/20">
                <Wifi class="h-5 w-5 text-purple-600 dark:text-purple-500" />
              </div>
              <div>
                <h3 class="text-sm font-semibold text-foreground">Minimal network usage</h3>
                <p class="text-xs text-muted-foreground mt-0.5">Only connects when you explicitly authorize it</p>
              </div>
            </div>
            
            <div class="space-y-3">
              <Collapsible
                v-for="faq in technicalFAQs"
                :key="faq.question"
                :label="faq.question"
                class="border border-border rounded-lg"
              >
                <p class="text-sm text-foreground leading-relaxed">{{ faq.answer }}</p>
              </Collapsible>
            </div>
          </div>
        </SectionCard>
      </section>
    </div>
  </PageContainer>
</template>