"use client";

import { cn } from "@/lib/utils";
import { Cpu, Zap, Code2 } from "lucide-react";
import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";

export default function InvestorFeatures() {
    return (
        <section id="features" className="relative py-24 px-4 flex flex-col items-center">
            <div className="absolute inset-0 bg-gradient-to-b from-transparent via-rose-pine-iris/10 to-transparent pointer-events-none" />

            <div className="relative z-10 max-w-6xl w-full">
                <motion.div
                    initial={{ opacity: 0, y: 20 }}
                    whileInView={{ opacity: 1, y: 0 }}
                    viewport={{ once: true }}
                    transition={{ duration: 0.6 }}
                    className="text-center mb-16"
                >
                    <h2 className="text-4xl md:text-5xl font-bold text-rose-pine-iris mb-4 drop-shadow-md">The Paradigm Shift</h2>
                    <p className="text-rose-pine-subtle text-lg max-w-2xl mx-auto">
                        5IVE is a Layer 1.5 that runs entirely on Solana. We help Solana scale by reducing compute costs and unlocking new use cases.
                    </p>
                </motion.div>

                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 relative z-10 w-full max-w-7xl mx-auto">
                    {/* Feature 1: Cost/Scale */}
                    <motion.div
                        whileHover={{ y: -5 }}
                        className="group relative p-8 rounded-2xl border border-[var(--glass-border)] bg-[var(--glass-bg)] backdrop-blur-xl shadow-lg transition-all duration-300 hover:shadow-[0_0_30px_rgba(235,111,146,0.15)] hover:border-l-rose-pine-love hover:border-t-rose-pine-love/50 min-h-[200px] flex flex-col"
                    >
                        <div className="absolute inset-0 rounded-2xl bg-gradient-to-br from-rose-pine-love/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none" />
                        <div className="w-12 h-12 rounded-xl bg-rose-pine-love/10 flex items-center justify-center mb-6 group-hover:bg-rose-pine-love/20 transition-colors">
                            <Cpu className="w-6 h-6 text-rose-pine-love" />
                        </div>
                        <h3 className="text-xl font-bold text-rose-pine-text mb-3 tracking-tight group-hover:text-rose-pine-love transition-colors">Scaling Solana</h3>
                        <div className="text-rose-pine-muted leading-relaxed font-light mt-auto">
                            <CostTicker />
                        </div>
                    </motion.div>

                    {/* Feature 2: Programs as Tokens */}
                    <motion.div
                        whileHover={{ y: -5 }}
                        className="group relative p-8 rounded-2xl border border-[var(--glass-border)] bg-[var(--glass-bg)] backdrop-blur-xl shadow-lg transition-all duration-300 hover:shadow-[0_0_30px_rgba(196,167,231,0.15)] hover:border-l-rose-pine-iris hover:border-t-rose-pine-iris/50 min-h-[200px] flex flex-col"
                    >
                        <div className="absolute inset-0 rounded-2xl bg-gradient-to-br from-rose-pine-iris/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none" />
                        <div className="w-12 h-12 rounded-xl bg-rose-pine-iris/10 flex items-center justify-center mb-6 group-hover:bg-rose-pine-iris/20 transition-colors">
                            <Code2 className="w-6 h-6 text-rose-pine-iris" />
                        </div>
                        <h3 className="text-xl font-bold text-rose-pine-text mb-3 tracking-tight group-hover:text-rose-pine-iris transition-colors">Programs are Tokens</h3>
                        <div className="text-rose-pine-muted leading-relaxed font-light mt-auto">
                            <TokenProgramTicker />
                        </div>
                    </motion.div>

                    {/* Feature 3: The Moat */}
                    <motion.div
                        whileHover={{ y: -5 }}
                        className="group relative p-8 rounded-2xl border border-[var(--glass-border)] bg-[var(--glass-bg)] backdrop-blur-xl shadow-lg transition-all duration-300 hover:shadow-[0_0_30px_rgba(156,207,216,0.15)] hover:border-l-rose-pine-foam hover:border-t-rose-pine-foam/50 min-h-[200px] flex flex-col"
                    >
                        <div className="absolute inset-0 rounded-2xl bg-gradient-to-br from-rose-pine-foam/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none" />
                        <div className="w-12 h-12 rounded-xl bg-rose-pine-foam/10 flex items-center justify-center mb-6 group-hover:bg-rose-pine-foam/20 transition-colors">
                            <Zap className="w-6 h-6 text-rose-pine-foam" />
                        </div>
                        <h3 className="text-xl font-bold text-rose-pine-text mb-3 tracking-tight group-hover:text-rose-pine-foam transition-colors">The 5IVE Moat</h3>
                        <div className="text-rose-pine-muted leading-relaxed font-light mt-auto">
                            <MoatTicker />
                        </div>
                    </motion.div>
                </div>
            </div>
        </section>
    );
}

