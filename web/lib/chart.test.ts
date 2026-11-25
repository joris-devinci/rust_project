import assert from "node:assert/strict";
import { test } from "node:test";

import { heightOf, scaleFor, ticksFor, waterline } from "./chart.ts";

function points(...percents: number[]) {
  return percents.map((percent, index) => ({ time: index * 1000, percent }));
}

test("a low level is drawn on the full scale", () => {
  assert.deepEqual(scaleFor(points(40)), { bottom: 0, top: 110 });
});

test("the scale tightens near the limit", () => {
  assert.deepEqual(scaleFor(points(92)), { bottom: 90, top: 101 });
  assert.equal(scaleFor(points(99.9)).bottom, 99.5);
});

test("the scale makes room above the limit", () => {
  assert.ok(scaleFor(points(99.9, 104)).top > 104);
});

test("old points do not widen the scale", () => {
  const recent = Array.from({ length: 5 }, () => 99.9);
  assert.equal(scaleFor(points(0, ...recent)).bottom, 99.5);
});

test("heights are fractions of the scale, kept inside it", () => {
  const scale = { bottom: 90, top: 110 };
  assert.equal(heightOf(100, scale), 0.5);
  assert.equal(heightOf(0, scale), 0);
  assert.equal(heightOf(200, scale), 1);
});

test("a level holds until the next observation, the last one for a polling period", () => {
  const line = waterline(points(0, 50, 100), { bottom: 0, top: 100 }, 200, 100);
  assert.equal(line, "M0,100 H50 V50 H100 V0 H200");
});

test("a single observation is a flat level", () => {
  assert.equal(waterline(points(50), { bottom: 0, top: 100 }, 200, 100), "M0,50 H200");
});

test("the full scale is graduated, with the warning mark", () => {
  assert.deepEqual(ticksFor({ bottom: 0, top: 110 }), [0, 25, 50, 75, 90, 100]);
});

test("a tight scale is graduated from its floor to the limit", () => {
  assert.deepEqual(ticksFor({ bottom: 99.5, top: 100.05 }), [99.5, 99.6, 99.7, 99.8, 99.9, 100]);
});

test("graduations that would overlap are dropped, never the limit", () => {
  const ticks = ticksFor({ bottom: 99.5, top: 110 });
  assert.deepEqual(ticks, [100]);
});
