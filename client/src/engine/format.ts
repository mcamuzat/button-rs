/** Engine scores are in half-points: 7 → "3,5". */
export function formatPoints(half: number): string {
  const sign = half < 0 ? "−" : "";
  const abs = Math.abs(half);
  const whole = Math.floor(abs / 2);
  return abs % 2 === 0 ? `${sign}${whole}` : `${sign}${whole},5`;
}
