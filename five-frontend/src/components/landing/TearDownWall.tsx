"use client";

import { motion, useScroll, useTransform } from "framer-motion";
import { useRef } from "react";
import { ShieldCheck, Coins, Unlink } from "lucide-react";

export default function TearDownWall() {
    const containerRef = useRef<HTMLDivElement>(null);
    const { scrollYProgress } = useScroll({
        target: containerRef,
        offset: ["start center", "end start"],
    });

    // Create a grid of "bricks" for the wall
    const rows = 8;
    const cols = 12;
    const blocks = Array.from({ length: rows * cols }, (_, i) => ({
        id: i,
        // Calculate random explosion vectors
        x: (Math.random() - 0.5) * 500,
        y: (Math.random() - 0.5) * 500,
        rotate: (Math.random() - 0.5) * 360,
        delay: Math.random() * 0.5, // Staggered explosion
    }));

    return (
        <section ref={containerRef} className="relative py-32 px-4 overflow-hidden min-h-[80vh] flex items-center justify-center">
             {/* Background Atmosphere - what is revealed */}
             <div className="absolute inset-0 bg-gradient-to-b from-transparent via-rose-pine-base/50 to-transparent z-0" />
            
             <div className="relative z-10 w-full max-w-6xl mx-auto text-center">
                
                {/* The Message Behind the Wall */}
                <div className="relative z-0 py-20">
                    <motion.div
                        initial={{ opacity: 0, scale: 0.8 }}
                        whileInView={{ opacity: 1, scale: 1 }}
                        transition={{ duration: 1, delay: 0.5 }}
                    >
                        <h2 className="text-5xl md:text-8xl font-black mb-6 tracking-tighter">
                            <span className="text-transparent bg-clip-text bg-gradient-to-r from-rose-pine-iris to-rose-pine-foam">
                                BUILD THE MOAT
                            </span>
                        </h2>
                        <p className="text-xl md:text-2xl text-rose-pine-subtle max-w-2xl mx-auto font-medium">
                            The barrier is gone. The entire world plays on Mainnet now.
                        </p>
                    </motion.div>
                </div>

                {/* The Digital Wall Overlay */}
                <div className="absolute inset-0 z-10 flex flex-wrap content-center justify-center pointer-events-none">
                    {blocks.map((block) => (
                        <WallBlock 
                            key={block.id} 
                            block={block} 
                            total={blocks.length} 
                        />
                    ))}
                </div>

             </div>
        </section>
    );
}

function WallBlock({ block, total }: { block: any, total: number }) {
    // Determine content: Mostly binary, some hex
    const content = Math.random() > 0.5 ? (Math.random() > 0.5 ? "1" : "0") : (Math.random() > 0.5 ? "0x" : "FF");
    
    // Gradient logic for the wall look
    const isAccent = Math.random() > 0.8;
    
    return (
        <motion.div
            initial={{ 
                x: 0, 
                y: 0, 
                rotate: 0, 
                opacity: 1,
                scale: 1 
            }}
            whileInView={{ 
                x: block.x * 2, // Explode outwards
                y: block.y * 2 + 100, // Fall down a bit
                rotate: block.rotate, 
                opacity: 0,
                scale: 0 
            }}
            viewport={{ once: true, margin: "-100px" }}
            transition={{ 
                duration: 1.5, 
                ease: [0.22, 1, 0.36, 1], // Custom cubic bezier
                delay: block.delay 
            }}
            className={`
                w-12 h-16 md:w-20 md:h-12 m-1 
                flex items-center justify-center 
                rounded-sm border border-rose-pine-hl-low/30
                backdrop-blur-md
                font-mono font-bold text-xs md:text-sm
                shadow-lg
                ${isAccent 
                    ? "bg-rose-pine-love/20 text-rose-pine-love border-rose-pine-love/40" 
                    : "bg-rose-pine-surface/80 text-rose-pine-subtle/50 bg-opacity-90"}
            `}
        >
            {content}
        </motion.div>
    );
}
