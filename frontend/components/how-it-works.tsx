"use client";

import { motion } from "framer-motion";

const STEPS = [
  {
    n: "01",
    title: "Ingest",
    body: "Point it at a folder. Markdown and plaintext, parsed and chunked by heading, not arbitrary character windows.",
  },
  {
    n: "02",
    title: "Embed",
    body: "Local embeddings, computed on your own machine in Rust. No GPU required, nothing sent to a cloud model.",
  },
  {
    n: "03",
    title: "Retrieve",
    body: "Full-text and vector search, fused with Reciprocal Rank Fusion. Catches exact terms and related concepts alike.",
  },
  {
    n: "04",
    title: "Answer",
    body: "Every claim traceable to a passage you actually wrote. Cited inline, or it says it doesn't know.",
  },
];

export function HowItWorks() {
  return (
    <section id="how" className="border-t border-line/60">
      <div className="max-w-6xl mx-auto px-6 py-32">
        <motion.p
          initial={{ opacity: 0, y: 8 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="font-mono text-xs uppercase tracking-widest text-moss mb-4"
        >
          How it works
        </motion.p>

        <motion.h2
          initial={{ opacity: 0, y: 8 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5, delay: 0.05 }}
          className="text-3xl md:text-4xl font-semibold tracking-tight mb-20 max-w-xl"
        >
          Four steps. In order. Every time.
        </motion.h2>

        {/* Connecting trail — desktop only */}
        <div className="hidden md:block relative">
          <div className="absolute top-[13px] left-0 right-0 h-px bg-line" />

          <div className="grid grid-cols-4 gap-8 relative">
            {STEPS.map((step, i) => (
              <motion.div
                key={step.n}
                initial={{ opacity: 0, y: 16 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.5, delay: i * 0.1 }}
              >
                <div className="relative flex items-center mb-6">
                  <div className="w-[27px] h-[27px] rounded-full border border-line bg-bg flex items-center justify-center relative z-10">
                    <div className="w-1.5 h-1.5 rounded-full bg-moss" />
                  </div>
                </div>
                <span className="font-mono text-xs text-moss">{step.n}</span>
                <h3 className="text-xl font-semibold mt-2 mb-3">
                  {step.title}
                </h3>
                <p className="text-stone text-sm leading-relaxed">
                  {step.body}
                </p>
              </motion.div>
            ))}
          </div>
        </div>

        {/* Stacked trail — mobile */}
        <div className="md:hidden relative pl-8">
          <div className="absolute top-0 bottom-0 left-[13px] w-px bg-line" />

          <div className="flex flex-col gap-12">
            {STEPS.map((step, i) => (
              <motion.div
                key={step.n}
                initial={{ opacity: 0, x: -12 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.5, delay: i * 0.1 }}
                className="relative"
              >
                <div className="absolute -left-8 top-1 w-[27px] h-[27px] rounded-full border border-line bg-bg flex items-center justify-center">
                  <div className="w-1.5 h-1.5 rounded-full bg-moss" />
                </div>
                <span className="font-mono text-xs text-moss">{step.n}</span>
                <h3 className="text-xl font-semibold mt-2 mb-3">
                  {step.title}
                </h3>
                <p className="text-stone text-sm leading-relaxed">
                  {step.body}
                </p>
              </motion.div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}
