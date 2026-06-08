"use client";

import { motion } from "framer-motion";
import { Globe, Mail } from "lucide-react";

function GithubIcon(props: React.ComponentProps<"svg">) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      {...props}
    >
      <path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4" />
      <path d="M9 18c-4.51 2-5-2-7-2" />
    </svg>
  );
}

export default function Founder() {
  return (
    <section id="founder" className="py-32 relative overflow-hidden bg-bg-secondary/10">
      {/* Background soft glow */}
      <div className="absolute top-1/2 right-1/4 w-[500px] h-[500px] radial-glow-cyan pointer-events-none opacity-10" />
      <div className="absolute bottom-1/2 left-1/4 w-[500px] h-[500px] radial-glow-violet pointer-events-none opacity-10" />

      <div className="max-w-5xl mx-auto px-6 flex flex-col gap-16 items-center z-10 relative">
        <div className="flex flex-col items-center gap-4 text-center">
          <div className="text-accent-electric text-xs font-mono tracking-widest uppercase">
            Leadership
          </div>
          <h2 className="text-4xl md:text-5xl font-semibold tracking-tight text-white">
            Behind the OS
          </h2>
          <p className="text-text-secondary max-w-xl font-light text-base leading-relaxed">
            The mission to make desktop computing private, voice-first, and fully autonomous.
          </p>
        </div>

        {/* Founder Glassmorphic Card */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.8, ease: [0.16, 1, 0.3, 1] }}
          className="w-full glass-panel rounded-3xl p-8 md:p-12 border border-glass-border grid grid-cols-1 md:grid-cols-12 gap-10 items-center relative overflow-hidden"
        >
          {/* Subtle top sweeping glow */}
          <div className="absolute top-0 left-0 w-full h-[2px] bg-gradient-to-r from-transparent via-accent-electric to-transparent opacity-50" />

          {/* Left Side: Avatar Profile Picture */}
          <div className="md:col-span-4 flex justify-center relative">
            <div className="relative w-48 h-48 md:w-56 md:h-56 rounded-full overflow-hidden border-2 border-accent-electric/30 shadow-glow-cyan/20 group">
              <div className="absolute inset-0 bg-gradient-to-tr from-accent-electric/10 to-accent-violet/10 z-10 pointer-events-none group-hover:opacity-0 transition-opacity duration-300" />
              <img
                src="/founder.png"
                alt="Alok Kumar Sahu"
                className="w-full h-full object-cover scale-[1.02] group-hover:scale-[1.05] transition-transform duration-500"
              />
            </div>

            {/* Tech stats badge */}
            <div className="absolute -bottom-3 bg-black/80 border border-glass-border-glow text-accent-electric px-3 py-1 rounded-full text-[10px] font-mono tracking-wider shadow-md">
              CREATOR & LEAD ARCHITECT
            </div>
          </div>

          {/* Right Side: Description Biography */}
          <div className="md:col-span-8 flex flex-col gap-6 text-left">
            <div className="flex flex-col gap-1.5">
              <h3 className="text-3xl font-semibold text-white">Alok Kumar Sahu</h3>
              <span className="text-xs font-mono text-accent-cyan tracking-wider uppercase">Lead Developer, ALOK JARVIS OS</span>
            </div>

            <p className="text-text-secondary text-sm md:text-base font-light leading-relaxed">
              &ldquo;I built ALOK OS to solve a personal friction: the overhead of executing daily developer workflows, organizing messy directory trees, and managing files using traditional keyboard-and-mouse commands.
              ALOK is designed to be a private, voice-first companion that virtualizes operations and automates steps using swarm intelligence. Everything runs 100% locally on your desktop to respect your digital privacy.&rdquo;
            </p>

            {/* Social links */}
            <div className="flex items-center gap-4 pt-4 border-t border-glass-border/30">
              <a
                href="https://github.com/alokkumar2510"
                target="_blank"
                rel="noreferrer"
                className="p-2.5 rounded-xl border border-glass-border bg-white/2 hover:bg-white/5 text-text-secondary hover:text-accent-electric transition-colors"
                title="GitHub Profile"
              >
                <GithubIcon className="w-4.5 h-4.5" />
              </a>
              <a
                href="https://alokkumarsahu.in"
                target="_blank"
                rel="noreferrer"
                className="p-2.5 rounded-xl border border-glass-border bg-white/2 hover:bg-white/5 text-text-secondary hover:text-accent-electric transition-colors"
                title="Personal Website"
              >
                <Globe className="w-4.5 h-4.5" />
              </a>
              <a
                href="mailto:alok.vssut28@gmail.com"
                className="p-2.5 rounded-xl border border-glass-border bg-white/2 hover:bg-white/5 text-text-secondary hover:text-accent-electric transition-colors"
                title="Contact Mail"
              >
                <Mail className="w-4.5 h-4.5" />
              </a>
            </div>
          </div>
        </motion.div>
      </div>
    </section>
  );
}
