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

/** Pinback badge: ink rim, inner ring, hard drop shadow (theme « Badge »). */
const STATE: Record<ChipState, string> = {
  idle: "bg-card text-ink",
  selectable: "bg-card text-ink outline-3 outline-offset-3 outline-dashed outline-accent hover:-translate-y-0.5",
  selected: "bg-accent text-white -translate-y-1 -rotate-6",
};

export function DieChip({ die, state, onClick, small = false }: Props) {
  const skills = useContext(SkillsContext);
  const letters = die.skills.map((s) => skills.get(s)?.letter ?? "").join("");
  const size = small ? "h-11 w-11 text-sm" : "h-16 w-16 text-xl";
  return (
    <motion.button
      layout
      type="button"
      onClick={onClick}
      disabled={!onClick}
      title={skillTitle(die, skills)}
      className={`relative flex ${size} flex-col items-center justify-center rounded-full border-[3px] border-ink font-display shadow-[inset_0_0_0_3px_var(--color-card),inset_0_0_0_5px_var(--color-ink),0_3px_0_var(--color-ink)] transition ${STATE[state]} ${die.dizzy ? "opacity-50" : ""}`}
    >
      <motion.span
        key={`${die.id}-${die.value}`}
        initial={{ rotate: -120, scale: 0.6, opacity: 0 }}
        animate={{ rotate: 0, scale: 1, opacity: 1 }}
        transition={{ type: "spring", stiffness: 300, damping: 18 }}
      >
        {die.value}
      </motion.span>
      <span className={`mt-0.5 font-sans text-[0.6rem] leading-none font-extrabold ${state === "selected" ? "text-white/85" : "text-muted"}`}>
        {letters && <span className={`mr-0.5 ${state === "selected" ? "text-white" : "text-accent"}`}>{letters}</span>}
        {die.max}
      </span>
    </motion.button>
  );
}
