"use client";

import { motion } from "framer-motion";
import { Mic, Cpu, Database, Eye, CheckCircle2, ShieldAlert } from "lucide-react";

interface FeatureCardProps {
  icon: React.ReactNode;
  title: string;
  description: string;
  index: number;
}

function FeatureCard({ icon, title, description, index }: FeatureCardProps) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 30 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: "-100px" }}
      transition={{ duration: 0.6, delay: index * 0.1, ease: [0.16, 1, 0.3, 1] }}
      className="glass-panel rounded-2xl p-8 flex flex-col gap-5 hover:translate-y-[-4px] transition-all duration-300 relative group overflow-hidden"
    >
      {/* Background card highlights */}
      <div className="absolute top-0 left-0 w-full h-[2px] bg-gradient-to-r from-accent-electric via-accent-cyan to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
      <div className="absolute -right-8 -bottom-8 w-24 h-24 bg-accent-electric/5 rounded-full blur-xl group-hover:bg-accent-cyan/10 transition-colors duration-300" />

      <div className="p-3 rounded-xl bg-white/3 border border-glass-border w-fit text-accent-electric group-hover:text-accent-cyan transition-colors">
        {icon}
      </div>

      <div className="flex flex-col gap-2.5">
        <h3 className="text-xl font-medium text-white">{title}</h3>
        <p className="text-text-secondary text-sm font-light leading-relaxed">{description}</p>
      </div>
    </motion.div>
  );
}

export default function Features() {
  const featureList = [
    {
      icon: <Mic className="w-5 h-5" />,
      title: "Voice First",
      description: "Talk naturally with ALOK. The voice-first VAD (Voice Activity Detection) system pauses tasks and redirects conversations instantly on wake-word interrupts.",
    },
    {
      icon: <Cpu className="w-5 h-5" />,
      title: "Universal PC Control",
      description: "Open applications, manage windows, control sound levels, inspect battery parameters, and automate complex workflows using natural speech.",
    },
    {
      icon: <Database className="w-5 h-5" />,
      title: "Memory Engine",
      description: "ALOK remembers conversations, preferences, projects, and goals. It uses local semantic vector memory to build personalized user graphs.",
    },
    {
      icon: <CheckCircle2 className="w-5 h-5" />,
      title: "Agent Intelligence",
      description: "Give high-level goals instead of simple commands. The system splits complex goals into sequential stages, self-corrects on failure, and learns habits.",
    },
    {
      icon: <Eye className="w-5 h-5" />,
      title: "Always Available",
      description: "Continuous wake word activation. A transparent, custom-rendered overlay HUD sits on top of your OS, listening when you say 'Hey Alok'.",
    },
    {
      icon: <ShieldAlert className="w-5 h-5" />,
      title: "Privacy First",
      description: "Everything runs locally. Your SQLite database, Piper voice synthesis engine, and local Whisper speech recognition operate 100% on-device.",
    },
  ];

  return (
    <section id="features" className="py-32 relative overflow-hidden">
      {/* Background soft glow */}
      <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] radial-glow-cyan pointer-events-none opacity-20" />

      <div className="max-w-7xl mx-auto px-6 flex flex-col gap-16 text-center z-10 relative">
        <div className="flex flex-col items-center gap-4">
          <motion.div
            initial={{ opacity: 0, y: 15 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="text-accent-electric text-xs font-mono tracking-widest uppercase"
          >
            Capabilities
          </motion.div>
          <motion.h2
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6 }}
            className="text-4xl md:text-5xl font-semibold tracking-tight text-white"
          >
            Built for total desktop autonomy.
          </motion.h2>
          <motion.p
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6, delay: 0.1 }}
            className="text-text-secondary max-w-xl font-light text-base leading-relaxed"
          >
            ALOK integrates closely with Windows subsystems, using advanced local voice engines and AI agents to act as your desktop coordinator.
          </motion.p>
        </div>

        {/* Feature Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
          {featureList.map((f, i) => (
            <FeatureCard
              key={i}
              icon={f.icon}
              title={f.title}
              description={f.description}
              index={i}
            />
          ))}
        </div>
      </div>
    </section>
  );
}
