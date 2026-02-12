"use client";

import React from "react";
import { motion } from "framer-motion";
import { TrendingUp, Globe, FileCode, Coins, Smile, Sparkles, Box, ArrowRight, Import } from "lucide-react";
import { useMarketData } from "@/contexts/MarketDataContext";

/**
 * AgentEconomy Component
 * "The Infrastructure for the Agent Age"
 * Focus: Market Size, Growth (J-Curve), and Capabilities.
 */
const AgentEconomy = React.memo(function AgentEconomy() {
    const { price, marketCap } = useMarketData();
    const scaleCost = price ? (0.8 * price).toFixed(0) : "100";
    const formattedMarketCap = marketCap ? `$${(marketCap / 1e9).toFixed(1)}B` : "$100B+";

    return (
        <section className="relative py-20 px-4 overflow-hidden">
            {/* Background Elements */}
            <div className="absolute inset-0 pointer-events-none">
                <div className="absolute top-1/4 left-1/4 w-[500px] h-[500px] bg-rose-pine-iris/5 rounded-full blur-[100px]" />
                <div className="absolute bottom-1/4 right-1/4 w-[400px] h-[400px] bg-rose-pine-love/5 rounded-full blur-[100px]" />
            </div>

            <div className="max-w-7xl mx-auto relative z-10">

                {/* Header Section */}
                <div className="text-center mb-16">
                    <motion.div
                        initial={{ opacity: 0, scale: 0.9 }}
                        whileInView={{ opacity: 1, scale: 1 }}
                        viewport={{ once: true }}
                        className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-rose-pine-highlight-low/10 border border-white/5 mb-6"
                    >
                        <div className="w-2 h-2 rounded-full bg-rose-pine-gold animate-pulse" />
                        <span className="text-xs font-bold text-rose-pine-gold tracking-widest uppercase">Global Access</span>
                    </motion.div>

                    <h2 className="text-5xl md:text-7xl font-black text-rose-pine-text mb-6 tracking-tight">
                        Infrastructure for <span className="text-transparent bg-clip-text bg-gradient-to-r from-rose-pine-gold via-rose-pine-iris to-rose-pine-love">Everyone</span>
                    </h2>
                    <p className="text-xl text-rose-pine-muted max-w-3xl mx-auto font-light leading-relaxed">
                        From Lagos to London, indie devs to institutions. <br className="hidden md:block" />
                        The first platform cheap enough for anyone to deploy their own infrastructure.
                    </p>
                </div>

                <div className="grid lg:grid-cols-2 gap-16 items-center">

                    {/* Left Col: The "Super Bullish" Chart */}
                    <div className="space-y-8">
                        <div className="bg-white/80 dark:bg-[#191724]/60 backdrop-blur-xl border border-white/5 rounded-3xl p-8 h-[500px] flex flex-col relative overflow-hidden group">
                            <div className="absolute inset-0 bg-gradient-to-b from-transparent to-rose-pine-surface/10 pointer-events-none" />

                            <div className="flex justify-between items-start mb-8 relative z-10">
                                <div>
                                    <h3 className="text-2xl font-bold text-rose-pine-text mb-1">On-Chain Deployments</h3>
                                    <p className="text-sm text-rose-pine-muted">Projected Protocol Launches (2025-2030)</p>
                                </div>
                                <div className="px-3 py-1 bg-rose-pine-foam/10 border border-rose-pine-foam/20 rounded-lg">
                                    <span className="text-xs font-bold text-rose-pine-foam">+10,000% Growth</span>
                                </div>
                            </div>

                            {/* The J-Curve Chart */}
                            <div className="flex-1 relative w-full flex items-end">
                                <ChartVisual />
                            </div>

                            {/* Legend */}
                            <div className="flex gap-6 mt-6 pt-6 border-t border-white/5 relative z-10">
                                <div className="flex items-center gap-2">
                                    <div className="w-3 h-3 rounded-full bg-rose-pine-subtle" />
                                    <span className="text-xs text-rose-pine-muted font-mono uppercase">Human TXs</span>
                                </div>
                                <div className="flex items-center gap-2">
                                    <div className="w-3 h-3 rounded-full bg-rose-pine-gold shadow-[0_0_10px_rgba(246,193,119,0.5)]" />
                                    <span className="text-xs text-rose-pine-gold font-mono uppercase font-bold">5IVE Agent Actions</span>
                                </div>
                            </div>
                        </div>

                        {/* Chart Caption/Stat */}
                        <div className="grid grid-cols-2 gap-4">
                            <div className="p-6 rounded-2xl bg-white/60 dark:bg-[#191724]/40 border border-white/5">
                                <div className="text-3xl font-black text-rose-pine-text mb-1">{formattedMarketCap}</div>
                                <div className="text-xs text-rose-pine-muted uppercase tracking-wider">Addressable Market</div>
                            </div>
                            <div className="p-6 rounded-2xl bg-white/60 dark:bg-[#191724]/40 border border-white/5">
                                <div className="text-3xl font-black text-rose-pine-love mb-1">Deflationary</div>
                                <div className="text-xs text-rose-pine-muted uppercase tracking-wider">Token Model</div>
                            </div>
                        </div>
                    </div>

                    {/* Right Col: The Agent Capabilities (Cards) */}
                    <div className="space-y-6">
                        <UtilityCard
                            title="Extended Reach"
                            description="With the 5ive SDK and MCP Server, developers and AI agents alike can write, deploy, and interact with smart contracts natively."
                            icon={<ReachAnim />}
                            color="text-rose-pine-foam"
                            delay={0.1}
                        />
                        <UtilityCard
                            title="Cost to Scale"
                            description={`Legacy Solana costs $${scaleCost}+ to deploy. 5IVE costs pennies. Agents can spawn thousands of disposable contracts without burning their budget.`}
                            icon={<TrendingUp className="w-6 h-6" />}
                            color="text-rose-pine-iris"
                            delay={0.2}
                        />
                    </div>

                </div>

                <div className="mt-20">
                    <ProgramTokenShowcase />
                </div>
            </div>
        </section>
    );
});

