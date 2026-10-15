import { useContext, type ReactNode } from "react";
import type { SwingRange } from "../engine/generated/SwingRange";
import { SkillsContext } from "../table/SkillsContext";
import { skillsByName } from "./rules";

interface Props {
  swingRanges: SwingRange[];
  onBack: () => void;
}

/** A static badge illustrating a die notation (not a game die). */
function Badge({ top, bottom, accent = false }: { top: string; bottom?: string; accent?: boolean }) {
  return (
    <span
      className={`inline-flex h-14 w-14 shrink-0 flex-col items-center justify-center rounded-full border-[3px] border-ink font-display shadow-[inset_0_0_0_3px_var(--color-card),inset_0_0_0_5px_var(--color-ink),0_3px_0_var(--color-ink)] ${accent ? "bg-accent text-white" : "bg-card text-ink"}`}
    >
      <span className={`leading-none ${top.length > 4 ? "text-[0.7rem]" : "text-base"}`}>{top}</span>
      {bottom && <span className="mt-0.5 font-sans text-[0.6rem] font-extrabold leading-none">{bottom}</span>}
    </span>
  );
}

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="print-card flex flex-col gap-3 p-5">
      <h2 className="font-display text-xl">{title}</h2>
      {children}
    </section>
  );
}

/** Summary of the rules (docs/RULES.md R 1–R 5) and every skill's engine-provided help. */
export function Rules({ swingRanges, onBack }: Props) {
  const skills = skillsByName(useContext(SkillsContext));
  return (
    <main className="mx-auto flex max-w-3xl flex-col gap-5 p-6">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <h1 className="font-display text-3xl">Règles du jeu</h1>
        <button type="button" onClick={onBack} className="font-extrabold text-muted hover:text-ink">
          ← Accueil
        </button>
      </div>

      <Section title="Le but">
        <p>
          Chaque joueur choisit un bouton : un personnage et ses dés. Un match se joue en manches ; le premier à gagner
          3 manches gagne le match.
        </p>
      </Section>

      <Section title="Une manche">
        <ol className="flex list-decimal flex-col gap-1 pl-5">
          <li>Chacun choisit en secret la taille de ses dés Swing et Option.</li>
          <li>Tous les dés sont lancés.</li>
          <li>Le joueur qui a le plus petit dé commence (en cas d'égalité, on compare le suivant).</li>
          <li>
            À son tour, on <strong>doit attaquer</strong> si on le peut ; on ne passe que si aucune attaque n'est
            possible.
          </li>
          <li>La manche s'arrête quand un joueur n'a plus de dés, ou quand les deux passent l'un après l'autre.</li>
        </ol>
      </Section>

      <Section title="Les attaques">
        <div className="flex items-center gap-4">
          <Badge top="9" bottom="12" accent />
          <p>
            <strong>Power</strong> : un de vos dés capture un dé adverse de valeur <strong>inférieure ou égale</strong>{" "}
            à la sienne.
          </p>
        </div>
        <div className="flex items-center gap-4">
          <span className="flex -space-x-3">
            <Badge top="3" bottom="6" accent />
            <Badge top="4" bottom="8" accent />
          </span>
          <p>
            <strong>Skill</strong> : un ou plusieurs de vos dés capturent un dé adverse dont la valeur est{" "}
            <strong>exactement la somme</strong> des leurs (3 + 4 capturent un 7).
          </p>
        </div>
        <p className="text-muted">
          Les dés capturés vont dans votre pile ; vos dés attaquants sont relancés. Les compétences ajoutent d'autres
          attaques (Shadow, Speed, Trip, Berserk, Rush, Boom).
        </p>
      </Section>

      <Section title="Les dés">
        <ul className="flex flex-col gap-3">
          <li className="flex items-center gap-4">
            <Badge top="(8)" />
            <span>
              <strong>Normal</strong> : un dé à 8 faces.
            </span>
          </li>
          <li className="flex items-center gap-4">
            <Badge top="(X)" />
            <span>
              <strong>Swing</strong> : vous choisissez sa taille dans la plage de sa lettre. Tous vos dés de même
              lettre ont la même taille.
            </span>
          </li>
          <li className="flex items-center gap-4">
            <Badge top="(4/12)" />
            <span>
              <strong>Option</strong> : vous choisissez l'une des deux tailles, 4 ou 12.
            </span>
          </li>
          <li className="flex items-center gap-4">
            <Badge top="(4,6)" />
            <span>
              <strong>Twin</strong> : deux dés lancés ensemble qui comptent comme un seul (taille 10, valeur de 2 à 10).
            </span>
          </li>
        </ul>
        <div className="overflow-x-auto">
          <table className="border-collapse font-mono text-sm">
            <caption className="mb-1 text-left font-sans text-sm text-muted">Plages des lettres Swing</caption>
            <tbody>
              <tr>
                {swingRanges.map((r) => (
                  <th key={r.letter} className="border-2 border-ink bg-ink px-2 py-1 text-paper">
                    {r.letter}
                  </th>
                ))}
              </tr>
              <tr>
                {swingRanges.map((r) => (
                  <td key={r.letter} className="border-2 border-ink px-2 py-1 whitespace-nowrap">
                    {r.min}–{r.max}
                  </td>
                ))}
              </tr>
            </tbody>
          </table>
        </div>
        <p className="text-muted">
          Après une manche, seul le perdant peut changer ses tailles Swing et Option ; après une égalité, personne.
        </p>
      </Section>

      <Section title="Le calcul du score">
        <p>
          À la fin de la manche : chaque dé que vous avez <strong>gardé</strong> vaut <strong>la moitié</strong> de sa
          taille, chaque dé que vous avez <strong>capturé</strong> vaut <strong>sa taille entière</strong>.
        </p>
        <div className="border-2 border-dashed border-ink bg-paper p-3">
          <p className="font-extrabold">Exemple</p>
          <p>Vous gardez un d12 et un d6 : 6 + 3 = 9 points.</p>
          <p>Vous avez capturé un d10 et un d20 : 10 + 20 = 30 points.</p>
          <p className="mt-1 font-display text-lg">Total : 39 points</p>
        </div>
        <p>Le score le plus haut gagne la manche ; une égalité donne une manche nulle.</p>
        <p className="text-muted">
          Certaines compétences changent ce calcul : Poison (points négatifs), Null (zéro point), Value (la valeur
          affichée remplace la taille).
        </p>
      </Section>

      <Section title="Les compétences">
        <p className="text-muted">
          La lettre d'une compétence s'écrit devant le dé dans la recette d'un bouton, par exemple <code>p(4)</code>{" "}
          pour un d4 Poison.
        </p>
        <ul className="grid gap-3 sm:grid-cols-2">
          {skills.map((s) => (
            <li key={s.skill} className="flex gap-3">
              <Badge top={s.letter} accent />
              <span className="min-w-0">
                <span className="font-display">{s.name}</span>{" "}
                <span className="font-mono text-xs text-muted">{s.rule}</span>
                <span className="block text-sm">{s.description}</span>
              </span>
            </li>
          ))}
        </ul>
      </Section>
    </main>
  );
}
