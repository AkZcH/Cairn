const API_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8000";

function getToken(): string | null {
  if (typeof window === "undefined") return null;
  return localStorage.getItem("cairn_token");
}

export function setToken(token: string) {
  localStorage.setItem("cairn_token", token);
}

export function clearToken() {
  localStorage.removeItem("cairn_token");
}

export function hasToken(): boolean {
  return !!getToken();
}

async function authedFetch(path: string, options: RequestInit = {}) {
  const token = getToken();
  const res = await fetch(`${API_URL}${path}`, {
    ...options,
    headers: {
      ...options.headers,
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
    },
  });
  if (!res.ok) {
    const body = await res.text();
    throw new Error(`${res.status}: ${body}`);
  }
  return res.json();
}

export type Citation = {
  chunk_id: string;
  document_id: string;
  title: string | null;
  content: string;
};

export type ChatResponse = {
  conversation_id: string;
  answer: string;
  citations: Citation[];
};

export async function sendMessage(
  question: string,
  conversationId?: string | null,
): Promise<ChatResponse> {
  return authedFetch("/chat", {
    method: "POST",
    body: JSON.stringify({ question, conversation_id: conversationId }),
  });
}

export type ConversationSummary = {
  id: string;
  title: string | null;
  updated_at: string;
};

export async function listConversations(): Promise<ConversationSummary[]> {
  return authedFetch("/conversations");
}

export type Message = {
  role: "user" | "assistant";
  content: string;
  citations: Citation[] | null;
  created_at: string;
};

export async function getConversationMessages(id: string): Promise<Message[]> {
  return authedFetch(`/conversations/${id}/messages`);
}
export async function login(email: string, password: string) {
  const res = await fetch(`${API_URL}/auth/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email, password }),
  });
  if (!res.ok) throw new Error(await res.text());
  return res.json() as Promise<{ token: string; api_key: string | null }>;
}

export async function signup(email: string, password: string) {
  const res = await fetch(`${API_URL}/auth/signup`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email, password }),
  });
  if (!res.ok) throw new Error(await res.text());
  return res.json() as Promise<{ token: string; api_key: string }>;
}
export async function uploadText(title: string, content: string) {
  return authedFetch("/upload/text", {
    method: "POST",
    body: JSON.stringify({ title, content }),
  });
}

export async function uploadFile(file: File) {
  const token = getToken();
  const formData = new FormData();
  formData.append("file", file);
  const res = await fetch(`${API_URL}/upload/file`, {
    method: "POST",
    headers: { Authorization: `Bearer ${token}` },
    body: formData,
  });
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}
