import type { Account, Health } from "@/lib/api";
import { scaleFor, waterline } from "@/lib/chart";
import { formatTime, formatUsd, HEALTH_LABELS } from "@/lib/format";
import { marginUsd, utilization } from "@/lib/position";
import type { Point, Session } from "@/lib/session";

import { Gauge } from "./Gauge";

const WIDTH = 900;
const HEIGHT = 300;

const HEALTH_STYLES: Record<Health, string> = {
  NoDebt: "border-mist/60",
  Safe: "border-mist",
  Unsafe: "border-amber text-amber",
  Warning: "border-amber bg-amber text-ink",
  Liquidatable: "border-white bg-limit",
};

type Props = {
  account: Account;
  points: Point[];
  liquidation: Session["liquidation"];
  /** The last refresh failed: these are the last known values. */
  stale: boolean;
};

export function UtilizationPanel({ account, points, liquidation, stale }: Props) {
  // While a liquidation is on screen, so are the levels that led to it
  const frozen = liquidation?.before.id === account.id ? liquidation : null;
  return (
    <section aria-labelledby="utilization" className="on-river overflow-hidden rounded-lg bg-river text-white">
      <div className="flex flex-wrap items-end gap-x-8 gap-y-4 p-5 sm:p-7">
        <div>
          <p className="text-6xl leading-none font-bold tracking-tight sm:text-8xl">{utilization(account)}</p>
          <h2 id="utilization" className="mt-2">
            of borrowing power used{stale && ", last known value"}
          </h2>
        </div>
        <div>
          <p className="text-2xl font-semibold text-balance sm:text-3xl">{describeMargin(account)}</p>
          <p
            role="status"
            className={`mt-2 inline-block rounded border-2 px-2.5 py-0.5 text-lg font-semibold ${HEALTH_STYLES[account.health]}`}
          >
            {HEALTH_LABELS[account.health]}
          </p>
        </div>
      </div>
      <WaterChart points={frozen ? frozen.points : points} frozenAt={frozen?.time} stale={stale} />
    </section>
  );
}

function describeMargin(account: Account): string {
  const margin = marginUsd(account);
  if (account.health === "Liquidatable") return `${formatUsd(margin.abs())} past the limit`;
  if (account.health === "NoDebt") return `${formatUsd(margin)} available to borrow`;
  return `${formatUsd(margin)} before liquidation`;
}

function WaterChart({ points, frozenAt, stale }: { points: Point[]; frozenAt?: number; stale: boolean }) {
  const scale = scaleFor(points);
  const line = waterline(points, scale, WIDTH, HEIGHT);
  const latest = points[points.length - 1];
  const end = frozenAt
    ? `Frozen before the liquidation, detected ${formatTime(frozenAt)}`
    : `${stale ? "Last update" : "Now"} ${formatTime(latest.time)}`;
  return (
    <figure>
      <Gauge scale={scale}>
        <svg
          role="img"
          aria-label={`Borrowing power used over time, last at ${latest.percent}%. Liquidation above 100%.`}
          viewBox={`0 0 ${WIDTH} ${HEIGHT}`}
          preserveAspectRatio="none"
          className="absolute inset-0 size-full"
        >
          <path d={`${line} V${HEIGHT} H0 Z`} className="fill-water" />
          <path d={line} className="fill-none stroke-white stroke-2" vectorEffect="non-scaling-stroke" />
        </svg>
      </Gauge>
      <figcaption className="flex flex-wrap justify-between gap-x-6 bg-river-deep px-5 py-2.5 text-sm text-mist sm:px-7">
        <span>Observed since {formatTime(points[0].time)}</span>
        {scale.bottom > 0 && <span className="font-semibold text-white">Zoomed in from {scale.bottom}%</span>}
        <span className={frozenAt ? "font-semibold text-white" : ""}>{end}</span>
      </figcaption>
    </figure>
  );
}