AgentEconomy.displayName = "AgentEconomy";
export default AgentEconomy;


// --- Visual Components ---

function ProgramTokenShowcase() {
    return (
        <motion.div
            initial={{ opacity: 0, y: 40 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="rounded-3xl bg-white/80 dark:bg-[#191724]/60 backdrop-blur-xl border border-white/5 p-12 lg:p-16 flex flex-col relative shadow-2xl"
        >
            <div className="absolute inset-0 bg-gradient-to-b from-rose-pine-gold/5 via-transparent to-rose-pine-love/5 pointer-events-none" />

            <div className="relative z-10 flex flex-col items-center text-center">
                <div className="flex items-center gap-3 mb-6">
                    <div className="w-12 h-12 rounded-2xl bg-rose-pine-gold/20 flex items-center justify-center border border-rose-pine-gold/20 shadow-[0_0_20px_rgba(246,193,119,0.3)]">
                        <Sparkles className="w-6 h-6 text-rose-pine-gold" />
                    </div>
                    <h3 className="text-4xl md:text-5xl font-black text-rose-pine-text tracking-tight">
                        Programs = <span className="text-rose-pine-gold">Tokens</span>
                    </h3>
                </div>

                <p className="max-w-3xl mx-auto text-rose-pine-muted text-lg leading-relaxed mb-12 font-light">
                    In 5IVE, a "program" isn't a separate executable account. It's a token.
                    This means you can transfer ownership of a smart contract as easily as sending USDC.
                    <span className="text-rose-pine-text font-medium"> Earn royalties</span> on code usage <span className="text-rose-pine-muted text-sm">(Coming Soon)</span>, trade algorithms on AMMs, and speculate on the utility of an agent's logic.
                </p>

                <div className="grid md:grid-cols-3 gap-8 w-full max-w-5xl">
                    <FeatureBox
                        title="Import"
                        subtitle="Import logic like a library"
                        icon={<ImportAnim />}
                        color="bg-rose-pine-foam/10 border-rose-pine-foam/20 text-rose-pine-foam"
                        delay={0.2}
                    />
                    <FeatureBox
                        title="Trade"
                        subtitle="Buy/Sell contract logic"
                        icon={<TradeAnim />}
                        color="bg-rose-pine-love/10 border-rose-pine-love/20 text-rose-pine-love"
                        delay={0.3}
                    />
                    <FeatureBox
                        title="Evolve"
                        subtitle="Governance upgrades"
                        icon={<EvolveAnim />}
                        color="bg-rose-pine-iris/10 border-rose-pine-iris/20 text-rose-pine-iris"
                        delay={0.4}
                    />
                </div>
            </div>
        </motion.div>
    );
}

function FeatureBox({ title, subtitle, icon, color, delay }: { title: string, subtitle: string, icon: React.ReactNode, color: string, delay: number }) {
    return (
        <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ delay }}
            className={`group rounded-2xl border p-8 flex flex-col items-center gap-4 transition-all duration-300 hover:scale-105 ${color.replace('text-', 'border-').split(' ')[1]} bg-white/60 dark:bg-[#1f1d2e]/50 backdrop-blur-sm`}
        >
            <div className={`w-16 h-16 rounded-xl flex items-center justify-center text-3xl ${color} bg-current/10 border border-current/20 shadow-[0_0_15px_currentColor]`}>
                {icon}
            </div>
            <div className="text-center">
                <h4 className="text-xl font-bold text-rose-pine-text mb-2 tracking-tight">{title}</h4>
                <p className="text-sm text-rose-pine-muted">{subtitle}</p>
            </div>
        </motion.div>
    )
}

