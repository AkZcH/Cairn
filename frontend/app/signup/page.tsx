"use client";

import { useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { signup, setToken } from "@/lib/api";
import { AuthShell } from "@/components/auth/auth-shell";

export default function SignupPage() {
  const router = useRouter();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [apiKey, setApiKey] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    setLoading(true);
    try {
      const res = await signup(email, password);
      setToken(res.token);
      setApiKey(res.api_key);
    } catch {
      setError(
        "Couldn't create an account. That email may already be registered.",
      );
    } finally {
      setLoading(false);
    }
  }

  if (apiKey) {
    return (
      <AuthShell eyebrow="One-time key" title="Save your API key">
        <p className="text-stone text-sm leading-relaxed mb-6">
          This key authenticates{" "}
          <code className="text-moss font-mono text-xs">cairn-ingest</code> on
          your machine. It&apos;s shown once, right now, and can&apos;t be
          retrieved again, only regenerated.
        </p>
        <div className="bg-surface border border-line rounded-lg p-4 mb-4">
          <code className="font-mono text-xs text-moss break-all">
            {apiKey}
          </code>
        </div>
        <button
          onClick={() => {
            navigator.clipboard.writeText(apiKey);
            setCopied(true);
          }}
          className="w-full border border-line rounded-full px-6 py-3 text-sm font-medium hover:border-moss/50 transition-colors focus-ring mb-3"
        >
          {copied ? "Copied" : "Copy key"}
        </button>
        <button
          onClick={() => router.push("/dashboard")}
          className="w-full bg-moss text-bg font-medium px-6 py-3 rounded-full hover:bg-moss-soft transition-colors focus-ring"
        >
          Continue to dashboard
        </button>
      </AuthShell>
    );
  }

  return (
    <AuthShell eyebrow="Get started" title="Create your account">
      <form onSubmit={handleSubmit} className="flex flex-col gap-4">
        <input
          type="email"
          required
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          placeholder="you@example.com"
          className="w-full bg-surface border border-line rounded-lg px-4 py-3 text-sm text-ink placeholder:text-stone/50 focus-ring"
        />
        <input
          type="password"
          required
          minLength={8}
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          placeholder="Password"
          className="w-full bg-surface border border-line rounded-lg px-4 py-3 text-sm text-ink placeholder:text-stone/50 focus-ring"
        />
        {error && <p className="text-sm text-red-400">{error}</p>}
        <button
          type="submit"
          disabled={loading}
          className="w-full bg-moss text-bg font-medium px-6 py-3 rounded-full hover:bg-moss-soft transition-colors focus-ring disabled:opacity-50"
        >
          {loading ? "Creating account..." : "Create account"}
        </button>
      </form>
      <p className="text-stone text-sm text-center mt-6">
        Already have an account?{" "}
        <Link
          href="/login"
          className="text-moss hover:text-moss-soft transition-colors focus-ring"
        >
          Sign in
        </Link>
      </p>
    </AuthShell>
  );
}
