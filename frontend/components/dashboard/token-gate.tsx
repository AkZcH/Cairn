"use client";

import { useState } from "react";
import { setToken } from "@/lib/api";

export function TokenGate({ onConnected }: { onConnected: () => void }) {
  const [value, setValue] = useState("");

  return (
    <div className="min-h-screen bg-bg text-ink flex items-center justify-center px-6">
      <div className="w-full max-w-md">
        <p className="font-mono text-xs uppercase tracking-widest text-moss mb-4">
          Temporary session bridge
        </p>
        <h1 className="text-2xl font-semibold tracking-tight mb-3">
          Paste a session token
        </h1>
        <p className="text-stone text-sm leading-relaxed mb-8">
          Real login pages don&apos;t exist yet. Get a token from your
          backend&apos;s{" "}
          <code className="text-moss font-mono text-xs">POST /auth/login</code>,
          paste it below. This screen is replaced once real auth pages are
          built.
        </p>
        <textarea
          value={value}
          onChange={(e) => setValue(e.target.value)}
          placeholder="eyJhbGciOi..."
          rows={4}
          className="w-full bg-surface border border-line rounded-lg px-4 py-3 font-mono text-xs text-ink placeholder:text-stone/50 focus-ring resize-none mb-4"
        />
        <button
          onClick={() => {
            if (value.trim()) {
              setToken(value.trim());
              onConnected();
            }
          }}
          className="w-full bg-moss text-bg font-medium px-6 py-3 rounded-full hover:bg-moss-soft transition-colors focus-ring"
        >
          Connect
        </button>
      </div>
    </div>
  );
}