function ImportAnim() {
    return (
        <div className="relative w-8 h-8 flex items-center justify-center text-rose-pine-foam">
            <div className="absolute inset-0 grid grid-cols-2 gap-1 opacity-20">
                <div className="bg-current rounded-[1px]" />
                <div className="bg-current rounded-[1px]" />
                <div className="bg-current rounded-[1px]" />
                <div className="bg-current rounded-[1px]" />
            </div>
            <motion.div
                className="absolute w-4 h-4 bg-current rounded-sm shadow-[0_0_10px_currentColor]"
                initial={{ y: -20, opacity: 0, scale: 0.5 }}
                animate={{ y: 0, opacity: 1, scale: 1 }}
                transition={{ duration: 2, repeat: Infinity, repeatDelay: 0.5, ease: "backOut" }}
            />
            <motion.div
                className="absolute inset-0 border-2 border-current rounded-md"
                initial={{ opacity: 0, scale: 1 }}
                animate={{ opacity: [0, 1, 0], scale: 1.2 }}
                transition={{ duration: 2, repeat: Infinity, repeatDelay: 0.5, delay: 0.2 }}
            />
        </div>
    )
}

function TradeAnim() {
    return (
        <div className="relative w-8 h-8 flex items-end justify-center gap-1 pb-1 text-rose-pine-love">
            {[0, 1, 2].map((i) => (
                <motion.div
                    key={i}
                    className="w-1.5 bg-current rounded-t-[1px]"
                    animate={{
                        height: [4 + i * 4, 12 + i * 2, 8 + i * 4, 16 + i * 3],
                        opacity: [0.5, 1, 0.7, 1]
                    }}
                    transition={{ duration: 2, repeat: Infinity, repeatType: "mirror", delay: i * 0.1 }}
                />
            ))}
            <svg className="absolute inset-0 w-full h-full overflow-visible">
                <motion.path
                    d="M 0,25 L 10,20 L 20,10 L 32,0"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    initial={{ pathLength: 0, opacity: 0 }}
                    animate={{ pathLength: 1, opacity: 1 }}
                    transition={{ duration: 2, repeat: Infinity, delay: 0.5 }}
                />
            </svg>
        </div>
    )
}

function EvolveAnim() {
    return (
        <div className="relative w-8 h-8 flex items-center justify-center text-rose-pine-iris">
            <motion.div
                className="absolute inset-0 border-2 border-current rounded-full"
                style={{ borderRadius: "40% 60% 70% 30% / 40% 50% 60% 50%" }}
                animate={{
                    rotate: 360,
                    borderRadius: [
                        "40% 60% 70% 30% / 40% 50% 60% 50%",
                        "60% 40% 30% 70% / 60% 50% 40% 50%",
                        "40% 60% 70% 30% / 40% 50% 60% 50%"
                    ]
                }}
                transition={{ duration: 4, repeat: Infinity, ease: "linear" }}
            />
            <motion.div
                className="w-3 h-3 bg-current rounded-full shadow-[0_0_10px_currentColor]"
                animate={{ scale: [1, 1.5, 1], opacity: [0.5, 1, 0.5] }}
                transition={{ duration: 2, repeat: Infinity }}
            />
            <motion.div
                className="absolute top-0 w-1 h-1 bg-current rounded-full"
                animate={{ y: [-10, -20], opacity: [1, 0] }}
                transition={{ duration: 1.5, repeat: Infinity, delay: 0.5 }}
            />
        </div>
    )
}

