"use client";

import React from "react";
import { Building2, Coins, BarChart3, Zap, Users } from "lucide-react";
import { Section, UseCaseCard } from "@/components/ui/investor-components";

/**
 * UseCasesSection Component
 * Showcases real-world applications optimized by Five Protocol
 * Includes dynamic NFTs, DeFi, gaming, and enterprise governance use cases
 */
const UseCasesSection = React.memo(function UseCasesSection() {
  return (
    <Section
      title="Designed For Real Applications"
      subtitle="Five Protocol optimizes for the highest-value blockchain use cases."
      icon={<Building2 className="w-8 h-8 text-rose-pine-love" />}
      color="love"
    >
      <div className="grid md:grid-cols-2 gap-8 w-full">
        <UseCaseCard
          title="Dynamic NFTs & Tickets"
          icon={<Coins className="w-6 h-6" />}
          description="Update NFT metadata, fighter stats, and game items in real-time with sub-cent costs. BKFC fighter cards demonstrate the economics."
          color="love"
          delay={0.1}
        />
        <UseCaseCard
          title="Decentralized Finance"
          icon={<BarChart3 className="w-6 h-6" />}
          description="Complete DEX implementation in Five DSL. AMMs, liquidity pools, and token swaps with minimal transaction costs."
          color="gold"
          delay={0.2}
        />
        <UseCaseCard
          title="High-Frequency Gaming"
          icon={<Zap className="w-6 h-6" />}
          description="Real-time player progression, item updates, and state changes without the overhead of traditional blockchain constraints."
          color="iris"
          delay={0.3}
        />
        <UseCaseCard
          title="Enterprise Governance"
          icon={<Users className="w-6 h-6" />}
          description="On-chain voting, proposal management, and treasury control at institutional scale without prohibitive costs."
          color="foam"
          delay={0.4}
        />
      </div>
    </Section>
  );
});

UseCasesSection.displayName = "UseCasesSection";

export default UseCasesSection;
