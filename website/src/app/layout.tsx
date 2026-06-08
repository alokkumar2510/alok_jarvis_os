import type { Metadata, Viewport } from "next";
import { Outfit, JetBrains_Mono } from "next/font/google";
import "./globals.css";

const outfit = Outfit({
  variable: "--font-sans",
  subsets: ["latin"],
  weight: ["200", "300", "400", "500", "600", "700"],
});

const jetbrainsMono = JetBrains_Mono({
  variable: "--font-mono",
  subsets: ["latin"],
  weight: ["300", "400", "500", "600"],
});

export const metadata: Metadata = {
  title: "ALOK JARVIS OS | Your Personal AI Operating System",
  description: "ALOK is a voice-first AI companion that understands, remembers, automates, and helps you control your entire PC naturally.",
  keywords: [
    "AI Assistant",
    "Personal AI",
    "JARVIS",
    "Windows Assistant",
    "Voice Assistant",
    "Automation",
    "Windows AI Companion",
    "OS Agent"
  ],
  authors: [{ name: "Alok Kumar Sahu", url: "https://alokkumarsahu.in" }],
  openGraph: {
    title: "ALOK JARVIS OS | Your Personal AI Operating System",
    description: "ALOK is a voice-first AI companion that understands, remembers, automates, and helps you control your entire PC naturally.",
    url: "https://jarvis.alokkumarsahu.in",
    siteName: "ALOK JARVIS OS",
    type: "website",
    locale: "en_US",
  },
  twitter: {
    card: "summary_large_image",
    title: "ALOK JARVIS OS | Your Personal AI Operating System",
    description: "ALOK is a voice-first AI companion that understands, remembers, automates, and helps you control your entire PC naturally.",
  },
  icons: {
    icon: "/favicon.ico",
  },
};

export const viewport: Viewport = {
  themeColor: "#0B0F19",
  width: "device-width",
  initialScale: 1,
  maximumScale: 1,
  userScalable: false,
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className={`${outfit.variable} ${jetbrainsMono.variable}`}>
      <body className="antialiased bg-bg-primary text-text-primary">
        {children}
      </body>
    </html>
  );
}
