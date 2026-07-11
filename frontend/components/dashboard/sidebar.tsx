"use client";

import { useEffect, useState } from "react";
import {
  listConversations,
  type ConversationSummary,
  clearToken,
} from "@/lib/api";

export function Sidebar({
  activeId,
  onSelect,
  onNewChat,
  refreshKey,
}: {
  activeId: string | null;
  onSelect: (id: string) => void;
  onNewChat: () => void;
  refreshKey: number;
}) {
  const [conversations, setConversations] = useState<ConversationSummary[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    listConversations()
      .then(setConversations)
      .catch(() => setConversations([]))
      .finally(() => setLoading(false));
  }, [refreshKey]);

  return (
    <aside className="w-64 shrink-0 border-r border-line/60 h-screen flex flex-col bg-black/30">
      <div className="p-4 border-b border-line/60">
        <div className="flex items-center gap-2 mb-6">
          <div className="w-5 h-5 rounded-sm bg-moss" />
          <span className="font-semibold tracking-tight text-sm">Cairn</span>
        </div>
        <button
          onClick={onNewChat}
          className="w-full text-left text-sm bg-surface border border-line rounded-lg px-3 py-2.5 hover:border-moss/50 transition-colors focus-ring"
        >
          + New chat
        </button>
      </div>

      <div className="flex-1 overflow-y-auto p-4">
        <p className="font-mono text-[10px] uppercase tracking-widest text-stone mb-3">
          Conversations
        </p>
        {loading && <p className="text-stone text-xs">Loading...</p>}
        {!loading && conversations.length === 0 && (
          <p className="text-stone text-xs leading-relaxed">
            No conversations yet. Ask something to start one.
          </p>
        )}
        <ul className="flex flex-col gap-1">
          {conversations.map((c) => (
            <li key={c.id}>
              <button
                onClick={() => onSelect(c.id)}
                className={`w-full text-left text-sm px-3 py-2 rounded-md truncate transition-colors focus-ring ${
                  activeId === c.id
                    ? "bg-surface text-ink"
                    : "text-stone hover:text-ink hover:bg-surface/50"
                }`}
              >
                {c.title || "Untitled"}
              </button>
            </li>
          ))}
        </ul>
      </div>

      <div className="p-4 border-t border-line/60">
        <button
          onClick={() => {
            clearToken();
            window.location.reload();
          }}
          className="font-mono text-[10px] uppercase tracking-widest text-stone hover:text-ink transition-colors focus-ring"
        >
          Disconnect
        </button>
      </div>
    </aside>
  );
}
