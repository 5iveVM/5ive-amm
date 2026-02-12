"use client";

import { motion } from "framer-motion";
import { Coins, ShieldCheck, Unlink, Hammer } from "lucide-react";

export default function NapkinToMainnet() {
    return (
        <section className="relative py-32 px-4 overflow-hidden bg-transparent">
            {/* Background Atmosphere */}
            <div className="absolute inset-0 pointer-events-none">
                <div className="absolute top-0 left-0 w-[500px] h-[500px] bg-rose-pine-iris/5 rounded-full blur-[120px]" />
            </div>

            <div className="relative z-10 max-w-7xl mx-auto space-y-40">
                <div className="grid lg:grid-cols-2 gap-16 items-center">

                    {/* Visual Comparison (Left) */}
                    <div className="order-2 lg:order-1 relative">
                        <div className="absolute -inset-10 bg-gradient-to-tr from-rose-pine-iris/20 to-transparent blur-3xl opacity-50" />

                        <motion.div
                            initial={{ opacity: 0, y: 20 }}
                            whileInView={{ opacity: 1, y: 0 }}
                            viewport={{ once: true }}
                            transition={{ duration: 0.8 }}
                            className="rounded-3xl border border-rose-pine-hl-low/20 bg-rose-pine-surface overflow-hidden shadow-2xl shadow-rose-pine-iris/10"
                        >
                            {/* Header */}
                            <div className="flex border-b border-rose-pine-hl-low/10">
                                <div className="flex-1 p-4 bg-rose-pine-base text-center text-xs font-mono uppercase tracking-widest text-rose-pine-subtle border-r border-rose-pine-hl-low/10 opacity-70 text-contrast">
                                    The Barrier
                                </div>
                                <div className="flex-1 p-4 bg-rose-pine-surface text-center text-xs font-bold font-mono uppercase tracking-widest text-rose-pine-iris">
                                    The Breakthrough
                                </div>
                            </div>

                            {/* Body - The Wall Crumbling Visual */}
                            <div className="grid grid-cols-2 h-[320px] relative">
                                {/* The Wall (Native Solana) */}
                                <div className="relative p-6 bg-rose-pine-base flex flex-col items-center justify-center border-r border-rose-pine-hl-low/10 group overflow-hidden">
                                     {/* Cracks and Debris Effect */}
                                    <div className="absolute inset-0 bg-[url('/noise.png')] opacity-20" />
                                    <div className="absolute top-0 right-0 w-full h-full border-r-[1px] border-r-rose-pine-love/20 skew-x-12 origin-bottom-right scale-y-110 opacity-0 group-hover:opacity-100 transition-opacity duration-700" />
                                    
                                    <div className="z-10 text-center">
                                        <div className="text-3xl font-bold text-rose-pine-subtle/50 mb-2">Higher complexity</div>
                                        <div className="text-xs font-mono text-rose-pine-love uppercase tracking-widest font-bold">Manual CPI + wiring</div>
                                    </div>

                                    {/* Simulation of crumbling blocks */}
                                    <div className="absolute bottom-0 w-full h-1/3 bg-gradient-to-t from-rose-pine-base to-transparent pointer-events-none" />
                                </div>

                                {/* The Breakthrough (5IVE) */}
                                <div className="relative p-6 bg-rose-pine-surface flex flex-col items-center justify-center overflow-hidden">
                                    <div className="absolute inset-0 bg-rose-pine-iris/5 animate-pulse" />
                                    <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-32 h-32 bg-rose-pine-iris/20 blur-3xl rounded-full" />
                                    
                                    <div className="z-10 text-center">
                                        <div className="text-5xl font-black text-transparent bg-clip-text bg-gradient-to-b from-white to-rose-pine-iris drop-shadow-[0_0_25px_rgba(196,167,231,0.6)]">
                                            Leaner path
                                        </div>
                                        <div className="text-xs font-mono text-rose-pine-iris uppercase tracking-widest font-bold mt-2">External calls + focused bytecode</div>
                                    </div>
                                </div>
                                
                                {/* Center "Break" Icon */}
                                <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 bg-rose-pine-surface border border-rose-pine-hl-low/20 p-2 rounded-full shadow-lg z-20 text-rose-pine-love">
                                    <Hammer size={20} className="transform -scale-x-100" />
                                </div>
                            </div>

                        </motion.div>
                    </div>

                    {/* Text Context (Right) */}
                    <div className="order-1 lg:order-2">
                        <motion.div
                            initial={{ opacity: 0, x: 20 }}
                            whileInView={{ opacity: 1, x: 0 }}
                            viewport={{ once: true }}
                            transition={{ duration: 0.6 }}
                        >
                            <div className="flex items-center gap-3 mb-6">
                                <div className="p-2 rounded-lg bg-rose-pine-iris/10 border border-rose-pine-iris/20 text-rose-pine-iris">
                                    <Coins size={24} />
                                </div>
                                <h2 className="text-3xl md:text-5xl font-black text-rose-pine-text tracking-tight">
                                    Tear Down Mainnet Friction <br /> <span className="text-transparent bg-clip-text bg-gradient-to-r from-rose-pine-iris to-rose-pine-foam">Open Solana to the Masses</span>
                                </h2>
                            </div>

                            <div className="mb-8">
                                <p className="text-xl text-rose-pine-subtle leading-relaxed mb-4 text-contrast">
                                    Five is built to make deployable contracts fast and accessible, with a deployment profile aimed at everyday builders, not just protocol teams.
                                </p>
                                <p className="text-lg font-medium text-rose-pine-text italic border-l-4 border-rose-pine-iris pl-4 py-2 bg-rose-pine-iris/5 rounded-r-lg">
                                    External bytecode calls stay non-CPI inside Five VM. Interfaces remain available for true CPI paths.
                                </p>
                            </div>

                            <ul className="space-y-4 mb-10 text-contrast">
                                <li className="flex items-start gap-3 text-rose-pine-subtle">
                                    <ShieldCheck className="mt-1 text-rose-pine-foam shrink-0" size={20} />
                                    <span><b>Inherit Solana security</b> - no bridge trust assumptions added by Five itself</span>
                                </li>
                                <li className="flex items-start gap-3 text-rose-pine-subtle">
                                    <Hammer className="mt-1 text-rose-pine-foam shrink-0" size={20} />
                                    <span><b>Less than a loaf of bread</b> - deploy economics designed for broad participation</span>
                                </li>
                                <li className="flex items-start gap-3 text-rose-pine-subtle">
                                    <Unlink className="mt-1 text-rose-pine-foam shrink-0" size={20} />
                                    <span><b>Mainnet for humans and agents</b> - composable bytecode unlocks workflows for both builders and LLMs</span>
                                </li>
                            </ul>

                        </motion.div>
                    </div>

                </div>
            </div>
        </section>
    );
}
