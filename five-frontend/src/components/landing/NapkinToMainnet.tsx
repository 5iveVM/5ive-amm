"use client";

import { motion } from "framer-motion";
import { useSolPrice } from "@/hooks/useSolPrice";
import { Coins, ShieldCheck, Unlink } from "lucide-react";

export default function NapkinToMainnet() {
    return (
        <section className="relative py-32 px-4 overflow-hidden bg-[#191724]">
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
                            className="rounded-3xl border border-rose-pine-hl-low/20 bg-[#1f1d2e] overflow-hidden shadow-2xl shadow-rose-pine-iris/10"
                        >
                            {/* Header */}
                            <div className="flex border-b border-rose-pine-hl-low/10">
                                <div className="flex-1 p-4 bg-[#232136] text-center text-xs font-mono uppercase tracking-widest text-rose-pine-subtle border-r border-rose-pine-hl-low/10 opacity-70">
                                    Native Solana
                                </div>
                                <div className="flex-1 p-4 bg-[#2a273f] text-center text-xs font-bold font-mono uppercase tracking-widest text-rose-pine-iris">
                                    5IVE L1.5
                                </div>
                            </div>

                            {/* Body */}
                            <div className="grid grid-cols-2 divide-x divide-rose-pine-hl-low/10 h-[300px]">

                                {/* Legacy Cost */}
                                <div className="relative p-8 bg-[#1f1d2e] flex flex-col items-center justify-center gap-2 opacity-60">
                                    <CostTicker isLegacy={true} />
                                    <span className="text-[10px] font-mono uppercase text-rose-pine-subtle mt-4">Native Binary (~2MB)</span>
                                </div>

                                {/* 5IVE Cost */}
                                <div className="relative p-8 bg-[#2a273f]/50 flex flex-col items-center justify-center gap-2">
                                    <div className="absolute top-0 right-0 p-2">
                                        <div className="flex gap-1.5">
                                            <div className="w-2 h-2 rounded-full bg-rose-pine-iris animate-pulse" />
                                        </div>
                                    </div>

                                    {/* Glow behind value */}
                                    <div className="absolute inset-0 bg-rose-pine-iris/10 blur-xl rounded-full scale-50" />

                                    <CostTicker isLegacy={false} />

                                    <span className="text-[10px] font-bold font-mono uppercase text-rose-pine-iris mt-4 tracking-widest bg-rose-pine-iris/10 px-2 py-1 rounded">
                                        5IVE Bytecode (&lt;1KB)
                                    </span>
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
                                    The First <br /> <span className="text-transparent bg-clip-text bg-gradient-to-r from-rose-pine-iris to-rose-pine-foam">Layer 1.5</span>
                                </h2>
                            </div>

                            <p className="text-xl text-rose-pine-muted font-light leading-relaxed mb-8">
                                Drastically reduce executable size. Deploy continuously.
                                <span className="block mt-2 text-rose-pine-text font-medium">5IVE compresses logic into raw bytecode, reducing executable footprint by 1000x.</span>
                            </p>

                            <ul className="space-y-4 mb-10">
                                <li className="flex items-start gap-3 text-rose-pine-subtle">
                                    <ShieldCheck className="mt-1 text-rose-pine-foam shrink-0" size={20} />
                                    <span><b>Inherit L1 Security</b> - No multisig bridges</span>
                                </li>
                                <li className="flex items-start gap-3 text-rose-pine-subtle">
                                    <Coins className="mt-1 text-rose-pine-foam shrink-0" size={20} />
                                    <span><b>$1.00 Deployment</b> (Inc. Fees) vs $400+ Standard</span>
                                </li>
                                <li className="flex items-start gap-3 text-rose-pine-subtle">
                                    <Unlink className="mt-1 text-rose-pine-foam shrink-0" size={20} />
                                    <span><b>No Bloat</b> - Pay for logic, not Rust boilerplate</span>
                                </li>
                            </ul>

                        </motion.div>
                    </div>

                </div>
            </div>
        </section>
    );
}

function CostTicker({ isLegacy }: { isLegacy: boolean }) {
    const { price: solPrice } = useSolPrice();
    // Legacy: ~5 SOL, 5ive: ~0.005 SOL (Increased for honesty: Rent + TX Fee)
    const costSOL = isLegacy ? 5.0 : 0.005;
    const costUSD = solPrice ? (costSOL * solPrice).toFixed(2) : (isLegacy ? "1000+" : "1.00");

    return (
        <div className="flex flex-col items-center relative z-10">
            <div className={`text-4xl md:text-6xl font-black tabular-nums tracking-tighter ${isLegacy
                ? "text-rose-pine-subtle drop-shadow-none"
                : "text-transparent bg-clip-text bg-gradient-to-b from-white to-rose-pine-iris drop-shadow-[0_0_20px_rgba(196,167,231,0.5)]"
                }`}>
                <span className="text-2xl md:text-3xl align-top mr-1 opacity-50">$</span>
                {costUSD}
            </div>
            <div className={`mt-1 font-mono text-[10px] tracking-widest uppercase ${isLegacy ? "text-rose-pine-subtle/50" : "text-rose-pine-iris"}`}>
                ~{costSOL} SOL
            </div>
        </div>
    );
}
