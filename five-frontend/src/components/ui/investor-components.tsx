"use client";

import React, { ReactNode } from "react";
import { motion } from "framer-motion";
import { cn } from "@/lib/utils";

/**
 * Props for the Section wrapper component
 */
interface SectionProps {
  title: string;
  subtitle: string;
  icon: ReactNode;
  children: ReactNode;
  color?: "iris" | "gold" | "love" | "foam" | "pine" | "rose";
  align?: "left" | "right";
}

/**
 * Section wrapper component with consistent styling and animations
 * Used to wrap major page sections with header, subtitle, and content
 */
const Section = React.memo(function Section({
  title,
  subtitle,
  icon,
  children,
  color = "iris",
  align = "left",
}: SectionProps) {
  return (
    <section className="relative w-full py-24 px-4 flex flex-col items-center">
      <div className="max-w-6xl w-full">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-100px" }}
          className={cn(
            "flex flex-col mb-12",
            align === "right" ? "items-end text-right" : "items-start text-left"
          )}
        >
          <div
            className={cn(
              "flex items-center gap-3 mb-4",
              align === "right" && "flex-row-reverse"
            )}
          >
            <div
              className={cn(
                "p-3 rounded-2xl backdrop-blur-xl border bg-opacity-10",
                `bg-rose-pine-${color}/10 border-rose-pine-${color}/20`
              )}
            >
              {icon}
            </div>
            <h2
              className={cn(
                "text-4xl md:text-5xl font-bold tracking-tight",
                `text-rose-pine-${color}`
              )}
            >
              {title}
            </h2>
          </div>
          <p className="text-xl md:text-2xl text-rose-pine-subtle max-w-2xl font-light">
            {subtitle}
          </p>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: "-50px" }}
          transition={{ delay: 0.2 }}
        >
          {children}
        </motion.div>
      </div>
    </section>
  );
});

/**
 * Props for the generic Card component
 */
interface CardProps {
  title: string;
  description: string;
  icon: ReactNode;
  delay?: number;
}

/**
 * Generic card component for displaying features and protocol info
 * Used in Protocol section for feature cards
 */
const Card = React.memo(function Card({
  title,
  description,
  icon,
  delay = 0,
}: CardProps) {
  return (
    <motion.div
      whileHover={{ y: -5 }}
      initial={{ opacity: 0, y: 20 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
      transition={{ delay }}
      className="p-8 rounded-3xl border border-rose-pine-hl-low/20 bg-rose-pine-surface/40 backdrop-blur-md hover:border-rose-pine-hl-med/40 transition-all duration-300 shadow-lg hover:shadow-rose-pine-iris/10"
    >
      <div className="mb-6">{icon}</div>
      <h3 className="text-xl font-bold text-rose-pine-text mb-3">{title}</h3>
      <p className="text-rose-pine-muted leading-relaxed font-light">
        {description}
      </p>
    </motion.div>
  );
});

/**
 * Props for the MetricCard component
 */
interface MetricCardProps {
  label: string;
  value: string;
  description: string;
  delay?: number;
}

/**
 * MetricCard component for displaying economic metrics
 * Used in Economics section for metric displays
 */
const MetricCard = React.memo(function MetricCard({
  label,
  value,
  description,
  delay = 0,
}: MetricCardProps) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
      transition={{ delay }}
      className="p-6 rounded-2xl border border-rose-pine-gold/20 bg-rose-pine-gold/5 backdrop-blur-md"
    >
      <p className="text-sm uppercase tracking-wider text-rose-pine-subtle mb-2">
        {label}
      </p>
      <p className="text-4xl font-black text-rose-pine-gold mb-2">{value}</p>
      <p className="text-sm text-rose-pine-muted">{description}</p>
    </motion.div>
  );
});

/**
 * Props for the UseCaseCard component
 */
interface UseCaseCardProps {
  title: string;
  icon: ReactNode;
  description: string;
  color: "love" | "gold" | "iris" | "foam" | "pine" | "rose";
  delay?: number;
}

/**
 * UseCaseCard component for displaying use case scenarios
 * Used in Use Cases section for application examples
 */
