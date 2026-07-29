import type { Locale } from "./dictionaries";

export function formatDzd(minorUnits: number, locale: Locale): string {
  return new Intl.NumberFormat(locale, {
    style: "currency",
    currency: "DZD",
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(minorUnits / 100);
}

export function formatNumber(value: number, locale: Locale, maximumFractionDigits = 2): string {
  return new Intl.NumberFormat(locale, {
    maximumFractionDigits,
  }).format(value);
}

export function formatDate(isoDate: string, locale: Locale): string {
  const [year, month, day] = isoDate.split("-").map(Number);
  if (!year || !month || !day) {
    return isoDate;
  }

  return new Intl.DateTimeFormat(locale, {
    year: "numeric",
    month: "short",
    day: "numeric",
    timeZone: "Africa/Algiers",
  }).format(new Date(Date.UTC(year, month - 1, day, 12)));
}
