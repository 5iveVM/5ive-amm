"use client";

import React from "react";
import { motion } from "framer-motion";
import { TrendingUp, Zap, Wallet, Banknote, Database, Flame, ArrowDown } from "lucide-react";

/**
 * TokenSection Component - Visual Upgrade
 * "The Wealth Machine" with animated energy flows.
 */
const TokenSection = React.memo(function TokenSection() {
  return (
    <section className="relative py-20 px-4 overflow-hidden">
      {/* Background Elements */}
      <div className="absolute inset-0 pointer-events-none">
        <div className="absolute top-1/4 left-1/4 w-[500px] h-[500px] bg-rose-pine-iris/5 rounded-full blur-[100px]" />
        <div className="absolute bottom-1/4 right-1/4 w-[400px] h-[400px] bg-rose-pine-love/5 rounded-full blur-[100px]" />
      </div>

      <div className="max-w-7xl mx-auto relative z-10">

        {/* Header Section */}
        <div className="text-center mb-12">
          <motion.div
            initial={{ opacity: 0, scale: 0.9 }}
            whileInView={{ opacity: 1, scale: 1 }}
            viewport={{ once: true }}
            className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-rose-pine-love/10 border border-white/5 mb-6"
          >
            <div className="w-2 h-2 rounded-full bg-rose-pine-love animate-pulse" />
            <span className="text-xs font-bold text-rose-pine-love tracking-widest uppercase">The Wealth Machine</span>
          </motion.div>

          <h3 className="text-4xl md:text-6xl font-black text-rose-pine-text mb-6">Converging Value Capture</h3>
          <p className="text-xl text-rose-pine-muted max-w-2xl mx-auto font-light leading-relaxed">
            Every agent deployed, every protocol replaced, and every game engine hosted drives value to a single point.
          </p>
        </div>

        {/* The Machine Visualization */}
        <div className="relative max-w-5xl mx-auto flex flex-col items-center">

          {/* Row 1: Inputs */}
          <div className="grid md:grid-cols-3 gap-6 w-full relative z-10">
            <ValueCard
              icon={<Zap className="w-5 h-5 text-rose-pine-iris" />}
              title="Agent Compute"
              desc="Humans & Agents paying protocol fees for instant on-chain deployment."
              color="iris"
            />
            <ValueCard
              icon={<TrendingUp className="w-5 h-5 text-rose-pine-gold" />}
              title="Legacy Displacement"
              desc="Legacy protocols must integrate 5ive or be displaced by small, fast teams."
              color="gold"
            />
            <ValueCard
              icon={<Database className="w-5 h-5 text-rose-pine-foam" />}
              title="Moat Staking"
              desc="Staking determines official Moat inclusion. Studios deploy custom Moats."
              color="foam"
            />
          </div>

          {/* Connector 1: Converging */}
          <ConvergingConnector />

          {/* Row 2: The Aggregator (Treasury) */}
          <div className="relative z-10 w-full flex justify-center">
            <div className="relative group w-full max-w-2xl bg-white/80 dark:bg-[#191724] border border-rose-pine-highlight-med/20 p-1 rounded-3xl shadow-[0_0_50px_rgba(235,111,146,0.1)]">
              <div className="absolute inset-0 bg-gradient-to-r from-rose-pine-iris via-rose-pine-gold to-rose-pine-love opacity-20 blur-xl group-hover:opacity-40 transition-opacity duration-500" />

              <div className="relative bg-white dark:bg-[#1f1d2e] rounded-[22px] p-8 flex flex-col md:flex-row items-center gap-8 text-center md:text-left overflow-hidden">
                {/* Top Connector Node for Treasury */}
                <div className="absolute -top-1.5 left-1/2 -translate-x-1/2 w-3 h-3 rounded-full bg-[#26233a] border border-white/20" />

                {/* Flow Animation Inside */}
                <div className="absolute inset-0 bg-[url('/grid-pattern.svg')] opacity-10" />

                <div className="w-16 h-16 rounded-2xl bg-gradient-to-br from-rose-pine-love/20 to-transparent border border-rose-pine-love/30 flex items-center justify-center shrink-0 shadow-[0_0_30px_rgba(235,111,146,0.2)]">
                  <Banknote className="w-8 h-8 text-rose-pine-love" />
                </div>

                <div className="flex-1">
                  <div className="text-xs font-bold text-rose-pine-muted uppercase tracking-wider mb-2">Protocol Treasury</div>
                  <h4 className="text-2xl font-black text-rose-pine-text dark:text-white mb-2">The Consolidation Point</h4>
                  <p className="text-rose-pine-subtle">
                    100% of all protocol revenue is programmatically routed here. No leaks. No middlemen.
                  </p>
                </div>

                <div className="shrink-0 flex flex-col items-center gap-2">
                  <div className="text-3xl font-black text-transparent bg-clip-text bg-gradient-to-r from-rose-pine-gold to-rose-pine-love">
                    100%
                  </div>
                  <div className="text-[10px] font-bold text-rose-pine-muted uppercase">Efficiency</div>
                </div>

                {/* Bottom Connector Node for Treasury */}
                <div className="absolute -bottom-1.5 left-1/2 -translate-x-1/2 w-3 h-3 rounded-full bg-[#26233a] border border-white/20" />
              </div>
            </div>
          </div>

          {/* Connector 2: Straight Down */}
          <StraightConnector />

          {/* Row 3: The Output (Burn) */}
          <div className="relative z-10 w-full flex justify-center">
            <div className="relative w-full max-w-lg">
              <div className="absolute -inset-1 bg-gradient-to-b from-rose-pine-love to-purple-600 rounded-full blur-2xl opacity-30 animate-pulse" />
              <div className="relative bg-white/90 dark:bg-black/80 backdrop-blur-xl border border-rose-pine-love/30 rounded-3xl p-8 text-center flex flex-col items-center">

                {/* Top Connector Node for Burn */}
                <div className="absolute -top-1.5 left-1/2 -translate-x-1/2 w-3 h-3 rounded-full bg-[#26233a] border border-white/20" />

                <div className="w-24 h-24 rounded-full bg-rose-pine-base dark:bg-black border border-rose-pine-love flex items-center justify-center mb-6 shadow-[0_0_40px_rgba(235,111,146,0.4)] relative overflow-hidden group">
                  <div className="absolute inset-0 bg-rose-pine-love/20 blur-xl animate-pulse" />
                  <Flame className="w-10 h-10 text-rose-pine-love relative z-10" />

                  {/* Particle Effects */}
                  {[...Array(6)].map((_, i) => (
                    <motion.div
                      key={i}
                      className="absolute w-1 h-1 bg-white rounded-full"
                      initial={{ opacity: 0, y: 0, x: 0 }}
                      animate={{
                        opacity: [0, 1, 0],
                        y: -40,
                        x: (Math.random() - 0.5) * 40
                      }}
                      transition={{
                        duration: 1 + Math.random(),
                        repeat: Infinity,
                        delay: Math.random() * 2
                      }}
                    />
                  ))}
                </div>

                <h4 className="text-3xl font-black text-rose-pine-text mb-2 tracking-tight">Systematic Supply Decay</h4>
                <p className="text-rose-pine-muted font-light leading-relaxed">
                  SOL, USDC & BONK revenue is <span className="text-rose-pine-love font-medium">auto-swapped</span> to Buy & Burn <span className="text-rose-pine-muted text-xs">(Launching Soon)</span>.<br />
                  Every interaction reduces supply. <span className="text-rose-pine-love font-bold">Programmatic Scarcity.</span>
                </p>
              </div>
            </div>
          </div>

        </div>

      </div>
    </section>
  );
});

