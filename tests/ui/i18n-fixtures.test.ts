import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import {
  DEFAULT_LOCALE,
  arMessages,
  directionForLocale,
  frMessages,
} from "../../src/i18n/dictionaries.ts";
import { formatDate, formatDzd } from "../../src/i18n/formatters.ts";
import {
  filterProducts,
  invoiceFixtureIsConsistent,
  productFixtures,
} from "../../src/features/ui-gallery/fixtures/index.ts";

const runtimeMessageKeys = [
  "runtime.initializing",
  "runtime.ready",
  "runtime.preview",
  "runtime.errorTitle",
  "runtime.errorGeneric",
  "runtime.retry",
  "runtime.schemaVersion",
  "runtime.migrationCount",
  "runtime.journalMode",
  "runtime.foreignKeys",
  "runtime.foreignKeysEnabled",
  "runtime.retrying",
] as const;

test("Arabic is the default RTL locale", () => {
  assert.equal(DEFAULT_LOCALE, "ar-DZ");
  assert.equal(directionForLocale(DEFAULT_LOCALE), "rtl");
  assert.equal(directionForLocale("fr-DZ"), "ltr");
});

test("Arabic and French dictionaries have identical complete keys", () => {
  assert.deepEqual(Object.keys(frMessages).sort(), Object.keys(arMessages).sort());
  for (const [key, value] of Object.entries(arMessages)) {
    assert.ok(value.trim(), `Arabic translation is empty: ${key}`);
    assert.ok(frMessages[key as keyof typeof frMessages].trim(), `French translation is empty: ${key}`);
  }
});

test("runtime messages exist and are non-empty in Arabic and French", () => {
  for (const key of runtimeMessageKeys) {
    assert.ok(arMessages[key].trim(), `Arabic runtime translation is empty: ${key}`);
    assert.ok(frMessages[key].trim(), `French runtime translation is empty: ${key}`);
  }
});

test("DZD and dates are localized through Intl", () => {
  const arMoney = formatDzd(517650, "ar-DZ");
  const frMoney = formatDzd(517650, "fr-DZ");
  assert.match(arMoney, /5[.\s\u00a0\u202f]?176|٥/);
  assert.match(frMoney, /5[.\s\u00a0\u202f]?176/);
  assert.ok(arMoney.includes("د.ج") || arMoney.includes("DZD"));
  assert.ok(frMoney.includes("DZD") || frMoney.includes("DA"));
  assert.notEqual(formatDate("2026-07-29", "ar-DZ"), "2026-07-29");
  assert.notEqual(formatDate("2026-07-29", "fr-DZ"), "2026-07-29");
});

test("Invoice fixture totals are internally consistent", () => {
  assert.equal(invoiceFixtureIsConsistent(), true);
});

test("Product filter supports localized text, code, and empty results", () => {
  assert.equal(filterProducts(productFixtures, "HUI-001", "all", "ar-DZ").length, 1);
  assert.equal(filterProducts(productFixtures, "huile", "all", "fr-DZ").length, 1);
  assert.equal(filterProducts(productFixtures, "غير موجود", "all", "ar-DZ").length, 0);
});

test("UI foundation contains reduced-motion and avoids prohibited visual patterns", () => {
  const foundationCss = readFileSync(new URL("../../src/styles/ui-foundation.css", import.meta.url), "utf8");
  const runtimeCss = readFileSync(new URL("../../src/features/runtime/runtime-status.css", import.meta.url), "utf8");
  const css = `${foundationCss}\n${runtimeCss}`;
  assert.match(css, /prefers-reduced-motion/);
  assert.doesNotMatch(css, /linear-gradient|radial-gradient|backdrop-filter/i);
  assert.doesNotMatch(css, /border-radius:\s*(?:1[2-9]|[2-9]\d)px/i);
});
