export type Phase06Locale = "ar-DZ" | "fr-DZ";
export type Phase06Screen =
  | "overview" | "ledger" | "opening" | "transfer" | "adjustment" | "count"
  | "reservations" | "orders" | "orderEditor" | "receipt" | "invoice" | "direct"
  | "return" | "reconciliation";

export type Phase06Copy = {
  title: string; inventory: string; purchasing: string; overview: string; ledger: string;
  opening: string; transfer: string; adjustment: string; count: string; reservations: string;
  orders: string; orderEditor: string; receipt: string; invoice: string; direct: string;
  return: string; reconciliation: string; refresh: string; loading: string; empty: string;
  retry: string; save: string; review: string; post: string; confirm: string; cancel: string;
  release: string; consume: string; rebuild: string; product: string; warehouse: string;
  location: string; onHand: string; reserved: string; available: string; cump: string;
  value: string; quantity: string; cost: string; price: string; date: string; reason: string;
  source: string; destination: string; supplier: string; status: string; document: string;
  movement: string; variance: string; systemQuantity: string; countedQuantity: string;
  mismatch: string; projection: string; rebuilt: string; success: string; error: string;
  postedLocked: string; negativeWarning: string; overrideConfirm: string; idempotency: string;
  documentId: string; sourceLineId: string; rowVersion: string; countNumber: string;
  notes: string; sourceMovement: string; directNote: string; preview: string; confirmPosting: string;
};

export const PHASE06_COPY: Record<Phase06Locale, Phase06Copy> = {
  "ar-DZ": {
    title: "المخزون والمشتريات", inventory: "المخزون", purchasing: "المشتريات",
    overview: "نظرة المخزون", ledger: "سجل الحركات", opening: "المخزون الافتتاحي",
    transfer: "تحويل المخزون", adjustment: "تسوية المخزون", count: "الجرد الفعلي",
    reservations: "الحجوزات", orders: "أوامر الشراء", orderEditor: "محرر أمر الشراء",
    receipt: "استلام المشتريات", invoice: "فاتورة المورد", direct: "استلام وفوترة مباشرة",
    return: "مرتجع المشتريات", reconciliation: "مطابقة الأرصدة", refresh: "تحديث",
    loading: "جارٍ تحميل السجل المحلي…", empty: "لا توجد بيانات مطابقة.", retry: "إعادة المحاولة",
    save: "حفظ المسودة", review: "إرسال للمراجعة", post: "ترحيل", confirm: "تأكيد",
    cancel: "إلغاء", release: "تحرير", consume: "استهلاك", rebuild: "إعادة بناء projection",
    product: "المادة", warehouse: "المستودع", location: "الموقع", onHand: "الموجود",
    reserved: "المحجوز", available: "المتاح", cump: "CUMP", value: "قيمة المخزون",
    quantity: "الكمية", cost: "تكلفة الوحدة", price: "سعر الوحدة", date: "التاريخ",
    reason: "السبب", source: "المصدر", destination: "الوجهة", supplier: "المورد",
    status: "الحالة", document: "المستند", movement: "نوع الحركة", variance: "الفرق",
    systemQuantity: "الكمية النظامية", countedQuantity: "الكمية المعدودة", mismatch: "عدم تطابق",
    projection: "القيمة الحالية", rebuilt: "القيمة المعاد بناؤها", success: "تمت العملية محليًا.",
    error: "تعذر إكمال العملية المحلية.", postedLocked: "المستند المرحّل مقفل ولا يقبل التعديل.",
    negativeWarning: "سيؤدي هذا الإجراء إلى مخزون سالب. الاستثناء حساس ويُسجل في التدقيق.",
    overrideConfirm: "أؤكد استعمال الاستثناء الإداري مع السبب المدخل", idempotency: "مفتاح عدم التكرار",
    documentId: "معرف المستند", sourceLineId: "معرف سطر المصدر", rowVersion: "نسخة السجل",
    countNumber: "رقم الجرد", notes: "ملاحظات", sourceMovement: "حركة الخروج المرتبطة",
    directNote: "يُنشئ الاستلام والفاتورة والحركة مرة واحدة داخل معاملة واحدة.",
    preview: "معاينة اختبار المتصفح — البيانات اصطناعية",
    confirmPosting: "سيُنشئ هذا الإجراء سجلًا غير قابل للتعديل. هل تريد المتابعة؟",
  },
  "fr-DZ": {
    title: "Stock et achats", inventory: "Stock", purchasing: "Achats", overview: "Vue du stock",
    ledger: "Journal des mouvements", opening: "Stock initial", transfer: "Transfert de stock",
    adjustment: "Ajustement", count: "Inventaire physique", reservations: "Réservations",
    orders: "Commandes fournisseur", orderEditor: "Éditeur de commande", receipt: "Réception achat",
    invoice: "Facture fournisseur", direct: "Réception et facture directe", return: "Retour fournisseur",
    reconciliation: "Rapprochement des soldes", refresh: "Actualiser", loading: "Chargement du registre local…",
    empty: "Aucune donnée correspondante.", retry: "Réessayer", save: "Enregistrer le brouillon",
    review: "Soumettre à la revue", post: "Valider", confirm: "Confirmer", cancel: "Annuler",
    release: "Libérer", consume: "Consommer", rebuild: "Reconstruire la projection",
    product: "Article", warehouse: "Dépôt", location: "Emplacement", onHand: "En stock",
    reserved: "Réservé", available: "Disponible", cump: "CUMP", value: "Valeur du stock",
    quantity: "Quantité", cost: "Coût unitaire", price: "Prix unitaire", date: "Date",
    reason: "Motif", source: "Source", destination: "Destination", supplier: "Fournisseur",
    status: "Statut", document: "Document", movement: "Mouvement", variance: "Écart",
    systemQuantity: "Quantité système", countedQuantity: "Quantité comptée", mismatch: "Écart détecté",
    projection: "Projection actuelle", rebuilt: "Valeur reconstruite", success: "Opération enregistrée localement.",
    error: "L’opération locale n’a pas pu être terminée.", postedLocked: "Le document validé est verrouillé.",
    negativeWarning: "Cette opération crée un stock négatif. La dérogation sensible est auditée.",
    overrideConfirm: "Je confirme la dérogation administrative avec le motif saisi",
    idempotency: "Clé d’idempotence", documentId: "Identifiant du document",
    sourceLineId: "Ligne source", rowVersion: "Version de ligne", countNumber: "Numéro d’inventaire",
    notes: "Notes", sourceMovement: "Mouvement de sortie lié",
    directNote: "Crée la réception, la facture et un seul mouvement dans la même transaction.",
    preview: "Aperçu de test navigateur — données synthétiques",
    confirmPosting: "Cette action crée un registre immuable. Voulez-vous continuer ?",
  },
};

export const PHASE06_SCREENS: readonly Phase06Screen[] = [
  "overview", "ledger", "opening", "transfer", "adjustment", "count", "reservations",
  "orders", "orderEditor", "receipt", "invoice", "direct", "return", "reconciliation",
];