TokenSection.displayName = "TokenSection";
export default TokenSection;

function ConvergingConnector() {
  return (
    <div className="w-full h-32 relative overflow-visible pointer-events-none hidden md:block mt-[-2px] mb-[-2px] z-0">
      <svg className="w-full h-full drop-shadow-[0_0_10px_rgba(196,167,231,0.3)]" viewBox="0 0 100 100" preserveAspectRatio="none">
        <defs>
          <linearGradient id="flowGradient" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor="#c4a7e7" stopOpacity="0" />
            <stop offset="50%" stopColor="#c4a7e7" stopOpacity="1" />
            <stop offset="100%" stopColor="#eb6f92" stopOpacity="1" />
          </linearGradient>
        </defs>

        {/* STATIC BASE PATHS (DIM) */}
        <path d="M 16.66 0 C 16.66 40, 50 40, 50 100" fill="none" stroke="#c4a7e7" strokeWidth="0.5" strokeOpacity="0.2" vectorEffect="non-scaling-stroke" />
        <path d="M 50 0 C 50 40 50 60 50 98" fill="none" stroke="#c4a7e7" strokeWidth="0.5" strokeOpacity="0.2" vectorEffect="non-scaling-stroke" />
        <path d="M 83.33 0 C 83.33 40, 50 40, 50 100" fill="none" stroke="#c4a7e7" strokeWidth="0.5" strokeOpacity="0.2" vectorEffect="non-scaling-stroke" />

        {/* DYNAMIC FLOW PATHS (ANIMATED) */}
        <motion.path
          d="M 16.66 0 C 16.66 40, 50 40, 50 100"
          fill="none" stroke="url(#flowGradient)" strokeWidth="2" strokeDasharray="10 10" strokeLinecap="round"
          vectorEffect="non-scaling-stroke"
          initial={{ strokeDashoffset: 0 }}
          animate={{ strokeDashoffset: -200 }}
          transition={{ duration: 1.5, repeat: Infinity, ease: "linear" }}
        />
        <motion.path
          d="M 50 0 C 50 40 50 60 50 98"
          fill="none" stroke="url(#flowGradient)" strokeWidth="2" strokeDasharray="10 10" strokeLinecap="round"
          vectorEffect="non-scaling-stroke"
          initial={{ strokeDashoffset: 0 }}
          animate={{ strokeDashoffset: -200 }}
          transition={{ duration: 1.5, repeat: Infinity, ease: "linear" }}
        />
        <motion.path
          d="M 83.33 0 C 83.33 40, 50 40, 50 100"
          fill="none" stroke="url(#flowGradient)" strokeWidth="2" strokeDasharray="10 10" strokeLinecap="round"
          vectorEffect="non-scaling-stroke"
          initial={{ strokeDashoffset: 0 }}
          animate={{ strokeDashoffset: -200 }}
          transition={{ duration: 1.5, repeat: Infinity, ease: "linear" }}
        />
      </svg>
    </div>
  )
}

