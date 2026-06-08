"use client";

import { Terminal } from "lucide-react";

export default function Footer() {
  const currentYear = new Date().getFullYear();

  return (
    <footer className="border-t border-glass-border bg-black/40 py-16 relative overflow-hidden">
      <div className="max-w-7xl mx-auto px-6 grid grid-cols-1 md:grid-cols-12 gap-12 items-center z-10 relative">
        
        {/* Left column: Branding */}
        <div className="md:col-span-5 flex flex-col gap-4 items-start text-left">
          <div className="flex items-center gap-2">
            <span className="text-xl font-bold tracking-wider text-white">
              ALOK<span className="text-accent-electric font-mono text-xs ml-1">OS_v3</span>
            </span>
          </div>
          <p className="text-xs font-light text-text-secondary max-w-sm leading-relaxed">
            A voice-first, private AI operating system coordinator for Windows desktops. Automate your workspace completely offline.
          </p>
        </div>

        {/* Right column: Links */}
        <div className="md:col-span-7 flex flex-wrap md:justify-end gap-x-16 gap-y-8 text-sm text-left">
          <div className="flex flex-col gap-3">
            <span className="text-xs font-mono tracking-wider text-text-muted uppercase font-semibold">RESOURCES</span>
            <a href="https://github.com/alokkumar2510/alok_jarvis_os" className="text-text-secondary hover:text-accent-electric transition-colors">GitHub Repository</a>
            <a href="https://github.com/alokkumar2510/alok_jarvis_os#readme" className="text-text-secondary hover:text-accent-electric transition-colors">Documentation</a>
          </div>
          
          <div className="flex flex-col gap-3">
            <span className="text-xs font-mono tracking-wider text-text-muted uppercase font-semibold">PRODUCT</span>
            <a href="#features" className="text-text-secondary hover:text-accent-electric transition-colors">Features</a>
            <a href="#demo" className="text-text-secondary hover:text-accent-electric transition-colors">Interactive HUD</a>
            <a href="#download" className="text-text-secondary hover:text-accent-electric transition-colors">Downloads</a>
          </div>

          <div className="flex flex-col gap-3">
            <span className="text-xs font-mono tracking-wider text-text-muted uppercase font-semibold">LEGAL & DOMAIN</span>
            <span className="text-text-secondary">alokkumarsahu.in</span>
            <a href="#" className="text-text-secondary hover:text-accent-electric transition-colors">Privacy Policy</a>
            <a href="#" className="text-text-secondary hover:text-accent-electric transition-colors">Terms of Service</a>
          </div>
        </div>

      </div>

      {/* Copyright telemetry bar */}
      <div className="max-w-7xl mx-auto px-6 mt-12 pt-8 border-t border-glass-border/30 flex flex-col sm:flex-row justify-between items-center text-xs font-mono text-text-muted gap-4">
        <span>© {currentYear} ALOK JARVIS OS. ALL RIGHTS RESERVED.</span>
        <div className="flex items-center gap-2">
          <span className="inline-block w-2 h-2 rounded-full bg-accent-emerald animate-pulse" />
          <span>SERVED FROM CLOUDFLARE EDGE</span>
        </div>
      </div>
    </footer>
  );
}
