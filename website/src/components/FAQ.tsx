"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { ChevronDown, HelpCircle } from "lucide-react";

interface FAQItemProps {
  question: string;
  answer: string;
  isOpen: boolean;
  onToggle: () => void;
}

function FAQItem({ question, answer, isOpen, onToggle }: FAQItemProps) {
  return (
    <div className="border-b border-glass-border">
      <button
        onClick={onToggle}
        className="w-full py-6 flex justify-between items-center text-left text-white hover:text-accent-electric transition-colors duration-200 group"
      >
        <span className="text-lg font-medium tracking-tight pr-4">{question}</span>
        <ChevronDown
          className={`w-5 h-5 text-text-muted group-hover:text-accent-electric transition-transform duration-300 shrink-0 ${
            isOpen ? "rotate-180" : ""
          }`}
        />
      </button>

      <AnimatePresence initial={false}>
        {isOpen && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.35, ease: [0.16, 1, 0.3, 1] }}
            className="overflow-hidden"
          >
            <p className="pb-6 text-text-secondary text-sm font-light leading-relaxed max-w-3xl">
              {answer}
            </p>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

export default function FAQ() {
  const [openIdx, setOpenIdx] = useState<number | null>(0);

  const faqs = [
    {
      question: "Is ALOK free?",
      answer: "Yes, ALOK is free to download and use. It runs entirely on your local machine using open-source models, so there are no recurring subscription fees."
    },
    {
      question: "Does it work offline?",
      answer: "Yes! The core speech-to-text (Whisper) and text-to-speech (Piper) engines, along with the SQLite semantic memory database, run entirely offline on your local computer. Internet is only required on first launch to automatically download model assets, and when sending reasoning prompts to the Groq API."
    },
    {
      question: "What platforms are supported?",
      answer: "Currently, ALOK is optimized for Windows 10 and 11 (64-bit). Linux and macOS versions are in the evolutionary roadmap, but direct integrations with system controls (winmm audio playback, windows task management, process invocation) are optimized for Windows."
    },
    {
      question: "How does memory work?",
      answer: "ALOK uses a local SQLite semantic database to store conversation contexts, entities (e.g., projects, files, browser urls), and relationships. It uses local embeddings to rank and retrieve memories dynamically, creating a personal knowledge network graph that updates as you talk to the OS."
    },
    {
      question: "Does it require an API key?",
      answer: "Yes, it requires a free Groq API key to perform intent classification, planner step generation, and dialogue reasoning. Your API key is stored locally on your device in the SQLite database settings table and is never uploaded to any external server."
    }
  ];

  return (
    <section id="faq" className="py-32 relative overflow-hidden">
      {/* Background glow overlay */}
      <div className="absolute top-1/2 left-1/4 -translate-x-1/2 -translate-y-1/2 w-[500px] h-[500px] radial-glow-cyan pointer-events-none opacity-10" />

      <div className="max-w-4xl mx-auto px-6 flex flex-col gap-16 items-center z-10 relative">
        <div className="flex flex-col items-center gap-4 text-center">
          <div className="text-accent-electric text-xs font-mono tracking-widest uppercase">
            Questions
          </div>
          <h2 className="text-4xl md:text-5xl font-semibold tracking-tight text-white">
            Frequently Asked Questions
          </h2>
          <p className="text-text-secondary font-light text-base leading-relaxed">
            Quick answers to pricing, offline capabilities, platform support, and local memory security.
          </p>
        </div>

        {/* Accordions List */}
        <div className="w-full glass-panel rounded-2xl p-8 border border-glass-border">
          {faqs.map((faq, idx) => (
            <FAQItem
              key={idx}
              question={faq.question}
              answer={faq.answer}
              isOpen={openIdx === idx}
              onToggle={() => setOpenIdx(openIdx === idx ? null : idx)}
            />
          ))}
        </div>
      </div>
    </section>
  );
}
