import Link from "next/link";
import { ArrowLeft, ShieldAlert } from "lucide-react";

export const metadata = {
  title: "Terms and Conditions | ALOK JARVIS OS",
  description: "Terms and Conditions for using ALOK JARVIS OS desktop application and website.",
};

export default function TermsPage() {
  return (
    <div className="relative min-h-screen bg-bg-primary overflow-hidden py-20 px-6">
      {/* Background soft glows */}
      <div className="absolute top-0 left-1/4 w-[500px] h-[500px] radial-glow-cyan pointer-events-none opacity-10" />
      <div className="absolute bottom-0 right-1/4 w-[500px] h-[500px] radial-glow-violet pointer-events-none opacity-10" />

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
          <div className="inline-flex items-center gap-2 text-xs font-mono text-accent-electric uppercase tracking-widest">
            <ShieldAlert className="w-4 h-4" />
            <span>Legal Agreement</span>
          </div>
          <h1 className="text-4xl md:text-5xl font-bold tracking-tight text-white">
            Terms and Conditions
          </h1>
          <p className="text-xs font-mono text-text-muted">
            LAST UPDATED: JUNE 8, 2026
          </p>
        </div>

        {/* Content Panel */}
        <div className="glass-panel rounded-2xl p-8 md:p-10 border border-glass-border flex flex-col gap-8 text-left text-text-secondary font-light text-sm md:text-base leading-relaxed">
          <section className="flex flex-col gap-4">
            <h2 className="text-xl font-semibold text-white">1. Agreement to Terms</h2>
            <p>
              By accessing, downloading, or using <strong>ALOK JARVIS OS</strong> (referred to as the &ldquo;Application,&rdquo; &ldquo;Software,&rdquo; or &ldquo;Service&rdquo;), you agree to be bound by these Terms and Conditions. If you do not agree with any of these terms, you are prohibited from using or accessing this software.
            </p>
          </section>

          <section className="flex flex-col gap-4">
            <h2 className="text-xl font-semibold text-white">2. License Grant & Restrictions</h2>
            <p>
              ALOK JARVIS OS is distributed as a local utility. You are granted a personal, non-transferable, non-exclusive license to use the software on your personal Windows desktop devices under the following conditions:
            </p>
            <ul className="list-disc pl-6 flex flex-col gap-2">
              <li>You shall not decompile, reverse-engineer, or disassemble the binary packages unless explicitly permitted by open-source licensing components.</li>
              <li>You shall not use the software to execute malicious commands, automate spam operations, or interfere with third-party operating systems or APIs.</li>
              <li>You agree that the software operates with high-level OS execution permissions to facilitate desktop automations. You assume all responsibility for commands triggered via voice activation.</li>
            </ul>
          </section>

          <section className="flex flex-col gap-4">
            <h2 className="text-xl font-semibold text-white">3. Offline Operation & Model Ownership</h2>
            <p>
              ALOK JARVIS OS executes speech-to-text (Whisper) and text-to-speech (Piper) models locally on your system. You are responsible for ensuring your system has sufficient hardware capability to run these models. Any third-party APIs integrated by you (such as custom LLM endpoints or web search APIs) are subject to their respective terms of service and usage billing.
            </p>
          </section>

          <section className="flex flex-col gap-4">
            <h2 className="text-xl font-semibold text-white">4. Disclaimer of Warranty</h2>
            <p className="italic bg-white/2 border border-glass-border rounded-xl p-5 text-text-muted">
              &ldquo;The software is provided &lsquo;as is&rsquo;, without warranty of any kind, express or implied, including but not limited to the warranties of merchantability, fitness for a particular purpose and noninfringement. In no event shall the authors or copyright holders be liable for any claim, damages or other liability, whether in an action of contract, tort or otherwise, arising from, out of or in connection with the software or the use or other dealings in the software.&rdquo;
            </p>
          </section>

          <section className="flex flex-col gap-4">
            <h2 className="text-xl font-semibold text-white">5. Limitation of Liability</h2>
            <p>
              Under no circumstances shall Alok Kumar Sahu or contributors be liable for any direct, indirect, incidental, special, or consequential damages (including loss of data, system file corruption, or command execution anomalies) resulting from the use or inability to use the Software.
            </p>
          </section>

          <section className="flex flex-col gap-4 border-t border-glass-border/30 pt-6">
            <h2 className="text-xl font-semibold text-white">6. Changes to Terms</h2>
            <p>
              We reserve the right to revise these terms at any time. By continuing to use the Software after changes are published, you agree to be bound by the updated terms.
            </p>
          </section>
        </div>
      </div>
    </div>
  );
}
