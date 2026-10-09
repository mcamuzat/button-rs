import { motion } from "motion/react";
import { useContext } from "react";
import type { DieView } from "../engine/generated/DieView";
import { skillTitle } from "./labels";
import { SkillsContext } from "./SkillsContext";

export type ChipState = "idle" | "selectable" | "selected";

interface Props {
  die: DieView;
  state: ChipState;
  onClick?: () => void;
  small?: boolean;
}

const RING: Record<ChipState, string> = {
  idle: "ring-1 ring-ink/20",
  selectable: "ring-2 ring-gold/70 hover:ring-gold",
  selected: "ring-4 ring-gold -translate-y-1",
};

export function DieChip({ die, state, onClick, small = false }: Props) {
  const skills = useContext(SkillsContext);
  const letters = die.skills.map((s) => skills.get(s)?.letter ?? "").join("");
  const size = small ? "h-12 w-12 text-lg" : "h-20 w-20 text-3xl";
  return (
    <motion.button
      layout
      type="button"
      onClick={onClick}
      disabled={!onClick}
      title={skillTitle(die, skills)}
      className={`relative flex ${size} flex-col items-center justify-center rounded-xl bg-ivory font-display text-ink shadow-md transition ${RING[state]} ${die.dizzy ? "opacity-50" : ""}`}
    >
      <motion.span
        key={`${die.id}-${die.value}`}
        initial={{ rotate: -120, scale: 0.6, opacity: 0 }}
        animate={{ rotate: 0, scale: 1, opacity: 1 }}
        transition={{ type: "spring", stiffness: 300, damping: 18 }}
      >
        {die.value}
      </motion.span>
      <span className="text-[0.65rem] leading-none text-ink/70">
        {letters && <span className="mr-0.5 font-bold text-danger">{letters}</span>}
        {die.max}
      </span>
    </motion.button>
  );
}
