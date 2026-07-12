"use client";

import { useRef, useState } from "react";
import { uploadFile, uploadText } from "@/lib/api";

export function UploadModal({ onClose }: { onClose: () => void }) {
  const [tab, setTab] = useState<"paste" | "file">("paste");
  const [title, setTitle] = useState("");
  const [content, setContent] = useState("");
  const [dragOver, setDragOver] = useState(false);
  const [status, setStatus] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  function describe(res: { status: string; chunks: number }) {
    const verb =
      res.status === "inserted"
        ? "Added"
        : res.status === "updated"
          ? "Updated"
          : "Already up to date";
    return `${verb} — ${res.chunks} chunk(s)`;
  }

  async function handlePasteSubmit() {
    if (!content.trim()) return;
    setBusy(true);
    setStatus(null);
    try {
      const res = await uploadText(title || "Untitled paste", content);
      setStatus(describe(res));
      setContent("");
      setTitle("");
    } catch {
      setStatus("Upload failed. Try again.");
    } finally {
      setBusy(false);
    }
  }

  async function handleFile(file: File) {
    setBusy(true);
    setStatus(null);
    try {
      const res = await uploadFile(file);
      setStatus(describe(res));
    } catch {
      setStatus("Upload failed. Only .md, .markdown, .txt are supported.");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-50 px-6">
      <div className="w-full max-w-lg bg-surface border border-line rounded-xl p-6">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-lg font-semibold tracking-tight">
            Add to your knowledge
          </h2>
          <button
            onClick={onClose}
            className="text-stone hover:text-ink transition-colors focus-ring"
          >
            ✕
          </button>
        </div>

        <div className="flex gap-2 mb-6">
          {(["paste", "file"] as const).map((t) => (
            <button
              key={t}
              onClick={() => setTab(t)}
              className={`font-mono text-xs uppercase tracking-widest px-3 py-1.5 rounded-full transition-colors focus-ring ${
                tab === t ? "bg-moss text-bg" : "text-stone hover:text-ink"
              }`}
            >
              {t === "paste" ? "Paste text" : "Upload file"}
            </button>
          ))}
        </div>

        {tab === "paste" ? (
          <div className="flex flex-col gap-3">
            <input
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="Title (optional)"
              className="bg-bg border border-line rounded-lg px-3 py-2 text-sm text-ink placeholder:text-stone/50 focus-ring"
            />
            <textarea
              value={content}
              onChange={(e) => setContent(e.target.value)}
              placeholder="Paste markdown or plain text..."
              rows={8}
              className="bg-bg border border-line rounded-lg px-3 py-2 text-sm text-ink placeholder:text-stone/50 focus-ring resize-none font-mono"
            />
            <button
              onClick={handlePasteSubmit}
              disabled={busy || !content.trim()}
              className="bg-moss text-bg font-medium px-6 py-2.5 rounded-full hover:bg-moss-soft transition-colors focus-ring disabled:opacity-40"
            >
              {busy ? "Ingesting..." : "Add"}
            </button>
          </div>
        ) : (
          <div
            onDragOver={(e) => {
              e.preventDefault();
              setDragOver(true);
            }}
            onDragLeave={() => setDragOver(false)}
            onDrop={(e) => {
              e.preventDefault();
              setDragOver(false);
              const file = e.dataTransfer.files?.[0];
              if (file) handleFile(file);
            }}
            onClick={() => fileInputRef.current?.click()}
            className={`border-2 border-dashed rounded-lg py-12 text-center cursor-pointer transition-colors ${
              dragOver
                ? "border-moss bg-moss/5"
                : "border-line hover:border-moss/40"
            }`}
          >
            <p className="text-sm text-stone mb-1">
              Drag a file here, or click to browse
            </p>
            <p className="font-mono text-xs text-stone/60">
              .md · .markdown · .txt
            </p>
            <input
              ref={fileInputRef}
              type="file"
              accept=".md,.markdown,.txt"
              className="hidden"
              onChange={(e) => {
                const file = e.target.files?.[0];
                if (file) handleFile(file);
              }}
            />
          </div>
        )}

        {status && <p className="font-mono text-xs text-moss mt-4">{status}</p>}
      </div>
    </div>
  );
}
