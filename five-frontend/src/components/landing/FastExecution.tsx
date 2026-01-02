"use client";

import { motion } from "framer-motion";
import { Zap, Timer, Server, Cpu, ArrowRight } from "lucide-react";

export default function FastExecution() {
    return (
        <section className="relative py-32 px-4 overflow-hidden bg-[#191724]">
            {/* Background Atmosphere */}
            <div className="absolute inset-0 pointer-events-none">
                <div className="absolute top-1/2 left-0 -translate-y-1/2 w-[800px] h-[600px] bg-rose-pine-love/5 rounded-full blur-[120px]" />
            </div>

            <div className="relative z-10 max-w-7xl mx-auto">
                <div className="grid lg:grid-cols-2 gap-16 items-center">

                    {/* Visual: Execution Timeline (Flame Graph) */}
                    <div className="order-1 relative">
                        {/* Background Splashes */}
                        <div className="absolute -inset-10 bg-gradient-to-tr from-rose-pine-love/10 to-transparent blur-3xl opacity-50" />

                        <motion.div
                            initial={{ opacity: 0, scale: 0.95 }}
                            whileInView={{ opacity: 1, scale: 1 }}
                            viewport={{ once: true }}
                            transition={{ duration: 0.8 }}
                            className="rounded-3xl border border-rose-pine-hl-low/20 bg-[#1f1d2e] shadow-2xl overflow-hidden p-6 md:p-8"
                        >
                            <div className="space-y-8">

                                {/* Timeline 1: Anchor (Slow) */}
                                <div>
                                    <div className="flex justify-between text-xs font-mono text-rose-pine-subtle mb-2 uppercase tracking-widest opacity-70">
                                        <span>Typical Anchor Program</span>
                                        <span>~5000 CU</span>
                                    </div>
                                    <div className="h-12 w-full flex rounded-lg overflow-hidden border border-rose-pine-hl-low/10 bg-[#26233a]">
                                        {/* Discriminator */}
                                        <div className="h-full bg-rose-pine-text/20 w-[15%] border-r border-[#1f1d2e] relative group">
                                            <div className="absolute inset-0 flex items-center justify-center text-[10px] text-rose-pine-subtle font-mono">Check</div>
                                            <div className="absolute -top-8 left-1/2 -translate-x-1/2 bg-[#2a273f] px-2 py-1 rounded text-[10px] text-rose-pine-text opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap border border-rose-pine-hl-low/20">
                                                Discriminator Check
                                            </div>
                                        </div>
                                        {/* Deserialization */}
                                        <div className="h-full bg-rose-pine-iris/20 w-[40%] border-r border-[#1f1d2e] relative group">
                                            <div className="absolute inset-0 flex items-center justify-center text-[10px] text-rose-pine-iris font-mono">Borsh Deserialize</div>
                                            <div className="absolute -top-8 left-1/2 -translate-x-1/2 bg-[#2a273f] px-2 py-1 rounded text-[10px] text-rose-pine-text opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap border border-rose-pine-hl-low/20">
                                                Arg Deserialization Overhead
                                            </div>
                                        </div>
                                        {/* Logic */}
                                        <div className="h-full bg-rose-pine-gold/20 w-[25%] border-r border-[#1f1d2e] relative group">
                                            <div className="absolute inset-0 flex items-center justify-center text-[10px] text-rose-pine-gold font-mono">Logic</div>
                                        </div>
                                        {/* Serialization */}
                                        <div className="h-full bg-rose-pine-iris/20 w-[20%] relative group">
                                            <div className="absolute inset-0 flex items-center justify-center text-[10px] text-rose-pine-iris font-mono">Serialize</div>
                                        </div>
                                    </div>
                                </div>

                                {/* Timeline 2: 5IVE (Fast) */}
                                <div>
                                    <div className="flex justify-between text-xs font-mono text-rose-pine-love mb-2 uppercase tracking-widest font-bold">
                                        <span>5IVE Program</span>
                                        <span>~400 CU</span>
                                    </div>
                                    <div className="relative">
                                        <div className="h-12 w-full flex rounded-lg overflow-hidden border border-rose-pine-love/30 bg-[#26233a]/50">
                                            {/* Native Execution */}
                                            <div className="h-full bg-gradient-to-r from-rose-pine-love to-rose-pine-iris w-[25%] relative group shadow-[0_0_20px_rgba(235,111,146,0.3)]">
                                                <div className="absolute inset-0 flex items-center justify-center text-[10px] text-[#191724] font-bold font-mono uppercase">Bytecode Exec</div>

                                                {/* Speed Lines */}
                                                <div className="absolute top-0 right-0 h-full w-[200%] bg-gradient-to-r from-white/20 to-transparent opacity-0 animate-[shimmer_1s_infinite]" />
                                            </div>
                                            <div className="flex-1 flex items-center px-4">
                                                <span className="text-[10px] text-rose-pine-subtle/50 font-mono italic">Zero Serialization Overhead</span>
                                            </div>
                                        </div>
                                    </div>
                                </div>

                            </div>
                        </motion.div>
                    </div>

                    {/* Text Context */}
                    <div className="order-2">
                        <motion.div
                            initial={{ opacity: 0, x: 20 }}
                            whileInView={{ opacity: 1, x: 0 }}
                            viewport={{ once: true }}
                            transition={{ duration: 0.6 }}
                        >
                            <div className="flex items-center gap-3 mb-6">
                                <div className="p-2 rounded-lg bg-rose-pine-love/10 border border-rose-pine-love/20 text-rose-pine-love">
                                    <Timer size={24} />
                                </div>
                                <h2 className="text-3xl md:text-5xl font-black text-rose-pine-text tracking-tight">
                                    Compute Unit <br /> <span className="text-transparent bg-clip-text bg-gradient-to-r from-rose-pine-love to-rose-pine-iris">Optimized</span>
                                </h2>
                            </div>

                            <p className="text-xl text-rose-pine-muted font-light leading-relaxed mb-8">
                                Optimized so you don't have to.
                                <span className="block mt-2 text-rose-pine-text font-medium">5IVE eliminates boilerplate overhead, leaving more compute for your logic.</span>
                            </p>

                            <ul className="space-y-6 mb-10">
                                <li className="flex gap-4">
                                    <div className="mt-1 p-1 bg-rose-pine-surface rounded text-rose-pine-love shrink-0">
                                        <Cpu size={20} />
                                    </div>
                                    <div>
                                        <h4 className="font-bold text-rose-pine-text text-lg">Zero-Copy Memory</h4>
                                        <p className="text-sm text-rose-pine-muted leading-relaxed">
                                            Operate directly on account data without deserializing it first. Save 5000+ CU per instruction.
                                        </p>
                                    </div>
                                </li>
                                <li className="flex gap-4">
                                    <div className="mt-1 p-1 bg-rose-pine-surface rounded text-rose-pine-love shrink-0">
                                        <Server size={20} />
                                    </div>
                                    <div>
                                        <h4 className="font-bold text-rose-pine-text text-lg">Instant Finality</h4>
                                        <p className="text-sm text-rose-pine-muted leading-relaxed">
                                            Smaller interactions mean more transactions per block and faster confirmation times.
                                        </p>
                                    </div>
                                </li>
                            </ul>
                        </motion.div>
                    </div>

                </div>
            </div>
        </section>
    );
}
