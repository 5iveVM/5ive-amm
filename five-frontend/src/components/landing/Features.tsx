"use client";

import { cn } from "@/lib/utils";
import { Link2, Box, Zap, Cloud, Check, Shield, RefreshCw } from "lucide-react";
import { motion } from "framer-motion";

export default function Features() {
    return (
        <section id="features" className="relative py-24 px-4 flex flex-col items-center">
            <div className="absolute inset-0 bg-gradient-to-b from-transparent via-rose-pine-iris/10 to-transparent pointer-events-none" />

            <div className="relative z-10 max-w-7xl w-full">
                <motion.div
                    initial={{ opacity: 0, y: 20 }}
                    whileInView={{ opacity: 1, y: 0 }}
                    viewport={{ once: true }}
                    transition={{ duration: 0.6 }}
                    className="text-center mb-16"
                >
                    <h2 className="text-4xl md:text-5xl font-bold text-rose-pine-iris mb-4 drop-shadow-md">The Biggest Shift Since Solana Began.</h2>
                    <p className="text-rose-pine-subtle text-lg max-w-2xl mx-auto">
                        The Wall is coming down. We are building the MOAT that unlocks Mainnet for the entire world.
                    </p>
                </motion.div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-6 relative z-10 w-full">
                    {/* Feature 1: The First Layer 1.5 */}
                    <motion.div
                        whileHover={{ y: -5 }}
                        className="group relative p-8 rounded-2xl border border-[var(--glass-border)] bg-[var(--glass-bg)] backdrop-blur-xl shadow-lg transition-all duration-300 hover:shadow-[0_0_30px_rgba(235,111,146,0.15)] hover:border-l-rose-pine-love hover:border-t-rose-pine-love/50 min-h-[200px] flex flex-col h-full"
                    >
                        <div className="absolute inset-0 rounded-2xl bg-gradient-to-br from-rose-pine-love/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none" />
                        <div className="w-12 h-12 rounded-xl bg-rose-pine-love/10 flex items-center justify-center mb-6 group-hover:bg-rose-pine-love/20 transition-colors">
                            <Link2 className="w-6 h-6 text-rose-pine-love" />
                        </div>
                        <h3 className="text-2xl font-bold text-rose-pine-text mb-2 tracking-tight group-hover:text-rose-pine-love transition-colors">The First Layer 1.5</h3>
                        <p className="text-rose-pine-subtle font-medium mb-4">TEAR DOWN THIS WALL. The cost of Mainnet has killed too many great ideas. 5IVE destroys the barrier, turning $1,000 deployments into $1.</p>

                        <ul className="space-y-3 mt-auto">
                            <li className="flex items-start gap-3 text-rose-pine-subtle text-sm">
                                <div className="mt-0.5 min-w-[18px]"><Shield className="w-4 h-4 text-rose-pine-love/80" /></div>
                                <span>L1 Security. L2 Economics. No Bridges.</span>
                            </li>
                            <li className="flex items-start gap-3 text-rose-pine-subtle text-sm">
                                <div className="mt-0.5 min-w-[18px]"><RefreshCw className="w-4 h-4 text-rose-pine-love/80" /></div>
                                <span>$1 Deployment vs $1,000+ Paywall</span>
                            </li>
                            <li className="flex items-start gap-3 text-rose-pine-subtle text-sm">
                                <div className="mt-0.5 min-w-[18px]"><Check className="w-4 h-4 text-rose-pine-love/80" /></div>
                                <span>Opportunity is Everything. Build on Mainnet.</span>
                            </li>
                        </ul>
                    </motion.div>

                    {/* Feature 2: Follow the Rules. Or Break Them. */}
                    <motion.div
                        whileHover={{ y: -5 }}
                        className="group relative p-8 rounded-2xl border border-[var(--glass-border)] bg-[var(--glass-bg)] backdrop-blur-xl shadow-lg transition-all duration-300 hover:shadow-[0_0_30px_rgba(196,167,231,0.15)] hover:border-l-rose-pine-iris hover:border-t-rose-pine-iris/50 min-h-[200px] flex flex-col h-full"
                    >
                        <div className="absolute inset-0 rounded-2xl bg-gradient-to-br from-rose-pine-iris/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none" />
                        <div className="w-12 h-12 rounded-xl bg-rose-pine-iris/10 flex items-center justify-center mb-6 group-hover:bg-rose-pine-iris/20 transition-colors">
                            <Box className="w-6 h-6 text-rose-pine-iris" />
                        </div>
                        <h3 className="text-2xl font-bold text-rose-pine-text mb-2 tracking-tight group-hover:text-rose-pine-iris transition-colors">Follow the Rules. Or Break Them.</h3>
                        <p className="text-rose-pine-subtle font-medium mb-4">Use the Standard Model for compatibility. Or break the rules with Unified State. 5IVE gives you the best of both worlds.</p>

                        <ul className="space-y-3 mt-auto">
                            <li className="flex items-start gap-3 text-rose-pine-subtle text-sm">
                                <div className="mt-0.5 min-w-[18px]"><Check className="w-4 h-4 text-rose-pine-iris/80" /></div>
                                <span>Follow Rules: Full Solana Compatibility</span>
                            </li>
                            <li className="flex items-start gap-3 text-rose-pine-subtle text-sm">
                                <div className="mt-0.5 min-w-[18px]"><Check className="w-4 h-4 text-rose-pine-iris/80" /></div>
                                <span>Break Rules: Combine Code & State</span>
                            </li>
                            <li className="flex items-start gap-3 text-rose-pine-subtle text-sm">
                                <div className="mt-0.5 min-w-[18px]"><Check className="w-4 h-4 text-rose-pine-iris/80" /></div>
                                <span>Single atomic unit for massive complexity reduction</span>
                            </li>
                        </ul>
                    </motion.div>

                    {/* Feature 3: The Import Revolution */}
                    <motion.div
                        whileHover={{ y: -5 }}
                        className="group relative p-8 rounded-2xl border border-[var(--glass-border)] bg-[var(--glass-bg)] backdrop-blur-xl shadow-lg transition-all duration-300 hover:shadow-[0_0_30px_rgba(246,193,119,0.15)] hover:border-l-rose-pine-gold hover:border-t-rose-pine-gold/50 min-h-[200px] flex flex-col h-full"
                    >
                        <div className="absolute inset-0 rounded-2xl bg-gradient-to-br from-rose-pine-gold/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none" />
                        <div className="w-12 h-12 rounded-xl bg-rose-pine-gold/10 flex items-center justify-center mb-6 group-hover:bg-rose-pine-gold/20 transition-colors">
                            <Zap className="w-6 h-6 text-rose-pine-gold" />
                        </div>
                        <h3 className="text-2xl font-bold text-rose-pine-text mb-2 tracking-tight group-hover:text-rose-pine-gold transition-colors">The Import Revolution</h3>
                        <p className="text-rose-pine-subtle font-medium mb-4">Stop wrestling with CPI boilerplate. 5IVE treats other programs like native libraries.</p>

                        <ul className="space-y-3 mt-auto">
                            <li className="flex items-start gap-3 text-rose-pine-subtle text-sm">
                                <div className="mt-0.5 min-w-[18px]"><Check className="w-4 h-4 text-rose-pine-gold/80" /></div>
                                <span>No manual account meta construction</span>
                            </li>
                            <li className="flex items-start gap-3 text-rose-pine-subtle text-sm">
                                <div className="mt-0.5 min-w-[18px]"><Check className="w-4 h-4 text-rose-pine-gold/80" /></div>
                                <span>No serialization overhead</span>
                            </li>
                            <li className="flex items-start gap-3 text-rose-pine-subtle text-sm">
                                <div className="mt-0.5 min-w-[18px]"><Check className="w-4 h-4 text-rose-pine-gold/80" /></div>
                                <span>Just import and call</span>
                            </li>
                        </ul>
                    </motion.div>

                    {/* Feature 4: The Browser is Your Devkit */}
                    <motion.div
                        whileHover={{ y: -5 }}
                        className="group relative p-8 rounded-2xl border border-[var(--glass-border)] bg-[var(--glass-bg)] backdrop-blur-xl shadow-lg transition-all duration-300 hover:shadow-[0_0_30px_rgba(156,207,216,0.15)] hover:border-l-rose-pine-foam hover:border-t-rose-pine-foam/50 min-h-[200px] flex flex-col h-full"
                    >
                        <div className="absolute inset-0 rounded-2xl bg-gradient-to-br from-rose-pine-foam/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none" />
                        <div className="w-12 h-12 rounded-xl bg-rose-pine-foam/10 flex items-center justify-center mb-6 group-hover:bg-rose-pine-foam/20 transition-colors">
                            <Cloud className="w-6 h-6 text-rose-pine-foam" />
                        </div>
                        <h3 className="text-2xl font-bold text-rose-pine-text mb-2 tracking-tight group-hover:text-rose-pine-foam transition-colors">The Browser is Your Devkit</h3>
                        <p className="text-rose-pine-subtle font-medium mb-4">No terminal to configure. No toolchain to break. Build, Deploy, and Execute from anywhere.</p>

                        <ul className="space-y-3 mt-auto">
                            <li className="flex items-start gap-3 text-rose-pine-subtle text-sm">
                                <div className="mt-0.5 min-w-[18px]"><Check className="w-4 h-4 text-rose-pine-foam/80" /></div>
                                <span>Client-side WASM Compilation</span>
                            </li>
                            <li className="flex items-start gap-3 text-rose-pine-subtle text-sm">
                                <div className="mt-0.5 min-w-[18px]"><Check className="w-4 h-4 text-rose-pine-foam/80" /></div>
                                <span>Zero local dependencies</span>
                            </li>
                            <li className="flex items-start gap-3 text-rose-pine-subtle text-sm">
                                <div className="mt-0.5 min-w-[18px]"><Check className="w-4 h-4 text-rose-pine-foam/80" /></div>
                                <span>Compatible with any device</span>
                            </li>
                        </ul>
                    </motion.div>
                </div>
            </div>
        </section>
    );
}
