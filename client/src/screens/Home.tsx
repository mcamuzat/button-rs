interface Props {
  onHotseat: () => void;
  onAi: () => void;
}

export function Home({ onHotseat, onAi }: Props) {
  const modes = [
    { title: "Deux joueurs, même écran", text: "Passez-vous l'appareil à chaque tour.", onClick: onHotseat },
    { title: "Contre l'IA", text: "Facile, Normal ou Difficile.", onClick: onAi },
    { title: "En ligne", text: "Bientôt.", onClick: undefined },
  ];
  return (
    <main className="mx-auto flex max-w-3xl flex-col gap-8 p-8">
      <header>
        <h1 className="font-display text-5xl">Button Men</h1>
        <p className="mt-2 text-muted">Le jeu de dés de James Ernest, dans le navigateur.</p>
      </header>
      <div className="grid gap-4 sm:grid-cols-3">
        {modes.map((m) => (
          <button
            key={m.title}
            type="button"
            disabled={!m.onClick}
            onClick={m.onClick}
            className="print-card p-5 text-left transition enabled:hover:-translate-y-0.5 enabled:hover:shadow-[6px_6px_0_var(--color-accent)] disabled:opacity-50"
          >
            <span className="block font-display text-xl">{m.title}</span>
            <span className="mt-1 block text-sm text-muted">{m.text}</span>
          </button>
        ))}
      </div>
    </main>
  );
}
