"use client";

import { ThemeToggle } from "@/components/ui/ThemeToggle";
import { NeonText } from "@/components/point-break/NeonText";
import { IDGenerator } from "@/components/point-break/IDGenerator";
import { motion } from "framer-motion";

export default function PointBreakPage() {
    return (
        <div className="min-h-screen bg-[#0f0f13] text-white font-sans selection:bg-[#ff00ff]/30 overflow-x-hidden">

            {/* Background Gradients/Glows */}
            <div className="fixed inset-0 z-0 pointer-events-none">
                <div className="absolute top-0 left-0 w-full h-[600px] bg-gradient-to-b from-[#ff00ff]/10 via-[#00ffff]/5 to-transparent blur-[120px]" />
                <div className="absolute bottom-0 right-0 w-[800px] h-[800px] bg-[#00ffff]/10 rounded-full blur-[150px]" />
                {/* Grid Overlay */}
                <div className="absolute inset-0 bg-[linear-gradient(to_right,#80808012_1px,transparent_1px),linear-gradient(to_bottom,#80808012_1px,transparent_1px)] bg-[size:40px_40px] opacity-20" />
            </div>

            {/* Header */}
            <header className="fixed top-0 left-0 right-0 h-16 border-b border-white/5 bg-[#0f0f13]/80 backdrop-blur-xl flex items-center justify-between px-6 z-50">
                <a href="/" className="font-black text-xl tracking-tighter bg-gradient-to-b from-white via-[#c4a7e7] to-[#eb6f92] bg-clip-text text-transparent hover:opacity-80 transition-opacity">
                    5IVE
                </a>
                <div className="flex items-center gap-4">
                    {/* Theme toggle could go here but forcing dark mode for this page aesthetics usually works better */}
                </div>
            </header>

            <main className="relative z-10 pt-32 pb-20 px-4 flex flex-col items-center min-h-screen">

                {/* Hero Section */}
                <motion.div
                    initial={{ opacity: 0, y: 20 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ duration: 0.8, ease: "circOut" }}
                    className="text-center mb-16 relative"
                >
                    <div className="mb-4">
                        <span className="inline-block px-3 py-1 bg-[#ff00ff]/20 border border-[#ff00ff]/50 rounded-full text-[#ff00ff] text-xs font-bold tracking-widest uppercase shadow-[0_0_15px_-3px_rgba(255,0,255,0.6)]">
                            Solana Breakpoint 2025
                        </span>
                    </div>
                    <NeonText text="POINT BREAK" />
                    <p className="mt-6 text-xl text-gray-400 max-w-lg mx-auto leading-relaxed">
                        Secure your <span className="text-[#00ffff] font-medium drop-shadow-[0_0_8px_rgba(0,255,255,0.4)]">Virtual ID</span>. Access the mainframe.
                    </p>
                </motion.div>

                {/* ID Generator */}
                <IDGenerator />

            </main>
        </div>
    );
}
