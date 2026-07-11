"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { hasToken } from "@/lib/api";
import { Sidebar } from "@/components/dashboard/sidebar";
import { ChatPanel } from "@/components/dashboard/chat-panel";

export default function DashboardPage() {
  const router = useRouter();
  const [checked, setChecked] = useState(false);
  const [activeId, setActiveId] = useState<string | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);

  useEffect(() => {
    if (!hasToken()) {
      router.push("/login");
    } else {
      setChecked(true);
    }
  }, [router]);

  if (!checked) return null;

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