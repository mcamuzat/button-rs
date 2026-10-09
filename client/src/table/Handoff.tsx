/** Hotseat: hides the screen between two secret choices. */
export function Handoff({ name, onReady }: { name: string; onReady: () => void }) {
  return (
    <div className="fixed inset-0 z-10 flex flex-col items-center justify-center gap-4 bg-felt-dark">
      <p className="font-display text-3xl">Passez l'écran à {name}</p>
      <button type="button" onClick={onReady} className="rounded-lg bg-gold px-6 py-3 font-semibold text-ink">
        C'est moi
      </button>
    </div>
  );
}
