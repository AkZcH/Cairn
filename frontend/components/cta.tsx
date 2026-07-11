"use client";

import { motion } from "framer-motion";

export function CTA() {
  return (
    <section className="relative overflow-hidden border-t border-line/60">
      <div className="pointer-events-none absolute left-1/2 top-1/2 h-[300px] w-[500px] -translate-x-1/2 -translate-y-1/2 rounded-full bg-moss/10 blur-[120px]" />

      <div className="relative mx-auto max-w-3xl px-6 py-32 text-center">
        <motion.h2
          initial={{ opacity: 0, y: 12 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mb-10 text-4xl font-semibold tracking-tight leading-[1.05] md:text-6xl"
        >
          Built to remember.
          <br />
          <span className="glow-text">Ready when you are.</span>
        </motion.h2>

        <motion.div
          initial={{ opacity: 0, y: 12 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6, delay: 0.1 }}
          className="flex flex-col items-center justify-center gap-4 sm:flex-row"
        >
          <a
            href="#"
            className="focus-ring rounded-full bg-moss px-6 py-3 font-medium text-bg transition-colors hover:bg-moss-soft"
          >
            Get Started
          </a>

          <a
            href="https://github.com/AkZcH/Cairn"
            target="_blank"
            rel="noopener noreferrer"
            className="focus-ring rounded-full border border-line px-6 py-3 text-ink transition-colors hover:border-moss/50"
          >
            View Source
          </a>
        </motion.div>
      </div>
    </section>
  );
}
