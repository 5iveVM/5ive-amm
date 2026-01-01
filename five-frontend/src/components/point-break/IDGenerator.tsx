"use client";

import { useState, useRef, useEffect } from "react";
import { GlassCard } from "@/components/ui/glass-card";
import { Download, RefreshCw, Share2, Copy } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import html2canvas from "html2canvas";

const THEMES = {
    miami: {
        name: "Miami Vice",
        bg: "bg-gradient-to-br from-[#ff00ff] via-[#2a004a] to-[#00ffff]",
        border: "border-[#ff00ff]/30",
        text: "text-white",
        accent: "text-[#00ffff]",
        badge: "bg-[#00ffff]/20 text-[#00ffff] border-[#00ffff]/50",
        overlay: "bg-[linear-gradient(45deg,transparent,rgba(255,0,255,0.1),transparent)]"
    },
    neon: {
        name: "Neon Night",
        bg: "bg-gradient-to-br from-[#0f0c29] via-[#302b63] to-[#24243e]",
        border: "border-[#7b2cbf]/50",
        text: "text-[#e0aaff]",
        accent: "text-[#9d4edd]",
        badge: "bg-[#9d4edd]/20 text-[#e0aaff] border-[#9d4edd]/50",
        overlay: "bg-[radial-gradient(circle_at_center,rgba(123,44,191,0.2),transparent)]"
    },
    core: {
        name: "5ive Core",
        bg: "bg-gradient-to-br from-[#191724] via-[#26233a] to-[#1f1d2e]",
        border: "border-[#ebbcba]/30",
        text: "text-[#e0def4]",
        accent: "text-[#eb6f92]",
        badge: "bg-[#eb6f92]/20 text-[#ebbcba] border-[#eb6f92]/50",
        overlay: "bg-[linear-gradient(to_bottom,transparent,rgba(235,188,186,0.05),transparent)]"
    }
};

const ROLES = ["Early Adopter", "Builder", "Architect", "Hacker", "Visionary"];

