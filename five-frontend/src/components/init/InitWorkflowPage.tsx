"use client";

import Link from "next/link";
import { LazyMotion, domAnimation, m } from "framer-motion";
import {
  ArrowRight,
  CheckCircle2,
  FileText,
  Rocket,
  Sparkles,
  Terminal,
} from "lucide-react";
import Background from "@/components/layout/Background";
import Header from "@/components/layout/Header";
import { GlassCard } from "@/components/ui/glass-card";
import { CodeBlock } from "@/components/ui/code-block";

const AGENTS_SNIPPET = `# AGENTS.md
## Project Goal
Generate a Solana escrow contract and client flow for milestone payments.

## Constraints
- Use 5IVE DSL with explicit @signer and @mut constraints
- Keep account model simple: Escrow + Milestone state
- Include tests for create, fund, release, and cancel flows

## Generation Rules
- Prefer clear function names over abstraction-heavy patterns
- Keep each instruction focused on one state transition
- Return a deployment-ready project layout with README + scripts`;

const QUICKSTART_SNIPPET = `npm i -g @5ive-tech/cli
5ive init my-app
cd my-app

# 1) shape your AGENTS.md for your target app
# 2) run your agent in a few shots to generate contracts + client

5ive build
5ive test
5ive deploy ./build/my-app.five --target devnet`;

const steps = [
  {
    icon: Terminal,
    label: "1) 5ive init",
    title: "Scaffold a clean starting point",
    body: "Initialize a project with the expected structure so your agent starts from a stable baseline.",
  },
  {
    icon: FileText,
    label: "2) Edit AGENTS.md",
    title: "Set intent and constraints",
    body: "Define the product goal, account model, and non-negotiables so generated code stays aligned with what you want to ship.",
  },
  {
    icon: Sparkles,
    label: "3) Generate in a few shots",
    title: "Iterate fast with focused prompts",
    body: "Use short prompt loops to generate contracts, tests, and client glue while AGENTS.md keeps output consistent.",
  },
  {
    icon: Rocket,
    label: "4) Deploy",
    title: "Ship the generated project",
    body: "Build, test, and deploy once the generated surface matches your requirements.",
  },
];

const proofCards = [
  {
    name: "5ive-escrow",
    summary: "Escrow-style flow generated from AGENTS.md constraints and iterated into a deployment-ready project.",
    points: [
      "State model scoped to escrow lifecycle",
      "Instruction flow shaped by explicit authority rules",
      "Project output includes contract + client workflow",
    ],
  },
  {
    name: "5ive-single-pool",
    summary: "Single-pool protocol surface generated through the same init-first, AGENTS-guided workflow.",
    points: [
      "Pool behavior defined up front in AGENTS.md",
      "Few-shot prompt loop used to refine function boundaries",
      "Result packaged for build/test/deploy workflow",
    ],
  },
];

