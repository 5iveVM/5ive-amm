"use client";

import React from "react";
import { motion } from "framer-motion";
import { Code2 } from "lucide-react";
import { Section, ComparisonItem } from "@/components/ui/investor-components";

/**
 * DeveloperFlywheel Component
 * Illustrates the developer economics and network effects of Five Protocol
 * Shows comparison metrics and the virtuous cycle of adoption
 */
const DeveloperFlywheel = React.memo(function DeveloperFlywheel() {
  return (
    <Section
      title="The Developer Flywheel"
      subtitle="How Five creates unstoppable network effects."
      icon={<Code2 className="w-8 h-8 text-rose-pine-iris" />}
      color="iris"
    >
      <div className="space-y-8">
        <div className="grid md:grid-cols-3 gap-8">
          <ComparisonItem
            metric="Time to Production"
            traditional="6-18 months"
            five="2-4 weeks"
            delay={0.1}
          />
          <ComparisonItem
            metric="Required Expertise"
            traditional="6+ months Rust/Solana"
            five="Basic programming"
            delay={0.2}
          />
          <ComparisonItem
            metric="AI Code Generation"
            traditional="Fails with Rust"
            five="Claude/GPT compatible"
            delay={0.3}
          />
        </div>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="p-8 rounded-3xl border border-rose-pine-iris/30 bg-rose-pine-iris/5 backdrop-blur-md"
        >
          <h3 className="text-2xl font-bold text-rose-pine-iris mb-6">The Network Effect</h3>
          <div className="flex flex-col md:flex-row items-center justify-between gap-4 text-center md:text-left">
            <div className="flex-1">
              <p className="text-sm uppercase text-rose-pine-subtle mb-2">More Developers</p>
              <p className="text-2xl font-bold text-rose-pine-iris">→</p>
            </div>
            <div className="flex-1">
              <p className="text-sm uppercase text-rose-pine-subtle mb-2">More Applications</p>
              <p className="text-2xl font-bold text-rose-pine-iris">→</p>
            </div>
            <div className="flex-1">
              <p className="text-sm uppercase text-rose-pine-subtle mb-2">More Libraries</p>
              <p className="text-2xl font-bold text-rose-pine-iris">→</p>
            </div>
            <div className="flex-1">
              <p className="text-sm uppercase text-rose-pine-subtle mb-2">Lower Barriers</p>
              <p className="text-2xl font-bold text-rose-pine-iris">↻</p>
            </div>
          </div>
        </motion.div>
      </div>
    </Section>
  );
});

DeveloperFlywheel.displayName = "DeveloperFlywheel";

export default DeveloperFlywheel;
