import Hero from "@/components/Hero";
import Features from "@/components/Features";
import Demo from "@/components/Demo";
import DownloadSection from "@/components/Download";
import Roadmap from "@/components/Roadmap";
import FAQ from "@/components/FAQ";
import Footer from "@/components/Footer";

export default function Home() {
  return (
    <div className="relative min-h-screen bg-bg-primary overflow-hidden">
      {/* Dynamic Glass Header Navigation Bar */}
      <header className="fixed top-0 left-0 w-full z-50 border-b border-glass-border bg-bg-primary/40 backdrop-blur-md">
        <div className="max-w-7xl mx-auto px-6 h-20 flex justify-between items-center">
          <div className="flex items-center gap-2 select-none">
            <span className="text-xl font-bold tracking-wider text-white">
              ALOK<span className="text-accent-electric font-mono text-xs ml-1">OS_v3</span>
            </span>
          </div>

          {/* Quick Page Anchors Navigation */}
          <nav className="hidden md:flex items-center gap-8 text-sm font-medium tracking-wide">
            <a href="#features" className="text-text-secondary hover:text-white transition-colors">Features</a>
            <a href="#demo" className="text-text-secondary hover:text-white transition-colors">HUD Console</a>
            <a href="#download" className="text-text-secondary hover:text-white transition-colors">Downloads</a>
            <a href="#roadmap" className="text-text-secondary hover:text-white transition-colors">Roadmap</a>
            <a href="#faq" className="text-text-secondary hover:text-white transition-colors">FAQ</a>
          </nav>

          {/* Nav CTA */}
          <div>
            <a
              href="#download"
              className="px-5 py-2.5 rounded-xl border border-glass-border bg-white/3 hover:bg-white/6 text-white text-xs font-semibold uppercase tracking-wider transition-all"
            >
              Get Installer
            </a>
          </div>
        </div>
      </header>

      {/* Landing Page Sections */}
      <main>
        <Hero />
        <Features />
        <Demo />
        <DownloadSection />
        <Roadmap />
        <FAQ />
      </main>

      <Footer />
    </div>
  );
}
