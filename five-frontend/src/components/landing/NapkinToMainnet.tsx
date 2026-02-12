"use client";

import { motion } from "framer-motion";
import { Coins, ShieldCheck, Unlink, Hammer, BadgeDollarSign, Rocket } from "lucide-react";
import { useSolPrice } from "@/hooks/useSolPrice";

const RENT_PER_BYTE_LAM = 6960;
const ACCOUNT_OVERHEAD_BYTES = 128;
const SCRIPT_HEADER_BYTES = 64;
const LAMPORTS_PER_SOL = 1_000_000_000;
const ANCHOR_HELLO_WORLD_BYTES = 100 * 1024; // 100KB
const FIVE_HELLO_WORLD_BYTES = 70;

function estimateDeploySol(bytecodeBytes: number): number {
    const space = SCRIPT_HEADER_BYTES + bytecodeBytes;
    const rentLamports = (ACCOUNT_OVERHEAD_BYTES + space) * RENT_PER_BYTE_LAM;
    return rentLamports / LAMPORTS_PER_SOL;
}

export default function NapkinToMainnet() {
    const { price: solPrice } = useSolPrice();
    const anchorCostSol = estimateDeploySol(ANCHOR_HELLO_WORLD_BYTES);
    const fiveCostSol = estimateDeploySol(FIVE_HELLO_WORLD_BYTES);
    const anchorCostUsd = solPrice ? anchorCostSol * solPrice : null;
    const fiveCostUsd = solPrice ? fiveCostSol * solPrice : null;

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
                                    The Devnet Trap
                                </div>
                                <div className="flex-1 p-4 bg-rose-pine-surface text-center text-xs font-bold font-mono uppercase tracking-widest text-rose-pine-iris">
                                    Compact Bytecode
                                </div>
                            </div>

                            {/* Body - The Wall Crumbling Visual */}
                            <div className="grid grid-cols-2 h-[320px] relative">
                                {/* The Wall (Native Solana) */}
                                <div className="relative p-6 bg-rose-pine-base flex flex-col items-center justify-center border-r border-rose-pine-hl-low/10 group overflow-hidden">
                                     {/* Cracks and Debris Effect */}
                                    <div className="absolute inset-0 bg-[url('/noise.png')] opacity-20" />
                                    <div className="absolute top-0 right-0 w-full h-full border-r-[1px] border-r-rose-pine-love/20 skew-x-12 origin-bottom-right scale-y-110 opacity-0 group-hover:opacity-100 transition-opacity duration-700" />
                                    
                                    <div className="z-10 w-full max-w-[220px] flex flex-col items-center gap-4">
                                        <div className="rounded-xl border border-rose-pine-love/25 bg-rose-pine-base/70 px-5 py-4 text-center">
                                            <div className="text-[10px] font-mono uppercase tracking-widest text-rose-pine-subtle mb-1">Anchor Hello World</div>
                                            <div className="text-3xl font-black text-rose-pine-love">&gt;100KB</div>
                                            <div className="mt-2 text-xs font-mono text-rose-pine-subtle">
                                                {anchorCostSol.toFixed(3)} SOL
                                            </div>
                                            <div className="text-xs font-mono text-rose-pine-love/90">
                                                {anchorCostUsd ? `$${anchorCostUsd.toFixed(2)}` : "--"}
                                            </div>
                                        </div>
                                        <BadgeDollarSign size={14} className="text-rose-pine-love/80" />
                                    </div>

                                    {/* Simulation of crumbling blocks */}
                                    <div className="absolute bottom-0 w-full h-1/3 bg-gradient-to-t from-rose-pine-base to-transparent pointer-events-none" />
                                </div>

                                {/* The Breakthrough (5IVE) */}
                                <div className="relative p-6 bg-rose-pine-surface flex flex-col items-center justify-center overflow-hidden">
                                    <div className="absolute inset-0 bg-rose-pine-iris/5 animate-pulse" />
                                    <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-32 h-32 bg-rose-pine-iris/20 blur-3xl rounded-full" />
                                    
                                    <div className="z-10 w-full max-w-[220px] flex flex-col items-center gap-4">
                                        <div className="rounded-xl border border-rose-pine-iris/30 bg-rose-pine-base/60 px-5 py-4 text-center">
                                            <div className="text-[10px] font-mono uppercase tracking-widest text-rose-pine-subtle mb-1">5ive Hello World</div>
                                            <div className="text-3xl font-black text-rose-pine-iris">&lt;70 bytes</div>
                                            <div className="mt-2 text-xs font-mono text-rose-pine-subtle">
                                                {fiveCostSol.toFixed(6)} SOL
                                            </div>
                                            <div className="text-xs font-mono text-rose-pine-iris/90">
                                                {fiveCostUsd ? `$${fiveCostUsd.toFixed(4)}` : "--"}
                                            </div>
                                        </div>
                                        <Rocket size={14} className="text-rose-pine-iris/90" />
                                    </div>
                                </div>
                                
                                {/* Center "Break" Icon */}
                                <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 bg-rose-pine-surface border border-rose-pine-hl-low/20 p-2 rounded-full shadow-lg z-20 text-rose-pine-love">
                                    <Hammer size={20} className="transform -scale-x-100" />
                                </div>
                            </div>
                            <div className="border-t border-rose-pine-hl-low/10 px-4 py-2 text-center">
                                <span className="text-[10px] font-mono uppercase tracking-wider text-rose-pine-subtle">
                                    {solPrice ? `@ $${solPrice.toFixed(2)} / SOL` : "@ loading SOL price..."}
                                </span>
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
                                <div>
                                    <p className="text-xs md:text-sm font-mono uppercase tracking-[0.18em] text-rose-pine-foam/90 mb-2">
                                        Tear down the wall starts here
                                    </p>
                                    <h2 className="text-3xl md:text-5xl font-black text-rose-pine-text tracking-tight">
                                        Mainnet Is the Goal. <br /> <span className="text-transparent bg-clip-text bg-gradient-to-r from-rose-pine-iris to-rose-pine-foam">Cost Is the Wall.</span>
                                    </h2>
                                </div>
                            </div>

                            <div className="mb-8">
                                <p className="text-xl text-rose-pine-subtle leading-relaxed mb-4 text-contrast">
                                    Most projects die on devnet because mainnet deploys are too expensive. 5IVE compiles contracts to compact bytecode to cut deploy cost.
                                </p>
                                <p className="text-lg font-medium text-rose-pine-text italic border-l-4 border-rose-pine-iris pl-4 py-2 bg-rose-pine-iris/5 rounded-r-lg">
                                    More projects can ship to mainnet instead of stalling on devnet.
                                </p>
                            </div>

                            <ul className="space-y-4 mb-10 text-contrast">
                                <li className="flex items-start gap-3 text-rose-pine-subtle">
                                    <ShieldCheck className="mt-1 text-rose-pine-foam shrink-0" size={20} />
                                    <span><b>The blocker is deploy cost</b> - not demand</span>
                                </li>
                                <li className="flex items-start gap-3 text-rose-pine-subtle">
                                    <Hammer className="mt-1 text-rose-pine-foam shrink-0" size={20} />
                                    <span><b>Compact bytecode</b> lowers deploy overhead</span>
                                </li>
                                <li className="flex items-start gap-3 text-rose-pine-subtle">
                                    <Unlink className="mt-1 text-rose-pine-foam shrink-0" size={20} />
                                    <span><b>Lower friction</b> makes mainnet viable for smaller teams</span>
                                </li>
                            </ul>

                        </motion.div>
                    </div>

                </div>
            </div>
        </section>
    );
}
