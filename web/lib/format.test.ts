import assert from "node:assert/strict";
import { test } from "node:test";

import { formatQuantity } from "./format.ts";

test("quantities are grouped without losing a decimal", () => {
  assert.equal(formatQuantity("1234567.123456789012345"), "1,234,567.123456789012345");
  assert.equal(formatQuantity("3596.4"), "3,596.4");
  assert.equal(formatQuantity("0.048"), "0.048");
});
