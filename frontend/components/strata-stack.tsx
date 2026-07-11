"use client";

import { motion } from "framer-motion";

const LAYERS = [
  { width: "72%", label: "raw notes" },
  { width: "58%", label: "chunked" },
  { width: "84%", label: "embedded" },
  { width: "46%", label: "retrieved" },
  { width: "66%", label: "answered" },
];

export function StrataStack() {
  return (
    <div className="flex flex-col-reverse gap-2 w-full max-w-md">
      {LAYERS.map((layer, i) => (
        <motion.div
          key={layer.label}
          initial={{ opacity: 0, y: 12, width: "0%" }}
          animate={{ opacity: 1, y: 0, width: layer.width }}
          transition={{
            delay: 0.15 * i,
            duration: 0.6,
            ease: [0.16, 1, 0.3, 1],
          }}
          className="group flex items-center gap-3"
        >
          <div className="h-8 rounded-sm bg-gradient-to-r from-moss/20 to-moss/5 border border-moss/30 flex-1" />
          <span className="font-mono text-[10px] uppercase tracking-widest text-stone opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap">
            {layer.label}
          </span>
        </motion.div>
      ))}
    </div>
  );
}