function CostTicker() {
    const [index, setIndex] = useState(0);
    const [solPrice, setSolPrice] = useState<number | null>(null);

    // Fetch SOL Price directly
    useEffect(() => {
        const fetchPrice = async () => {
            try {
                const response = await fetch('https://api.coingecko.com/api/v3/simple/price?ids=solana&vs_currencies=usd');
                const data = await response.json();
                if (data.solana?.usd) {
                    setSolPrice(data.solana.usd);
                }
            } catch (e) {
                console.warn("Failed to fetch SOL price", e);
            }
        };
        fetchPrice();
    }, []);

    const solPriceStr = solPrice ? `$${solPrice.toFixed(2)}` : "$150+";
    // 5ive cost: ~0.002 SOL (small account + bytecode) vs Legacy Anchor ~2+ SOL (large buffer)
    // Actually, user text: "same cost as deploying a spl token with metadata"
    // SPL Token + Metadata is roughly 0.002 - 0.005 SOL depending on rent.
    // 5ive programs are compressed bytecode + state in one.

    const items = [
        { label: "Legacy Anchor Cost", value: solPrice ? `$${(solPrice * 2).toFixed(2)}+` : "$300+", sub: "High Rent + Buffer", color: "text-rose-pine-love" },
        { label: "5IVE Cost", value: solPrice ? `$${(solPrice * 0.002).toFixed(3)}` : "$0.30", sub: "Same as SPL Token", color: "text-rose-pine-foam" },
        { label: "Efficiency", value: "-1000 CU", sub: "Cheaper Execution", color: "text-rose-pine-gold" }
    ];

    useEffect(() => {
        const timer = setInterval(() => {
            setIndex((prev) => (prev + 1) % items.length);
        }, 3000);
        return () => clearInterval(timer);
    }, [items.length]);

    const item = items[index];

    return (
        <div className="h-24 relative w-full overflow-hidden flex flex-col justify-center">
            <AnimatePresence mode="wait">
                <motion.div
                    key={index}
                    initial={{ y: 20, opacity: 0 }}
                    animate={{ y: 0, opacity: 1 }}
                    exit={{ y: -20, opacity: 0 }}
                    transition={{ duration: 0.3 }}
                    className="absolute inset-x-0 flex flex-col justify-center"
                >
                    <div className={`text-xs font-bold uppercase tracking-widest mb-1 ${item.color}`}>
                        {item.label}
                    </div>
                    <div className="flex flex-col gap-0.5">
                        <span className="text-3xl font-black text-rose-pine-text whitespace-nowrap tracking-tight">{item.value}</span>
                        <span className="text-sm font-medium text-rose-pine-subtle truncate">{item.sub}</span>
                    </div>
                </motion.div>
            </AnimatePresence>
        </div>
    );
}

function TokenProgramTicker() {
    const [index, setIndex] = useState(0);
    const items = [
        { label: "Innovation", value: "Meme Programs", sub: "Bonding Curves Inside", color: "text-rose-pine-love" },
        { label: "Architecture", value: "Code + State", sub: "Single Account", color: "text-rose-pine-foam" },
        { label: "Simplicity", value: "No Executable", sub: "Just One Token", color: "text-rose-pine-gold" }
    ];

    useEffect(() => {
        const timer = setInterval(() => {
            setIndex((prev) => (prev + 1) % items.length);
        }, 3000);
        return () => clearInterval(timer);
    }, []);

    const item = items[index];

    return (
        <div className="h-24 relative w-full overflow-hidden flex flex-col justify-center">
            <AnimatePresence mode="wait">
                <motion.div
                    key={index}
                    initial={{ y: 20, opacity: 0 }}
                    animate={{ y: 0, opacity: 1 }}
                    exit={{ y: -20, opacity: 0 }}
                    transition={{ duration: 0.3 }}
                    className="absolute inset-x-0 flex flex-col justify-center"
                >
                    <div className={`text-xs font-bold uppercase tracking-widest mb-1 ${item.color}`}>
                        {item.label}
                    </div>
                    <div className="flex flex-col gap-0.5">
                        <span className="text-3xl font-black text-rose-pine-text whitespace-nowrap tracking-tight">{item.value}</span>
                        <span className="text-sm font-medium text-rose-pine-subtle truncate">{item.sub}</span>
                    </div>
                </motion.div>
            </AnimatePresence>
        </div>
    );
}

function MoatTicker() {
    const [index, setIndex] = useState(0);
    const items = [
        { label: "Capacity", value: "1000+ Apps", sub: "In One Account", color: "text-rose-pine-love" },
        { label: "Ecosystem", value: "Everything", sub: "AMM, Vaults, DAO, Lending", color: "text-rose-pine-foam" },
        { label: "Synergy", value: "Layer 1.5", sub: "Universal Runtime", color: "text-rose-pine-gold" }
    ];

    useEffect(() => {
        const timer = setInterval(() => {
            setIndex((prev) => (prev + 1) % items.length);
        }, 3000);
        return () => clearInterval(timer);
    }, []);

    const item = items[index];

    return (
        <div className="h-24 relative w-full overflow-hidden flex flex-col justify-center">
            <AnimatePresence mode="wait">
                <motion.div
                    key={index}
                    initial={{ y: 20, opacity: 0 }}
                    animate={{ y: 0, opacity: 1 }}
                    exit={{ y: -20, opacity: 0 }}
                    transition={{ duration: 0.3 }}
                    className="absolute inset-x-0 flex flex-col justify-center"
                >
                    <div className={`text-xs font-bold uppercase tracking-widest mb-1 ${item.color}`}>
                        {item.label}
                    </div>
                    <div className="flex flex-col gap-0.5">
                        <span className="text-3xl font-black text-rose-pine-text whitespace-nowrap tracking-tight">{item.value}</span>
                        <span className="text-sm font-medium text-rose-pine-subtle truncate">{item.sub}</span>
                    </div>
                </motion.div>
            </AnimatePresence>
        </div>
    );
}
