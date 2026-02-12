"use client";

import React from "react";
import { motion } from "framer-motion";

/**
 * HeroSection Component
 * Main hero section for the investor page with animated gradient title
 * Displays the core 5IVE Protocol thesis with key metrics
 */
const HeroSection = React.memo(function HeroSection() {
  return (
    <section className="relative min-h-[60vh] flex flex-col justify-center items-center px-4 w-full max-w-7xl mx-auto pt-24 pb-20">
      {/* Background Elements */}
      <div className="absolute inset-0 pointer-events-none overflow-hidden">
        <div className="absolute top-1/4 left-1/4 w-[600px] h-[600px] bg-rose-pine-iris/5 rounded-full blur-[100px] will-change-transform translate-z-0" />
        <div className="absolute bottom-1/4 right-1/4 w-[500px] h-[500px] bg-rose-pine-love/5 rounded-full blur-[100px] will-change-transform translate-z-0" />
      </div>
      <motion.div
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ duration: 0.8, ease: "circOut" }}
        className="flex flex-col items-center text-center gap-6"
      >
        <h1 className="text-6xl md:text-8xl font-black tracking-tighter leading-none bg-clip-text text-transparent bg-gradient-to-b from-rose-pine-text via-rose-pine-iris to-rose-pine-love dark:from-rose-pine-text dark:via-rose-pine-iris dark:to-rose-pine-love drop-shadow-2xl">
          The 5IVE Thesis.
        </h1>
        <p className="text-xl md:text-2xl font-medium text-rose-pine-subtle max-w-3xl">
          Not a Layer 2. The world's first <span className="text-rose-pine-gold font-bold">Layer 1.5</span>.<br />
          A hyper-optimized execution layer embedded directly into Solana L1. <br />
          <span className="text-rose-pine-love font-bold">It changes everything.</span>
        </p>
      </motion.div>
    </section>
  );
});

HeroSection.displayName = "HeroSection";

export default HeroSection;
