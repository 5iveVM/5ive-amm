"use client";

import { motion } from "framer-motion";

export function NeonText({ text }: { text: string }) {
    return (
        <div className="relative inline-block">
            {/* Main Text */}
            <motion.h1
                initial={{ opacity: 0, scale: 0.9 }}
                animate={{ opacity: 1, scale: 1 }}
                transition={{ duration: 1, delay: 0.2 }}
                className="text-7xl md:text-9xl font-black tracking-tighter text-transparent bg-clip-text bg-gradient-to-b from-white to-[#ffb8ff] drop-shadow-[0_0_10px_rgba(255,0,255,0.5)] select-none z-10 relative"
                style={{
                    fontFamily: "'Orbitron', sans-serif", // Or fallback to standard if Orbitron isn't loaded
                }}
            >
                {text}
            </motion.h1>

            {/* Glow/Blur Layer */}
            <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: [0.5, 0.8, 0.5] }}
                transition={{ duration: 3, repeat: Infinity, ease: "easeInOut" }}
                className="absolute inset-0 blur-3xl opacity-50 z-0 select-none"
            >
                <h1 className="text-7xl md:text-9xl font-black tracking-tighter text-[#ff00ff]">
                    {text}
                </h1>
            </motion.div>

            {/* Secondary Glow Layer */}
            <motion.div
                className="absolute inset-0 blur-xl opacity-30 z-0 select-none mix-blend-screen"
            >
                <h1 className="text-7xl md:text-9xl font-black tracking-tighter text-[#00ffff]">
                    {text}
                </h1>
            </motion.div>
        </div>
    );
}
