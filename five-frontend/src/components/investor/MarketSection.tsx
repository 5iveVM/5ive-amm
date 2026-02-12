"use client";

import React from "react";
import { motion } from "framer-motion";
import { TrendingUp } from "lucide-react";
import { Section } from "@/components/ui/investor-components";

/**
 * GrowthVisualization Component
 * Displays market growth projection as animated bar chart
 */
const GrowthVisualization = React.memo(function GrowthVisualization() {
  const years = [
    { year: "2024", value: 67.4, label: "$67.4B" },
    { year: "2026", value: 160, label: "$160B" },
    { year: "2028", value: 380, label: "$380B" },
    { year: "2030", value: 943, label: "$943B" },
  ];

  const maxValue = 943;

  return (
    <div className="w-full h-full flex items-end justify-center gap-4 md:gap-6 px-4">
      {years.map((item, idx) => (
        <motion.div
          key={idx}
          initial={{ height: 0, opacity: 0 }}
          whileInView={{ height: `${(item.value / maxValue) * 100}%`, opacity: 1 }}
          viewport={{ once: true }}
          transition={{ delay: idx * 0.1, duration: 0.6 }}
          className="flex-1 flex flex-col items-center"
        >
          <div className="w-full rounded-t-2xl bg-gradient-to-t from-rose-pine-foam to-rose-pine-foam/60 min-h-16 flex items-end justify-center pb-2">
            <span className="text-[10px] md:text-xs font-bold text-rose-pine-base whitespace-nowrap">
              {item.label}
            </span>
          </div>
          <p className="text-xs text-rose-pine-muted mt-2 font-bold">{item.year}</p>
        </motion.div>
      ))}
    </div>
  );
});

GrowthVisualization.displayName = "GrowthVisualization";

/**
 * MarketSection Component
 * Presents the massive market opportunity for blockchain development platforms
 * Shows TAM, growth rates, and 2030 projection with interactive visualization
 */
const MarketSection = React.memo(function MarketSection() {
  return (
    <Section
      title="The Market Opportunity"
      subtitle="5IVE is entering a market experiencing explosive growth."
      icon={<TrendingUp className="w-8 h-8 text-rose-pine-foam" />}
      color="foam"
      align="right"
    >
      <div className="grid md:grid-cols-2 gap-8 items-center">
        <div className="order-2 md:order-1">
          <div className="space-y-8">
            <div className="flex flex-col gap-2">
              <p className="text-sm uppercase tracking-wider text-rose-pine-subtle">2024 TAM</p>
              <p className="text-5xl font-black text-rose-pine-foam">$67.4B</p>
              <p className="text-rose-pine-muted">Blockchain development platform market</p>
            </div>
            <div className="flex flex-col gap-2">
              <p className="text-sm uppercase tracking-wider text-rose-pine-subtle">Annual Growth</p>
              <p className="text-5xl font-black text-rose-pine-foam">68.4%</p>
              <p className="text-rose-pine-muted">CAGR through 2030</p>
            </div>
            <div className="flex flex-col gap-2 pt-4 border-t border-rose-pine-hl-low/20">
              <p className="text-sm uppercase tracking-wider text-rose-pine-foam font-bold">
                2030 Projection
              </p>
              <p className="text-5xl font-black bg-gradient-to-r from-rose-pine-foam to-rose-pine-gold bg-clip-text text-transparent">
                $943.3B
              </p>
              <p className="text-rose-pine-muted">5IVE is positioned to capture meaningful share</p>
            </div>
          </div>
        </div>
        <div className="order-1 md:order-2 flex items-center justify-center h-80">
          <GrowthVisualization />
        </div>
      </div>
    </Section>
  );
});

MarketSection.displayName = "MarketSection";

export default MarketSection;
