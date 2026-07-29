export type ProductStatus = "available" | "low" | "out";

export interface ProductFixture {
  id: string;
  code: string;
  barcode: string;
  nameAr: string;
  nameFr: string;
  familyAr: string;
  familyFr: string;
  salePriceMinor: number;
  taxRateBasisPoints: number;
  onHand: number;
  reserved: number;
  minimum: number;
  warehouseAr: string;
  warehouseFr: string;
  lastMovementAr: string;
  lastMovementFr: string;
  status: ProductStatus;
}

export const productFixtures: readonly ProductFixture[] = [
  {
    id: "prod-olive-oil",
    code: "HUI-001",
    barcode: "6130001000185",
    nameAr: "زيت زيتون بكر 1 لتر",
    nameFr: "Huile d’olive vierge 1 L",
    familyAr: "زيوت",
    familyFr: "Huiles",
    salePriceMinor: 185000,
    taxRateBasisPoints: 900,
    onHand: 36,
    reserved: 8,
    minimum: 12,
    warehouseAr: "المستودع الرئيسي",
    warehouseFr: "Dépôt principal",
    lastMovementAr: "استلام شراء +24 — 28 جويلية 2026",
    lastMovementFr: "Réception achat +24 — 28 juillet 2026",
    status: "available",
  },
  {
    id: "prod-flour",
    code: "ALI-014",
    barcode: "6130001001427",
    nameAr: "فرينة ممتازة 5 كغ",
    nameFr: "Farine supérieure 5 kg",
    familyAr: "مواد غذائية",
    familyFr: "Alimentation",
    salePriceMinor: 62000,
    taxRateBasisPoints: 900,
    onHand: 14,
    reserved: 6,
    minimum: 10,
    warehouseAr: "المستودع الرئيسي",
    warehouseFr: "Dépôt principal",
    lastMovementAr: "حجز طلب -6 — 29 جويلية 2026",
    lastMovementFr: "Réservation commande -6 — 29 juillet 2026",
    status: "low",
  },
  {
    id: "prod-detergent",
    code: "ENT-032",
    barcode: "6130001003247",
    nameAr: "منظف أرضيات 3 لتر",
    nameFr: "Nettoyant sols 3 L",
    familyAr: "تنظيف",
    familyFr: "Entretien",
    salePriceMinor: 98000,
    taxRateBasisPoints: 1900,
    onHand: 0,
    reserved: 0,
    minimum: 8,
    warehouseAr: "المستودع الرئيسي",
    warehouseFr: "Dépôt principal",
    lastMovementAr: "تسليم بيع -4 — 27 جويلية 2026",
    lastMovementFr: "Livraison vente -4 — 27 juillet 2026",
    status: "out",
  },
  {
    id: "prod-notebook",
    code: "BUR-008",
    barcode: "6130001000826",
    nameAr: "دفتر محاسبي 200 صفحة",
    nameFr: "Registre comptable 200 pages",
    familyAr: "مكتبية",
    familyFr: "Papeterie",
    salePriceMinor: 45000,
    taxRateBasisPoints: 1900,
    onHand: 72,
    reserved: 12,
    minimum: 20,
    warehouseAr: "مخزن العرض",
    warehouseFr: "Stock boutique",
    lastMovementAr: "تحويل داخلي +20 — 26 جويلية 2026",
    lastMovementFr: "Transfert interne +20 — 26 juillet 2026",
    status: "available",
  },
  {
    id: "prod-water",
    code: "BOI-021",
    barcode: "6130001002103",
    nameAr: "مياه معدنية 1.5 لتر × 6",
    nameFr: "Eau minérale 1,5 L × 6",
    familyAr: "مشروبات",
    familyFr: "Boissons",
    salePriceMinor: 30000,
    taxRateBasisPoints: 900,
    onHand: 22,
    reserved: 4,
    minimum: 18,
    warehouseAr: "المستودع الرئيسي",
    warehouseFr: "Dépôt principal",
    lastMovementAr: "بيع مباشر -3 — 29 جويلية 2026",
    lastMovementFr: "Vente directe -3 — 29 juillet 2026",
    status: "low",
  },
  {
    id: "prod-battery",
    code: "ELE-007",
    barcode: "6130001000710",
    nameAr: "بطاريات قلم AA × 4",
    nameFr: "Piles AA × 4",
    familyAr: "كهرباء",
    familyFr: "Électricité",
    salePriceMinor: 75000,
    taxRateBasisPoints: 1900,
    onHand: 48,
    reserved: 2,
    minimum: 10,
    warehouseAr: "مخزن العرض",
    warehouseFr: "Stock boutique",
    lastMovementAr: "استلام شراء +36 — 25 جويلية 2026",
    lastMovementFr: "Réception achat +36 — 25 juillet 2026",
    status: "available",
  },
] as const;

