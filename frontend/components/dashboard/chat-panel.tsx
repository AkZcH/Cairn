"use client";

import { useEffect, useRef, useState } from "react";
import {
  sendMessage,
  getConversationMessages,
  type Message,
  type Citation,
} from "@/lib/api";

function CitationChips({ citations }: { citations: Citation[] }) {
  const [open, setOpen] = useState<number | null>(null);

  return (
    <div className="flex flex-wrap gap-2 mt-3">
      {citations.map((c, i) => (
        <div key={c.chunk_id} className="relative">
          <button
            onClick={() => setOpen(open === i ? null : i)}
            className="font-mono text-[11px] text-moss border border-moss/30 rounded-full px-2.5 py-1 hover:bg-moss/10 transition-colors focus-ring"
          >
            [{i + 1}] {c.title || "untitled"}
          </button>
          {open === i && (
            <div className="absolute z-10 top-full mt-2 w-72 bg-surface border border-line rounded-lg p-4 shadow-xl">
              <p className="text-xs text-stone leading-relaxed">{c.content}</p>
            </div>
          )}
        </div>
      ))}
    </div>
  );
}

function Bubble({ message }: { message: Message }) {
  const isUser = message.role === "user";
  return (
    <div className={`flex ${isUser ? "justify-end" : "justify-start"}`}>
      <div className={`max-w-lg ${isUser ? "" : "w-full"}`}>
        <div
          className={
            isUser
              ? "bg-moss/15 border border-moss/20 rounded-2xl rounded-tr-sm px-4 py-3 text-sm text-ink"
              : "text-sm text-ink leading-relaxed"
          }
        >
          {message.content}
        </div>
        {!isUser && message.citations && message.citations.length > 0 && (
          <CitationChips citations={message.citations} />
        )}
      </div>
    </div>
  );
}

export function ChatPanel({
  activeId,
  onConversationStart,
}: {
  activeId: string | null;
  onConversationStart: (id: string) => void;
}) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [sending, setSending] = useState(false);
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!activeId) {
      setMessages([]);
      return;
    }
    getConversationMessages(activeId)
      .then(setMessages)
      .catch(() => setMessages([]));
  }, [activeId]);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  async function handleSend() {
    const question = input.trim();
    if (!question || sending) return;

    setInput("");
    setSending(true);
    setMessages((prev) => [
      ...prev,
      {
        role: "user",
        content: question,
        citations: null,
        created_at: new Date().toISOString(),
      },
    ]);

    try {
      const res = await sendMessage(question, activeId);
      setMessages((prev) => [
        ...prev,
        {
          role: "assistant",
          content: res.answer,
          citations: res.citations,
          created_at: new Date().toISOString(),
        },
      ]);
      if (!activeId) onConversationStart(res.conversation_id);
    } catch (err) {
      setMessages((prev) => [
        ...prev,
        {
          role: "assistant",
          content:
            "Something went wrong reaching Cairn. Check that the backend is running.",
          citations: null,
          created_at: new Date().toISOString(),
        },
      ]);
    } finally {
      setSending(false);
    }
  }

  const isEmpty = messages.length === 0;

  return (
    <div className="flex-1 flex flex-col h-screen">
      {isEmpty ? (
        <div className="flex-1 flex flex-col items-center justify-center px-6">
          <p className="font-mono text-xs uppercase tracking-widest text-moss mb-4">
            Ask your notes
          </p>
          <h2 className="text-2xl font-semibold tracking-tight mb-10 text-center">
            What do you want to know?
          </h2>
          <div className="w-full max-w-lg">
            <ChatInput
              value={input}
              onChange={setInput}
              onSend={handleSend}
              disabled={sending}
            />
          </div>
        </div>
      ) : (
        <>
          <div className="flex-1 overflow-y-auto px-6 py-8">
            <div className="max-w-2xl mx-auto flex flex-col gap-6">
              {messages.map((m, i) => (
                <Bubble key={i} message={m} />
              ))}
              {sending && (
                <div className="flex items-center gap-1.5 px-1">
                  <span className="w-1.5 h-1.5 rounded-full bg-stone animate-pulse" />
                  <span className="w-1.5 h-1.5 rounded-full bg-stone animate-pulse [animation-delay:150ms]" />
                  <span className="w-1.5 h-1.5 rounded-full bg-stone animate-pulse [animation-delay:300ms]" />
                </div>
              )}
              <div ref={bottomRef} />
            </div>
          </div>
          <div className="border-t border-line/60 px-6 py-4">
            <div className="max-w-2xl mx-auto">
              <ChatInput
                value={input}
                onChange={setInput}
                onSend={handleSend}
                disabled={sending}
              />
            </div>
          </div>
        </>
      )}
    </div>
  );
}

function ChatInput({
  value,
  onChange,
  onSend,
  disabled,
}: {
  value: string;
  onChange: (v: string) => void;
  onSend: () => void;
  disabled: boolean;
}) {
  return (
    <div className="flex items-center gap-2 bg-surface border border-line rounded-full px-4 py-2.5 focus-within:border-moss/50 transition-colors">
      <input
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter" && !e.shiftKey) {
            e.preventDefault();
            onSend();
          }
        }}
        placeholder="Ask anything about your notes..."
        className="flex-1 bg-transparent text-sm text-ink placeholder:text-stone/60 outline-none"
      />
      <button
        onClick={onSend}
        disabled={disabled || !value.trim()}
        className="bg-moss text-bg text-sm font-medium px-4 py-1.5 rounded-full hover:bg-moss-soft transition-colors disabled:opacity-40 disabled:cursor-not-allowed focus-ring"
      >
        Send
      </button>
    </div>
  );
}