export default function InitWorkflowPage() {
  return (
    <LazyMotion features={domAnimation}>
      <div className="min-h-screen bg-transparent text-rose-pine-text font-sans selection:bg-rose-pine-love/30 flex flex-col relative overflow-x-hidden">
        <Background />
        <Header />

        <main className="flex-1 relative z-10 w-full pt-24">
          <section className="relative min-h-[78vh] flex items-center px-4 py-20">
            <div className="absolute inset-0 pointer-events-none">
              <div className="absolute top-8 left-1/2 -translate-x-1/2 w-[800px] h-[500px] bg-rose-pine-iris/10 rounded-full blur-[80px]" />
            </div>

            <m.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.45, ease: "easeOut" }}
              className="relative z-10 max-w-5xl mx-auto text-center"
            >
              <p className="text-xs md:text-sm font-mono uppercase tracking-[0.18em] text-rose-pine-foam mb-5">
                AI-assisted Builder Workflow
              </p>
              <h1 className="text-4xl md:text-6xl font-black tracking-tight leading-tight">
                Start with <span className="text-rose-pine-iris">5ive init</span>.
                <br />
                Shape output with <span className="text-rose-pine-love">AGENTS.md</span>.
                <br />
                Ship in a few shots.
              </h1>
              <p className="mt-6 text-base md:text-lg text-rose-pine-subtle max-w-3xl mx-auto">
                5IVE now fits modern development loops: initialize fast, define constraints once, generate focused project code, and move directly into deployment workflows.
              </p>

              <div className="mt-10 flex flex-col sm:flex-row gap-4 justify-center">
                <a
                  href="#quickstart"
                  className="group inline-flex items-center justify-center gap-2 px-8 py-3 rounded-xl bg-gradient-to-r from-rose-pine-love to-rose-pine-iris text-rose-pine-base font-bold shadow-lg shadow-rose-pine-love/20 hover:brightness-110 transition-all"
                >
                  Start with 5ive init
                  <ArrowRight size={18} className="group-hover:translate-x-0.5 transition-transform" />
                </a>
                <Link
                  href="/ide"
                  className="inline-flex items-center justify-center gap-2 px-8 py-3 rounded-xl border border-rose-pine-hl-med/40 bg-rose-pine-surface/40 text-rose-pine-text font-semibold hover:border-rose-pine-iris/40 hover:bg-rose-pine-surface/70 transition-all"
                >
                  Launch IDE
                </Link>
              </div>
            </m.div>
          </section>

          <section className="px-4 py-16">
            <div className="max-w-6xl mx-auto">
              <m.div
                initial={{ opacity: 0, y: 16 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.35 }}
                className="mb-8"
              >
                <h2 className="text-3xl md:text-4xl font-black">How it works</h2>
                <p className="mt-3 text-rose-pine-subtle">
                  A practical loop for AI-assisted builders using 5IVE.
                </p>
              </m.div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
                {steps.map((step, idx) => {
                  const Icon = step.icon;
                  return (
                    <m.div
                      key={step.label}
                      initial={{ opacity: 0, y: 14 }}
                      whileInView={{ opacity: 1, y: 0 }}
                      viewport={{ once: true }}
                      transition={{ duration: 0.3, delay: idx * 0.05 }}
                    >
                      <GlassCard className="p-6 h-full">
                        <div className="flex items-start gap-4">
                          <div className="p-2 rounded-lg bg-rose-pine-iris/10 border border-rose-pine-iris/20 text-rose-pine-iris shrink-0">
                            <Icon size={18} />
                          </div>
                          <div>
                            <p className="text-xs uppercase tracking-widest text-rose-pine-foam font-mono">
                              {step.label}
                            </p>
                            <h3 className="mt-2 text-xl font-bold text-rose-pine-text">
                              {step.title}
                            </h3>
                            <p className="mt-2 text-sm text-rose-pine-subtle">{step.body}</p>
                          </div>
                        </div>
                      </GlassCard>
                    </m.div>
                  );
                })}
              </div>
            </div>
          </section>

          <section className="px-4 py-16">
            <div className="max-w-6xl mx-auto grid grid-cols-1 lg:grid-cols-[1fr_1.2fr] gap-8 items-start">
              <m.div
                initial={{ opacity: 0, y: 16 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.35 }}
              >
                <h2 className="text-3xl md:text-4xl font-black">AGENTS.md as the control surface</h2>
                <p className="mt-4 text-rose-pine-subtle">
                  Keep AGENTS.md concise and specific. The clearer your goal and constraints, the more reliable the generated project output across prompt iterations.
                </p>
                <ul className="mt-6 space-y-3">
                  <li className="flex items-start gap-3 text-rose-pine-subtle">
                    <CheckCircle2 className="text-rose-pine-foam shrink-0 mt-0.5" size={16} />
                    Capture project intent in one sentence.
                  </li>
                  <li className="flex items-start gap-3 text-rose-pine-subtle">
                    <CheckCircle2 className="text-rose-pine-foam shrink-0 mt-0.5" size={16} />
                    List hard constraints for account model and authority checks.
                  </li>
                  <li className="flex items-start gap-3 text-rose-pine-subtle">
                    <CheckCircle2 className="text-rose-pine-foam shrink-0 mt-0.5" size={16} />
                    Define expected deliverables (contract, tests, client flow).
                  </li>
                </ul>
              </m.div>

              <m.div
                initial={{ opacity: 0, y: 16 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.35, delay: 0.06 }}
              >
                <CodeBlock filename="AGENTS.md" code={AGENTS_SNIPPET} />
              </m.div>
            </div>
          </section>

          <section className="px-4 py-16">
            <div className="max-w-6xl mx-auto">
              <m.div
                initial={{ opacity: 0, y: 16 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.35 }}
                className="mb-8"
              >
                <h2 className="text-3xl md:text-4xl font-black">Generated project proof points</h2>
                <p className="mt-3 text-rose-pine-subtle">
                  Named examples from the same init and AGENTS-guided development model.
                </p>
              </m.div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
                {proofCards.map((card, idx) => (
                  <m.div
                    key={card.name}
                    initial={{ opacity: 0, y: 14 }}
                    whileInView={{ opacity: 1, y: 0 }}
                    viewport={{ once: true }}
                    transition={{ duration: 0.3, delay: idx * 0.05 }}
                  >
                    <GlassCard className="p-6 h-full">
                      <h3 className="text-2xl font-black text-rose-pine-iris">{card.name}</h3>
                      <p className="mt-3 text-rose-pine-subtle">{card.summary}</p>
                      <ul className="mt-5 space-y-2">
                        {card.points.map((point) => (
                          <li key={point} className="flex items-start gap-2 text-sm text-rose-pine-subtle">
                            <span className="mt-1 h-1.5 w-1.5 rounded-full bg-rose-pine-foam shrink-0" />
                            {point}
                          </li>
                        ))}
                      </ul>
                    </GlassCard>
                  </m.div>
                ))}
              </div>
            </div>
          </section>

          <section id="quickstart" className="px-4 py-16 scroll-mt-28">
            <div className="max-w-6xl mx-auto grid grid-cols-1 lg:grid-cols-[1fr_1.15fr] gap-8 items-start">
              <m.div
                initial={{ opacity: 0, y: 16 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.35 }}
              >
                <h2 className="text-3xl md:text-4xl font-black">Quickstart</h2>
                <p className="mt-4 text-rose-pine-subtle">
                  Use this as your default execution loop: initialize, define AGENTS.md, generate in a few shots, then build and deploy.
                </p>
              </m.div>

              <m.div
                initial={{ opacity: 0, y: 16 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.35, delay: 0.06 }}
              >
                <CodeBlock
                  filename="quickstart.sh"
                  language="shell"
                  code={QUICKSTART_SNIPPET}
                />
              </m.div>
            </div>
          </section>

          <section className="px-4 py-16">
            <m.div
              initial={{ opacity: 0, y: 16 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.35 }}
              className="max-w-6xl mx-auto"
            >
              <GlassCard className="p-8 md:p-10">
                <h2 className="text-2xl md:text-3xl font-black">
                  Ready to run the <code>5ive init -&gt; AGENTS.md -&gt; generate -&gt; deploy</code> loop?
                </h2>
                <p className="mt-3 text-rose-pine-subtle">
                  Start with quickstart commands, then move to docs when you need deeper DSL and runtime details.
                </p>
                <div className="mt-7 flex flex-col sm:flex-row gap-4">
                  <a
                    href="#quickstart"
                    className="inline-flex items-center justify-center gap-2 px-6 py-3 rounded-xl bg-rose-pine-iris text-rose-pine-base font-semibold hover:brightness-110 transition-all"
                  >
                    Go to Quickstart
                  </a>
                  <Link
                    href="/docs"
                    className="inline-flex items-center justify-center gap-2 px-6 py-3 rounded-xl border border-rose-pine-hl-med/40 bg-rose-pine-surface/40 text-rose-pine-text font-semibold hover:border-rose-pine-iris/40 transition-all"
                  >
                    Read Docs
                  </Link>
                </div>
              </GlassCard>
            </m.div>
          </section>
        </main>

        <footer className="py-8 border-t border-rose-pine-hl-low/20 text-center text-sm text-rose-pine-muted relative z-10">
          <p>© 2026 5ive Tech. All rights reserved.</p>
        </footer>
      </div>
    </LazyMotion>
  );
}
