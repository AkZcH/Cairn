const COLUMNS = [
  {
    title: "Product",
    links: [
      { label: "How it works", href: "#how" },
      { label: "Features", href: "#features" },
    ],
  },
  {
    title: "Resources",
    links: [
      { label: "Repository", href: "https://github.com/AkZcH/Cairn" },
      {
        label: "Changelog",
        href: "https://github.com/AkZcH/Cairn/commits/main",
      },
      { label: "Issues", href: "https://github.com/AkZcH/Cairn/issues" },
    ],
  },
];

export function Footer() {
  return (
    <footer className="border-t border-line/60 bg-black/40">
      <div className="max-w-6xl mx-auto px-6 py-16">
        <div className="grid grid-cols-2 md:grid-cols-4 gap-10 mb-16">
          <div className="col-span-2 md:col-span-2">
            <div className="flex items-center gap-2 mb-4">
              <div className="w-5 h-5 rounded-sm bg-moss" />
              <span className="font-semibold tracking-tight">Cairn</span>
            </div>
            <p className="text-stone text-sm leading-relaxed max-w-xs">
              A knowledge operating system. Local-first, cited, and built in the
              open.
            </p>
          </div>

          {COLUMNS.map((col) => (
            <div key={col.title}>
              <p className="font-mono text-xs uppercase tracking-widest text-stone mb-4">
                {col.title}
              </p>
              <ul className="flex flex-col gap-3">
                {col.links.map((link) => (
                  <li key={link.label}>
                    <a
                      href={link.href}
                      className="text-sm text-ink/80 hover:text-moss transition-colors focus-ring"
                    >
                      {link.label}
                    </a>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>

        <div className="pt-8 border-t border-line/60 flex flex-col md:flex-row items-center justify-between gap-4">
          <span className="font-mono text-xs text-stone">
            Built by Akshat, in the open.
          </span>
          <a
            href="https://github.com/AkZcH/Cairn"
            className="font-mono text-xs text-stone hover:text-ink transition-colors focus-ring"
          >
            github.com/AkZcH/Cairn
          </a>
        </div>
      </div>
    </footer>
  );
}
