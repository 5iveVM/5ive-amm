"use client";

import { motion } from "framer-motion";
import { Layers, Box, Cpu, Shield, Database, LayoutGrid } from "lucide-react";

export default function TheMoat() {
    return (
        <section className="relative py-32 px-4 overflow-hidden bg-[#191724]">
            {/* Background Atmosphere */}
            <div className="absolute inset-0 pointer-events-none">
                <div className="absolute bottom-0 left-0 w-[800px] h-[800px] bg-rose-pine-gold/5 rounded-full blur-[150px]" />
            </div>

            <div className="relative z-10 max-w-7xl mx-auto">
                <div className="grid lg:grid-cols-2 gap-16 items-center">

                    {/* Visual: The Megablock (10MB Container) */}
                    <div className="order-1 relative">
                        {/* Background Pulse */}
                        <div className="absolute -inset-10 bg-gradient-to-tr from-rose-pine-gold/10 to-transparent blur-3xl opacity-50" />

                        <motion.div
                            initial={{ opacity: 0, scale: 0.9 }}
                            whileInView={{ opacity: 1, scale: 1 }}
                            viewport={{ once: true }}
                            transition={{ duration: 0.8 }}
                            className="relative aspect-square max-w-md mx-auto"
                        >
                            {/* The Container Border */}
                            <div className="absolute inset-0 rounded-3xl border-2 border-rose-pine-gold/20 bg-[#1f1d2e]/80 shadow-[0_0_50px_rgba(246,193,119,0.1)] overflow-hidden flex flex-wrap content-start p-2 gap-1">

                                {/* Header Bar of the Container */}
                                <div className="w-full h-8 mb-2 flex items-center justify-between px-3 border-b border-rose-pine-gold/10">
                                    <span className="text-[10px] font-mono text-rose-pine-gold uppercase tracking-widest">Account.bin</span>
                                    <span className="text-[10px] font-mono text-rose-pine-subtle">10MB / 10MB</span>
                                </div>

                                {/* Protocol Blocks (Simulated Density) */}
                                {Array.from({ length: 64 }).map((_, i) => (
                                    <motion.div
                                        key={i}
                                        initial={{ opacity: 0, scale: 0 }}
                                        whileInView={{ opacity: 1, scale: 1 }}
                                        viewport={{ once: true }}
                                        transition={{ delay: i * 0.01, duration: 0.4 }}
                                        className={`w-8 h-8 rounded-md bg-rose-pine-surface border border-white/5 flex items-center justify-center ${i % 3 === 0 ? 'bg-rose-pine-love/10' : i % 4 === 0 ? 'bg-rose-pine-iris/10' : i % 5 === 0 ? 'bg-rose-pine-foam/10' : ''}`}
                                    >
                                        <div className={`w-3 h-3 rounded-sm ${i % 3 === 0 ? 'bg-rose-pine-love' : i % 4 === 0 ? 'bg-rose-pine-iris' : i % 5 === 0 ? 'bg-rose-pine-foam' : 'bg-rose-pine-muted/20'}`} />
                                    </motion.div>
                                ))}

                                {/* Overlay Text */}
                                <div className="absolute inset-0 flex items-center justify-center bg-black/40 backdrop-blur-[2px]">
                                    <div className="text-center">
                                        <div className="text-4xl font-black text-white drop-shadow-lg">1000+</div>
                                        <div className="text-sm font-bold text-rose-pine-gold uppercase tracking-widest">Protocols</div>
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
                                <div className="p-2 rounded-lg bg-rose-pine-gold/10 border border-rose-pine-gold/20 text-rose-pine-gold">
                                    <Layers size={24} />
                                </div>
                                <h2 className="text-3xl md:text-5xl font-black text-rose-pine-text tracking-tight">
                                    The Unforkable <br /> <span className="text-transparent bg-clip-text bg-gradient-to-r from-rose-pine-gold to-rose-pine-love">Moat</span>
                                </h2>
                            </div>

                            <p className="text-xl text-rose-pine-muted font-light leading-relaxed mb-8">
                                A single 10MB account containing over 1000 distinct programs.
                                <span className="block mt-2 text-rose-pine-text font-medium">Unlock entirely new crypto use cases.</span>
                            </p>

                            <ul className="space-y-6 mb-10">
                                <li className="flex gap-4">
                                    <div className="mt-1 p-1 bg-rose-pine-surface rounded text-rose-pine-gold shrink-0">
                                        <Database size={20} />
                                    </div>
                                    <div>
                                        <h4 className="font-bold text-rose-pine-text text-lg">Massive State Capacity</h4>
                                        <p className="text-sm text-rose-pine-muted leading-relaxed">
                                            Combine AMMs, Lending, Governance, and Identity in one high-performance memory space.
                                        </p>
                                    </div>
                                </li>
                                <li className="flex gap-4">
                                    <div className="mt-1 p-1 bg-rose-pine-surface rounded text-rose-pine-gold shrink-0">
                                        <Shield size={20} />
                                    </div>
                                    <div>
                                        <h4 className="font-bold text-rose-pine-text text-lg">Protocol Stability</h4>
                                        <p className="text-sm text-rose-pine-muted leading-relaxed">
                                            Once a Moat is deployed, it becomes unstoppable infrastructure. Zero maintenance, infinite uptime.
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
