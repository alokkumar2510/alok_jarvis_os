"use client";

import { useEffect, useRef, useState } from "react";
import { motion } from "framer-motion";
import { Download, Play, Terminal } from "lucide-react";

export default function Hero() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [downloadHovered, setDownloadHovered] = useState(false);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    let animationFrameId: number;
    let width = (canvas.width = 400);
    let height = (canvas.height = 400);

    // Track cursor coordinates relative to canvas center
    let targetMouseX = 0;
    let targetMouseY = 0;
    let currentMouseX = 0;
    let currentMouseY = 0;

    const handleMouseMove = (e: MouseEvent) => {
      const rect = canvas.getBoundingClientRect();
      const x = e.clientX - rect.left - rect.width / 2;
      const y = e.clientY - rect.top - rect.height / 2;
      
      // Limit magnitude of mouse response
      const dist = Math.sqrt(x*x + y*y);
      const limit = 120;
      if (dist > limit) {
        targetMouseX = (x / dist) * limit;
        targetMouseY = (y / dist) * limit;
      } else {
        targetMouseX = x;
        targetMouseY = y;
      }
    };

    const handleMouseLeave = () => {
      targetMouseX = 0;
      targetMouseY = 0;
    };

    window.addEventListener("mousemove", handleMouseMove);
    canvas.addEventListener("mouseleave", handleMouseLeave);

    // Initialize 3D particles
    const particleCount = 180;
    const particles: { x: number; y: number; z: number; baseRadius: number; speed: number; angleOffset: number }[] = [];
    
    for (let i = 0; i < particleCount; i++) {
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(Math.random() * 2 - 1);
      
      // 3D coordinates on a sphere
      const baseRadius = 110 + Math.random() * 20;
      const x = baseRadius * Math.sin(phi) * Math.cos(theta);
      const y = baseRadius * Math.sin(phi) * Math.sin(theta);
      const z = baseRadius * Math.cos(phi);
      
      particles.push({
        x,
        y,
        z,
        baseRadius,
        speed: 0.005 + Math.random() * 0.008,
        angleOffset: Math.random() * Math.PI * 2,
      });
    }

    let time = 0;

    // Render loop (120 FPS targets)
    const render = () => {
      time += 0.015;
      
      // Smooth interpolation for mouse movements
      currentMouseX += (targetMouseX - currentMouseX) * 0.08;
      currentMouseY += (targetMouseY - currentMouseY) * 0.08;

      ctx.clearRect(0, 0, width, height);

      // Radial gradients background inside orb
      const orbGrad = ctx.createRadialGradient(
        width / 2 + currentMouseX * 0.15, 
        height / 2 + currentMouseY * 0.15, 
        10, 
        width / 2, 
        height / 2, 
        150
      );
      orbGrad.addColorStop(0, "rgba(0, 245, 255, 0.12)");
      orbGrad.addColorStop(0.4, "rgba(138, 43, 226, 0.06)");
      orbGrad.addColorStop(1, "rgba(11, 15, 25, 0)");

      ctx.fillStyle = orbGrad;
      ctx.beginPath();
      ctx.arc(width / 2, height / 2, 160, 0, Math.PI * 2);
      ctx.fill();

      // Project and draw 3D particles
      particles.forEach((p) => {
        // Rotate particle mathematically
        const rotX = time * p.speed * 0.5;
        const rotY = time * p.speed;

        // Apply rotation matrices around X and Y axes
        let x1 = p.x * Math.cos(rotY) - p.z * Math.sin(rotY);
        let z1 = p.x * Math.sin(rotY) + p.z * Math.cos(rotY);
        let y2 = p.y * Math.cos(rotX) - z1 * Math.sin(rotX);
        let z2 = p.y * Math.sin(rotX) + z1 * Math.cos(rotX);

        // Apply dynamic wave warping (morphing shape effect)
        const warp = 1.0 + Math.sin(time * 2.5 + p.angleOffset) * 0.08;
        x1 *= warp;
        y2 *= warp;

        // Projections relative to center + cursor tracking inertia
        const screenX = width / 2 + x1 + currentMouseX * 0.35;
        const screenY = height / 2 + y2 + currentMouseY * 0.35;
        const depth = (z2 + 150) / 300; // Normalized 0.0 - 1.0 depth scale

        if (screenX >= 0 && screenX <= width && screenY >= 0 && screenY <= height) {
          // Draw connecting links to nearby particles (simulates neural net)
          particles.forEach((other) => {
            const dx = other.x - p.x;
            const dy = other.y - p.y;
            const dz = other.z - p.z;
            const distSq = dx*dx + dy*dy + dz*dz;
            
            if (distSq < 1600 && Math.random() < 0.08) {
              ctx.strokeStyle = `rgba(0, 245, 255, ${0.03 * depth})`;
              ctx.lineWidth = 0.5;
              ctx.beginPath();
              ctx.moveTo(screenX, screenY);
              
              // Project other particle coordinate
              const ox = width / 2 + (other.x * warp) + currentMouseX * 0.35;
              const oy = height / 2 + (other.y * warp) + currentMouseY * 0.35;
              ctx.lineTo(ox, oy);
              ctx.stroke();
            }
          });

          // Draw the particle
          const size = (1.5 + depth * 2.0) * (p.baseRadius > 125 ? 1.2 : 0.8);
          
          // Color changes based on depth (electric blue in front, purple in back)
          const r = Math.floor(138 - depth * 138);
          const g = Math.floor(43 + depth * 202);
          const b = Math.floor(226 + depth * 29);
          ctx.fillStyle = `rgba(${r}, ${g}, ${b}, ${0.15 + depth * 0.85})`;

          ctx.beginPath();
          ctx.arc(screenX, screenY, size, 0, Math.PI * 2);
          ctx.fill();

          // Highlight glow on closest particles
          if (depth > 0.8) {
            ctx.shadowColor = "#00F5FF";
            ctx.shadowBlur = 10;
            ctx.fillStyle = "rgba(255, 255, 255, 0.9)";
            ctx.beginPath();
            ctx.arc(screenX, screenY, size * 0.8, 0, Math.PI * 2);
            ctx.fill();
            ctx.shadowBlur = 0; // Reset shadow
          }
        }
      });

      // Animated Voice Waveform underneath the Orb
      ctx.strokeStyle = "rgba(0, 245, 255, 0.25)";
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      
      const wavePoints = 40;
      for (let i = 0; i <= wavePoints; i++) {
        const x = (width / wavePoints) * i;
        const normX = (i / wavePoints) * 2 - 1; // -1.0 to 1.0
        
        // Bell curve envelope to pin the ends to zero
        const envelope = Math.exp(-3.0 * normX * normX);
        
        // Complex trigonometric waves
        const yOffset = 30 * Math.sin(time * 5.0 + normX * 4.0) * Math.cos(time * 2.0 + normX * 1.5) * envelope;
        const targetY = height - 40 + yOffset;
        
        if (i === 0) {
          ctx.moveTo(x, targetY);
        } else {
          ctx.lineTo(x, targetY);
        }
      }
      ctx.stroke();

      // Mirror soft secondary wave (violet colored)
      ctx.strokeStyle = "rgba(138, 43, 226, 0.15)";
      ctx.lineWidth = 1.0;
      ctx.beginPath();
      for (let i = 0; i <= wavePoints; i++) {
        const x = (width / wavePoints) * i;
        const normX = (i / wavePoints) * 2 - 1;
        const envelope = Math.exp(-2.5 * normX * normX);
        const yOffset = 18 * Math.sin(time * -4.0 + normX * 5.0) * Math.cos(time * 3.0 + normX * 2.0) * envelope;
        const targetY = height - 40 + yOffset;
        if (i === 0) ctx.moveTo(x, targetY);
        else ctx.lineTo(x, targetY);
      }
      ctx.stroke();

      animationFrameId = requestAnimationFrame(render);
    };

    render();

    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      cancelAnimationFrame(animationFrameId);
    };
  }, []);

  return (
    <section ref={containerRef} className="relative min-h-screen flex items-center justify-center pt-24 overflow-hidden grid-bg">
      {/* Background radial overlays */}
      <div className="absolute top-1/4 left-1/4 -translate-x-1/2 -translate-y-1/2 w-[500px] h-[500px] radial-glow-cyan pointer-events-none" />
      <div className="absolute bottom-1/4 right-1/4 translate-x-1/2 translate-y-1/2 w-[500px] h-[500px] radial-glow-violet pointer-events-none" />

      <div className="max-w-7xl mx-auto px-6 grid grid-cols-1 lg:grid-cols-12 gap-16 items-center z-10 w-full">
        {/* Headline & CTAs */}
        <div className="lg:col-span-7 flex flex-col gap-8 text-left">
          <motion.div
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8, ease: [0.16, 1, 0.3, 1] }}
            className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-glass-border bg-white/3 text-accent-electric text-xs font-mono tracking-wider w-fit"
          >
            <Terminal className="w-3.5 h-3.5" />
            <span>VERSION 0.1.0 IS NOW PUBLIC</span>
          </motion.div>

          <div className="flex flex-col gap-4">
            <motion.h1
              initial={{ opacity: 0, y: 30 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.8, delay: 0.1, ease: [0.16, 1, 0.3, 1] }}
              className="text-5xl md:text-7xl font-semibold tracking-tight text-white leading-[1.05]"
            >
              Your Personal <br />
              <span className="bg-gradient-to-r from-accent-electric via-accent-cyan to-accent-violet bg-clip-text text-transparent">
                AI Operating System
              </span>
            </motion.h1>

            <motion.p
              initial={{ opacity: 0, y: 30 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.8, delay: 0.2, ease: [0.16, 1, 0.3, 1] }}
              className="text-lg md:text-xl text-text-secondary font-light max-w-xl leading-relaxed"
            >
              ALOK is a voice-first AI companion that understands, remembers, automates, and helps you control your entire PC naturally.
            </motion.p>
          </div>

          <motion.div
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8, delay: 0.3, ease: [0.16, 1, 0.3, 1] }}
            className="flex flex-wrap items-center gap-4"
          >
            <a
              href="https://github.com/alokkumar2510/alok_jarvis_os/releases/download/v0.1.0/alok_jarvis_os_0.1.0_x64-setup.exe"
              className="relative px-6 py-3.5 rounded-xl bg-gradient-to-r from-accent-electric to-accent-cyan text-bg-primary font-medium flex items-center gap-2.5 transition-transform duration-200 active:scale-95 group shadow-glow-cyan"
              onMouseEnter={() => setDownloadHovered(true)}
              onMouseLeave={() => setDownloadHovered(false)}
            >
              <Download className="w-5 h-5 group-hover:translate-y-0.5 transition-transform" />
              <span>Download for Windows</span>
            </a>

            <a
              href="#demo"
              className="px-6 py-3.5 rounded-xl border border-glass-border bg-white/2 hover:bg-white/5 text-white font-medium flex items-center gap-2.5 transition-all duration-200"
            >
              <Play className="w-4 h-4 fill-white" />
              <span>Watch Demo</span>
            </a>
          </motion.div>
        </div>

        {/* 3D Orb Canvas Wrapper */}
        <motion.div
          initial={{ opacity: 0, scale: 0.85 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ duration: 1.0, ease: [0.16, 1, 0.3, 1] }}
          className="lg:col-span-5 flex justify-center items-center relative"
        >
          {/* Glass background glowing card ring */}
          <div className="absolute w-[360px] h-[360px] rounded-full border border-glass-border bg-white/2 backdrop-blur-md -z-10 shadow-glow-violet opacity-30 animate-pulse-slow" />
          
          <div className="relative w-[400px] h-[400px]">
            <canvas ref={canvasRef} className="block w-full h-full cursor-pointer" />
          </div>
        </motion.div>
      </div>

      {/* Slide down arrow indicator */}
      <div className="absolute bottom-10 left-1/2 -translate-x-1/2 flex flex-col items-center gap-2 text-text-muted select-none pointer-events-none">
        <span className="text-[10px] font-mono tracking-widest">SCROLL TO DISCOVER</span>
        <div className="w-0.5 h-10 bg-gradient-to-b from-text-muted to-transparent rounded-full" />
      </div>
    </section>
  );
}
