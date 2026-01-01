"use client";

import React from "react";
import { motion } from "framer-motion";
import { Layers, ArrowRight } from "lucide-react";

export default function RuntimeDeepDive() {
    return (
        <section className="relative py-20 px-4 flex flex-col items-center overflow-hidden">
            {/* Background Elements */}
            <div className="absolute inset-0 pointer-events-none">
                <div className="absolute top-1/4 left-1/4 w-[500px] h-[500px] bg-rose-pine-iris/5 rounded-full blur-[100px]" />
                <div className="absolute bottom-1/4 right-1/4 w-[400px] h-[400px] bg-rose-pine-love/5 rounded-full blur-[100px]" />
            </div>

            <div className="relative z-10 max-w-7xl w-full">
                <div className="flex flex-col lg:flex-row items-center gap-16 mb-16">
                    {/* Text Content */}
                    <div className="flex-1 space-y-10">
                        <motion.div
                            initial={{ opacity: 0, scale: 0.9 }}
                            whileInView={{ opacity: 1, scale: 1 }}
                            viewport={{ once: true }}
                            className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-rose-pine-iris/10 border border-rose-pine-iris/20"
                        >
                            <Layers className="w-4 h-4 text-rose-pine-iris" />
                            <span className="text-xs font-bold text-rose-pine-iris uppercase tracking-wide">Unstoppable World Logic</span>
                        </motion.div>

                        <h2 className="text-5xl md:text-7xl font-black text-rose-pine-text leading-tight tracking-tight">
                            The 5IVE Engine.<br />
                            <span className="text-rose-pine-subtle">Build Worlds, not just Apps.</span>
                        </h2>
                        <motion.p
                            initial={{ opacity: 0, y: 20 }}
                            whileInView={{ opacity: 1, y: 0 }}
                            viewport={{ once: true }}
                            transition={{ delay: 0.1 }}
                            className="text-xl text-rose-pine-muted font-light leading-relaxed max-w-xl"
                        >
                            Moats allow studios to deploy persistent world logic on-chain.
                            Host complex economies, recursive quest lines, and governance hooks in a single 10MB account.
                            It's a verified, moddable, and unstoppable backend for your IP.
                        </motion.p>

                        <ul className="space-y-6 pt-4">
                            <ListItem>Admin Hooks for IP Control</ListItem>
                            <ListItem> thousands of Bytecode Scripts (NPCs, Quests)</ListItem>
                            <ListItem>Unified State (Items, Reputation, Evolution)</ListItem>
                        </ul>
                    </div>

                    {/* Visual Diagram */}
                    <div className="flex-1 w-full flex justify-center">
                        <div className="relative w-full max-w-[500px] aspect-square">
                            {/* Layer 1 Base */}
                            <div className="absolute inset-x-0 bottom-0 h-1/3 bg-rose-pine-hl-low dark:bg-[#1e1e2e] rounded-b-3xl border border-white/10 flex items-center justify-center translate-y-4 opacity-50 scale-95">
                                <span className="text-rose-pine-subtle font-mono text-sm tracking-widest uppercase">Solana L1 / SVM</span>
                            </div>

                            {/* The Moat */}
                            <motion.div
                                initial={{ y: 20, opacity: 0 }}
                                whileInView={{ y: 0, opacity: 1 }}
                                viewport={{ once: true }}
                                className="absolute inset-0 bg-gradient-to-b from-rose-pine-surface to-rose-pine-base dark:from-[#1f1d2e] dark:to-[#121118] rounded-3xl border border-rose-pine-highlight-med/30 shadow-2xl flex flex-col items-center p-8 backdrop-blur-xl"
                            >
                                <div className="w-full text-center border-b border-white/5 pb-6 mb-8">
                                    <h3 className="text-xl font-bold text-rose-pine-text">5IVE Moat (Layer 1.5)</h3>
                                </div>

                                {/* Apps Inside */}
                                <div className="grid grid-cols-2 gap-4 w-full h-full">
                                    <AppBlock name="Economy Logic" color="bg-rose-pine-love" />
                                    <AppBlock name="Admin Hooks" color="bg-rose-pine-foam" />
                                    <AppBlock name="NPC Scripts" color="bg-rose-pine-gold" />
                                    <AppBlock name="Item State" color="bg-rose-pine-iris" />
                                </div>
                            </motion.div>
                        </div>
                    </div>
                </div>

            </div>
        </section >
    );
}

function ListItem({ children }: { children: React.ReactNode }) {
    return (
        <li className="flex items-center gap-3 text-rose-pine-text">
            <div className="w-6 h-6 rounded-full bg-rose-pine-iris/20 flex items-center justify-center text-rose-pine-iris">
                <ArrowRight size={14} />
            </div>
            {children}
        </li>
    );
}

function AppBlock({ name, color }: { name: string, color: string }) {
    return (
        <div className="relative group overflow-hidden rounded-xl bg-white/60 dark:bg-[#121118]/60 border border-white/5 p-4 flex flex-col items-center justify-center transition-all hover:bg-white/80 dark:hover:bg-[#121118]/80 hover:scale-105 cursor-default">
            <div className={`w-8 h-8 rounded-lg ${color} opacity-80 mb-3 shadow-[0_0_15px_rgba(0,0,0,0.5)]`} />
            <span className="font-medium text-rose-pine-text">{name}</span>
            <div className={`absolute inset-0 ${color} opacity-0 group-hover:opacity-10 transition-opacity`} />
        </div>
    );
}


