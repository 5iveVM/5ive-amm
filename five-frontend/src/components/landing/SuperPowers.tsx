"use client";

import { motion } from "framer-motion";
import { Share2, Box, Database, Code, CheckCircle2 } from "lucide-react";

export default function SuperPowers() {
    return (
        <section className="relative py-32 px-4 overflow-hidden bg-[#191724]">
            {/* Background Elements */}
            <div className="absolute inset-0 pointer-events-none">
                <div className="absolute top-1/2 right-0 -translate-y-1/2 w-[800px] h-[800px] bg-rose-pine-love/5 rounded-full blur-[150px]" />
            </div>

            <div className="relative z-10 max-w-7xl mx-auto space-y-40">
                <div className="grid lg:grid-cols-2 gap-16 items-center">

                    {/* Text Context */}
                    <div className="order-1 lg:order-1">
                        <motion.div
                            initial={{ opacity: 0, x: -20 }}
                            whileInView={{ opacity: 1, x: 0 }}
                            viewport={{ once: true }}
                            transition={{ duration: 0.6 }}
                        >
                            <div className="flex items-center gap-3 mb-6">
                                <div className="p-2 rounded-lg bg-rose-pine-love/10 border border-rose-pine-love/20 text-rose-pine-love">
                                    <Box size={24} />
                                </div>
                                <h2 className="text-3xl md:text-5xl font-black text-rose-pine-text tracking-tight">
                                    Follow the Rules. <br /> <span className="text-transparent bg-clip-text bg-gradient-to-r from-rose-pine-love to-rose-pine-iris">Or Break Them.</span>
                                </h2>
                            </div>

                            <p className="text-xl text-rose-pine-muted font-light leading-relaxed mb-8">
                                Use the Standard Model for compatibility. Or break the rules with Unified State.
                                <span className="block mt-2 text-rose-pine-text font-medium">5IVE gives you the best of both worlds.</span>
                            </p>

                            <ul className="space-y-4 mb-10">
                                <li className="flex items-start gap-3 text-rose-pine-subtle">
                                    <CheckCircle2 className="mt-1 text-rose-pine-foam shrink-0" size={20} />
                                    <span><b>Follow Rules</b>: Full Solana Compatibility</span>
                                </li>
                                <li className="flex items-start gap-3 text-rose-pine-subtle">
                                    <CheckCircle2 className="mt-1 text-rose-pine-foam shrink-0" size={20} />
                                    <span><b>Break Rules</b>: Combine Code & State</span>
                                </li>
                                <li className="flex items-start gap-3 text-rose-pine-subtle">
                                    <CheckCircle2 className="mt-1 text-rose-pine-foam shrink-0" size={20} />
                                    <span>Single atomic unit for massive complexity reduction</span>
                                </li>
                            </ul>
                        </motion.div>
                    </div>

                    {/* Comparison Visual */}
                    <div className="order-2 lg:order-2 relative">
                        {/* Background Splashes */}
                        <div className="absolute -inset-10 bg-gradient-to-tr from-rose-pine-love/20 to-transparent blur-3xl opacity-50" />

                        <motion.div
                            initial={{ opacity: 0, y: 20 }}
                            whileInView={{ opacity: 1, y: 0 }}
                            viewport={{ once: true }}
                            transition={{ duration: 0.8 }}
                            className="rounded-3xl border border-rose-pine-hl-low/20 bg-[#1f1d2e] overflow-hidden shadow-2xl shadow-rose-pine-iris/10"
                        >
                            {/* Header */}
                            <div className="flex border-b border-rose-pine-hl-low/10">
                                <div className="flex-1 p-4 bg-[#232136] text-center text-xs font-mono uppercase tracking-widest text-rose-pine-subtle border-r border-rose-pine-hl-low/10 opacity-70">
                                    Follow Rules
                                </div>
                                <div className="flex-1 p-4 bg-[#2a273f] text-center text-xs font-bold font-mono uppercase tracking-widest text-rose-pine-love">
                                    Break Rules
                                </div>
                            </div>

                            {/* Body */}
                            <div className="grid grid-cols-2 divide-x divide-rose-pine-hl-low/10 h-[300px]">

                                {/* Left: Standard (Separated) */}
                                <div className="relative p-6 bg-[#1f1d2e] flex flex-col items-center justify-center gap-6 opacity-60">
                                    {/* Program */}
                                    <div className="flex flex-col items-center gap-2">
                                        <div className="w-12 h-12 rounded-lg bg-[#232136] border border-dashed border-rose-pine-text/20 flex items-center justify-center">
                                            <Code size={20} className="text-rose-pine-subtle" />
                                        </div>
                                        <span className="text-[10px] uppercase font-mono text-rose-pine-subtle">Program</span>
                                    </div>

                                    {/* Link */}
                                    <div className="h-8 w-[1px] border-l border-dashed border-rose-pine-subtle/20" />

                                    {/* State */}
                                    <div className="flex flex-col items-center gap-2">
                                        <div className="w-12 h-12 rounded-lg bg-[#232136] border border-dashed border-rose-pine-text/20 flex items-center justify-center">
                                            <Database size={20} className="text-rose-pine-subtle" />
                                        </div>
                                        <span className="text-[10px] uppercase font-mono text-rose-pine-subtle">State</span>
                                    </div>
                                </div>

                                {/* Right: Unified (Fused) */}
                                <div className="relative p-6 bg-[#2a273f]/50 flex flex-col items-center justify-center">
                                    <div className="absolute top-0 right-0 p-2">
                                        <div className="flex gap-1.5">
                                            <div className="w-2 h-2 rounded-full bg-rose-pine-love animate-pulse" />
                                        </div>
                                    </div>

                                    {/* The Atom */}
                                    <div className="relative">
                                        <div className="absolute inset-0 bg-rose-pine-love/20 blur-xl rounded-full animate-pulse" />

                                        <div className="relative w-24 h-24 rounded-full bg-gradient-to-br from-rose-pine-love to-rose-pine-iris flex items-center justify-center shadow-lg border border-white/10">
                                            <div className="flex flex-col items-center">
                                                <div className="flex gap-1 mb-1">
                                                    <Code size={14} className="text-white" />
                                                    <Database size={14} className="text-white" />
                                                </div>
                                                <span className="text-[10px] font-black text-white uppercase tracking-widest">Atom</span>
                                            </div>

                                            {/* Orbitals */}
                                            <div className="absolute inset-0 rounded-full border border-white/20 w-[120%] h-[120%] -left-[10%] -top-[10%] animate-[spin_10s_linear_infinite]" />
                                            <div className="absolute inset-0 rounded-full border border-white/10 w-[150%] h-[150%] -left-[25%] -top-[25%] animate-[spin_15s_linear_infinite_reverse]" />
                                        </div>
                                    </div>

                                    <span className="mt-8 text-xs font-bold text-rose-pine-text uppercase tracking-widest">
                                        Rules Broken
                                    </span>
                                </div>

                            </div>
                        </motion.div>
                    </div>
                </div>
            </div>
        </section>
    );
}
