"use client";

import React from "react";
import { motion } from "framer-motion";
import { Sparkles, Bot, Terminal, Cpu } from "lucide-react";

export default function AIAdvantageSection() {
    return (
        <section className="relative py-20 px-4 flex flex-col items-center overflow-hidden">
            {/* Background Elements */}
            <div className="absolute inset-0 pointer-events-none">
                <div className="absolute top-1/4 left-1/4 w-[500px] h-[500px] bg-rose-pine-iris/5 rounded-full blur-[100px]" />
                <div className="absolute bottom-1/4 right-1/4 w-[400px] h-[400px] bg-rose-pine-love/5 rounded-full blur-[100px]" />
            </div>

            <div className="relative z-10 max-w-7xl w-full flex flex-col gap-16">
                {/* Header */}
                <div className="text-center space-y-6">
                    <motion.div
                        initial={{ opacity: 0, scale: 0.9 }}
                        whileInView={{ opacity: 1, scale: 1 }}
                        viewport={{ once: true }}
                        className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-rose-pine-highlight-low/10 border border-rose-pine-highlight-low/20 backdrop-blur-md mb-4"
                    >
                        <Sparkles className="w-4 h-4 text-rose-pine-gold" />
                        <span className="text-sm font-medium text-rose-pine-gold uppercase tracking-wider">
                            Built for Builders
                        </span>
                    </motion.div>

                    <motion.h2
                        initial={{ opacity: 0, y: 20 }}
                        whileInView={{ opacity: 1, y: 0 }}
                        viewport={{ once: true }}
                        className="text-5xl md:text-7xl font-black text-rose-pine-text leading-tight tracking-tight"
                    >
                        Human First.<br />
                        <span className="text-transparent bg-clip-text bg-gradient-to-r from-rose-pine-iris via-[#c4a7e7] to-rose-pine-love dark:from-rose-pine-iris dark:via-[#c4a7e7] dark:to-rose-pine-love drop-shadow-sm">AI Ready</span>.
                    </motion.h2>

                    <motion.p
                        initial={{ opacity: 0, y: 20 }}
                        whileInView={{ opacity: 1, y: 0 }}
                        viewport={{ once: true }}
                        transition={{ delay: 0.1 }}
                        className="text-xl md:text-2xl text-rose-pine-subtle max-w-3xl mx-auto leading-relaxed"
                    >
                        The Five DSL is engineered for speed and simplicity. Write contracts in minutes, not months. Readable enough for humans, concise enough for AI.
                    </motion.p>
                </div>

                {/* Deep Dive Grid */}
                <div className="grid md:grid-cols-2 gap-8 lg:gap-16 items-center">
                    {/* Left: Content */}
                    <div className="space-y-12">
                        <FeatureBlock
                            icon={<Bot />}
                            title="Concise Syntax"
                            description="Five scripts are 10x more concise than Rust or Solidity. This means AI agents can generate and manage entire protocol architectures within a single context window, reducing hallucinations."
                        />
                        <FeatureBlock
                            icon={<Terminal />}
                            title="Type-Safe Generation"
                            description="The compiler enforces strict constraints that LLMs often miss. Agents can write code, compile it, read the error message, and self-correct in a tight loop without human intervention."
                        />
                        <FeatureBlock
                            icon={<Cpu />}
                            title="Deterministic Execution"
                            description="No undefined behavior. An agent simulating a transaction knows exactly what will happen, making it safe for autonomous economic operations."
                        />
                    </div>

                    {/* Right: Visual */}
                    <motion.div
                        initial={{ opacity: 0, x: 20 }}
                        whileInView={{ opacity: 1, x: 0 }}
                        viewport={{ once: true }}
                        className="relative"
                    >
                        <div className="absolute -inset-4 bg-gradient-to-r from-rose-pine-iris/20 to-rose-pine-love/20 rounded-3xl blur-xl" />
                        <div className="relative rounded-2xl border border-white/10 bg-rose-pine-surface/90 dark:bg-[#121118]/90 backdrop-blur-xl p-6 shadow-2xl">
                            <div className="flex items-center gap-2 border-b border-white/5 pb-4 mb-4">
                                <div className="flex gap-1.5">
                                    <div className="w-3 h-3 rounded-full bg-rose-500/50" />
                                    <div className="w-3 h-3 rounded-full bg-amber-500/50" />
                                    <div className="w-3 h-3 rounded-full bg-emerald-500/50" />
                                </div>
                                <div className="text-xs text-rose-pine-muted font-mono ml-2">agent_context.v</div>
                            </div>
                            <pre className="font-mono text-xs md:text-sm leading-relaxed overflow-x-auto text-rose-pine-subtle">
                                <code>
                                    <span className="text-rose-pine-iris">account</span> <span className="text-rose-pine-gold">BondingCurve</span> {"{"}
                                    {"\n"}  <span className="text-rose-pine-text">supply:</span> <span className="text-rose-pine-love">u64</span>;
                                    {"\n"}  <span className="text-rose-pine-text">reserve:</span> <span className="text-rose-pine-love">u64</span>;
                                    {"\n"}{"}"}
                                    {"\n"}
                                    {"\n"}<span className="text-rose-pine-subtle">// AI generated logic</span>
                                    {"\n"}<span className="text-rose-pine-iris">pub</span> <span className="text-rose-pine-foam">calculate_price</span>(@state <span className="text-rose-pine-text">curve</span>) {"{"}
                                    {"\n"}  <span className="text-rose-pine-iris">return</span> <span className="text-rose-pine-text">curve.reserve</span> / <span className="text-rose-pine-text">curve.supply</span>;
                                    {"\n"}{"}"}
                                </code>
                            </pre>
                            <div className="mt-4 pt-4 border-t border-white/5 flex items-center justify-between text-xs">
                                <span className="text-rose-pine-muted">Context Usage</span>
                                <span className="text-emerald-400 font-mono">~100 tokens (High Efficiency)</span>
                            </div>
                        </div>
                    </motion.div>
                </div>
            </div>
        </section>
    );
}

function FeatureBlock({ icon, title, description }: { icon: React.ReactNode, title: string, description: string }) {
    return (
        <motion.div
            initial={{ opacity: 0, y: 10 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="flex gap-6"
        >
            <div className="flex-shrink-0 w-12 h-12 rounded-xl bg-rose-pine-surface/50 border border-white/5 flex items-center justify-center text-rose-pine-iris">
                {icon}
            </div>
            <div>
                <h3 className="text-xl font-bold text-rose-pine-text mb-2 tracking-tight">{title}</h3>
                <p className="text-rose-pine-subtle leading-relaxed font-medium">
                    {description}
                </p>
            </div>
        </motion.div>
    );
}
