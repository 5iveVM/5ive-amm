"use client";

import { motion } from "framer-motion";
import { Terminal, Cloud, Zap, ArrowRight, CheckCircle2, XCircle, Globe, Laptop } from "lucide-react";
import Link from "next/link";

export default function DeveloperExperience() {
    return (
        <section className="relative py-32 px-4 overflow-hidden bg-[#191724]">
            {/* Background Atmosphere */}
            <div className="absolute inset-0 pointer-events-none">
                <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[1000px] h-[1000px] bg-rose-pine-iris/5 rounded-full blur-[150px]" />
            </div>

            <div className="relative z-10 max-w-7xl mx-auto space-y-40">

                {/* SECTION 1: THE IMPORT REVOLUTION */}
                <div className="grid lg:grid-cols-2 gap-16 items-center">

                    {/* Text Context */}
                    <div className="order-1 lg:order-2">
                        <motion.div
                            initial={{ opacity: 0, x: 20 }}
                            whileInView={{ opacity: 1, x: 0 }}
                            viewport={{ once: true }}
                            transition={{ duration: 0.6 }}
                        >
                            <div className="flex items-center gap-3 mb-6">
                                <div className="p-2 rounded-lg bg-rose-pine-love/10 border border-rose-pine-love/20 text-rose-pine-love">
                                    <Zap size={24} />
                                </div>
                                <h2 className="text-3xl md:text-5xl font-black text-rose-pine-text tracking-tight">
                                    The Import <br /> <span className="text-transparent bg-clip-text bg-gradient-to-r from-rose-pine-love to-rose-pine-iris">Revolution</span>
                                </h2>
                            </div>

                            <p className="text-xl text-rose-pine-muted font-light leading-relaxed mb-8">
                                Stop wrestling with CPI boilerplate. 5IVE treats other programs like native libraries.
                            </p>

                            <ul className="space-y-4 mb-10">
                                <li className="flex items-start gap-3 text-rose-pine-subtle">
                                    <XCircle className="mt-1 text-rose-pine-muted shrink-0 opacity-50" size={20} />
                                    <span>No manual account meta construction</span>
                                </li>
                                <li className="flex items-start gap-3 text-rose-pine-subtle">
                                    <XCircle className="mt-1 text-rose-pine-muted shrink-0 opacity-50" size={20} />
                                    <span>No serialization overhead</span>
                                </li>
                                <li className="flex items-start gap-3 text-rose-pine-text font-medium">
                                    <CheckCircle2 className="mt-1 text-rose-pine-foam shrink-0" size={20} />
                                    <span>Just <code className="bg-rose-pine-overlay px-1.5 py-0.5 rounded text-rose-pine-foam">import</code> and call</span>
                                </li>
                            </ul>
                        </motion.div>
                    </div>

                    {/* Code Comparison Visual */}
                    <div className="order-2 lg:order-1 relative">
                        {/* Background Splashes */}
                        <div className="absolute -inset-10 bg-gradient-to-tr from-rose-pine-iris/20 to-transparent blur-3xl opacity-50" />

                        <div className="relative grid gap-6">

                            {/* Comparison Card */}
                            <motion.div
                                initial={{ opacity: 0, y: 20 }}
                                whileInView={{ opacity: 1, y: 0 }}
                                viewport={{ once: true }}
                                transition={{ duration: 0.8 }}
                                className="rounded-3xl border border-rose-pine-hl-low/20 bg-[#1f1d2e] overflow-hidden shadow-2xl shadow-rose-pine-iris/10"
                            >
                                {/* Header */}
                                <div className="flex border-b border-rose-pine-hl-low/10">
                                    <div className="flex-1 p-4 bg-[#232136] text-center text-xs font-mono uppercase tracking-widest text-rose-pine-subtle border-r border-rose-pine-hl-low/10 opacity-50">
                                        Legacy CPI
                                    </div>
                                    <div className="flex-1 p-4 bg-[#2a273f] text-center text-xs font-bold font-mono uppercase tracking-widest text-rose-pine-iris">
                                        5IVE Import
                                    </div>
                                </div>

                                {/* Editor Body */}
                                <div className="grid grid-cols-2 divide-x divide-rose-pine-hl-low/10 font-mono text-[10px] md:text-xs leading-relaxed">

                                    {/* Legacy Code (Dimmed) */}
                                    <div className="p-6 bg-[#1f1d2e] opacity-40 select-none overflow-hidden relative">
                                        <div className="absolute inset-0 bg-gradient-to-b from-transparent via-transparent to-[#1f1d2e] z-10" />
                                        <span className="text-rose-pine-love">let</span> ix = Instruction {"{"}<br />
                                        &nbsp;&nbsp;program_id: pid,<br />
                                        &nbsp;&nbsp;accounts: <span className="text-rose-pine-foam">vec!</span>[<br />
                                        &nbsp;&nbsp;&nbsp;&nbsp;AccountMeta::new(..),<br />
                                        &nbsp;&nbsp;&nbsp;&nbsp;AccountMeta::readonly(..),<br />
                                        &nbsp;&nbsp;],<br />
                                        &nbsp;&nbsp;data: args.try_to_vec()?,<br />
                                        {"}"};<br />
                                        invoke_signed(&ix, accounts, ..)?;
                                    </div>

                                    {/* 5IVE Code (Bright) */}
                                    <div className="p-6 bg-[#2a273f]/50 relative">
                                        <div className="absolute top-0 right-0 p-2">
                                            <div className="flex gap-1.5">
                                                <div className="w-2 h-2 rounded-full bg-rose-pine-foam animate-pulse" />
                                            </div>
                                        </div>
                                        <span className="text-rose-pine-iris">import</span> <span className="text-rose-pine-gold">"@solana/spl-token"</span>;<br />
                                        <br />
                                        <span className="text-rose-pine-subtle">// Just call it.</span><br />
                                        SPLToken.transfer(from, to, amount);
                                    </div>
                                </div>
                            </motion.div>
                        </div>
                    </div>
                </div>


                {/* SECTION 2: ZERO INSTALL / BROWSER BASED */}
                <div className="grid lg:grid-cols-2 gap-16 items-center">

                    {/* Visuals (Browser/Cloud IDE) */}
                    <div className="relative flex justify-center lg:justify-start order-2">

                        {/* Floating IDE Window */}
                        <motion.div
                            initial={{ opacity: 0, scale: 0.95, y: 20 }}
                            whileInView={{ opacity: 1, scale: 1, y: 0 }}
                            viewport={{ once: true }}
                            transition={{ duration: 0.8, ease: "circOut" }}
                            className="relative w-full max-w-lg rounded-2xl border border-rose-pine-hl-low/20 bg-[#1f1d2e] shadow-[0_0_50px_rgba(196,167,231,0.1)] overflow-hidden z-10"
                        >
                            {/* Window Title Bar */}
                            <div className="h-10 bg-[#26233a] border-b border-white/5 flex items-center px-4 justify-between">
                                <div className="flex gap-2">
                                    <div className="w-3 h-3 rounded-full bg-rose-pine-love/50" />
                                    <div className="w-3 h-3 rounded-full bg-rose-pine-gold/50" />
                                    <div className="w-3 h-3 rounded-full bg-rose-pine-foam/50" />
                                </div>
                                <div className="text-[10px] font-mono text-rose-pine-subtle opacity-50 flex items-center gap-2">
                                    <Globe size={10} />
                                    ide.five.org
                                </div>
                            </div>

                            {/* Editor Content */}
                            <div className="p-6 relative">
                                {/* Success toast simulation */}
                                <div className="absolute top-4 right-4 bg-rose-pine-foam/10 border border-rose-pine-foam/20 text-rose-pine-foam px-3 py-1.5 rounded text-xs font-medium flex items-center gap-2">
                                    <CheckCircle2 size={12} /> Compiled (WASM)
                                </div>

                                <div className="space-y-1 font-mono text-xs md:text-sm text-rose-pine-text leading-relaxed opacity-90">
                                    <div><span className="text-rose-pine-subtle select-none mr-4">1</span><span className="text-rose-pine-subtle">// Clean State</span></div>
                                    <div><span className="text-rose-pine-subtle select-none mr-4">2</span><span className="text-rose-pine-iris">account</span> Counter {"{"}</div>
                                    <div><span className="text-rose-pine-subtle select-none mr-4">3</span>&nbsp;&nbsp;val: u64;</div>
                                    <div><span className="text-rose-pine-subtle select-none mr-4">4</span>{"}"}</div>
                                    <div><span className="text-rose-pine-subtle select-none mr-4">5</span></div>
                                    <div><span className="text-rose-pine-subtle select-none mr-4">6</span><span className="text-rose-pine-iris">pub</span> <span className="text-rose-pine-love">increment</span>(state: Counter) {"{"}</div>
                                    <div><span className="text-rose-pine-subtle select-none mr-4">7</span>&nbsp;&nbsp;state.val += 1;</div>
                                    <div><span className="text-rose-pine-subtle select-none mr-4">8</span>{"}"}</div>
                                </div>

                                <div className="mt-8 pt-4 border-t border-white/5 flex justify-between items-center">
                                    <div className="text-[10px] text-rose-pine-subtle flex items-center gap-2">
                                        <div className="w-2 h-2 rounded-full bg-rose-pine-foam animate-pulse" />
                                        Ready to deploy
                                    </div>
                                    <button className="px-4 py-2 rounded bg-gradient-to-r from-rose-pine-love to-rose-pine-iris text-white text-xs font-bold hover:brightness-110 transition-all">
                                        Deploy Now
                                    </button>
                                </div>
                            </div>
                        </motion.div>

                        {/* Floating 'No Tools' badges (Re-positioned) */}
                        <motion.div
                            initial={{ opacity: 0, x: -20 }}
                            whileInView={{ opacity: 1, x: 0 }}
                            viewport={{ once: true }}
                            transition={{ duration: 0.8, delay: 0.3 }}
                            className="absolute -left-4 md:-left-12 top-1/2 -translate-y-1/2 flex flex-col gap-3 z-20 pointer-events-none"
                        >
                            <div className="px-4 py-2 rounded-lg bg-[#26233a]/90 backdrop-blur border border-rose-pine-love/20 text-rose-pine-subtle text-xs font-mono shadow-xl whitespace-nowrap opacity-60 line-through decoration-rose-pine-love decoration-2">
                                rustup update
                            </div>
                            <div className="px-4 py-2 rounded-lg bg-[#26233a]/90 backdrop-blur border border-rose-pine-love/20 text-rose-pine-subtle text-xs font-mono shadow-xl whitespace-nowrap opacity-60 line-through decoration-rose-pine-love decoration-2 ml-4">
                                solana-install
                            </div>
                            <div className="px-4 py-2 rounded-lg bg-[#26233a]/90 backdrop-blur border border-rose-pine-love/20 text-rose-pine-subtle text-xs font-mono shadow-xl whitespace-nowrap opacity-60 line-through decoration-rose-pine-love decoration-2">
                                cargo build-bpf
                            </div>
                        </motion.div>

                    </div>


                    {/* Text Context (Right aligned now for balance) */}
                    <div className="order-1">
                        <motion.div
                            initial={{ opacity: 0, x: -20 }}
                            whileInView={{ opacity: 1, x: 0 }}
                            viewport={{ once: true }}
                            transition={{ duration: 0.6 }}
                        >
                            <div className="flex items-center gap-3 mb-6">
                                <div className="p-2 rounded-lg bg-rose-pine-foam/10 border border-rose-pine-foam/20 text-rose-pine-foam">
                                    <Cloud size={24} />
                                </div>
                                <h2 className="text-3xl md:text-5xl font-black text-rose-pine-text tracking-tight">
                                    The Browser is <br /> <span className="text-transparent bg-clip-text bg-gradient-to-r from-rose-pine-foam to-rose-pine-iris">Your Devkit</span>
                                </h2>
                            </div>

                            <p className="text-xl text-rose-pine-muted font-light leading-relaxed mb-8">
                                No terminal to configure. No toolchain to break.
                                <span className="block mt-2 text-rose-pine-text font-medium">Build, Deploy, and Execute from anywhere.</span>
                            </p>

                            <ul className="space-y-4 mb-10">
                                <li className="flex items-center gap-3 text-rose-pine-subtle">
                                    <CheckCircle2 className="text-rose-pine-foam shrink-0" size={20} />
                                    <span>Client-side WASM Compilation</span>
                                </li>
                                <li className="flex items-center gap-3 text-rose-pine-subtle">
                                    <CheckCircle2 className="text-rose-pine-foam shrink-0" size={20} />
                                    <span>Zero local dependencies</span>
                                </li>
                                <li className="flex items-center gap-3 text-rose-pine-subtle">
                                    <CheckCircle2 className="text-rose-pine-foam shrink-0" size={20} />
                                    <span>Compatible with any device</span>
                                </li>
                            </ul>

                            <Link href="/ide">
                                <button className="group flex items-center gap-2 text-rose-pine-foam font-bold hover:gap-4 transition-all">
                                    Launch Web IDE <ArrowRight size={20} />
                                </button>
                            </Link>

                        </motion.div>
                    </div>

                </div>

            </div>
        </section>
    );
}
