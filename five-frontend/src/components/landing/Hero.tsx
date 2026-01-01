"use client";

import { ArrowRight, Terminal } from "lucide-react";
import Link from "next/link";
import { m, LazyMotion, domAnimation } from "framer-motion";

export default function Hero() {
    return (
        <LazyMotion features={domAnimation}>
            <section className="relative min-h-[90vh] flex flex-col justify-center items-center px-4 pt-20 pb-20">

                {/* Background Glows (Moved grid to page.tsx) */}
                <m.div
                    animate={{ opacity: [0.3, 0.6, 0.3], scale: [1, 1.1, 1] }}
                    transition={{ duration: 8, repeat: Infinity, ease: "easeInOut" }}
                    className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[800px] h-[800px] bg-[#c4a7e7]/10 rounded-full blur-[60px] pointer-events-none will-change-transform translate-z-0"
                />
                <div className="absolute top-0 right-0 w-[500px] h-[500px] bg-[#eb6f92]/5 rounded-full blur-[60px] pointer-events-none will-change-transform translate-z-0" />
                <div className="absolute bottom-0 left-0 w-[600px] h-[600px] bg-[#9ccfd8]/5 rounded-full blur-[60px] pointer-events-none will-change-transform translate-z-0" />

                <div className="relative z-10 max-w-6xl w-full flex flex-col items-center text-center gap-10">

                    {/* Badge */}


                    {/* Hero Container (Simplified) */}
                    <m.div
                        initial={{ opacity: 0, scale: 0.95 }}
                        animate={{ opacity: 1, scale: 1 }}
                        transition={{ duration: 0.8, ease: "circOut" }}
                        className="relative flex flex-col items-center mb-10 group"
                    >


                        {/* Main Title */}
                        <div className="relative z-10">
                            <m.h1
                                initial={{ opacity: 0, scale: 0.9, filter: "blur(10px)" }}
                                animate={{ opacity: 1, scale: 1, filter: "blur(0px)" }}
                                transition={{ duration: 0.8, ease: "circOut" }}
                                className="text-8xl md:text-[10rem] font-black tracking-tighter leading-none bg-clip-text text-transparent bg-gradient-to-b from-rose-pine-iris via-[#c4a7e7] to-rose-pine-love dark:from-rose-pine-text dark:via-rose-pine-iris dark:to-rose-pine-love drop-shadow-2xl select-none mb-4"
                            >
                                5IVE
                            </m.h1>
                        </div>

                        <m.p
                            initial={{ opacity: 0 }}
                            animate={{ opacity: 1 }}
                            transition={{ delay: 0.4 }}
                            className="relative z-10 text-3xl md:text-5xl font-bold text-rose-pine-subtle tracking-tight max-w-2xl mx-auto drop-shadow-md"
                        >
                            Build the Moat.
                        </m.p>
                    </m.div>

                    {/* CTAs */}
                    <m.div
                        initial={{ opacity: 0, y: 20 }}
                        animate={{ opacity: 1, y: 0 }}
                        transition={{ delay: 0.5, duration: 0.6 }}
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
                </div >
            </section >
        </LazyMotion>
    );
}
