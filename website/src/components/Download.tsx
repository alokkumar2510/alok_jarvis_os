"use client";

import { useState } from "react";
import { motion } from "framer-motion";
import { Download, Check, Copy, AlertTriangle, FileText, Info } from "lucide-react";

export default function DownloadSection() {
  const [copied, setCopied] = useState(false);
  const checksum = "b07ea0b1b4115a38e1a7b07debf581f0b77d999925f8acb8f39d322b0ba0a822";

  const copyChecksum = () => {
    navigator.clipboard.writeText(checksum);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <section id="download" className="py-32 relative overflow-hidden">
      {/* Background soft glow */}
      <div className="absolute top-1/2 left-1/4 -translate-x-1/2 -translate-y-1/2 w-[500px] h-[500px] radial-glow-cyan pointer-events-none opacity-20" />

      <div className="max-w-7xl mx-auto px-6 grid grid-cols-1 lg:grid-cols-12 gap-16 items-start z-10 relative">
        
        {/* Left Side: CTAs & System Requirements */}
        <div className="lg:col-span-6 flex flex-col gap-10">
          <div className="flex flex-col gap-4 text-left">
            <div className="text-accent-electric text-xs font-mono tracking-widest uppercase">
              Distribution
            </div>
            <h2 className="text-4xl md:text-5xl font-semibold tracking-tight text-white leading-tight">
              Get started with ALOK OS
            </h2>
            <p className="text-text-secondary font-light text-base leading-relaxed">
              Download the native Windows desktop client. The automatic downloader will handle Whisper and Piper settings on first run.
            </p>
          </div>

          <div className="glass-panel rounded-2xl p-8 flex flex-col gap-6">
            {/* Version Telemetry */}
            <div className="grid grid-cols-2 md:grid-cols-4 gap-6 border-b border-glass-border pb-6 text-left">
              <div>
                <span className="block text-[10px] font-mono text-text-muted uppercase">LATEST RELEASE</span>
                <strong className="text-white font-medium text-base">v0.1.0</strong>
              </div>
              <div>
                <span className="block text-[10px] font-mono text-text-muted uppercase">RELEASE DATE</span>
                <strong className="text-white font-medium text-base">June 8, 2026</strong>
              </div>
              <div>
                <span className="block text-[10px] font-mono text-text-muted uppercase">FILE SIZE</span>
                <strong className="text-white font-medium text-base">16.2 MB (EXE)</strong>
              </div>
              <div>
                <span className="block text-[10px] font-mono text-text-muted uppercase">PLATFORM</span>
                <strong className="text-white font-medium text-base">Windows 10/11</strong>
              </div>
            </div>

            {/* Main Buttons */}
            <div className="flex flex-col sm:flex-row gap-4">
              <a
                href="/alok_jarvis_os_0.1.0_x64-setup.exe"
                className="flex-1 py-4 px-6 rounded-xl bg-gradient-to-r from-accent-electric to-accent-cyan text-bg-primary font-medium flex items-center justify-center gap-2.5 active:scale-98 transition-transform shadow-glow-cyan"
              >
                <Download className="w-5 h-5" />
                <span>Download Installer (.exe)</span>
              </a>

              <a
                href="https://github.com/alokkumar2510/alok_jarvis_os/releases/download/v0.1.0/alok_jarvis_os_0.1.0_x64_en-US.msi"
                className="py-4 px-6 rounded-xl border border-glass-border bg-white/2 hover:bg-white/5 text-white font-medium flex items-center justify-center gap-2.5 transition-all"
              >
                <span>Download MSI Bundle</span>
              </a>
            </div>

            {/* Checksum Hash Copy */}
            <div className="flex items-center justify-between bg-black/30 border border-glass-border rounded-xl p-4">
              <div className="flex flex-col text-left gap-1 min-w-0">
                <span className="text-[10px] font-mono text-text-muted uppercase">SHA-256 CHECKSUM</span>
                <code className="text-xs font-mono text-text-secondary truncate pr-4">{checksum}</code>
              </div>
              <button
                onClick={copyChecksum}
                className="p-2.5 rounded-lg border border-glass-border bg-white/2 hover:bg-white/5 text-text-secondary hover:text-white transition-all shrink-0"
                title="Copy SHA-256 Hash"
              >
                {copied ? <Check className="w-4 h-4 text-accent-emerald" /> : <Copy className="w-4 h-4" />}
              </button>
            </div>
          </div>

          {/* System Requirements Warning Panel */}
          <div className="flex gap-4 p-5 rounded-2xl border border-glass-border bg-white/1 text-left text-xs font-light text-text-secondary">
            <Info className="w-5 h-5 text-accent-electric shrink-0" />
            <div className="flex flex-col gap-1.5">
              <strong className="text-white font-medium">System Prerequisites</strong>
              <p className="leading-relaxed">
                Requires a 64-bit edition of Windows 10 or 11. An active internet connection is required on first launch to automatically retrieve and extract the voice models. Direct local execution of Piper TTS and Whisper requires standard CPU instructions.
              </p>
            </div>
          </div>
        </div>

        {/* Right Side: Release Notes */}
        <div className="lg:col-span-6 flex flex-col gap-6 text-left w-full h-full">
          <div className="glass-panel rounded-2xl p-8 flex flex-col gap-6 h-full justify-between">
            <div className="flex flex-col gap-6">
              <div className="flex items-center gap-3 border-b border-glass-border pb-4">
                <FileText className="w-5 h-5 text-accent-electric" />
                <h3 className="text-xl font-medium text-white">Release Notes: v0.1.0</h3>
              </div>
              
              <ul className="flex flex-col gap-4 text-sm font-light text-text-secondary leading-relaxed">
                <li className="flex gap-3">
                  <span className="w-1.5 h-1.5 rounded-full bg-accent-electric mt-2 shrink-0" />
                  <p>
                    <strong className="text-white font-medium">Dynamic Path resolution:</strong> Eliminated developer-specific hardcoded paths. All data, settings, databases, and third-party binaries are virtualized dynamically relative to the local user's AppData directory.
                  </p>
                </li>
                <li className="flex gap-3">
                  <span className="w-1.5 h-1.5 rounded-full bg-accent-electric mt-2 shrink-0" />
                  <p>
                    <strong className="text-white font-medium">Automatic Dependency downloader:</strong> Added automatic first-time downloading of Whisper CLI and Piper TTS engines. The app handles ZIP extraction and downloads models directly from Hugging Face.
                  </p>
                </li>
                <li className="flex gap-3">
                  <span className="w-1.5 h-1.5 rounded-full bg-accent-electric mt-2 shrink-0" />
                  <p>
                    <strong className="text-white font-medium">winmm Speech play purges:</strong> Replaced blocking synchronous sound playback with asynchronous memory buffers, preventing win32 FFI sound play thread corruption and crashes during interruptions.
                  </p>
                </li>
                <li className="flex gap-3">
                  <span className="w-1.5 h-1.5 rounded-full bg-accent-electric mt-2 shrink-0" />
                  <p>
                    <strong className="text-white font-medium">Voice interruption logic:</strong> Enabled higher-priority interruptions. Speech loops stop instantly when the wake-word is detected or emergency commands are spoken.
                  </p>
                </li>
              </ul>
            </div>

            <div className="pt-4 border-t border-glass-border flex justify-between items-center text-xs font-mono text-text-muted">
              <span>CONTRIBUTORS: 1</span>
              <span>BRANCH: MASTER</span>
            </div>
          </div>
        </div>

      </div>
    </section>
  );
}
