"use client";

import { ArrowRight, Terminal } from "lucide-react";
import Link from "next/link";
import { m, LazyMotion, domAnimation } from "framer-motion";

export default function Hero() {
    return (
        <LazyMotion features={domAnimation}>
            <section className="relative min-h-[90vh] flex flex-col justify-center items-center px-4 pt-20 pb-20 overflow-hidden">
                <m.div
                    animate={{ opacity: [0.35, 0.6, 0.35], scale: [1, 1.08, 1] }}
                    transition={{ duration: 8, repeat: Infinity, ease: "easeInOut" }}
                    className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[840px] h-[840px] bg-rose-pine-iris/10 rounded-full blur-[70px] pointer-events-none"
                />
                <div className="absolute top-0 right-0 w-[500px] h-[500px] bg-rose-pine-love/5 rounded-full blur-[60px] pointer-events-none" />
                <div className="absolute bottom-0 left-0 w-[600px] h-[600px] bg-rose-pine-foam/5 rounded-full blur-[60px] pointer-events-none" />

                <div className="relative z-10 max-w-6xl w-full flex flex-col items-center text-center gap-10">
                    <m.div
                        initial={{ opacity: 0, scale: 0.95, y: 10 }}
                        animate={{ opacity: 1, scale: 1, y: 0 }}
                        transition={{ duration: 0.5, delay: 0.2, ease: "circOut" }}
                        className="relative flex flex-col items-center mb-10"
                    >
                        <div className="relative z-10">
                            <h1 className="text-8xl md:text-[10rem] font-black tracking-tighter leading-none bg-clip-text text-transparent bg-gradient-to-b from-rose-pine-iris via-rose-pine-iris to-rose-pine-love drop-shadow-2xl select-none mb-4">
                                5IVE
                            </h1>
                        </div>
                        <p className="relative z-10 mb-3 text-xs md:text-sm font-mono uppercase tracking-[0.2em] text-rose-pine-foam/90">
                            Layer 1.5 for Solana
                        </p>

                        <p className="relative z-10 text-3xl md:text-5xl font-bold text-rose-pine-subtle tracking-tight max-w-4xl mx-auto drop-shadow-md">
                            Tear down the wall. <span className="text-rose-pine-iris">Build the moat.</span>
                        </p>
                        <p className="relative z-10 mt-4 text-base md:text-lg text-rose-pine-muted max-w-3xl mx-auto">
                            5IVE is built to make Solana stronger, not compete with it. When barriers fall, ecosystems open. 5IVE brings Solana mainnet to more builders and agentic workflows with fast, composable contracts.
                        </p>
                    </m.div>

                    <m.div
                        initial={{ opacity: 0, y: 24 }}
                        animate={{ opacity: 1, y: 0 }}
                        transition={{ delay: 0.4, duration: 0.4, ease: "circOut" }}
                        className="flex flex-col sm:flex-row items-center gap-5 mt-6"
                    >
                        <Link href="/ide">
                            <m.button
                                whileHover={{ scale: 1.05, boxShadow: "0 0 30px -5px var(--color-rose-pine-iris)" }}
                                whileTap={{ scale: 0.95 }}
                                className="group relative px-10 py-4 rounded-2xl bg-gradient-to-r from-rose-pine-love to-rose-pine-iris text-rose-pine-base font-bold text-lg shadow-xl shadow-rose-pine-love/20 transition-all flex items-center gap-3 overflow-hidden"
                            >
                                <span className="relative z-10">Launch IDE</span>
                                <ArrowRight size={20} className="relative z-10 group-hover:translate-x-1 transition-transform" />
                                <div className="absolute inset-0 bg-white/20 opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
                                <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/30 to-transparent -translate-x-[200%] group-hover:animate-[shimmer_1.5s_infinite]" />
                            </m.button>
                        </Link>

                        <Link href="/docs">
                            <m.button
                                whileHover={{ scale: 1.05, backgroundColor: "var(--color-rose-pine-surface)", borderColor: "var(--color-rose-pine-iris)" }}
                                whileTap={{ scale: 0.95 }}
                                className="px-10 py-4 rounded-2xl bg-rose-pine-surface/40 dark:bg-rose-pine-surface/40 border border-rose-pine-hl-low text-rose-pine-text font-semibold text-lg hover:border-rose-pine-hl-med transition-all backdrop-blur-md flex items-center gap-3 shadow-lg"
                            >
                                <Terminal size={20} className="text-rose-pine-foam" />
                                Read Docs
                            </m.button>
                        </Link>
                    </m.div>
                </div>
            </section>
        </LazyMotion>
    );
}
