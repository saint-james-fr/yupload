# YouTube API Verification & Compliance

This document explains the **unverified project restriction** for the YouTube Data API, how to get your app verified, and an analysis of whether this app follows YouTube’s rules.

---

## 1. Is the Restriction Real?

**Yes, but with important distinctions.** For API projects created **after July 28, 2020** that have **not** passed a YouTube API compliance audit:

- All uploads via the `videos.insert` endpoint are **forced to private**, regardless of the `privacyStatus` you send (`"public"`, `"unlisted"`, `"scheduled"`, etc.).
- There is no way to make those uploads public, unlisted, or scheduled without the project being verified (audited).

**However**, there are two separate verification processes that can cause confusion:

### OAuth App Test Mode vs. YouTube API Verification

| Type | What it controls | Where it's set |
|------|------------------|----------------|
| **OAuth Consent Screen** (Test Mode) | Who can authenticate with your app | Google Cloud Console → OAuth consent screen |
| **YouTube API Compliance Audit** | Whether uploads can be public/unlisted | YouTube API Services audit form |

**Important:** Your OAuth app being in "test mode" (OAuth consent screen) is **different** from YouTube API project verification. These are separate processes:

- **OAuth Test Mode**: Limits who can authenticate (only test users you add). This does **not** affect whether uploads can be public.
- **YouTube API Verification**: Determines if uploads can be public/unlisted vs. forced to private.

### Why You Might Be Able to Upload Public Videos

If you successfully uploaded a public video, it could be because:

1. **Your project was created before July 28, 2020** — Projects created before this date are grandfathered and can upload public videos without verification.
2. **Your project has already been verified** — You may have already passed the YouTube API compliance audit (check your Google Cloud project history or emails from YouTube API Services).
3. **Different project** — You might be using a different Google Cloud project that was created earlier or is already verified.

**Note:** OAuth app "test mode" (consent screen) is separate from YouTube API verification. Test mode only controls who can authenticate, not whether uploads can be public. If you're in test mode and can upload public videos, it's likely because your project was created before July 2020 or has already been verified.