export interface TodayOperationFixture {
  id: string;
  group: "intervention" | "delivery" | "invoice" | "unposted" | "stock" | "recent";
  taskAr: string;
  taskFr: string;
  reference: string;
  ownerAr: string;
  ownerFr: string;
  time: string;
  status: "confirmed" | "pending" | "shortage" | "draft" | "posted";
}

export const todayOperationFixtures: readonly TodayOperationFixture[] = [
  { id: "today-1", group: "intervention", taskAr: "تأكيد فرق كمية في التسليم", taskFr: "Confirmer l’écart de quantité à livrer", reference: "BL-2026-0048", ownerAr: "سميرة", ownerFr: "Samira", time: "09:20", status: "pending" },
  { id: "today-2", group: "delivery", taskAr: "طلب عميل مكتمل التحضير", taskFr: "Commande client entièrement préparée", reference: "CMD-2026-0114", ownerAr: "مراد", ownerFr: "Mourad", time: "10:05", status: "confirmed" },
  { id: "today-3", group: "invoice", taskAr: "تحويل سند تسليم إلى فاتورة", taskFr: "Transformer le bon de livraison en facture", reference: "BL-2026-0046", ownerAr: "سميرة", ownerFr: "Samira", time: "10:32", status: "pending" },
  { id: "today-4", group: "unposted", taskAr: "مراجعة فاتورة قبل الترحيل", taskFr: "Contrôler la facture avant comptabilisation", reference: "FAC-2026-0072", ownerAr: "نادية", ownerFr: "Nadia", time: "11:10", status: "draft" },
  { id: "today-5", group: "stock", taskAr: "منظف أرضيات وصل إلى الصفر", taskFr: "Le nettoyant sols est épuisé", reference: "ENT-032", ownerAr: "المخزن", ownerFr: "Stock", time: "11:35", status: "shortage" },
  { id: "today-6", group: "recent", taskAr: "فاتورة بيع رُحّلت", taskFr: "Facture de vente comptabilisée", reference: "FAC-2026-0071", ownerAr: "نادية", ownerFr: "Nadia", time: "12:02", status: "posted" },
] as const;

export interface OpeningStockLineFixture {
  id: string;
  productAr: string;
  productFr: string;
  quantity: number;
  unitCostMinor: number;
  totalMinor: number;
}

export const openingStockFixtures: readonly OpeningStockLineFixture[] = [
  { id: "opening-1", productAr: "زيت زيتون بكر 1 لتر", productFr: "Huile d’olive vierge 1 L", quantity: 24, unitCostMinor: 146000, totalMinor: 3504000 },
  { id: "opening-2", productAr: "فرينة ممتازة 5 كغ", productFr: "Farine supérieure 5 kg", quantity: 40, unitCostMinor: 49000, totalMinor: 1960000 },
  { id: "opening-3", productAr: "دفتر محاسبي 200 صفحة", productFr: "Registre comptable 200 pages", quantity: 60, unitCostMinor: 32000, totalMinor: 1920000 },
] as const;

