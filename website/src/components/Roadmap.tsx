"use client";

import { motion } from "framer-motion";
import { Milestone, GitMerge, Compass, Layers } from "lucide-react";

export default function Roadmap() {
  const phases = [
    {
      badge: "PHASE 1 - AVAILABLE NOW",
      title: "Local Foundations",
      description: "Establishing low-latency local execution blocks to ensure voice conversations and PC automations run offline instantly.",
      items: [
        "Local Whisper STT & Piper TTS sub-processes",
        "Voice-first VAD interruption & emergency stopping",
        "Smart SQLite Memory database and entity relationships",
        "Desktop windowing process controller and CLI automation"
      ],
      icon: <Layers className="w-5 h-5" />,
      color: "border-accent-electric text-accent-electric bg-accent-electric/5"
    },
    {
      badge: "PHASE 2 - UPCOMING Q3 2026",
      title: "Swarm Coordinator",
      description: "Developing cross-agent pipeline execution where multiple sub-agents collaborate autonomously to achieve complex goals.",
      items: [
        "Multi-step planner self-correction algorithms",
        "Habit sweep tracker and proactive reminders",
        "Semantic memory decay (decaying older irrelevant memories)",
        "Browser context scraping and screen OCR context integration"
      ],
      icon: <GitMerge className="w-5 h-5" />,
      color: "border-accent-cyan text-accent-cyan bg-accent-cyan/5"
    },
    {
      badge: "PHASE 3 - FUTURE VISION",
      title: "Autonomous Workspace Companion",
      description: "Integrating deep local context modeling and custom local LLM tuning to create a companion that understands what you are doing before you ask.",
      items: [
        "On-device custom fine-tuned GGUF/exl2 models",
        "Cross-device synchronization (PC, Mobile, Web HUD)",
        "Proactive context awareness via audio/camera feed (Optional)",
        "Self-improvement code generation and runtime optimization"
      ],
      icon: <Compass className="w-5 h-5" />,
      color: "border-accent-violet text-accent-violet bg-accent-violet/5"
    }
  ];

  return (
    <section id="roadmap" className="py-32 relative overflow-hidden bg-bg-secondary/20">
      {/* Background glow overlay */}
      <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[700px] h-[700px] radial-glow-violet pointer-events-none opacity-10" />

      <div className="max-w-7xl mx-auto px-6 flex flex-col gap-16 items-center z-10 relative">
        <div className="flex flex-col items-center gap-4 text-center">
          <div className="text-accent-electric text-xs font-mono tracking-widest uppercase">
            Milestones
          </div>
          <h2 className="text-4xl md:text-5xl font-semibold tracking-tight text-white">
            OS Evolutionary Roadmap
          </h2>
          <p className="text-text-secondary max-w-xl font-light text-base leading-relaxed">
            Discover the roadmap for building the ultimate private and autonomous workspace companion.
          </p>
        </div>

        {/* Timeline Layout */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8 w-full">
          {phases.map((phase, idx) => (
            <motion.div
              key={idx}
              initial={{ opacity: 0, y: 30 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.6, delay: idx * 0.15, ease: [0.16, 1, 0.3, 1] }}
              className="glass-panel rounded-2xl p-8 flex flex-col justify-between border border-glass-border hover:border-glass-border-glow transition-all duration-300 relative text-left"
            >
              <div className="flex flex-col gap-6">
                <div className="flex justify-between items-center">
                  <span className={`px-3 py-1 rounded-full border text-[9px] font-mono tracking-wider ${phase.color}`}>
                    {phase.badge}
                  </span>
                  <div className="text-text-muted p-2 bg-white/2 rounded-lg border border-glass-border">
                    {phase.icon}
                  </div>
                </div>

                <div className="flex flex-col gap-3">
                  <h3 className="text-2xl font-medium text-white">{phase.title}</h3>
                  <p className="text-text-secondary text-sm font-light leading-relaxed">{phase.description}</p>
                </div>

                {/* Sub items */}
                <ul className="flex flex-col gap-3.5 pt-4 border-t border-glass-border/30">
                  {phase.items.map((item, itemIdx) => (
                    <li key={itemIdx} className="flex gap-2.5 items-start text-xs font-light text-text-secondary leading-normal">
                      <span className="w-1.5 h-1.5 rounded-full bg-accent-electric mt-1.5 shrink-0" />
                      <span>{item}</span>
                    </li>
                  ))}
                </ul>
              </div>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}