To check your project's verification status, look for:
- Emails from YouTube API Services about audit approval
- Your Google Cloud project creation date (if before July 28, 2020, you're exempt)
- The project's API quota page in Google Cloud Console

So the note is accurate:

> *Unverified API projects (created after July 2020) are limited to private uploads until the app is verified. Public/unlisted/scheduled will work for verified projects or during testing with private/scheduled and draft.*

---

## 2. Official Sources

| Source | URL | What it says |
|--------|-----|--------------|
| **Quota and Compliance Audits** | [developers.google.com/youtube/v3/guides/quota_and_compliance_audits](https://developers.google.com/youtube/v3/guides/quota_and_compliance_audits) | Describes the audit requirement and links to the YouTube API audit form. |
| **Videos: insert** | [developers.google.com/youtube/v3/docs/videos/insert](https://developers.google.com/youtube/v3/docs/videos/insert) | Reference for the upload endpoint; behavior aligns with the private-only policy for unverified projects. |
| **Uploading a video** | [developers.google.com/youtube/v3/guides/uploading_a_video](https://developers.google.com/youtube/v3/guides/uploading_a_video) | Documents `privacyStatus` and upload flow. |
| **Developer Policies** | [developers.google.com/youtube/terms/developer-policies-guide](https://developers.google.com/youtube/terms/developer-policies-guide) | Compliance requirements for API use. |
| **API Services Terms of Service** | [developers.google.com/youtube/terms/api-services-terms-of-service](https://developers.google.com/youtube/terms/api-services-terms-of-service) | Legal terms for using the API. |

The **July 28, 2020** date and the “unverified → private only” behavior are widely referenced (e.g. Stack Overflow, Google’s own summaries) and match the purpose of the **Audit and Quota Extension** process.

---

## 3. How to Verify the App (Enable Public/Unlisted/Scheduled)

“Verification” here means **your YouTube Data API project passes a compliance audit**. That is separate from generic “OAuth app verification” for sensitive scopes.

### Step 1: Request an audit

Use the official form:

- **YouTube API Services – Audit and Quota Extension Form**  
  **https://support.google.com/youtube/contact/yt_api_form**

Submit it for the Google Cloud project that uses the YouTube Data API. A YouTube API Services team member will respond.

### Step 2: What the audit evaluates

You must show that the application:

- Uses the API in line with the **YouTube API Services Terms of Service**
- Follows **YouTube’s Developer Policies** (and any other policies referenced in the form/docs)

They typically review:

- What the app does (especially uploads and any automation)
- How you use quota
- That you don’t circumvent YouTube’s rules or abuse the API

The main reference is:

- [Quota and Compliance Audits](https://developers.google.com/youtube/v3/guides/quota_and_compliance_audits)

### Step 3: Other forms (after audit or special cases)

- **Audited Developer Requests** (e.g. extra quota after a recent audit):  
  https://support.google.com/youtube/contact/yt_api_audited_developer_requests_form  
- **Appeals** if your audit is rejected  
- **Periodic / change-of-control** forms when YouTube or the form instructs you to use them

### Step 4: Until and after approval

- **Before approval:** only private uploads will work for that project.
- **After approval:** public, unlisted, and scheduled uploads via `videos.insert` can work as documented (subject to normal quota and product rules).
- “Testing with private/scheduled and draft” matches: private always works; scheduling/draft behavior may depend on current YouTube testing allowances; the main requirement for “public/unlisted/scheduled in production” is passing this audit.

---

## 4. Compliance Analysis: Does This App Follow the Rules?

This section checks this codebase against YouTube’s usual requirements (Developer Policies, API Services Terms of Service, Quota and Compliance Audits). It is not legal advice.

### 4.1 API usage & intent

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Use API for intended, compliant use | ✅ | Desktop app for **user-initiated** video creation and uploads. No automated/bot uploads, no scraping. |
| User-initiated uploads | ✅ | All uploads go through `upload_video_to_youtube` / `upload_bulk_videos_to_youtube`; triggered by user actions in `VideoUpload.vue`. |
| No quota circumvention | ✅ | Single OAuth client, single project, standard flows. No sharding or multi-project tricks. |
| Standard OAuth + scopes | ✅ | Uses `youtube.upload` and `youtube`, OAuth 2.0 PKCE for desktop (`youtube_client.rs`). |

### 4.2 Data & privacy

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Minimal user data | ✅ | Only OAuth tokens (access/refresh) for the YouTube account the user connects. |
| Token storage | ✅ | Tokens stored locally in app store, encrypted (`decrypt_token` / `encrypt_token` in `youtube_client.rs`). |
| No harvesting/surveillance | ✅ | No background collection of user or viewer data. |
| User control | ✅ | User connects/disconnects YouTube in Settings; tokens deleted on disconnect. |

**Gap:** A **privacy policy** and clear disclosure of data use are typically expected for verification. The codebase does not define or host one; you will need a privacy policy (and possibly a small in-app or download-page notice) for the audit.

### 4.3 Metadata & API limits

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Title length | ✅ | Enforced in `validate_youtube_title`: max 100 characters (`validation.rs`). |
| Description length | ✅ | Enforced in `validate_youtube_description`: max 5000 characters (`validation.rs`). |
| Tags | ⚠️ | Tags are sent as provided (`youtube_models.VideoUploadRequest.tags` → API). YouTube limits: total ~500 characters, each tag ≤30 characters. There is **no client-side validation** of tag count/length; long or huge tag lists could hit API errors. |
| Category | ✅ | `category_id` sent as chosen by user (e.g. Music `"10"`). |
| `privacyStatus` / `publishAt` | ✅ | `public` / `private` / `unlisted` and optional `publish_at` for private/scheduled; draft mapped to private before send. Behavior matches API; actual visibility still subject to project verification. |

### 4.4 Prohibited behaviors

| Requirement | Status | Evidence |
|-------------|--------|----------|
| No selling ads on YouTube content | ✅ | No ad or monetization logic. |
| No circumventing restrictions | ✅ | Uses documented `videos.insert` and resumable upload protocol. |
| No replicating core YouTube experience | ✅ | Tool assists creation/upload; does not replace youtube.com. |
| Independent value | ✅ | Video creation from audio + images, normalization, Discogs metadata, bulk upload. |

### 4.5 Scopes & OAuth

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Needed scopes only | ✅ | `youtube.upload` and `youtube`; used for uploads, playlists.list, playlistItems.insert. |
| Desktop / public client | ✅ | PKCE, no client secret, local callback server; matches guidance for installed apps. |

### 4.6 Summary

- The app’s **purpose and use** of the API (user-driven uploads, no abuse, no quota tricks) align well with YouTube’s typical compliance expectations.
- **Metadata validation** is mostly correct; the main technical gap is **tags** (no enforcement of total length or per-tag length).
- The main **non-code** gap for verification is a **privacy policy** and, where relevant, a short in-app or web notice about how you use Google/YouTube data.

---

## 5. Recommendations Before Requesting Verification

1. **Add a privacy policy**  
   Describe what data you collect (e.g. OAuth tokens, local upload history), how it’s stored, and that you don’t sell or share it. Host it (e.g. on a project or product site) and, if applicable, link it from the app or installer.

2. **Optional: validate tags before upload**  
   - Total tags string length ≤ 500 characters (YouTube’s approximate limit).  
   - Each tag ≤ 30 characters.  
   This avoids avoidable API errors and shows careful use of the API.

3. **Prepare audit materials**  
   - Short description of the app (creation + upload tool for content creators).  
   - How you use the YouTube Data API (upload, playlists list, playlistItems insert).  
   - How you store and protect tokens (local, encrypted).  
   - Link to privacy policy (and, if any, terms of use).

4. **In-app note about unverified projects**  
   In the upload or YouTube-settings UI, you can add a line such as:  
   *“If your Google Cloud project is not yet verified by YouTube, uploads will be private until verification. To request verification, use the [YouTube API audit form](https://support.google.com/youtube/contact/yt_api_form).”*

---

## 6. Quick reference

| Item | Value |
|------|--------|
| Unverified projects (created after July 28, 2020) | Uploads forced to **private** |
| To enable public/unlisted/scheduled | Pass **YouTube API compliance audit** |
| Request audit | [YouTube API Services – Audit and Quota Extension Form](https://support.google.com/youtube/contact/yt_api_form) |
| Official guide | [Quota and Compliance Audits](https://developers.google.com/youtube/v3/guides/quota_and_compliance_audits) |
| Developer policies | [Complying with YouTube's Developer Policies](https://developers.google.com/youtube/terms/developer-policies-guide) |
