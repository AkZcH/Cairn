"use client";

import { motion } from "framer-motion";
import { StrataStack } from "./strata-stack";

export function Hero() {
  return (
    <section className="relative overflow-hidden">
      {/* Grid + glow backdrop */}
      <div className="absolute inset-0 bg-grid hero-fade pointer-events-none" />
      <div className="absolute top-0 left-1/2 -translate-x-1/2 w-[600px] h-[400px] bg-moss/20 rounded-full blur-[120px] pointer-events-none" />

      <div className="relative max-w-4xl mx-auto px-6 pt-32 pb-24 flex flex-col items-center text-center">
        <motion.div
          initial={{ opacity: 0, y: 8 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5 }}
          className="inline-flex items-center gap-2 rounded-full border border-line bg-surface/60 px-4 py-1.5 mb-8"
        >
          <span className="w-1.5 h-1.5 rounded-full bg-moss animate-pulse" />
          <span className="font-mono text-xs uppercase tracking-widest text-stone">
            A knowledge operating system
          </span>
        </motion.div>

        <motion.h1
          initial={{ opacity: 0, y: 12 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.1 }}
          className="text-5xl md:text-7xl font-semibold tracking-tight leading-[1.05] mb-6"
        >
          Your knowledge,
          <br />
          built up in <span className="glow-text">layers.</span>
        </motion.h1>

        <motion.p
          initial={{ opacity: 0, y: 12 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.2 }}
          className="text-stone text-base md:text-lg leading-relaxed mb-10 max-w-2xl"
        >
          Cairn ingests your notes, retrieves with hybrid search, and answers
          with citations, never a guess dressed up as fact.
        </motion.p>

        <motion.div
          initial={{ opacity: 0, y: 12 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.3 }}
          className="flex items-center gap-4 mb-5"
        >
          <a
            href="/signup"
            className="bg-moss text-bg font-medium px-6 py-3 rounded-md hover:bg-moss-soft transition-colors focus-ring"
          >
            Get started
          </a>
          <a
            href="#how"
            className="border border-line text-ink px-6 py-3 rounded-md hover:border-moss/50 transition-colors focus-ring"
          >
            See how it works
          </a>
        </motion.div>

        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 0.6, delay: 0.4 }}
          className="font-mono text-xs text-stone bg-surface border border-line rounded-full px-5 py-2.5 mb-16"
        >
          <span className="text-moss">$</span> git clone
          https://github.com/AkZcH/Cairn.git
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 16 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.5 }}
          className="w-full max-w-md mx-auto"
        >
          <StrataStack />
        </motion.div>
      </div>
    </section>
  );
}
