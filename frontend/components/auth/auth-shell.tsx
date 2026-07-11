export function AuthShell({
  eyebrow,
  title,
  children,
}: {
  eyebrow: string;
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div className="min-h-screen bg-bg text-ink flex items-center justify-center px-6 relative overflow-hidden">
      <div className="absolute top-1/3 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[500px] h-[300px] bg-moss/10 rounded-full blur-[120px] pointer-events-none" />

      <div className="relative w-full max-w-sm">
        <div className="flex items-center gap-2 justify-center mb-10">
          <div className="w-5 h-5 rounded-sm bg-moss" />
          <span className="font-semibold tracking-tight">Cairn</span>
        </div>

        <p className="font-mono text-xs uppercase tracking-widest text-moss text-center mb-3">
          {eyebrow}
        </p>
        <h1 className="text-2xl font-semibold tracking-tight text-center mb-8">
          {title}
        </h1>

        {children}
      </div>
    </div>
  );
}
