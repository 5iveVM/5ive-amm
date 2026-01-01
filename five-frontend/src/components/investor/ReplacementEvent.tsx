"use client";

import React from "react";
import { motion } from "framer-motion";
import { AlertTriangle, ArrowRight, Zap, ShieldAlert, Skull, Database, Server } from "lucide-react";
import { useSolPrice } from "@/hooks/useSolPrice";

export default function ReplacementEvent() {
    const { price } = useSolPrice();
    // 10 SOL is a realistic estimate for a full program deployment (Hello World ~1.26 SOL, SPL ~9 SOL)
    const legacyCost = price ? (10.0 * price).toFixed(2) : "2000.00";

    return (
        <section className="relative py-20 px-4 flex flex-col items-center overflow-hidden">
            {/* Background Elements */}
            <div className="absolute inset-0 pointer-events-none">
                <div className="absolute top-1/4 left-1/4 w-[500px] h-[500px] bg-rose-pine-iris/5 rounded-full blur-[100px]" />
                <div className="absolute bottom-1/4 right-1/4 w-[400px] h-[400px] bg-rose-pine-love/5 rounded-full blur-[100px]" />
            </div>

            <div className="relative z-10 max-w-7xl w-full">

                {/* Header */}
                <div className="text-center mb-16 max-w-4xl mx-auto">
                    <motion.div
                        initial={{ opacity: 0, scale: 0.9 }}
                        whileInView={{ opacity: 1, scale: 1 }}
                        viewport={{ once: true }}
                        className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-rose-pine-love/10 border border-rose-pine-love/20 mb-6"
                    >
                        <AlertTriangle className="w-3 h-3 text-rose-pine-love" />
                        <span className="text-xs font-bold text-rose-pine-love tracking-widest uppercase">The Replacement Event</span>
                    </motion.div>

                    <motion.h2
                        initial={{ opacity: 0, y: 20 }}
                        whileInView={{ opacity: 1, y: 0 }}
                        viewport={{ once: true }}
                        className="text-5xl md:text-7xl font-black text-rose-pine-text mb-8 tracking-tight leading-tight"
                    >
                        Adapt or <span className="text-rose-pine-love">Die.</span>
                    </motion.h2>

                    <motion.p
                        initial={{ opacity: 0, y: 20 }}
                        whileInView={{ opacity: 1, y: 0 }}
                        viewport={{ once: true }}
                        transition={{ delay: 0.1 }}
                        className="text-xl md:text-2xl text-rose-pine-muted font-light leading-relaxed"
                    >
                        Existing protocols are paying <span className="text-rose-pine-love font-bold">1000x too much</span> for <span className="text-rose-pine-love font-bold">100x less performance</span>.
                        The inefficiency gap is no longer sustainable. In a matter of weeks, legacy stacks become obsolete.
                    </motion.p>
                </div>

                {/* Comparison Visual */}
                <div className="grid md:grid-cols-2 gap-8 lg:gap-16 items-stretch">

                    {/* Legacy Stack (The Old Way) */}
                    <motion.div
                        initial={{ opacity: 0, x: -20 }}
                        whileInView={{ opacity: 1, x: 0 }}
                        viewport={{ once: true }}
                        className="relative p-10 rounded-3xl border border-rose-pine-love/10 bg-white/80 dark:bg-[#121118]/80 backdrop-blur-xl overflow-hidden group grayscale hover:grayscale-0 transition-all duration-500"
                    >
                        <div className="absolute inset-0 bg-gradient-to-br from-rose-pine-love/5 via-transparent to-transparent opacity-50" />

                        <div className="relative z-10 flex flex-col h-full">
                            <div className="flex items-center gap-3 mb-8">
                                <div className="p-3 rounded-xl bg-rose-pine-love/10 border border-rose-pine-love/20">
                                    <Database className="w-6 h-6 text-rose-pine-love" />
                                </div>
                                <div>
                                    <h3 className="text-2xl font-bold text-rose-pine-text">Legacy Stack</h3>
                                    <p className="text-rose-pine-muted text-sm uppercase tracking-wider">Bloated & Expensive</p>
                                </div>
                            </div>

                            <div className="space-y-6 flex-1">
                                <LegacyItem label="Deployment Cost" value={`$${legacyCost}`} sub="Per Program" />
                                <LegacyItem label="Time to Market" value="6 Months" sub="Rust Boilerplate" />
                                <LegacyItem label="Code Complexity" value="High Friction" sub="Heavy Boilerplate" />
                            </div>

                            <div className="mt-8 pt-6 border-t border-white/5">
                                <div className="flex items-center gap-2 text-rose-pine-love text-sm font-bold animate-pulse">
                                    <Skull className="w-4 h-4" />
                                    <span>Facing Extinction</span>
                                </div>
                            </div>
                        </div>
                    </motion.div>

                    {/* 5IVE Stack (The New Way) */}
                    <motion.div
                        initial={{ opacity: 0, x: 20 }}
                        whileInView={{ opacity: 1, x: 0 }}
                        viewport={{ once: true }}
                        className="relative p-10 rounded-3xl border border-rose-pine-foam/20 bg-white/80 dark:bg-[#121118]/80 backdrop-blur-xl overflow-hidden shadow-[0_0_50px_rgba(49,116,143,0.1)]"
                    >
                        <div className="absolute inset-0 bg-gradient-to-tl from-rose-pine-foam/10 via-transparent to-transparent opacity-50" />

                        <div className="relative z-10 flex flex-col h-full">
                            <div className="flex items-center gap-3 mb-8">
                                <div className="p-3 rounded-xl bg-rose-pine-foam/10 border border-rose-pine-foam/20 shadow-[0_0_15px_rgba(49,116,143,0.3)]">
                                    <Zap className="w-6 h-6 text-rose-pine-foam" />
                                </div>
                                <div>
                                    <h3 className="text-2xl font-bold text-white">5IVE Stack</h3>
                                    <p className="text-rose-pine-foam text-sm uppercase tracking-wider">Lean & Autonomous</p>
                                </div>
                            </div>

                            <div className="space-y-6 flex-1">
                                <FiveItem label="Deployment Cost" value="$0.10" sub="Per Program" />
                                <FiveItem label="Time to Market" value="2 Weeks" sub="DSL + AI Generation" />
                                <FiveItem label="Code Complexity" value="Streamlined" sub="Pure Logic Focus" />
                            </div>

                            <div className="mt-8 pt-6 border-t border-rose-pine-foam/20">
                                <div className="flex items-center gap-2 text-rose-pine-foam text-sm font-bold">
                                    <Server className="w-4 h-4" />
                                    <span>The New Standard</span>
                                </div>
                            </div>
                        </div>
                    </motion.div>

                </div>
            </div>
        </section>
    )
}

function LegacyItem({ label, value, sub }: { label: string, value: string, sub: string }) {
    return (
        <div className="flex items-center justify-between group">
            <span className="text-rose-pine-muted font-light">{label}</span>
            <div className="text-right">
                <div className="text-xl font-bold text-rose-pine-love group-hover:scale-105 transition-transform">{value}</div>
                <div className="text-xs text-rose-pine-subtle opacity-60">{sub}</div>
            </div>
        </div>
    )
}

function FiveItem({ label, value, sub }: { label: string, value: string, sub: string }) {
    return (
        <div className="flex items-center justify-between group">
            <span className="text-rose-pine-text font-medium">{label}</span>
            <div className="text-right">
                <div className="text-xl font-bold text-rose-pine-foam bg-rose-pine-foam/10 px-2 py-0.5 rounded-md border border-rose-pine-foam/20 group-hover:scale-105 transition-transform inline-block shadow-[0_0_10px_rgba(49,116,143,0.2)]">{value}</div>
                <div className="text-xs text-rose-pine-foam opacity-80 mt-1">{sub}</div>
            </div>
        </div>
    )
}
