"use client";

import React from "react";
import { motion } from "framer-motion";
import { Lock } from "lucide-react";
import { Section, MoatCard } from "@/components/ui/investor-components";

/**
 * MoatSection Component
 * Explains 5IVE Protocol's unforkable competitive advantages
 * Covers integrated ecosystem, architecture, bytecode format, and protocol stability
 */
const MoatSection = React.memo(function MoatSection() {
  return (
    <Section
      title="The Unforkable Moat"
      subtitle="5IVE isn't just a language. It's an integrated Layer 1.5 ecosystem."
      icon={<Lock className="w-8 h-8 text-rose-pine-love" />}
      color="love"
    >
      <div className="space-y-12">
        <div className="grid md:grid-cols-2 gap-8">
          <MoatCard
            title="1000 Programs, One Account"
            description="A single Moat account (10MB) can contain over 1000 distinct bytecode programs. AMMs, Lending protocols, Vaults, Escrows, DAOs, and Tokens all live together in one high-performance memory space."
            delay={0.1}
          />
          <MoatCard
            title="Universal Runtime"
            description="The 5VM is a portable execution layer that runs entirely on Solana L1, but is designed to run on any SVM chain. 5IVE succeeds only if Solana succeeds, helping it scale by compressing compute."
            delay={0.2}
          />
          <MoatCard
            title="Direct Bytecode Calls"
            description="Programs within a Moat call each other directly using internal byte code jumps. This bypasses CPI overhead, saving 1000+ CU per call and enabling complex composability at 0 cost."
            delay={0.3}
          />
          <MoatCard
            title="Protocol Stability"
            description="Versioned opcode set (Layer 1.5) ensures backward compatibility while enabling upgrades. Once a Moat is deployed, it is a permanent, unstoppable infrastructure."
            delay={0.4}
          />
        </div>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="p-8 rounded-3xl bg-rose-pine-love/5 border border-rose-pine-love/20"
        >
          <h3 className="text-2xl font-bold text-rose-pine-love mb-4">
            Unlock Entirely New Use Cases
          </h3>
          <p className="text-rose-pine-muted leading-relaxed">
            By combining storage and execution in a single Moat, developers can build complex
            systems that were previously impossible on Solana due to CU limits. Imagine an entire
            DeFi ecosystem (Dex + Lend + Stake + Vote) running in a single transaction.
            This is the power of the 5IVE Moat.
          </p>
        </motion.div>
      </div>
    </Section>
  );
});

MoatSection.displayName = "MoatSection";

export default MoatSection;
