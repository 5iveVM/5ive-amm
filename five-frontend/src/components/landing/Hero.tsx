"use client";

import { ArrowRight, Terminal } from "lucide-react";
import Link from "next/link";
import { m, LazyMotion, domAnimation } from "framer-motion";

const WALL_ROWS = 7;
const WALL_COLS = 12;
const BLOCK_W = 64;
const BLOCK_H = 38;
const BLOCK_GAP = 6;

export default function Hero() {
    const wallBlocks = Array.from({ length: WALL_ROWS * WALL_COLS }, (_, i) => {
        const row = Math.floor(i / WALL_COLS);
        const col = i % WALL_COLS;
        const x = (col - (WALL_COLS - 1) / 2) * (BLOCK_W + BLOCK_GAP);
        const y = (row - (WALL_ROWS - 1) / 2) * (BLOCK_H + BLOCK_GAP);
        const side = col < WALL_COLS / 2 ? -1 : 1;
        const colDistance = Math.abs(col - (WALL_COLS - 1) / 2);
        const delay = 1.05 + colDistance * 0.025 + row * 0.012;

        return { id: i, row, col, x, y, side, delay };
    });

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

                <div className="absolute inset-0 z-30 pointer-events-none flex items-center justify-center">
                    <div className="relative w-[920px] h-[560px]">
                        <m.div
                            initial={{ opacity: 0.9 }}
                            animate={{ opacity: 0 }}
                            transition={{ duration: 0.55, delay: 1.35, ease: "easeOut" }}
                            className="absolute inset-0 z-20 flex flex-col items-center justify-center"
                        >
                            <h2 className="text-[8rem] md:text-[10rem] font-black leading-none text-rose-pine-love/90 tracking-tight select-none">
                                THE WALL
                            </h2>
                            <div className="-mt-4 px-5 py-1.5 bg-rose-pine-love text-rose-pine-base text-xl font-black uppercase tracking-[0.2em] rounded-sm">
                                Mainnet Barrier
                            </div>
                        </m.div>

                        <m.div
                            initial={{ scaleY: 0, opacity: 0 }}
                            animate={{ scaleY: [0, 1, 1], opacity: [0, 0.8, 0] }}
                            transition={{ duration: 1.2, delay: 0.8, ease: "easeInOut" }}
                            className="absolute left-1/2 top-10 bottom-10 w-[2px] bg-gradient-to-b from-transparent via-rose-pine-love to-transparent origin-top z-10"
                        />

                        {wallBlocks.map((block) => (
                            <m.div
                                key={block.id}
                                initial={{ x: block.x, y: block.y, opacity: 1, rotate: 0, scale: 1 }}
                                animate={{
                                    x: block.x + block.side * (140 + Math.abs(block.col - WALL_COLS / 2) * 10),
                                    y: block.y + 220 + block.row * 16,
                                    rotate: block.side * (8 + (block.row % 3) * 2),
                                    opacity: 0,
                                    scale: 0.92,
                                }}
                                transition={{
                                    duration: 0.75,
                                    delay: block.delay,
                                    ease: [0.2, 0.7, 0.3, 1],
                                }}
                                className={`absolute left-1/2 top-1/2 border rounded-sm shadow-[2px_2px_4px_rgba(0,0,0,0.45)] ${
                                    block.row % 2 === 0
                                        ? "bg-rose-pine-overlay border-rose-pine-hl-low/45"
                                        : "bg-rose-pine-muted/90 border-rose-pine-subtle/50"
                                }`}
                                style={{
                                    width: BLOCK_W,
                                    height: BLOCK_H,
                                    marginLeft: -(BLOCK_W / 2),
                                    marginTop: -(BLOCK_H / 2),
                                }}
                            />
                        ))}
                    </div>
                </div>

                <div className="relative z-10 max-w-6xl w-full flex flex-col items-center text-center gap-10">
                    <m.div
                        initial={{ opacity: 0, scale: 0.95, y: 10 }}
                        animate={{ opacity: 1, scale: 1, y: 0 }}
                        transition={{ duration: 0.7, delay: 1.8, ease: "circOut" }}
                        className="relative flex flex-col items-center mb-10"
                    >
                        <div className="relative z-10">
                            <h1 className="text-8xl md:text-[10rem] font-black tracking-tighter leading-none bg-clip-text text-transparent bg-gradient-to-b from-rose-pine-iris via-rose-pine-iris to-rose-pine-love drop-shadow-2xl select-none mb-4">
                                5IVE
                            </h1>
                        </div>

                        <p className="relative z-10 text-3xl md:text-5xl font-bold text-rose-pine-subtle tracking-tight max-w-4xl mx-auto drop-shadow-md">
                            Tear down the wall. <span className="text-rose-pine-iris">Build the moat.</span>
                        </p>
                        <p className="relative z-10 mt-4 text-base md:text-lg text-rose-pine-muted max-w-3xl mx-auto">
                            When barriers fall, ecosystems open. Five brings Solana mainnet to more builders and agentic workflows with fast, composable contracts.
                        </p>
                    </m.div>

                    <m.div
                        initial={{ opacity: 0, y: 24 }}
                        animate={{ opacity: 1, y: 0 }}
                        transition={{ delay: 2.0, duration: 0.55 }}
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
