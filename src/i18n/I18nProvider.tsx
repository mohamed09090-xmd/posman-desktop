import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import {
  DEFAULT_LOCALE,
  dictionaries,
  directionForLocale,
  type Locale,
  type MessageKey,
} from "./dictionaries";
import { formatDate, formatDzd, formatNumber } from "./formatters";

type I18nValue = {
  locale: Locale;
  direction: "rtl" | "ltr";
  setLocale: (locale: Locale) => void;
  t: (key: MessageKey) => string;
  formatMoney: (minorUnits: number) => string;
  formatNumber: (value: number, maximumFractionDigits?: number) => string;
  formatDate: (isoDate: string) => string;
};

const I18nContext = createContext<I18nValue | null>(null);

export function I18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocale] = useState<Locale>(DEFAULT_LOCALE);
  const direction = directionForLocale(locale);

  useEffect(() => {
    document.documentElement.lang = locale;
    document.documentElement.dir = direction;
  }, [direction, locale]);

  const t = useCallback((key: MessageKey) => dictionaries[locale][key], [locale]);
  const formatMoney = useCallback((minorUnits: number) => formatDzd(minorUnits, locale), [locale]);
  const localizedNumber = useCallback(
    (value: number, maximumFractionDigits = 2) =>
      formatNumber(value, locale, maximumFractionDigits),
    [locale],
  );
  const localizedDate = useCallback((isoDate: string) => formatDate(isoDate, locale), [locale]);

  const value = useMemo<I18nValue>(
    () => ({
      locale,
      direction,
      setLocale,
      t,
      formatMoney,
      formatNumber: localizedNumber,
      formatDate: localizedDate,
    }),
    [direction, formatMoney, locale, localizedDate, localizedNumber, t],
  );

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n(): I18nValue {
  const context = useContext(I18nContext);
  if (!context) {
    throw new Error("useI18n must be used inside I18nProvider");
  }
  return context;
}
