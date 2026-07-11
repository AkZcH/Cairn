"use client";

import { useState } from "react";
import { hasToken } from "@/lib/api";
import { TokenGate } from "@/components/dashboard/token-gate";
import { Sidebar } from "@/components/dashboard/sidebar";
import { ChatPanel } from "@/components/dashboard/chat-panel";

export default function DashboardPage() {
  const [connected, setConnected] = useState(false);
  const [activeId, setActiveId] = useState<string | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);

  const isConnected = connected || hasToken();

  if (!isConnected) {
    return <TokenGate onConnected={() => setConnected(true)} />;
  }

  return (
    <div className="flex h-screen bg-bg text-ink">
      <Sidebar
        activeId={activeId}
        onSelect={setActiveId}
        onNewChat={() => setActiveId(null)}
        refreshKey={refreshKey}
      />
      <ChatPanel
        activeId={activeId}
        onConversationStart={(id) => {
          setActiveId(id);
          setRefreshKey((k) => k + 1);
        }}
      />
    </div>
  );
}
