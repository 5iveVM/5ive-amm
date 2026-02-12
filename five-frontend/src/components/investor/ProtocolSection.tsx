"use client";

import React from "react";
import { Layers, Shield, Zap, Globe } from "lucide-react";
import { Section, Card } from "@/components/ui/investor-components";

/**
 * ProtocolSection Component
 * Showcases the core features and benefits of the 5IVE Protocol
 * Highlights integrated ecosystem, production-ready status, and execution speed
 */
const ProtocolSection = React.memo(function ProtocolSection() {
  return (
    <Section
      title="The Protocol"
      subtitle="A complete, integrated runtime for high-performance Solana applications."
      icon={<Globe className="w-8 h-8 text-rose-pine-foam" />}
      color="foam"
    >
      <div className="grid md:grid-cols-3 gap-8 w-full">
        <Card
          title="Complete Ecosystem"
          icon={<Layers className="w-6 h-6 text-rose-pine-foam" />}
          description="5IVE DSL language + optimizing compiler + zero-allocation VM + Solana program + JavaScript SDK. No fragmentation, no integration nightmares."
          delay={0.1}
        />
        <Card
          title="Production-Ready"
          icon={<Shield className="w-6 h-6 text-rose-pine-foam" />}
          description="Real Solana integration with actual clock sysvar, rent calculations, PDA derivation, and CPI support. What you test locally works identically on mainnet."
          delay={0.2}
        />
        <Card
          title="Sub-Second Execution"
          icon={<Zap className="w-6 h-6 text-rose-pine-foam" />}
          description="Synchronous composability and instant finality. Build applications that feel responsive in real-time, not blockchain-time."
          delay={0.3}
        />
      </div>
    </Section>
  );
});

ProtocolSection.displayName = "ProtocolSection";

export default ProtocolSection;
