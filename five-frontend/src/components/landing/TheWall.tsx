"use client";

import { motion } from "framer-motion";
import { ShieldCheck, Coins, Unlink, Hammer, XCircle, CheckCircle2 } from "lucide-react";

export default function TheWall() {
    return (
        <section className="relative py-32 px-4 overflow-hidden bg-transparent">
            {/* Background Atmosphere */}
            <div className="absolute inset-0 pointer-events-none">
                <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] bg-rose-pine-love/5 rounded-full blur-[120px]" />
            </div>

            <div className="relative z-10 max-w-7xl mx-auto">
                <div className="grid lg:grid-cols-2 gap-16 items-center">

                    {/* Text Context (Left for variety - Napkin was Right) */}
                    <div className="order-2 lg:order-1">
                        <motion.div
                            initial={{ opacity: 0, x: -20 }}
                            whileInView={{ opacity: 1, x: 0 }}
                            viewport={{ once: true }}
                            transition={{ duration: 0.6 }}
                        >
                            <div className="flex items-center gap-3 mb-6">
                                <div className="p-2 rounded-lg bg-rose-pine-love/10 border border-rose-pine-love/20 text-rose-pine-love">
                                    <Hammer size={24} />
                                </div>
                                <h2 className="text-3xl md:text-5xl font-black text-rose-pine-text tracking-tight">
                                    Tear Down <br /> <span className="text-transparent bg-clip-text bg-gradient-to-r from-rose-pine-love to-rose-pine-iris">This Wall</span>
                                </h2>
                            </div>

                            <p className="text-xl text-rose-pine-subtle leading-relaxed mb-8 text-contrast">
                                The cost of Mainnet has killed too many great ideas.
                                <span className="block mt-2 text-rose-pine-text font-medium">5IVE destroys the barrier, turning $1,000 deployments into $1.</span>
                            </p>

                            <div className="space-y-4 mb-10 text-contrast">
                                <p className="text-lg font-medium text-rose-pine-text italic border-l-4 border-rose-pine-love pl-4 py-2 bg-rose-pine-love/5 rounded-r-lg">
                                    "Devnet is no longer the graveyard to great ideas. Let your ideas flourish. Let them bloom. We will tear down this wall and open it up for all."
                                </p>
                            </div>

                            <ul className="space-y-4 mb-10 text-contrast">
                                <li className="flex items-start gap-3 text-rose-pine-subtle">
                                    <ShieldCheck className="mt-1 text-rose-pine-iris shrink-0" size={20} />
                                    <span><b>L1 Security. L2 Economics.</b> No Bridges.</span>
                                </li>
                                <li className="flex items-start gap-3 text-rose-pine-subtle">
                                    <Unlink className="mt-1 text-rose-pine-iris shrink-0" size={20} />
                                    <span><b>Opportunity is Everything.</b> Build on Mainnet.</span>
                                </li>
                            </ul>

                        </motion.div>
                    </div>

                    {/* Visual: The Wall vs The Gate (Right) */}
                    <div className="order-1 lg:order-2 relative">
                        <div className="absolute -inset-10 bg-gradient-to-bl from-rose-pine-love/20 to-transparent blur-3xl opacity-50" />

                        <motion.div
                            initial={{ opacity: 0, scale: 0.95 }}
                            whileInView={{ opacity: 1, scale: 1 }}
                            viewport={{ once: true }}
                            transition={{ duration: 0.8 }}
                            className="rounded-3xl border border-rose-pine-hl-low/20 bg-rose-pine-surface overflow-hidden shadow-2xl shadow-rose-pine-love/10 grid grid-rows-2 divide-y divide-rose-pine-hl-low/10"
                        >
                            {/* The Old Way (The Wall) */}
                            <div className="p-8 bg-rose-pine-base/50 flex items-center justify-between group">
                                <div>
                                    <div className="text-xs font-mono uppercase tracking-widest text-rose-pine-subtle mb-1">Standard SOL</div>
                                    <div className="text-2xl font-bold text-rose-pine-love flex items-center gap-2">
                                        <XCircle size={20} />
                                        $1,000+
                                    </div>
                                    <div className="text-xs text-rose-pine-subtle mt-2">PAYWALL BARRIER</div>
                                </div>
                                <div className="h-12 w-1 bg-rose-pine-love/20 rounded-full" />
                                <div className="text-right opacity-50 grayscale group-hover:grayscale-0 transition-all">
                                    <div className="text-xs font-mono text-rose-pine-subtle">Deploy</div>
                                    <div className="text-sm font-bold text-rose-pine-text">Failed</div>
                                </div>
                            </div>

                            {/* The New Way (The Open Door) */}
                            <div className="p-8 bg-rose-pine-surface relative overflow-hidden">
                                <div className="absolute top-0 left-0 w-1 h-full bg-rose-pine-iris" />
                                <div className="absolute inset-0 bg-rose-pine-iris/5" />

                                <div className="flex items-center justify-between relative z-10">
                                    <div>
                                        <div className="text-xs font-mono uppercase tracking-widest text-rose-pine-iris mb-1">5IVE Network</div>
                                        <div className="text-4xl font-black text-rose-pine-text flex items-center gap-2">
                                            <CheckCircle2 size={32} className="text-rose-pine-iris" />
                                            $1.00
                                        </div>
                                        <div className="text-xs text-rose-pine-iris mt-2 font-bold tracking-wide">PERMISSIONLESS</div>
                                    </div>

                                    <div className="text-right">
                                        <div className="text-xs font-mono text-rose-pine-iris">Status</div>
                                        <div className="text-lg font-bold text-rose-pine-text">Mainnet Live</div>
                                        <div className="text-[10px] text-rose-pine-subtle mt-1">Ready for the World</div>
                                    </div>
                                </div>
                            </div>

                        </motion.div>
                    </div>

                </div>
            </div>
        </section>
    );
}
