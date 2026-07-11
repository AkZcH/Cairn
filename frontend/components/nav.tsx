export function Nav() {
  return (
    <nav className="sticky top-0 z-50 border-b border-line/60 bg-bg/85 backdrop-blur-md">
      <div className="max-w-7xl mx-auto px-6 h-16 flex items-center justify-between">
        <div className="flex items-center gap-5">
          <div className="flex items-center gap-3.5">
            <div className="w-3.5 h-3.5 rotate-45 bg-moss" />
            <span className="h-6 w-px bg-line -skew-x-[24deg]" />
            <span className="font-semibold text-lg tracking-tight">Cairn</span>
          </div>
        </div>

        <div className="hidden md:flex items-center gap-7 text-sm text-stone">
          <a
            href="#how"
            className="hover:text-ink transition-colors focus-ring"
          >
            How it works
          </a>
          <a
            href="#features"
            className="hover:text-ink transition-colors focus-ring"
          >
            Features
          </a>
          <a
            href="https://github.com/AkZcH/Cairn"
            className="hover:text-ink transition-colors focus-ring"
          >
            GitHub
          </a>
        </div>

        <a
          href="/login"
          className="hidden md:block font-mono text-xs uppercase tracking-widest text-stone hover:text-ink transition-colors focus-ring"
        >
          Sign in
        </a>

        <a
          href="/signup"
          className="font-medium text-sm bg-moss text-bg px-4 py-2 rounded-md hover:bg-moss-soft transition-colors focus-ring"
        >
          Get started
        </a>
      </div>
    </nav>
  );
}
