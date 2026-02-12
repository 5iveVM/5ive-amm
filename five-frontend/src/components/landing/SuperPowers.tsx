"use client";

import { motion } from "framer-motion";
import { Box, Database, Code, CheckCircle2 } from "lucide-react";

export default function SuperPowers() {
    return (
        <section className="relative py-32 px-4 overflow-hidden bg-transparent">
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
                                    Build the Moat: <br /> <span className="text-transparent bg-clip-text bg-gradient-to-r from-rose-pine-love to-rose-pine-iris">Distribution, Not Just Deployment.</span>
                                </h2>
                            </div>

                            <p className="text-xl text-rose-pine-subtle leading-relaxed mb-8 text-contrast">
                                Solana composition is constrained by account-list limits, even with LUTs. 5IVE keeps the same account model and packs reusable code into a moat account for native-like execution.
                                <span className="block mt-2 text-rose-pine-text font-medium">That means thousands of programs, templates, and interfaces can compose from one deploy surface.</span>
                            </p>

                            <ul className="space-y-4 mb-10 text-contrast">
                                <li className="flex items-start gap-3 text-rose-pine-subtle">
                                    <CheckCircle2 className="mt-1 text-rose-pine-foam shrink-0" size={20} />
                                    <span><b>Same Solana pattern</b>: standard accounts still work, with a denser execution surface</span>
                                </li>
                                <li className="flex items-start gap-3 text-rose-pine-subtle">
                                    <CheckCircle2 className="mt-1 text-rose-pine-foam shrink-0" size={20} />
                                    <span><b>Composition moat</b>: execute reusable modules from one account (risk engine + vault + AMM + settlement)</span>
                                </li>
                                <li className="flex items-start gap-3 text-rose-pine-subtle">
                                    <CheckCircle2 className="mt-1 text-rose-pine-foam shrink-0" size={20} />
                                    <span><b>New contract classes</b>: strategy marketplaces, agent swarms, and vertical app stacks</span>
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
                            className="rounded-3xl border border-rose-pine-hl-low/20 bg-rose-pine-surface overflow-hidden shadow-2xl shadow-rose-pine-iris/10"
                        >
                            {/* Header */}
                            <div className="flex border-b border-rose-pine-hl-low/10">
                                <div className="flex-1 p-4 bg-rose-pine-base text-center text-xs font-mono uppercase tracking-widest text-rose-pine-subtle border-r border-rose-pine-hl-low/10 opacity-70">
                                    Fragmented Stack
                                </div>
                                <div className="flex-1 p-4 bg-rose-pine-surface text-center text-xs font-bold font-mono uppercase tracking-widest text-rose-pine-love">
                                    5IVE Moat Stack
                                </div>
                            </div>

                            {/* Body */}
                            <div className="grid grid-cols-2 divide-x divide-rose-pine-hl-low/10 h-[300px]">

                                {/* Left: Program -> CPI -> Program + separate state */}
                                <div className="relative p-6 bg-rose-pine-surface flex flex-col items-center justify-center gap-5 opacity-65">
                                    <div className="w-full max-w-[190px] flex items-center justify-between">
                                        <div className="w-[68px] rounded-lg border border-dashed border-rose-pine-text/20 bg-rose-pine-base/50 p-2 flex flex-col items-center gap-1">
                                            <Code size={14} className="text-rose-pine-subtle" />
                                            <span className="text-[9px] uppercase font-mono text-rose-pine-subtle">Program A</span>
                                        </div>
                                        <div className="flex flex-col items-center gap-1 px-1">
                                            <span className="text-[9px] uppercase font-mono text-rose-pine-love">CPI</span>
                                            <span className="text-rose-pine-subtle text-xs">-&gt;</span>
                                        </div>
                                        <div className="w-[68px] rounded-lg border border-dashed border-rose-pine-text/20 bg-rose-pine-base/50 p-2 flex flex-col items-center gap-1">
                                            <Code size={14} className="text-rose-pine-subtle" />
                                            <span className="text-[9px] uppercase font-mono text-rose-pine-subtle">Program B</span>
                                        </div>
                                    </div>

                                    <div className="w-full max-w-[190px] flex items-center justify-between">
                                        <div className="h-4 w-[1px] border-l border-dashed border-rose-pine-subtle/25 ml-[33px]" />
                                        <div className="h-4 w-[1px] border-l border-dashed border-rose-pine-subtle/25 mr-[33px]" />
                                    </div>

                                    <div className="w-full max-w-[190px] flex items-center justify-between">
                                        <div className="w-[72px] rounded-lg border border-dashed border-rose-pine-text/20 bg-rose-pine-base/50 p-2 flex flex-col items-center gap-1">
                                            <Database size={14} className="text-rose-pine-subtle" />
                                            <span className="text-[9px] uppercase font-mono text-rose-pine-subtle">State A</span>
                                        </div>
                                        <div className="w-[72px] rounded-lg border border-dashed border-rose-pine-text/20 bg-rose-pine-base/50 p-2 flex flex-col items-center gap-1">
                                            <Database size={14} className="text-rose-pine-subtle" />
                                            <span className="text-[9px] uppercase font-mono text-rose-pine-subtle">State B</span>
                                        </div>
                                    </div>
                                    <span className="mt-2 text-[10px] uppercase font-mono text-rose-pine-subtle text-center text-contrast">
                                        Account-list constraints (even with LUTs)
                                    </span>
                                </div>

                                {/* Right: Moat Account + separate state */}
                                <div className="relative p-6 bg-rose-pine-surface/50 flex flex-col items-center justify-center">
                                    <div className="absolute top-0 right-0 p-2">
                                        <div className="flex gap-1.5">
                                            <div className="w-2 h-2 rounded-full bg-rose-pine-love animate-pulse" />
                                        </div>
                                    </div>

                                    <div className="relative w-full max-w-[190px] rounded-xl border border-rose-pine-love/30 bg-gradient-to-br from-rose-pine-love/20 to-rose-pine-iris/20 p-4 shadow-lg">
                                        <div className="text-[10px] uppercase font-mono text-rose-pine-text mb-3 text-center tracking-widest">
                                            Moat Account
                                        </div>
                                        <div className="grid grid-cols-2 gap-2 text-[9px] font-mono uppercase">
                                            <div className="rounded bg-rose-pine-base/70 px-2 py-1 text-rose-pine-foam text-center">Templates</div>
                                            <div className="rounded bg-rose-pine-base/70 px-2 py-1 text-rose-pine-foam text-center">Interfaces</div>
                                            <div className="rounded bg-rose-pine-base/70 px-2 py-1 text-rose-pine-foam text-center">Apps</div>
                                            <div className="rounded bg-rose-pine-base/70 px-2 py-1 text-rose-pine-foam text-center">Modules</div>
                                        </div>
                                        <div className="mt-3 text-center text-xs font-black text-rose-pine-love">
                                            1000+
                                        </div>
                                        <div className="text-center text-[10px] uppercase font-mono text-rose-pine-subtle">
                                            composable units
                                        </div>
                                    </div>

                                    <div className="h-4 w-[1px] border-l border-dashed border-rose-pine-subtle/35" />

                                    <div className="w-full max-w-[120px] rounded-lg border border-dashed border-rose-pine-text/25 bg-rose-pine-base/50 p-2 flex flex-col items-center gap-1">
                                        <Database size={14} className="text-rose-pine-subtle" />
                                        <span className="text-[9px] uppercase font-mono text-rose-pine-subtle">State Account</span>
                                    </div>

                                </div>

                            </div>
                        </motion.div>
                    </div>
                </div>
            </div>
        </section>
    );
}
