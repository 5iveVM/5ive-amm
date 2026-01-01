"use client";

import React from "react";
import { motion } from "framer-motion";
import { DollarSign } from "lucide-react";
import { Section, MetricCard } from "@/components/ui/investor-components";

/**
 * EconomicsSection Component
 * Displays the economic advantages of Five Protocol
 * Includes cost comparison, key metrics, and real-world examples
 */
const EconomicsSection = React.memo(function EconomicsSection() {
  return (
    <Section
      title="The Economics"
      subtitle="Why Five Protocol changes the game for blockchain development."
      icon={<DollarSign className="w-8 h-8 text-rose-pine-gold" />}
      color="gold"
    >
      <div className="space-y-12">
        {/* Cost Comparison */}
        <div className="grid md:grid-cols-2 gap-8">
          <motion.div
            whileHover={{ scale: 1.02 }}
            initial={{ opacity: 0, x: -20 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            className="p-8 rounded-3xl border border-rose-pine-hl-med/20 bg-rose-pine-surface/40 backdrop-blur-md"
          >
            <p className="text-sm uppercase tracking-wider text-rose-pine-subtle mb-4">
              Traditional (Anchor)
            </p>
            <div className="text-6xl font-black text-rose-pine-subtle mb-4">
              $126
            </div>
            <p className="text-rose-pine-muted">Cost per smart contract deployment to Solana</p>
            <div className="mt-6 text-xs uppercase text-rose-pine-muted opacity-60">
              1.26 SOL at current prices
            </div>
          </motion.div>

          <motion.div
            whileHover={{ scale: 1.02 }}
            initial={{ opacity: 0, x: 20 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            transition={{ delay: 0.1 }}
            className="p-8 rounded-3xl border-2 border-rose-pine-gold/40 bg-gradient-to-br from-rose-pine-gold/10 to-transparent backdrop-blur-md"
          >
            <p className="text-sm uppercase tracking-wider text-rose-pine-gold mb-4 font-bold">
              5IVE Protocol
            </p>
            <div className="text-6xl font-black text-rose-pine-gold mb-4">$0.002</div>
            <p className="text-rose-pine-text font-medium">Cost per smart contract deployment</p>
            <div className="mt-6 text-xs uppercase text-rose-pine-gold opacity-80 font-bold">
              99.8% Reduction
            </div>
          </motion.div>
        </div>

        {/* Key Metrics Grid */}
        <div className="grid md:grid-cols-3 gap-6">
          <MetricCard
            label="Bytecode Efficiency"
            value="800x"
            description="Smaller file sizes enable affordable scaling"
            delay={0.1}
          />
          <MetricCard
            label="Compute Reduction"
            value="70%"
            description="Less compute units per transaction"
            delay={0.2}
          />
          <MetricCard
            label="Dev Time Savings"
            value="30x"
            description="2-4 weeks vs 6-18 months to first dapp"
            delay={0.3}
          />
        </div>

        {/* Real World Example */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="p-8 rounded-3xl border border-rose-pine-gold/30 bg-rose-pine-gold/5 backdrop-blur-md"
        >
          <h3 className="text-2xl font-bold text-rose-pine-gold mb-6">
            Real-World Example: BKFC Dynamic NFTs
          </h3>
          <div className="grid md:grid-cols-3 gap-6">
            <div className="flex flex-col gap-2">
              <p className="text-sm uppercase tracking-wider text-rose-pine-subtle">
                20,000 Fighter Tickets
              </p>
              <p className="text-3xl font-bold text-rose-pine-gold">$40</p>
              <p className="text-xs text-rose-pine-muted">Deploy + dynamic updates with 5IVE</p>
            </div>
            <div className="flex items-center justify-center text-rose-pine-subtle">
              <div className="text-4xl">→</div>
            </div>
            <div className="flex flex-col gap-2">
              <p className="text-sm uppercase tracking-wider text-rose-pine-subtle">
                Traditional Approach
              </p>
              <p className="text-3xl font-bold text-rose-pine-subtle">$10K-50K</p>
              <p className="text-xs text-rose-pine-muted">Setup + infrastructure costs</p>
            </div>
          </div>
        </motion.div>
      </div>
    </Section>
  );
});

EconomicsSection.displayName = "EconomicsSection";

export default EconomicsSection;
