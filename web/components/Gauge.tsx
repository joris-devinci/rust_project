import type { ReactNode } from "react";

import { heightOf, type Scale, ticksFor } from "@/lib/chart";

const MARKS: Record<number, { name: string; line: string }> = {
  75: { name: "unsafe", line: "border-mist border-t" },
  90: { name: "warning", line: "border-amber border-t-2" },
  100: { name: "limit", line: "border-limit-light border-t-4" },
};

/** A graduated staff on the left, and the marks across the water, edged in dark to stay readable over it. */
export function Gauge({ scale, children }: { scale: Scale; children?: ReactNode }) {
  return (
    <div className="flex pb-4 pl-5 sm:pl-7">
      <div aria-hidden className="relative h-60 w-24 shrink-0 border-r-2 border-mist text-sm sm:h-72 sm:w-28">
        {ticksFor(scale).map((level) => (
          <div
            key={level}
            className="absolute inset-x-0 flex translate-y-1/2 items-center justify-between leading-none"
            style={{ bottom: `${heightOf(level, scale) * 100}%` }}
          >
            <span className={MARKS[level] ? "font-semibold" : "text-mist"}>
              {level}% {MARKS[level]?.name}
            </span>
            <span className="w-2.5 border-t-2 border-mist" />
          </div>
        ))}
      </div>
      <div className="relative h-60 min-w-0 flex-1 sm:h-72">
        {children}
        {Object.entries(MARKS).map(
          ([level, { line }]) =>
            Number(level) >= scale.bottom && (
              <div
                key={level}
                className={`absolute inset-x-0 translate-y-1/2 outline-2 outline-river-deep ${line}`}
                style={{ bottom: `${heightOf(Number(level), scale) * 100}%` }}
              />
            ),
        )}
      </div>
    </div>
  );
}
