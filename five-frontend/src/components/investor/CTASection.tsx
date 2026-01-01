"use client";

import React from "react";
import { motion } from "framer-motion";
import { ChevronRight } from "lucide-react";
import Link from "next/link";

/**
 * CTASection Component
 * Final call-to-action section encouraging users to try the IDE or view the source code
 * Includes animated buttons with hover effects
 */
const CTASection = React.memo(function CTASection() {
  return (
    <section className="relative w-full py-32 px-4 flex flex-col items-center overflow-hidden">
      {/* Background Elements */}
      <div className="absolute inset-0 pointer-events-none">
        <div className="absolute top-1/4 left-1/4 w-[500px] h-[500px] bg-rose-pine-iris/5 rounded-full blur-[100px]" />
        <div className="absolute bottom-1/4 right-1/4 w-[400px] h-[400px] bg-rose-pine-love/5 rounded-full blur-[100px]" />
      </div>

      <div className="relative z-10 max-w-5xl w-full text-center flex flex-col items-center gap-8">
        <motion.h2
          initial={{ opacity: 0, y: -20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="text-4xl md:text-5xl font-bold text-rose-pine-text"
        >
          Ready to Build the Future?
        </motion.h2>
        <motion.p
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ delay: 0.1 }}
          className="text-lg text-rose-pine-muted max-w-2xl"
        >
          Five Protocol is ready for production. Developers can start building today.
        </motion.p>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.2 }}
          className="flex flex-col sm:flex-row gap-4 w-full sm:w-auto justify-center"
        >
          <Link href="/ide">
            <motion.button
              whileHover={{ scale: 1.05, boxShadow: "0 0 30px -5px var(--color-rose-pine-iris)" }}
              whileTap={{ scale: 0.95 }}
              className="group relative px-10 py-4 rounded-2xl bg-gradient-to-r from-rose-pine-love to-rose-pine-iris text-rose-pine-base font-bold text-lg shadow-xl shadow-rose-pine-love/20 transition-all flex items-center gap-3 overflow-hidden whitespace-nowrap"
            >
              <span className="relative z-10">Try the IDE</span>
              <ChevronRight size={20} className="relative z-10 group-hover:translate-x-1 transition-transform" />
              <div className="absolute inset-0 bg-white/20 opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
            </motion.button>
          </Link>
          <motion.a
            href="https://github.com/five-org"
            target="_blank"
            whileHover={{ scale: 1.05 }}
            whileTap={{ scale: 0.95 }}
            className="px-10 py-4 rounded-2xl border-2 border-rose-pine-iris/40 bg-rose-pine-iris/5 text-rose-pine-iris font-bold text-lg hover:border-rose-pine-iris/70 transition-all flex items-center gap-3 justify-center whitespace-nowrap"
          >
            <span>View on GitHub</span>
            <ChevronRight size={20} />
          </motion.a>
        </motion.div>
      </div>
    </section>
  );
});

CTASection.displayName = "CTASection";

export default CTASection;
