import { PERIOD_MS, type Point } from "./session.ts";

export type Scale = { bottom: number; top: number };

export const FULL_SCALE: Scale = { bottom: 0, top: 110 };

/** The scale starts at the highest floor below the recent levels: fixed steps keep it still. */
const FLOORS = [0, 75, 90, 95, 99, 99.5];
const STEPS = [25, 5, 2, 1, 0.2, 0.1];
/** Smallest distance between two labels, as a fraction of the chart's height. */
const MIN_GAP = 0.08;
/** Ten seconds: a new loan is zoomed in on quickly, and a single tick does not move the scale. */
const RECENT_POINTS = 5;
const HEADROOM = 0.1;

/** Near the limit, a move of 0.1% decides everything: the scale tightens to show it. */
export function scaleFor(points: Point[]): Scale {
  const recent = points.slice(-RECENT_POINTS).map((point) => point.percent);
  const lowest = Math.min(...recent);
  const highest = Math.max(100, ...recent);
  const bottom = FLOORS.findLast((floor) => floor < lowest) ?? 0;
  return { bottom, top: highest + (highest - bottom) * HEADROOM };
}

/** Fraction of the chart's height, from 0 at the bottom to 1 at the top. */
export function heightOf(percent: number, { bottom, top }: Scale): number {
  return Math.min(1, Math.max(0, (percent - bottom) / (top - bottom)));
}

/** Levels to label on the gauge, from the bottom up; any that would overlap are dropped. */
export function ticksFor(scale: Scale): number[] {
  const step = STEPS[FLOORS.indexOf(scale.bottom)];
  const graduations = [];
  for (let level = scale.bottom; level < 100; level = round(level + step)) graduations.push(level);

  // The marks come first: they win over the graduations next to them
  const kept: number[] = [];
  for (const level of [100, 90, 75, ...graduations]) {
    const isClear = kept.every((other) => Math.abs(heightOf(level, scale) - heightOf(other, scale)) >= MIN_GAP);
    if (level >= scale.bottom && isClear) kept.push(level);
  }
  return kept.sort((a, b) => a - b);
}

/**
 * SVG path of the level over time: each level holds until the next observation, and the
 * last one for a polling period, so that a level is visible as soon as it is observed.
 */
export function waterline(points: Point[], scale: Scale, width: number, height: number): string {
  const start = points[0].time;
  const duration = points[points.length - 1].time + PERIOD_MS - start;
  const steps = points.map(({ time, percent }, index) => {
    const x = round(((time - start) / duration) * width);
    const y = round((1 - heightOf(percent, scale)) * height);
    return index === 0 ? `M${x},${y}` : `H${x} V${y}`;
  });
  return `${steps.join(" ")} H${width}`;
}

function round(value: number): number {
  return Math.round(value * 100) / 100;
}