export function IDGenerator() {
    const [name, setName] = useState("Anon");
    const [role, setRole] = useState(ROLES[0]);
    const [handle, setHandle] = useState("@username");
    const [theme, setTheme] = useState<keyof typeof THEMES>("miami");
    const [loading, setLoading] = useState(false);
    const cardRef = useRef<HTMLDivElement>(null);

    const activeTheme = THEMES[theme];

    const generateImage = async () => {
        if (!cardRef.current) return;
        setLoading(true);
        try {
            const canvas = await html2canvas(cardRef.current, {
                backgroundColor: null,
                scale: 2, // Higher resolution
                logging: false,
                useCORS: true
            } as any);

            const link = document.createElement('a');
            link.download = `five-point-break-${name.toLowerCase().replace(/\s+/g, '-')}.png`;
            link.href = canvas.toDataURL('image/png');
            link.click();
        } catch (err) {
            console.error("Failed to generate image", err);
        }
        setLoading(false);
    };

    const shareToTwitter = () => {
        const text = `I secured my Virtual ID for #SolanaBreakpoint 2025 with @five_org. \n\nGet yours here: https://five.org/point-break \n\n#5IVE #PointBreak`;
        window.open(`https://twitter.com/intent/tweet?text=${encodeURIComponent(text)}`, '_blank');
    };

    return (
        <div className="flex flex-col lg:flex-row gap-12 items-start w-full max-w-5xl">

            {/* Controls */}
            <GlassCard className="flex-1 w-full p-8 border-[#ff00ff]/20 bg-[#0f0f13]/60 backdrop-blur-2xl">
                <h3 className="text-2xl font-bold mb-6 text-white flex items-center gap-2">
                    <span className="w-2 h-8 bg-[#ff00ff] rounded-sm"></span>
                    Configure Credentials
                </h3>

                <div className="space-y-6">
                    <div>
                        <label className="block text-sm font-medium text-gray-400 mb-2">Code Name</label>
                        <input
                            type="text"
                            value={name}
                            onChange={(e) => setName(e.target.value)}
                            maxLength={16}
                            className="w-full bg-black/40 border border-white/10 rounded-lg px-4 py-3 text-white focus:outline-none focus:border-[#00ffff] focus:ring-1 focus:ring-[#00ffff]/50 transition-all font-mono"
                        />
                    </div>

                    <div>
                        <label className="block text-sm font-medium text-gray-400 mb-2">Comms Handle</label>
                        <input
                            type="text"
                            value={handle}
                            onChange={(e) => setHandle(e.target.value)}
                            className="w-full bg-black/40 border border-white/10 rounded-lg px-4 py-3 text-white focus:outline-none focus:border-[#00ffff] focus:ring-1 focus:ring-[#00ffff]/50 transition-all font-mono"
                        />
                    </div>

                    <div>
                        <label className="block text-sm font-medium text-gray-400 mb-2">Role Clearance</label>
                        <div className="grid grid-cols-2 gap-2">
                            {ROLES.map((r) => (
                                <button
                                    key={r}
                                    onClick={() => setRole(r)}
                                    className={`px-3 py-2 rounded-md text-sm font-medium transition-all text-left ${role === r
                                        ? "bg-white/10 text-white border border-white/20"
                                        : "text-gray-500 hover:text-gray-300 hover:bg-white/5"
                                        }`}
                                >
                                    {r}
                                </button>
                            ))}
                        </div>
                    </div>

                    <div>
                        <label className="block text-sm font-medium text-gray-400 mb-2">System Theme</label>
                        <div className="flex gap-3">
                            {(Object.keys(THEMES) as Array<keyof typeof THEMES>).map((t) => (
                                <button
                                    key={t}
                                    onClick={() => setTheme(t)}
                                    className={`flex-1 py-3 rounded-lg border transition-all text-sm font-bold relative overflow-hidden ${theme === t
                                        ? "border-white text-white shadow-[0_0_15px_-5px_rgba(255,255,255,0.3)]"
                                        : "border-white/10 text-gray-500 hover:border-white/20"
                                        }`}
                                >
                                    <span className="relative z-10">{THEMES[t].name}</span>
                                    {/* Theme preview generic gradient */}
                                    <div className={`absolute inset-0 opacity-20 ${THEMES[t].bg}`} />
                                </button>
                            ))}
                        </div>
                    </div>
                </div>
            </GlassCard>

            {/* Preview & Actions */}
            <div className="flex-1 w-full flex flex-col items-center gap-8">

                {/* ID Card Wrapper for centering/scaling if needed */}
                <div className="relative group perspective-1000">
                    <div
                        ref={cardRef}
                        className={`relative w-[400px] h-[600px] rounded-3xl overflow-hidden shadow-2xl transition-all duration-500 border ${activeTheme.border} ${activeTheme.bg} flex flex-col`}
                    >
                        {/* Overlay Texture */}
                        <div className={`absolute inset-0 ${activeTheme.overlay} pointer-events-none mix-blend-overlay`} />
                        <div className="absolute inset-0 bg-[url('https://grainy-gradients.vercel.app/noise.svg')] opacity-20 pointer-events-none brightness-150 contrast-150" />

                        {/* Header */}
                        <div className="p-6 flex justify-between items-start z-10">
                            <div className="bg-black/20 backdrop-blur-md px-3 py-1 rounded-full border border-white/10">
                                <span className={`text-xs font-bold tracking-wider ${activeTheme.accent}`}>5IVE // SYSTEM</span>
                            </div>
                            <div className="w-12 h-12 rounded-full border border-white/20 bg-white/5 flex items-center justify-center">
                                <span className="text-xl">ID</span>
                            </div>
                        </div>

                        {/* Content */}
                        <div className="flex-1 flex flex-col items-center justify-center text-center p-6 z-10">
                            {/* Avatar Placeholder */}
                            <div className={`w-36 h-36 rounded-2xl mb-6 relative overflow-hidden ring-4 ring-white/10 ${activeTheme.accent} bg-black/30 backdrop-blur-sm flex items-center justify-center`}>
                                <div className="absolute inset-0 bg-gradient-to-tr from-white/10 to-transparent"></div>
                                <span className="text-5xl font-black opacity-50">{name.charAt(0)}</span>
                            </div>

                            <h2 className={`text-4xl font-black tracking-tight mb-2 uppercase break-all ${activeTheme.text} drop-shadow-lg`}>
                                {name}
                            </h2>
                            <p className={`text-lg font-mono opacity-70 mb-6 ${activeTheme.text}`}>
                                {handle}
                            </p>

                            <span className={`px-4 py-2 rounded-lg text-sm font-bold uppercase tracking-widest ${activeTheme.badge}`}>
                                {role}
                            </span>
                        </div>

                        {/* Footer */}
                        <div className="p-6 z-10 relative">
                            <div className="w-full h-[1px] bg-gradient-to-r from-transparent via-white/20 to-transparent mb-4"></div>
                            <div className="flex justify-between items-end">
                                <div className="flex flex-col text-left">
                                    <span className="text-[10px] uppercase tracking-widest opacity-50 text-white">Access Level</span>
                                    <span className="text-sm font-mono text-white">UNRESTRICTED</span>
                                </div>
                                <div className="flex flex-col text-right">
                                    <span className="text-[10px] uppercase tracking-widest opacity-50 text-white">Event</span>
                                    <span className="text-sm font-mono text-white">SOLANA BREAKPOINT '25</span>
                                </div>
                            </div>
                            {/* Glitch bar */}
                            <div className="absolute bottom-0 left-0 right-0 h-1 bg-gradient-to-r from-[#ff00ff] via-[#00ffff] to-[#ff00ff] opacity-50"></div>
                        </div>
                    </div>
                </div>

                {/* Actions */}
                <div className="flex gap-4 w-full max-w-[400px]">
                    <motion.button
                        whileHover={{ scale: 1.02 }}
                        whileTap={{ scale: 0.98 }}
                        onClick={generateImage}
                        disabled={loading}
                        className="flex-1 bg-white text-black font-bold py-4 rounded-xl flex items-center justify-center gap-2 hover:bg-gray-100 transition-colors shadow-[0_0_20px_rgba(255,255,255,0.3)] disabled:opacity-50"
                    >
                        {loading ? <RefreshCw className="animate-spin" /> : <Download size={20} />}
                        Download ID
                    </motion.button>
                    <motion.button
                        whileHover={{ scale: 1.02 }}
                        whileTap={{ scale: 0.98 }}
                        onClick={shareToTwitter}
                        className="flex-1 bg-[#1DA1F2] text-white font-bold py-4 rounded-xl flex items-center justify-center gap-2 hover:bg-[#1a91da] transition-colors shadow-lg"
                    >
                        <Share2 size={20} />
                        Share
                    </motion.button>
                </div>
            </div>
        </div>
    );
}
