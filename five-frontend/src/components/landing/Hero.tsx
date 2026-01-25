"use client";

import { ArrowRight, Terminal, Hammer } from "lucide-react";
import Link from "next/link";
import { m, LazyMotion, domAnimation, AnimatePresence } from "framer-motion";
import { useState, useEffect } from "react";

export default function Hero() {
    // Generate blocks (client-side only to avoid hydration mismatch)
    const [blocks, setBlocks] = useState<any[]>([]);
    
    useEffect(() => {
        // More rows/cols for a denser wall
        const rows = 8;
        const cols = 12; 
        const generatedBlocks = Array.from({ length: rows * cols }, (_, i) => {
            const row = Math.floor(i / cols);
            const col = i % cols;
            
            // Stagger every other row for a brick pattern
            const xOffset = row % 2 === 0 ? 0 : 40; 
            
            return {
                id: i,
                // Grid position logic instead of random
                gridRow: row,
                gridCol: col,
                // Explosion vectors based on center of screen
                x: (Math.random() - 0.5) * 1200,
                y: (Math.random() - 0.5) * 800 + 200,
                rotate: (Math.random() - 0.5) * 720,
                delay: Math.random() * 0.3,
                
                content: `$${Math.floor(Math.random() * (900 - 200) + 200)}`,
                isAccent: Math.random() > 0.85
            };
        });
        setBlocks(generatedBlocks);
    }, []);

    return (
        <LazyMotion features={domAnimation}>
            <section className="relative min-h-[90vh] flex flex-col justify-center items-center px-4 pt-20 pb-20 overflow-hidden">

                {/* Background Glows */}
                <m.div
                    animate={{ opacity: [0.3, 0.6, 0.3], scale: [1, 1.1, 1] }}
                    transition={{ duration: 8, repeat: Infinity, ease: "easeInOut" }}
                    className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[800px] h-[800px] bg-rose-pine-iris/10 rounded-full blur-[60px] pointer-events-none will-change-transform translate-z-0"
                />
                <div className="absolute top-0 right-0 w-[500px] h-[500px] bg-rose-pine-love/5 rounded-full blur-[60px] pointer-events-none will-change-transform translate-z-0" />
                <div className="absolute bottom-0 left-0 w-[600px] h-[600px] bg-rose-pine-foam/5 rounded-full blur-[60px] pointer-events-none will-change-transform translate-z-0" />

                {/* THE WALL OVERLAY */}
                {/* z index 50 to sit on top of everything. Pointer events auto so we can inspect if needed, but none usually on wall */}
                <div className="absolute inset-0 top-0 z-50 pointer-events-none flex items-center justify-center">
                    {/* Container with constrained width to look like a wall section */}
                    <div className="relative w-[900px] h-[600px] flex flex-wrap content-center justify-center gap-1">
                        
                        {/* GRAFFITI LAYER - Absolute positioned over the wall blocks */}
                        <m.div
                            initial={{ opacity: 1, scale: 1, filter: "blur(0px)" }}
                            animate={{ opacity: 0, scale: 1.2, filter: "blur(10px)" }}
                            transition={{ duration: 1.2, ease: "easeIn", delay: 2.0 }}
                            className="absolute inset-0 z-20 flex flex-col items-center justify-center pointer-events-none mix-blend-hard-light"
                        >
                            <div className="relative">
                                {/* Spray Paint Drips/Splatter Effect (CSS shapes) */}
                                <div className="absolute -top-10 -left-10 w-32 h-32 bg-rose-pine-love/20 blur-xl rounded-full" />
                                <div className="absolute bottom-0 right-0 w-40 h-40 bg-rose-pine-love/20 blur-xl rounded-full" />
                                
                                <h2 
                                    className="text-[12rem] font-black leading-none text-rose-pine-love opacity-90 rotate-[-5deg] drop-shadow-lg font-mono tracking-tighter select-none"
                                    style={{ 
                                        maskImage: "url('/noise.png')",
                                        WebkitMaskImage: "url('/noise.png')",
                                        maskSize: "contain",
                                        WebkitMaskSize: "contain"
                                    }}
                                >
                                    THE WALL
                                </h2>
                            </div>
                            
                            <h3 className="text-5xl font-black text-rose-pine-base bg-rose-pine-love px-6 py-2 rotate-[2deg] -mt-10 uppercase tracking-widest shadow-lg transform skew-x-12 opacity-90">
                                Mainnet Barrier
                            </h3>
                        </m.div>

                        <AnimatePresence>
                            {blocks.map((block) => (
                                <m.div
                                    key={block.id}
                                    layoutId={`block-${block.id}`}
                                    initial={{ 
                                        x: 0, 
                                        y: 0, 
                                        rotate: 0, 
                                        opacity: 1,
                                        scale: 1 
                                    }}
                                    animate={{ 
                                        x: block.x, 
                                        y: block.y, 
                                        rotate: block.rotate, 
                                        opacity: 0,
                                        scale: 0.5 
                                    }}
                                    transition={{ 
                                        duration: 2.0,  
                                        ease: [0.22, 1, 0.36, 1], // Custom heavy ease
                                        delay: 2.0 + block.delay 
                                    }}
                                    className={`
                                        w-24 h-14 
                                        flex items-center justify-center 
                                        rounded-sm border-2
                                        font-mono font-bold text-lg
                                        shadow-[2px_2px_4px_rgba(0,0,0,0.4)]
                                        
                                        ${block.isAccent 
                                            ? "bg-rose-pine-muted text-rose-pine-base border-rose-pine-subtle/50" // Concrete Grey accent
                                            : "bg-rose-pine-overlay text-rose-pine-subtle border-rose-pine-hl-low/40"} // Dark concrete
                                    `}
                                    style={{
                                        // Slight brick offset logic handled by flex wrap gap, or we could strict grid it.
                                        // Flex wrap with gap-1 creates a decent 'piled' look.
                                    }}
                                >
                                    {block.content}
                                </m.div>
                            ))}
                        </AnimatePresence>
                    </div>
                </div>

                <div className="relative z-10 max-w-6xl w-full flex flex-col items-center text-center gap-10">

                    {/* Hero Container (Preserved) */}
                    <m.div
                        initial={{ opacity: 0, scale: 0.9 }}
                        animate={{ opacity: 1, scale: 1 }}
                        transition={{ duration: 0.8, delay: 2.5, ease: "circOut" }} // Delay until wall breaks
                        className="relative flex flex-col items-center mb-10 group"
                    >
                        {/* Main Title */}
                        <div className="relative z-10">
                            <h1 className="text-8xl md:text-[10rem] font-black tracking-tighter leading-none bg-clip-text text-transparent bg-gradient-to-b from-rose-pine-iris via-rose-pine-iris to-rose-pine-love drop-shadow-2xl select-none mb-4">
                                5IVE
                            </h1>
                        </div>

                        <p className="relative z-10 text-3xl md:text-5xl font-bold text-rose-pine-subtle tracking-tight max-w-4xl mx-auto drop-shadow-md">
                            Tear down the wall. <span className="text-rose-pine-iris">Build the Moat.</span>
                        </p>
                    </m.div>

                    {/* CTAs */}
                    <m.div
                        initial={{ opacity: 0, y: 30 }}
                        animate={{ opacity: 1, y: 0 }}
                        transition={{ delay: 2.8, duration: 0.6 }} // Reveal after title
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
