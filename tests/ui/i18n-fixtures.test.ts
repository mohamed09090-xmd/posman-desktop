import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { DEFAULT_LOCALE, directionForLocale } from "../../src/i18n/dictionaries.ts";
import { formatDate, formatDzd } from "../../src/i18n/formatters.ts";

test("Arabic remains the default RTL locale and French remains LTR", () => {
  assert.equal(DEFAULT_LOCALE, "ar-DZ");
  assert.equal(directionForLocale("ar-DZ"), "rtl");
  assert.equal(directionForLocale("fr-DZ"), "ltr");
});

test("DZD and dates remain localized through Intl", () => {
  assert.notEqual(formatDate("2026-08-01", "ar-DZ"), "2026-08-01");
  assert.notEqual(formatDate("2026-08-01", "fr-DZ"), "2026-08-01");
  assert.match(formatDzd(123450, "ar-DZ"), /د.ج|DZD/);
  assert.match(formatDzd(123450, "fr-DZ"), /DZD|DA/);
});

test("PHASE 05 interface is offline, reduced-motion aware, and avoids prohibited patterns", () => {
  const css = readFileSync(new URL("../../src/features/phase05/phase05.css", import.meta.url), "utf8");
  const app = readFileSync(new URL("../../src/features/phase05/Phase05App.tsx", import.meta.url), "utf8");
  assert.match(css, /prefers-reduced-motion/);
  assert.doesNotMatch(css, /linear-gradient|radial-gradient|backdrop-filter/i);
  assert.doesNotMatch(app, /fetch\(|XMLHttpRequest|WebSocket\(|https?:\/\//i);
  assert.match(app, /ADMIN_OVERRIDE/);
  assert.match(app, /BELOW_COST|belowCost/);
  assert.match(app, /ar-DZ/);
  assert.match(app, /fr-DZ/);
});
