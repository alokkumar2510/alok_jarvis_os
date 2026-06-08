# ALOK JARVIS OS - Landing Page & Auto-Updater Site

This is the startup-quality landing page website for **ALOK JARVIS OS**, built using Next.js 15, React, TypeScript, Tailwind CSS, and Framer Motion. It is optimized for static export and deployable directly on Cloudflare Pages.

## Tech Stack
* **Framework:** Next.js 15 (App Router, Static Export)
* **Styling:** Tailwind CSS (v4)
* **Animation:** Framer Motion (120 FPS targets, 3D Canvas Orb, interactive waveforms)
* **Icons:** Lucide React
* **Deployment:** Cloudflare Pages (via GitHub or Wrangler CLI)
* **Domain:** jarvis.alokkumarsahu.in

---

## Local Development

1. Navigate to the website sub-directory:
   ```bash
   cd website
   ```
2. Install dependencies:
   ```bash
   npm install
   ```
3. Run the local development server:
   ```bash
   npm run dev
   ```
4. Open [http://localhost:3000](http://localhost:3000) in your browser.

---

## Build for Static Export

To generate the static HTML/CSS/JS bundles for Cloudflare hosting:
```bash
npm run build
```
This command compiles the project and outputs all static assets to the `/out` directory.

---

## Cloudflare Pages Deployment

### Option A: Automatic Git Integration (Recommended)
1. Go to your **Cloudflare Dashboard** -> **Workers & Pages** -> **Create Application** -> **Pages** -> **Connect to Git**.
2. Select your repository `alokkumar2510/alok_jarvis_os`.
3. Set the following build parameters:
   * **Framework Preset:** `Next.js (Static HTML Export)`
   * **Root Directory:** `website`
   * **Build Command:** `npm run build`
   * **Build Output Directory:** `out`
   * **Compatibility Date:** `2026-06-08` or newer.
4. Click **Save and Deploy**. Cloudflare will rebuild and redeploy the site automatically on every commit to `master`.

### Option B: Wrangler CLI Manual Deployment
If you prefer deploying directly from your terminal using Wrangler:
1. Log in to your Cloudflare account via CLI:
   ```bash
   npx wrangler login
   ```
2. Build the website:
   ```bash
   npm run build
   ```
3. Deploy the `/out` folder:
   ```bash
   npx wrangler pages deploy out --project-name alok-jarvis-website
   ```

---

## Download & Auto-Updater System

* **Installer Hosting:** The download buttons in the Download section link directly to your GitHub release assets (e.g. `https://github.com/alokkumar2510/alok_jarvis_os/releases/download/v0.1.0/alok_jarvis_os_0.1.0_x64-setup.exe`). This offloads bandwidth from Cloudflare Pages, which has a 25MB single-asset size limit on free plans.
* **Auto-Update Endpoint:** Tauri’s native auto-updater queries the static JSON manifest hosted at `https://jarvis.alokkumarsahu.in/update.json`.
  To release a new update:
  1. Compile your new Tauri installer.
  2. Upload the installer `.exe` and `.msi` to a new GitHub Release.
  3. Edit [website/public/update.json](file:///e:/ALOK%20PC/alok_jarvis_os/website/public/update.json) with the new version number, date, and GitHub download URLs.
  4. Push to master. Cloudflare will automatically deploy the updated manifest, triggering the update prompt for all running desktop clients.