export const openingStockTotals = {
  quantity: openingStockFixtures.reduce((sum, line) => sum + line.quantity, 0),
  costMinor: openingStockFixtures.reduce((sum, line) => sum + line.totalMinor, 0),
} as const;

export interface InvoiceLineFixture {
  id: string;
  productAr: string;
  productFr: string;
  quantity: number;
  unitPriceMinor: number;
  grossMinor: number;
  lineDiscountMinor: number;
  generalDiscountAllocationMinor: number;
  netHtMinor: number;
  taxMinor: number;
  totalTtcMinor: number;
}

export const invoiceLineFixtures: readonly InvoiceLineFixture[] = [
  {
    id: "invoice-1",
    productAr: "زيت زيتون بكر 1 لتر",
    productFr: "Huile d’olive vierge 1 L",
    quantity: 2,
    unitPriceMinor: 125000,
    grossMinor: 250000,
    lineDiscountMinor: 10000,
    generalDiscountAllocationMinor: 8000,
    netHtMinor: 232000,
    taxMinor: 44080,
    totalTtcMinor: 276080,
  },
  {
    id: "invoice-2",
    productAr: "دفتر محاسبي 200 صفحة",
    productFr: "Registre comptable 200 pages",
    quantity: 6,
    unitPriceMinor: 35000,
    grossMinor: 210000,
    lineDiscountMinor: 0,
    generalDiscountAllocationMinor: 7000,
    netHtMinor: 203000,
    taxMinor: 38570,
    totalTtcMinor: 241570,
  },
] as const;

export const invoiceTotals = {
  subtotalMinor: 460000,
  lineDiscountMinor: 10000,
  generalDiscountMinor: 15000,
  netHtMinor: 435000,
  taxMinor: 82650,
  totalTtcMinor: 517650,
} as const;

export function filterProducts(
  products: readonly ProductFixture[],
  query: string,
  family: string,
  locale: "ar-DZ" | "fr-DZ",
): ProductFixture[] {
  const normalizedQuery = query.trim().toLocaleLowerCase(locale);
  return products.filter((product) => {
    const localizedName = locale === "ar-DZ" ? product.nameAr : product.nameFr;
    const localizedFamily = locale === "ar-DZ" ? product.familyAr : product.familyFr;
    const matchesFamily = family === "all" || localizedFamily === family;
    const searchable = [product.code, product.barcode, localizedName, localizedFamily]
      .join(" ")
      .toLocaleLowerCase(locale);
    return matchesFamily && (!normalizedQuery || searchable.includes(normalizedQuery));
  });
}

export function invoiceFixtureIsConsistent(): boolean {
  const subtotal = invoiceLineFixtures.reduce((sum, line) => sum + line.grossMinor, 0);
  const lineDiscount = invoiceLineFixtures.reduce((sum, line) => sum + line.lineDiscountMinor, 0);
  const generalDiscount = invoiceLineFixtures.reduce(
    (sum, line) => sum + line.generalDiscountAllocationMinor,
    0,
  );
  const netHt = invoiceLineFixtures.reduce((sum, line) => sum + line.netHtMinor, 0);
  const tax = invoiceLineFixtures.reduce((sum, line) => sum + line.taxMinor, 0);
  const totalTtc = invoiceLineFixtures.reduce((sum, line) => sum + line.totalTtcMinor, 0);

  return (
    subtotal === invoiceTotals.subtotalMinor &&
    lineDiscount === invoiceTotals.lineDiscountMinor &&
    generalDiscount === invoiceTotals.generalDiscountMinor &&
    netHt === invoiceTotals.netHtMinor &&
    tax === invoiceTotals.taxMinor &&
    totalTtc === invoiceTotals.totalTtcMinor &&
    subtotal - lineDiscount - generalDiscount === netHt &&
    netHt + tax === totalTtc
  );
}
