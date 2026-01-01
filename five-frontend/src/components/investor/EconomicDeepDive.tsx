"use client";

import { motion, AnimatePresence } from "framer-motion";
import { TrendingDown, Coins, Activity } from "lucide-react";
import { useSolPrice } from "@/hooks/useSolPrice";

export default function EconomicDeepDive() {
    return (
        <section className="relative py-32 px-4 overflow-hidden">
            {/* Background Elements */}
            <div className="absolute inset-0 pointer-events-none">
                <div className="absolute top-1/4 left-1/4 w-[500px] h-[500px] bg-rose-pine-iris/5 rounded-full blur-[100px]" />
                <div className="absolute bottom-1/4 right-1/4 w-[400px] h-[400px] bg-rose-pine-love/5 rounded-full blur-[100px]" />
            </div>

            <div className="relative z-10 max-w-7xl mx-auto flex flex-col items-center">
                {/* Header */}
                <div className="text-center mb-20 max-w-3xl">
                    <motion.div
                        initial={{ opacity: 0, y: 20 }}
                        whileInView={{ opacity: 1, y: 0 }}
                        viewport={{ once: true }}
                    >
                        <h2 className="text-4xl md:text-5xl font-black text-rose-pine-text mb-6 tracking-tight">Napkin to Mainnet</h2>
                        <p className="text-xl text-rose-pine-muted font-light leading-relaxed">
                            Legacy blockchains require thousands of dollars in audit and deployment costs just to test an idea.
                            With 5IVE, you can deploy a protocol for the price of a coffee.
                            It turns "deployment" from a major financial decision into a trivial experimental step.
                        </p>
                    </motion.div>
                </div>

                <div className="w-full grid md:grid-cols-2 gap-8 mb-16">
                    {/* Visual Comparison Card - Legacy */}
                    <motion.div
                        initial={{ opacity: 0, x: -20 }}
                        whileInView={{ opacity: 1, x: 0 }}
                        viewport={{ once: true }}
                        className="relative p-10 rounded-3xl border border-white/5 bg-[#121118]/80 backdrop-blur-xl overflow-hidden group"
                    >
                        <div className="absolute top-0 right-0 p-8 opacity-20 transition-opacity group-hover:opacity-40">
                            <TrendingDown className="w-32 h-32 text-rose-pine-love" />
                        </div>
                        <div className="relative z-10">
                            <h3 className="text-2xl font-bold text-rose-pine-text mb-2">Legacy Deployment</h3>
                            <p className="text-rose-pine-muted mb-8 text-sm uppercase tracking-wider font-medium">Anchor Program (Buffer + IDL)</p>
                            <CostComparison isLegacy={true} />
                        </div>
                    </motion.div>

                    {/* 5IVE Deployment Card */}
                    <motion.div
                        initial={{ opacity: 0, x: 20 }}
                        whileInView={{ opacity: 1, x: 0 }}
                        viewport={{ once: true }}
                        className="relative p-10 rounded-3xl border border-rose-pine-foam/20 bg-rose-pine-foam/5 backdrop-blur-xl overflow-hidden group"
                    >
                        <div className="absolute inset-0 bg-gradient-to-br from-rose-pine-foam/5 to-transparent opacity-50" />
                        <div className="absolute top-0 right-0 p-8 opacity-20 transition-opacity group-hover:opacity-40">
                            <Activity className="w-32 h-32 text-rose-pine-foam" />
                        </div>
                        <div className="relative z-10">
                            <h3 className="text-2xl font-bold text-rose-pine-text mb-2">5IVE Deployment</h3>
                            <p className="text-rose-pine-foam mb-8 text-sm uppercase tracking-wider font-medium">Optimized Bytecode + Minimal State</p>
                            <CostComparison isLegacy={false} />
                        </div>
                    </motion.div>
                </div>

                <div className="grid md:grid-cols-3 gap-6 w-full">
                    <MetricStat label="Code Efficiency" value="800x" sub="Smaller Footprint" />
                    <MetricStat label="Compute Savings" value="Max" sub="High Performance" />
                    <MetricStat label="Development Speed" value="10x" sub="Faster Time to Market" />
                </div>
            </div>
        </section>
    );
}

function CostComparison({ isLegacy }: { isLegacy: boolean }) {
    const { price: solPrice } = useSolPrice();

    // Legacy: ~10 SOL (Standard program deployment, e.g. SPL Token is ~9 SOL)
    // 5ive: ~0.002 SOL (Rent for small account)
    const costSOL = isLegacy ? 10.0 : 0.002;
    const costUSD = solPrice ? (costSOL * solPrice).toFixed(2) : (isLegacy ? "2000.00+" : "0.30");

    return (
        <div className="flex flex-col gap-4">
            <div className={`text-6xl font-black tracking-tighter ${isLegacy ? 'text-rose-pine-love' : 'text-rose-pine-foam'}`}>
                ${costUSD}
            </div>
            <div className="flex items-center gap-2 text-sm font-mono opacity-70">
                <span>{costSOL} SOL</span>
                <span className="w-1 h-1 rounded-full bg-current" />
                <span>@ ${solPrice?.toFixed(0) || "150"} / SOL</span>
            </div>
            {isLegacy ? (
                <div className="mt-4 p-3 bg-rose-pine-love/10 rounded-lg text-xs text-rose-pine-love border border-rose-pine-love/20 font-medium">
                    High barrier to entry for experimentation.
                </div>
            ) : (
                <div className="mt-4 p-3 bg-rose-pine-foam/10 rounded-lg text-xs text-rose-pine-foam border border-rose-pine-foam/20 font-medium">
                    Cheap enough to experiment freely. Build, test, iterate.
                </div>
            )}
        </div>
    );
}

function MetricStat({ label, value, sub }: { label: string, value: string, sub: string }) {
    return (
        <motion.div
            whileHover={{ y: -5 }}
            className="flex flex-col items-center text-center p-8 rounded-2xl bg-white/60 dark:bg-[#121118]/60 border border-white/5 backdrop-blur-md hover:bg-white/80 dark:hover:bg-[#121118]/80 transition-colors"
        >
            <span className="text-xs uppercase tracking-widest text-rose-pine-muted mb-3 font-semibold">{label}</span>
            <span className="text-5xl font-black text-rose-pine-text mb-2 tracking-tight">{value}</span>
            <span className="text-sm text-rose-pine-subtle">{sub}</span>
        </motion.div>
    )
}
