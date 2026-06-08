import Link from "next/link";
import { ArrowLeft, Shield } from "lucide-react";

export const metadata = {
  title: "Privacy Policy | ALOK JARVIS OS",
  description: "Privacy Policy for ALOK JARVIS OS - 100% local and secure desktop AI execution.",
};

export default function PrivacyPage() {
  return (
    <div className="relative min-h-screen bg-bg-primary overflow-hidden py-20 px-6">
      {/* Background soft glows */}
      <div className="absolute top-0 right-1/4 w-[500px] h-[500px] radial-glow-cyan pointer-events-none opacity-10" />
      <div className="absolute bottom-0 left-1/4 w-[500px] h-[500px] radial-glow-violet pointer-events-none opacity-10" />

      <div className="max-w-4xl mx-auto z-10 relative flex flex-col gap-10">
        {/* Back Link */}
        <Link
          href="/"
          className="inline-flex items-center gap-2 text-sm font-mono text-text-secondary hover:text-accent-electric transition-colors w-fit group"
        >
          <ArrowLeft className="w-4 h-4 group-hover:-translate-x-1 transition-transform" />
          <span>BACK TO HOME</span>
        </Link>

        {/* Page Header */}
        <div className="flex flex-col gap-4 border-b border-glass-border pb-8 text-left">
          <div className="inline-flex items-center gap-2 text-xs font-mono text-accent-cyan uppercase tracking-widest">
            <Shield className="w-4 h-4" />
            <span>Privacy Standards</span>
          </div>
          <h1 className="text-4xl md:text-5xl font-bold tracking-tight text-white">
            Privacy Policy
          </h1>
          <p className="text-xs font-mono text-text-muted">
            LAST UPDATED: JUNE 8, 2026
          </p>
        </div>

        {/* Content Panel */}
        <div className="glass-panel rounded-2xl p-8 md:p-10 border border-glass-border flex flex-col gap-8 text-left text-text-secondary font-light text-sm md:text-base leading-relaxed">
          <section className="flex flex-col gap-4">
            <h2 className="text-xl font-semibold text-white">1. Core Commitment</h2>
            <p>
              Privacy is not a feature; it is the default foundation of <strong>ALOK JARVIS OS</strong>. The software is designed from the ground up to keep your personal data, voice commands, and operating system activities entirely on your local machine.
            </p>
          </section>

          <section className="flex flex-col gap-4">
            <h2 className="text-xl font-semibold text-white">2. Data Localized by Design</h2>
            <p>
              Unlike traditional cloud-based voice assistants, ALOK JARVIS OS does not upload your telemetry, audio recordings, or text conversations to external servers.
            </p>
            <ul className="list-disc pl-6 flex flex-col gap-2">
              <li><strong>Voice Processing:</strong> Speech recognition (Whisper) and Voice synthesis (Piper) run completely offline on your local CPU. Your audio never leaves your RAM/disk.</li>
              <li><strong>Local Storage:</strong> All chat records, custom actions, environment paths, and local memory contexts are stored in a local SQLite database file in your device's <code>%APPDATA%</code> path.</li>
              <li><strong>Zero Tracking:</strong> No tracking cookies, telemetry SDKs, or third-party analytic services are bundled with the desktop client binary.</li>
            </ul>
          </section>

          <section className="flex flex-col gap-4">
            <h2 className="text-xl font-semibold text-white">3. Third-Party API Integrations</h2>
            <p>
              You may choose to connect external AI endpoints (e.g. Groq, OpenAI, or custom self-hosted LLM servers) or enable search integration (Tavily, Google API). If enabled:
            </p>
            <ul className="list-disc pl-6 flex flex-col gap-2">
              <li>Only the specific prompt payload required for the action is transmitted over HTTPS to the selected provider.</li>
              <li>Your API Keys are stored locally on your device in your configuration file, encrypted using local OS keyring standards. We have zero access to your API keys.</li>
            </ul>
          </section>

          <section className="flex flex-col gap-4">
            <h2 className="text-xl font-semibold text-white">4. Auto-Update Telemetry</h2>
            <p>
              The desktop application periodically fetches a public configuration manifest (<code>update.json</code>) from <code>https://jarvis.alokkumarsahu.in</code> to verify if a new installer bundle is available. This query does not transmit any identifiable user metrics, machine IDs, or metadata.
            </p>
          </section>

          <section className="flex flex-col gap-4 border-t border-glass-border/30 pt-6">
            <h2 className="text-xl font-semibold text-white">5. Contact and Concerns</h2>
            <p>
              Because your data stays entirely on your machine, we do not have the technical ability to retrieve, view, or delete your voice databases. If you wish to wipe your memory database, you can do so directly by deleting the local database file from your application settings or AppData folder.
            </p>
            <p className="mt-2">
              For any questions regarding open-source code compliance or privacy standards, contact us at: <a href="mailto:alok.vssut28@gmail.com" className="text-accent-cyan hover:underline">alok.vssut28@gmail.com</a>.
            </p>
          </section>
        </div>
      </div>
    </div>
  );
}
