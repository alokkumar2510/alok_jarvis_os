"use client";

import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Terminal, Send, Play, CheckCircle, RotateCcw, AlertTriangle } from "lucide-react";

interface SimulationStep {
  title: string;
  status: "completed" | "running" | "pending" | "failed";
  duration?: string;
}

interface DemoScenario {
  trigger: string;
  userMessage: string;
  assistantReply: string;
  plannerSteps: SimulationStep[];
  diagnostics: {
    ram: string;
    stt: string;
    action: string;
    groq: string;
  };
}

export default function Demo() {
  const scenarios: DemoScenario[] = [
    {
      trigger: "Open VS Code",
      userMessage: "Hey Alok, open VS Code and start the flutter project.",
      assistantReply: "Launching Visual Studio Code and starting your flutter workspace.",
      plannerSteps: [
        { title: "Identify environment targets", status: "completed", duration: "12ms" },
        { title: "Spawn process: vscode.exe", status: "completed", duration: "84ms" },
        { title: "Resolve project folder: 'flutter_app_v2'", status: "completed", duration: "45ms" },
        { title: "Run background task: 'flutter pub get'", status: "completed", duration: "1.2s" },
        { title: "Verify workspace compilation status", status: "completed", duration: "320ms" }
      ],
      diagnostics: {
        ram: "84.2 MB",
        stt: "140 ms",
        action: "250 ms",
        groq: "680 ms"
      }
    },
    {
      trigger: "Clean Downloads",
      userMessage: "Clean up my Downloads folder and organize the duplicate files.",
      assistantReply: "I've scanned your Downloads folder. Found 12 duplicate files and archived them to a backup folder.",
      plannerSteps: [
        { title: "Scan folder: C:\\Users\\User\\Downloads", status: "completed", duration: "64ms" },
        { title: "Identify duplicates via SHA-256 hash checks", status: "completed", duration: "112ms" },
        { title: "Create backup archive: Duplicates_Archive", status: "completed", duration: "10ms" },
        { title: "Transaction: Move duplicate files", status: "completed", duration: "25ms" },
        { title: "Generate folder sorting summary report", status: "completed", duration: "8ms" }
      ],
      diagnostics: {
        ram: "72.4 MB",
        stt: "125 ms",
        action: "320 ms",
        groq: "740 ms"
      }
    },
    {
      trigger: "Research GUI",
      userMessage: "Research Rust GUI frameworks and summarize the best options.",
      assistantReply: "Collected sources. The top frameworks are Slint, iced, and egui. Summarized features and comparison report below.",
      plannerSteps: [
        { title: "Query Groq API: Research Rust GUI frameworks", status: "completed", duration: "720ms" },
        { title: "Evaluate iced vs egui features", status: "completed", duration: "180ms" },
        { title: "Generate markdown research report", status: "completed", duration: "35ms" },
        { title: "Format response for voice delivery", status: "completed", duration: "12ms" }
      ],
      diagnostics: {
        ram: "112.5 MB",
        stt: "185 ms",
        action: "145 ms",
        groq: "980 ms"
      }
    }
  ];

  const [activeIdx, setActiveIdx] = useState(0);
  const [simulationState, setSimulationState] = useState<"idle" | "typing_user" | "thinking" | "speaking" | "finished">("idle");
  const [typedUser, setTypedUser] = useState("");
  const [revealedSteps, setRevealedSteps] = useState<number>(0);
  const [showReply, setShowReply] = useState(false);

  const activeScenario = scenarios[activeIdx];

  const startSimulation = (idx: number) => {
    setActiveIdx(idx);
    setSimulationState("typing_user");
    setTypedUser("");
    setRevealedSteps(0);
    setShowReply(false);
  };

  // Run simulation sequence
  useEffect(() => {
    if (simulationState === "typing_user") {
      let charIdx = 0;
      const text = activeScenario.userMessage;
      const interval = setInterval(() => {
        setTypedUser((prev) => prev + text.charAt(charIdx));
        charIdx++;
        if (charIdx >= text.length) {
          clearInterval(interval);
          setTimeout(() => {
            setSimulationState("thinking");
          }, 600);
        }
      }, 35);
      return () => clearInterval(interval);
    }

    if (simulationState === "thinking") {
      const timer = setTimeout(() => {
        setSimulationState("speaking");
      }, 1500);
      return () => clearTimeout(timer);
    }

    if (simulationState === "speaking") {
      setShowReply(true);
      let stepCount = 0;
      const stepsLength = activeScenario.plannerSteps.length;
      
      const interval = setInterval(() => {
        setRevealedSteps((prev) => prev + 1);
        stepCount++;
        if (stepCount >= stepsLength) {
          clearInterval(interval);
          setSimulationState("finished");
        }
      }, 400);
      return () => clearInterval(interval);
    }
  }, [simulationState, activeIdx]);

  return (
    <section id="demo" className="py-32 relative overflow-hidden bg-bg-secondary/40">
      {/* Background soft glow */}
      <div className="absolute top-1/4 right-0 w-[500px] h-[500px] radial-glow-violet pointer-events-none opacity-20" />
      <div className="absolute bottom-1/4 left-0 w-[500px] h-[500px] radial-glow-cyan pointer-events-none opacity-10" />

      <div className="max-w-7xl mx-auto px-6 flex flex-col gap-16 items-center z-10 relative">
        <div className="flex flex-col items-center gap-4 text-center">
          <div className="text-accent-electric text-xs font-mono tracking-widest uppercase">
            Interactive HUD
          </div>
          <h2 className="text-4xl md:text-5xl font-semibold tracking-tight text-white">
            See ALOK in Action
          </h2>
          <p className="text-text-secondary max-w-xl font-light text-base leading-relaxed">
            Select a scenario to trigger the autonomous planner, execution steps, and real-time diagnostics monitoring.
          </p>
        </div>

        {/* Tab Buttons */}
        <div className="flex bg-black/40 border border-glass-border p-1 rounded-xl gap-2">
          {scenarios.map((s, idx) => (
            <button
              key={idx}
              onClick={() => startSimulation(idx)}
              disabled={simulationState !== "idle" && simulationState !== "finished" && activeIdx === idx}
              className={`px-5 py-2.5 rounded-lg text-sm font-medium tracking-wide transition-all duration-300 ${
                activeIdx === idx
                  ? "bg-white/8 text-accent-electric border border-glass-border-glow shadow-glow-cyan/10"
                  : "text-text-secondary hover:text-white"
              }`}
            >
              {s.trigger}
            </button>
          ))}
        </div>

        {/* HUD UI Simulator */}
        <div className="w-full max-w-4xl glass-panel rounded-2xl border border-glass-border shadow-hud flex flex-col overflow-hidden">
          {/* Header Bar */}
          <div className="px-6 py-4 border-b border-glass-border flex justify-between items-center bg-black/20">
            <div className="flex items-center gap-2.5">
              <Terminal className="w-4 h-4 text-accent-electric" />
              <span className="text-sm font-mono text-white">jarvis_os_hud.exe</span>
            </div>
            <div className="flex gap-1.5">
              <div className="w-3 h-3 rounded-full bg-white/5" />
              <div className="w-3 h-3 rounded-full bg-white/5" />
              <div className="w-3 h-3 rounded-full bg-white/5" />
            </div>
          </div>

          {/* Grid Layout inside HUD */}
          <div className="grid grid-cols-1 md:grid-cols-12 min-h-[440px]">
            {/* Left side: Dialogue timeline */}
            <div className="md:col-span-7 p-6 flex flex-col justify-between border-r border-glass-border">
              <div className="flex flex-col gap-5 h-full overflow-y-auto">
                {/* Dialogue Timeline Stream */}
                {simulationState === "idle" && (
                  <div className="flex flex-col items-center justify-center h-full text-center text-text-muted gap-3 py-10">
                    <Terminal className="w-8 h-8 opacity-40 animate-pulse" />
                    <p className="text-sm font-light">Select a tab or click play to start the dialogue simulation.</p>
                    <button
                      onClick={() => startSimulation(activeIdx)}
                      className="px-4 py-2 border border-glass-border bg-white/3 text-white rounded-lg text-xs font-mono flex items-center gap-2 hover:bg-white/5"
                    >
                      <Play className="w-3.5 h-3.5 fill-white" />
                      <span>Start Simulation</span>
                    </button>
                  </div>
                )}

                {/* User message */}
                {simulationState !== "idle" && (
                  <div className="flex flex-col gap-1.5 items-end">
                    <div className="bg-accent-electric/5 border border-accent-electric/20 text-white rounded-2xl rounded-tr-sm px-4 py-3 max-w-[90%] text-sm font-light">
                      {typedUser}
                      {simulationState === "typing_user" && (
                        <span className="inline-block w-1.5 h-4 bg-accent-electric ml-1 animate-pulse" />
                      )}
                    </div>
                    <span className="text-[10px] font-mono text-text-muted uppercase">User (Alok)</span>
                  </div>
                )}

                {/* Thinking bubble */}
                {simulationState === "thinking" && (
                  <div className="flex flex-col gap-1.5 items-start">
                    <div className="bg-white/2 border border-glass-border text-text-secondary rounded-2xl rounded-tl-sm px-4 py-3 max-w-[80%] flex items-center gap-3">
                      <div className="flex gap-1.5">
                        <div className="w-2 h-2 bg-accent-electric rounded-full animate-bounce [animation-delay:-0.3s]" />
                        <div className="w-2 h-2 bg-accent-electric rounded-full animate-bounce [animation-delay:-0.15s]" />
                        <div className="w-2 h-2 bg-accent-electric rounded-full animate-bounce" />
                      </div>
                      <span className="text-xs font-mono">Thinking...</span>
                    </div>
                  </div>
                )}

                {/* Assistant reply */}
                {showReply && (
                  <div className="flex flex-col gap-1.5 items-start">
                    <motion.div
                      initial={{ opacity: 0, y: 10 }}
                      animate={{ opacity: 1, y: 0 }}
                      className="bg-white/2 border border-glass-border text-white rounded-2xl rounded-tl-sm px-4 py-3 max-w-[90%] text-sm font-light"
                    >
                      {activeScenario.assistantReply}
                    </motion.div>
                    <span className="text-[10px] font-mono text-accent-electric uppercase">ALOK JARVIS OS</span>
                  </div>
                )}
              </div>

              {/* Input simulator box */}
              <div className="mt-4 pt-4 border-t border-glass-border/30 flex items-center gap-3">
                <input
                  type="text"
                  disabled
                  placeholder={simulationState === "idle" ? "Select a tab above..." : "System processing locks input..."}
                  className="w-full bg-black/40 border border-glass-border rounded-xl px-4 py-3 text-xs text-text-muted outline-none"
                />
                <button disabled className="w-10 h-10 bg-white/2 border border-glass-border rounded-xl flex items-center justify-center text-text-muted">
                  <Send className="w-4 h-4" />
                </button>
              </div>
            </div>

            {/* Right side: Planner / Steps & Telemetry */}
            <div className="md:col-span-5 p-6 flex flex-col justify-between bg-black/10">
              <div className="flex flex-col gap-6">
                {/* Active Planner box */}
                <div className="flex flex-col gap-3">
                  <h3 className="text-xs font-mono uppercase tracking-widest text-text-secondary border-b border-glass-border pb-2">
                    Execution Planner
                  </h3>
                  <ul className="flex flex-col gap-2 min-h-[220px]">
                    {simulationState === "idle" && (
                      <div className="flex flex-col justify-center items-center h-full text-center text-text-muted py-12">
                        <span className="text-xs font-mono">PLANNER IDLE</span>
                      </div>
                    )}

                    <AnimatePresence>
                      {simulationState !== "idle" &&
                        activeScenario.plannerSteps.map((step, idx) => {
                          const isRevealed = revealedSteps > idx || simulationState === "finished";
                          if (!isRevealed) return null;

                          return (
                            <motion.li
                              key={idx}
                              initial={{ opacity: 0, x: 20 }}
                              animate={{ opacity: 1, x: 0 }}
                              className="flex justify-between items-center p-3 rounded-lg border border-glass-border/30 bg-white/1 text-xs"
                            >
                              <div className="flex items-center gap-2">
                                <CheckCircle className="w-3.5 h-3.5 text-accent-electric" />
                                <span className="text-white font-light">{step.title}</span>
                              </div>
                              <span className="font-mono text-text-muted text-[10px]">
                                {step.duration || "OK"}
                              </span>
                            </motion.li>
                          );
                        })}
                    </AnimatePresence>
                  </ul>
                </div>

                {/* Diagnostics telemetries */}
                <div className="flex flex-col gap-2.5">
                  <h3 className="text-xs font-mono uppercase tracking-widest text-text-secondary border-b border-glass-border pb-2">
                    Hardware Telemetry
                  </h3>
                  <div className="grid grid-cols-2 gap-3 font-mono text-[11px]">
                    <div className="bg-white/2 border border-glass-border/40 p-2.5 rounded-lg">
                      <span className="block text-text-muted uppercase text-[9px]">Memory Overhead</span>
                      <strong className="text-white">
                        {simulationState !== "idle" ? activeScenario.diagnostics.ram : "-- MB"}
                      </strong>
                    </div>
                    <div className="bg-white/2 border border-glass-border/40 p-2.5 rounded-lg">
                      <span className="block text-text-muted uppercase text-[9px]">Groq Latency</span>
                      <strong className="text-white">
                        {simulationState !== "idle" ? activeScenario.diagnostics.groq : "-- ms"}
                      </strong>
                    </div>
                    <div className="bg-white/2 border border-glass-border/40 p-2.5 rounded-lg">
                      <span className="block text-text-muted uppercase text-[9px]">Whisper STT</span>
                      <strong className="text-white">
                        {simulationState !== "idle" ? activeScenario.diagnostics.stt : "-- ms"}
                      </strong>
                    </div>
                    <div className="bg-white/2 border border-glass-border/40 p-2.5 rounded-lg">
                      <span className="block text-text-muted uppercase text-[9px]">Action Run</span>
                      <strong className="text-white">
                        {simulationState !== "idle" ? activeScenario.diagnostics.action : "-- ms"}
                      </strong>
                    </div>
                  </div>
                </div>
              </div>

              {/* Reset simulation */}
              {simulationState === "finished" && (
                <button
                  onClick={() => startSimulation(activeIdx)}
                  className="mt-4 px-4 py-2 border border-glass-border bg-white/3 hover:bg-white/6 text-accent-electric rounded-xl text-xs font-mono flex items-center gap-2 justify-center w-full transition-all"
                >
                  <RotateCcw className="w-3.5 h-3.5" />
                  <span>Replay Simulation</span>
                </button>
              )}
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
