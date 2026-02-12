"use client";

import React from "react";
import { motion } from "framer-motion";
import { Shield } from "lucide-react";
import { Section } from "@/components/ui/investor-components";
import { cn } from "@/lib/utils";

/**
 * ComparisonTable Component
 * Detailed comparison between 5IVE, Anchor, and Seahorse frameworks
 * Shows key differentiators including cost, development time, and features
 */
const ComparisonTable = React.memo(function ComparisonTable() {
  const rows = [
    { metric: "Deployment Cost", anchor: "$126", seahorse: "~$100", five: "$0.002" },
    { metric: "Development Time", anchor: "6-18 months", seahorse: "4-12 months", five: "2-4 weeks" },
    { metric: "Language", anchor: "Rust", seahorse: "Python + Rust", five: "5IVE DSL" },
    { metric: "AI Code Generation", anchor: "❌", seahorse: "❌", five: "✅" },
    { metric: "Zero-Allocation VM", anchor: "❌", seahorse: "❌", five: "✅" },
    { metric: "Integrated Toolchain", anchor: "Partial", seahorse: "Partial", five: "✅ Complete" },
  ];

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
      className="overflow-x-auto"
    >
      <div className="hidden md:block rounded-3xl border border-rose-pine-hl-low/20 bg-rose-pine-surface/20 backdrop-blur-md overflow-hidden">
        <table className="w-full">
          <thead>
            <tr className="border-b border-rose-pine-hl-low/20">
              <th className="px-6 py-4 text-left text-sm font-bold text-rose-pine-text">
                Feature
              </th>
              <th className="px-6 py-4 text-left text-sm font-bold text-rose-pine-muted">Anchor</th>
              <th className="px-6 py-4 text-left text-sm font-bold text-rose-pine-muted">
                Seahorse
              </th>
              <th className="px-6 py-4 text-left text-sm font-bold text-rose-pine-foam">5IVE</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((row, idx) => (
              <motion.tr
                key={idx}
                initial={{ opacity: 0 }}
                whileInView={{ opacity: 1 }}
                viewport={{ once: true }}
                transition={{ delay: idx * 0.05 }}
                className={cn(
                  "border-b border-rose-pine-hl-low/10 last:border-b-0",
                  idx % 2 === 0 && "bg-rose-pine-overlay/20"
                )}
              >
                <td className="px-6 py-4 text-sm font-medium text-rose-pine-text">
                  {row.metric}
                </td>
                <td className="px-6 py-4 text-sm text-rose-pine-muted">{row.anchor}</td>
                <td className="px-6 py-4 text-sm text-rose-pine-muted">{row.seahorse}</td>
                <td className="px-6 py-4 text-sm font-bold text-rose-pine-foam">{row.five}</td>
              </motion.tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Mobile Card View */}
      <div className="md:hidden space-y-4">
        {rows.map((row, idx) => (
          <motion.div
            key={idx}
            initial={{ opacity: 0, y: 10 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ delay: idx * 0.05 }}
            className="p-4 rounded-2xl border border-rose-pine-hl-low/20 bg-rose-pine-surface/30 backdrop-blur-md"
          >
            <p className="text-sm font-bold text-rose-pine-text mb-3">{row.metric}</p>
            <div className="space-y-2 text-xs">
              <div className="flex justify-between">
                <span className="text-rose-pine-muted">Anchor:</span>
                <span className="text-rose-pine-muted">{row.anchor}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-rose-pine-muted">Seahorse:</span>
                <span className="text-rose-pine-muted">{row.seahorse}</span>
              </div>
              <div className="flex justify-between font-bold text-rose-pine-foam">
                <span>5IVE:</span>
                <span>{row.five}</span>
              </div>
            </div>
          </motion.div>
        ))}
      </div>
    </motion.div>
  );
});

ComparisonTable.displayName = "ComparisonTable";

/**
 * CompetitiveSection Component
 * Presents competitive analysis against established frameworks
 * Shows why 5IVE is superior to Anchor and Seahorse alternatives
 */
const CompetitiveSection = React.memo(function CompetitiveSection() {
  return (
    <Section
      title="Why 5IVE, Not Anchor?"
      subtitle="An honest comparison with the status quo."
      icon={<Shield className="w-8 h-8 text-rose-pine-pine" />}
      color="pine"
    >
      <ComparisonTable />
    </Section>
  );
});

CompetitiveSection.displayName = "CompetitiveSection";

export default CompetitiveSection;