const UseCaseCard = React.memo(function UseCaseCard({
  title,
  icon,
  description,
  color,
  delay = 0,
}: UseCaseCardProps) {
  return (
    <motion.div
      whileHover={{ y: -5 }}
      initial={{ opacity: 0, y: 20 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
      transition={{ delay }}
      className={cn(
        "p-8 rounded-3xl border backdrop-blur-md transition-all duration-300",
        `border-rose-pine-${color}/20 bg-rose-pine-${color}/5 hover:border-rose-pine-${color}/40`
      )}
    >
      <div className={cn("mb-6 p-3 rounded-xl w-fit", `bg-rose-pine-${color}/20`)}>
        {icon}
      </div>
      <h3 className={cn("text-xl font-bold mb-3", `text-rose-pine-${color}`)}>
        {title}
      </h3>
      <p className="text-rose-pine-muted leading-relaxed">{description}</p>
    </motion.div>
  );
});

/**
 * Props for the ComparisonItem component
 */
interface ComparisonItemProps {
  metric: string;
  traditional: string;
  five: string;
  delay?: number;
}

/**
 * ComparisonItem component for developer economics comparison
 * Used in Developer Flywheel section for comparing traditional vs 5IVE
 */
const ComparisonItem = React.memo(function ComparisonItem({
  metric,
  traditional,
  five,
  delay = 0,
}: ComparisonItemProps) {
  const { Check, X } = require("lucide-react");

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
      transition={{ delay }}
      className="p-6 rounded-2xl border border-rose-pine-hl-low/20 bg-rose-pine-surface/30 backdrop-blur-md"
    >
      <p className="text-sm uppercase tracking-wider text-rose-pine-subtle mb-4 font-bold">
        {metric}
      </p>
      <div className="space-y-3">
        <div className="flex items-start gap-2">
          <X size={16} className="text-rose-pine-subtle mt-1 flex-shrink-0" />
          <p className="text-sm text-rose-pine-muted">{traditional}</p>
        </div>
        <div className="flex items-start gap-2">
          <Check size={16} className="text-rose-pine-foam mt-1 flex-shrink-0" />
          <p className="text-sm font-medium text-rose-pine-foam">{five}</p>
        </div>
      </div>
    </motion.div>
  );
});

/**
 * Props for the MoatCard component
 */
interface MoatCardProps {
  title: string;
  description: string;
  delay?: number;
}

/**
 * MoatCard component for displaying moat advantages
 * Used in Moat section for explaining competitive advantages
 */
const MoatCard = React.memo(function MoatCard({
  title,
  description,
  delay = 0,
}: MoatCardProps) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
      transition={{ delay }}
      className="p-6 rounded-2xl border border-rose-pine-love/20 bg-rose-pine-love/5 backdrop-blur-md"
    >
      <h3 className="text-lg font-bold text-rose-pine-love mb-2">{title}</h3>
      <p className="text-sm text-rose-pine-muted leading-relaxed">{description}</p>
    </motion.div>
  );
});

/**
 * Props for the TokenBenefit component
 */
interface TokenBenefitProps {
  icon: string;
  text: string;
}

/**
 * TokenBenefit component for displaying token benefits
 * Used in Token section for listing token utilities
 */
const TokenBenefit = React.memo(function TokenBenefit({
  icon,
  text,
}: TokenBenefitProps) {
  return (
    <motion.div
      initial={{ opacity: 0, x: -10 }}
      whileInView={{ opacity: 1, x: 0 }}
      viewport={{ once: true }}
      className="flex items-center gap-3"
    >
      <span className="text-rose-pine-gold font-bold">{icon}</span>
      <span className="text-rose-pine-text">{text}</span>
    </motion.div>
  );
});

// Export all components
export {
  Section,
  Card,
  MetricCard,
  UseCaseCard,
  ComparisonItem,
  MoatCard,
  TokenBenefit,
};

// Export types
export type {
  SectionProps,
  CardProps,
  MetricCardProps,
  UseCaseCardProps,
  ComparisonItemProps,
  MoatCardProps,
  TokenBenefitProps,
};
