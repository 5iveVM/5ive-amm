"use client";

import Link from "next/link";
import { cn } from "@/lib/utils";
import { Cpu, Zap, Code2 } from "lucide-react";
import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";

export default function Features() {
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
                    <h2 className="text-4xl md:text-5xl font-bold text-rose-pine-iris mb-4 drop-shadow-md">Why Build on 5IVE?</h2>
                    <p className="text-rose-pine-subtle text-lg max-w-2xl mx-auto">
                        Experience the next generation of blockchain development with features designed for scale and developer happiness.
                    </p>
                </motion.div>

                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 relative z-10 w-full max-w-7xl mx-auto">
                    {/* Feature 1: Cheaper Deployment */}
                    <Link href="/investor#adapt" className="block h-full">
                        <motion.div
                            whileHover={{ y: -5 }}
                            className="group relative p-8 rounded-2xl border border-[var(--glass-border)] bg-[var(--glass-bg)] backdrop-blur-xl shadow-lg transition-all duration-300 hover:shadow-[0_0_30px_rgba(235,111,146,0.15)] hover:border-l-rose-pine-love hover:border-t-rose-pine-love/50 min-h-[200px] flex flex-col h-full cursor-pointer"
                        >
                            <div className="absolute inset-0 rounded-2xl bg-gradient-to-br from-rose-pine-love/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none" />
                            <div className="w-12 h-12 rounded-xl bg-rose-pine-love/10 flex items-center justify-center mb-6 group-hover:bg-rose-pine-love/20 transition-colors">
                                <Cpu className="w-6 h-6 text-rose-pine-love" />
                            </div>
                            <h3 className="text-xl font-bold text-rose-pine-text mb-3 tracking-tight group-hover:text-rose-pine-love transition-colors">Cheaper Deployment</h3>
                            <div className="text-rose-pine-muted leading-relaxed font-light mt-auto">
                                <CostTicker />
                            </div>
                        </motion.div>
                    </Link>

                    {/* Feature 2: Super Powers */}
                    <Link href="/investor#engine" className="block h-full">
                        <motion.div
                            whileHover={{ y: -5 }}
                            className="group relative p-8 rounded-2xl border border-[var(--glass-border)] bg-[var(--glass-bg)] backdrop-blur-xl shadow-lg transition-all duration-300 hover:shadow-[0_0_30px_rgba(196,167,231,0.15)] hover:border-l-rose-pine-iris hover:border-t-rose-pine-iris/50 min-h-[200px] flex flex-col h-full cursor-pointer"
                        >
                            <div className="absolute inset-0 rounded-2xl bg-gradient-to-br from-rose-pine-iris/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none" />
                            <div className="w-12 h-12 rounded-xl bg-rose-pine-iris/10 flex items-center justify-center mb-6 group-hover:bg-rose-pine-iris/20 transition-colors">
                                <Code2 className="w-6 h-6 text-rose-pine-iris" />
                            </div>
                            <h3 className="text-xl font-bold text-rose-pine-text mb-3 tracking-tight group-hover:text-rose-pine-iris transition-colors">Super Powers</h3>
                            <div className="text-rose-pine-muted leading-relaxed font-light mt-auto">
                                <SuperPowersTicker />
                            </div>
                        </motion.div>
                    </Link>

                    {/* Feature 3: Universal Runtime */}
                    <Link href="/investor#ai" className="block h-full">
                        <motion.div
                            whileHover={{ y: -5 }}
                            className="group relative p-8 rounded-2xl border border-[var(--glass-border)] bg-[var(--glass-bg)] backdrop-blur-xl shadow-lg transition-all duration-300 hover:shadow-[0_0_30px_rgba(156,207,216,0.15)] hover:border-l-rose-pine-foam hover:border-t-rose-pine-foam/50 min-h-[200px] flex flex-col h-full cursor-pointer"
                        >
                            <div className="absolute inset-0 rounded-2xl bg-gradient-to-br from-rose-pine-foam/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none" />
                            <div className="w-12 h-12 rounded-xl bg-rose-pine-foam/10 flex items-center justify-center mb-6 group-hover:bg-rose-pine-foam/20 transition-colors">
                                <Zap className="w-6 h-6 text-rose-pine-foam" />
                            </div>
                            <h3 className="text-xl font-bold text-rose-pine-text mb-3 tracking-tight group-hover:text-rose-pine-foam transition-colors">Universal Runtime</h3>
                            <div className="text-rose-pine-muted leading-relaxed font-light mt-auto">
                                <ExecutionTicker />
                            </div>
                        </motion.div>
                    </Link>
                </div>
            </div>
        </section>
    );
}

function CostTicker() {
    const [index, setIndex] = useState(0);
    const items = [
        { label: "Traditional Anchor", value: "$400+", sub: "2+ SOL", color: "text-rose-pine-love" },
        { label: "5IVE VM", value: "$0.40", sub: "0.002 SOL", color: "text-rose-pine-foam" },
        { label: "Cost Reduction", value: "99.9%", sub: "1000x Cheaper", color: "text-rose-pine-gold" }
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

function SuperPowersTicker() {
    const [index, setIndex] = useState(0);
    const items = [
        { label: "Execution Model", value: "Code + State", sub: "Same Account", color: "text-rose-pine-love" },
        { label: "No More CPI", value: "Import / Use", sub: "Direct Calls", color: "text-rose-pine-foam" },
        { label: "Build Moats", value: "10MB Accounts", sub: "1000s of Programs", color: "text-rose-pine-gold" }
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

function ExecutionTicker() {
    const [index, setIndex] = useState(0);
    const items = [
        { label: "Efficiency", value: "Low CU Usage", sub: "Fast Execution", color: "text-rose-pine-love" },
        { label: "Scalability", value: "5IVE Scales Solana", sub: "Next-Gen Performance", color: "text-rose-pine-foam" },
        { label: "Environment", value: "WASM Native", sub: "100% Web Dev Exp", color: "text-rose-pine-gold" }
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
