"use client";

import { motion } from "framer-motion";

const FEATURES = [
  {
    title: "Structure-aware chunking",
    body: "Splits on headings, not fixed windows, so retrieved context stays coherent.",
  },
  {
    title: "Edit-aware ingestion",
    body: "Re-ingest a changed file and its old chunks are replaced, never forked.",
  },
  {
    title: "Hybrid retrieval",
    body: "\u201cpostgres\u201d and \u201cPostgreSQL\u201d resolve as the same concept, not two.",
  },
  {
    title: "Local embeddings",
    body: "Computed in Rust, on your machine. No GPU, no cloud model in the loop.",
  },
  {
    title: "Multi-turn memory",
    body: "Conversations persist. Context carries across questions, not just within one.",
  },
  {
    title: "Isolated by design",
    body: "Every query scoped to your account. Verified with real accounts, not assumed.",
  },
];

export function Features() {
  return (
    <section id="features" className="border-t border-line/60">
      <div className="max-w-6xl mx-auto px-6 py-32">
        <motion.p
          initial={{ opacity: 0, y: 8 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="font-mono text-xs uppercase tracking-widest text-moss mb-4"
        >
          Features
        </motion.p>

        <motion.h2
          initial={{ opacity: 0, y: 8 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5, delay: 0.05 }}
          className="text-3xl md:text-4xl font-semibold tracking-tight mb-16 max-w-xl"
        >
          Not another wrapper around a vector store.
        </motion.h2>

        <div className="grid md:grid-cols-3 gap-px bg-line rounded-xl overflow-hidden">
          {FEATURES.map((f, i) => (
            <motion.div
              key={f.title}
              initial={{ opacity: 0, y: 16 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: (i % 3) * 0.08 }}
              className="group bg-bg p-8 hover:bg-surface/60 transition-colors"
            >
              <div className="w-8 h-8 rounded-md border border-line group-hover:border-moss/50 flex items-center justify-center mb-6 transition-colors">
                <div className="w-1.5 h-1.5 rounded-sm bg-stone group-hover:bg-moss transition-colors" />
              </div>
              <h3 className="font-semibold mb-2">{f.title}</h3>
              <p className="text-stone text-sm leading-relaxed">{f.body}</p>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}