function ChartVisual() {
    return (
        <svg className="w-full h-full overflow-visible" preserveAspectRatio="none" viewBox="0 0 100 100">
            <defs>
                <linearGradient id="chartGradient" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stopColor="#eb6f92" stopOpacity="0.5" />
                    <stop offset="100%" stopColor="#eb6f92" stopOpacity="0" />
                </linearGradient>
                <linearGradient id="lineGradient" x1="0" y1="0" x2="1" y2="0">
                    <stop offset="0%" stopColor="#9ccfd8" />
                    <stop offset="50%" stopColor="#c4a7e7" />
                    <stop offset="100%" stopColor="#eb6f92" />
                </linearGradient>
            </defs>

            {/* Grid Lines */}
            <line x1="0" y1="25" x2="100" y2="25" stroke="rgba(255,255,255,0.05)" strokeWidth="0.5" strokeDasharray="2 2" />
            <line x1="0" y1="50" x2="100" y2="50" stroke="rgba(255,255,255,0.05)" strokeWidth="0.5" strokeDasharray="2 2" />
            <line x1="0" y1="75" x2="100" y2="75" stroke="rgba(255,255,255,0.05)" strokeWidth="0.5" strokeDasharray="2 2" />

            {/* The Curve */}
            <motion.path
                d="M 0,90 Q 30,85 50,70 T 100,10"
                fill="url(#chartGradient)"
                stroke="url(#lineGradient)"
                strokeWidth="2"
                strokeLinecap="round"
                initial={{ pathLength: 0, opacity: 0 }}
                whileInView={{ pathLength: 1, opacity: 1 }}
                viewport={{ once: true }}
                transition={{ duration: 1.5, ease: "easeOut" }}
            />

            {/* Glowing Tip */}
            <motion.circle
                cx="100" cy="10" r="2"
                fill="#eb6f92"
                initial={{ opacity: 0, scale: 0 }}
                whileInView={{ opacity: 1, scale: 1 }}
                transition={{ delay: 1.5, duration: 0.5 }}
                className="drop-shadow-[0_0_8px_#eb6f92]"
            />
        </svg>
    )
}


function UtilityCard({ title, description, icon, color, delay }: { title: string, description: string, icon: React.ReactNode, color: string, delay: number }) {
    return (
        <motion.div
            initial={{ opacity: 0, x: 20 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            transition={{ delay }}
            className="group flex gap-6 p-6 rounded-2xl bg-white/60 dark:bg-[#191724]/40 border border-white/5 hover:bg-white/80 dark:hover:bg-[#191724]/60 hover:border-white/10 transition-all duration-300"
        >
            <div className={`shrink-0 w-12 h-12 rounded-xl bg-white/5 flex items-center justify-center ${color} border border-white/5 group-hover:scale-110 transition-transform shadow-[0_0_20px_rgba(0,0,0,0.3)]`}>
                {icon}
            </div>
            <div>
                <h3 className="text-xl font-bold text-rose-pine-text mb-2 group-hover:text-white transition-colors">{title}</h3>
                <p className="text-sm text-rose-pine-muted leading-relaxed font-light group-hover:text-rose-pine-subtle transition-colors">
                    {description}
                </p>
            </div>
        </motion.div>
    );
}



function ReachAnim() {
    return (
        <div className="relative w-6 h-6">
            <motion.div
                animate={{ rotate: 360 }}
                transition={{ duration: 10, repeat: Infinity, ease: "linear" }}
            >
                <Globe className="w-6 h-6 text-rose-pine-foam" />
            </motion.div>
            <motion.div
                className="absolute inset-0 rounded-full border border-rose-pine-foam/50"
                animate={{ scale: [1, 1.5], opacity: [0.5, 0] }}
                transition={{ duration: 2, repeat: Infinity }}
            />
        </div>
    )
}