function StraightConnector() {
  return (
    <div className="w-full h-24 relative overflow-visible pointer-events-none hidden md:block mt-[-2px] mb-[-2px] z-0">
      <svg className="w-full h-full drop-shadow-[0_0_10px_rgba(235,111,146,0.3)]" viewBox="0 0 100 100" preserveAspectRatio="none">
        <defs>
          <linearGradient id="burnGradient" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor="#eb6f92" stopOpacity="1" />
            <stop offset="100%" stopColor="#b4637a" stopOpacity="0.5" />
          </linearGradient>
        </defs>

        {/* Static Base */}
        <path d="M 50 0 L 50 100" fill="none" stroke="#eb6f92" strokeWidth="1" strokeOpacity="0.2" vectorEffect="non-scaling-stroke" />

        {/* Dynamic Beam */}
        <motion.path
          d="M 50 0 L 50 100"
          fill="none" stroke="url(#burnGradient)" strokeWidth="4" strokeLinecap="round"
          vectorEffect="non-scaling-stroke"
          initial={{ pathLength: 0, opacity: 0 }}
          whileInView={{ pathLength: 1, opacity: 1 }}
          transition={{ duration: 0.5 }}
        />

        {/* Pulsing Core */}
        <motion.path
          d="M 50 0 L 50 100"
          fill="none" stroke="white" strokeWidth="1" strokeOpacity="0.5"
          vectorEffect="non-scaling-stroke"
          animate={{ opacity: [0.5, 1, 0.5] }}
          transition={{ duration: 1.5, repeat: Infinity }}
        />
      </svg>
    </div>
  )
}

function ValueCard({ icon, title, desc, color }: { icon: React.ReactNode, title: string, desc: string, color: string }) {
  const colors: Record<string, string> = {
    iris: "hover:border-rose-pine-iris/50 hover:shadow-[0_0_30px_rgba(196,167,231,0.15)]",
    gold: "hover:border-rose-pine-gold/50 hover:shadow-[0_0_30px_rgba(246,193,119,0.15)]",
    foam: "hover:border-rose-pine-foam/50 hover:shadow-[0_0_30px_rgba(156,207,216,0.15)]",
  }

  return (
    <motion.div
      whileHover={{ y: -5 }}
      className={`relative flex flex-col items-center text-center p-6 rounded-2xl border border-white/5 bg-white/60 dark:bg-[#191724]/60 backdrop-blur-md transition-all duration-300 ${colors[color]}`}
    >
      <div className="w-10 h-10 rounded-full bg-white/5 flex items-center justify-center mb-4 border border-white/5 shadow-inner">
        {icon}
      </div>
      <h4 className="text-lg font-bold text-rose-pine-text mb-2">{title}</h4>
      <p className="text-sm text-rose-pine-muted leading-relaxed font-light">{desc}</p>

      {/* Connector Node */}
      <div className="absolute -bottom-1.5 left-1/2 -translate-x-1/2 w-3 h-3 rounded-full bg-[#26233a] border border-white/20" />
    </motion.div>
  )
}
