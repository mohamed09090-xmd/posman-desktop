import { useCallback, useEffect, useMemo, useState, type FormEvent, type ReactNode } from "react";
import { useI18n } from "../../i18n/I18nProvider";
import {
  Phase05GatewayError,
  resolvePhase05Gateway,
  type BelowCostPolicy,
  type CompanyProfile,
  type DocumentSequenceView,
  type FiscalSetup,
  type Page,
  type PartnerView,
  type ProductView,
  type ReferenceRecord,
  type RoleView,
  type SessionView,
  type SetupStatus,
  type UserView,
} from "../../platform/tauri/phase05";
import "./phase05.css";

type ScreenId =
  | "company" | "fiscal" | "pricing" | "sequences" | "users" | "roles"
  | "families" | "units" | "warehouses" | "locations" | "taxes"
  | "customers" | "suppliers" | "products";
type LoadState = "idle" | "loading" | "ready" | "empty" | "error" | "denied";
type Locale = "ar-DZ" | "fr-DZ";

type Copy = {
  appName: string; local: string; preview: string; loading: string; retry: string; save: string; create: string;
  cancel: string; search: string; empty: string; denied: string; error: string; success: string; logout: string;
  setupTitle: string; setupLead: string; next: string; back: string; finish: string; recoveryKeep: string;
  loginTitle: string; username: string; password: string; recover: string; newPassword: string; recoveryCode: string;
  company: string; fiscal: string; pricing: string; sequences: string; users: string; roles: string;
  families: string; units: string; warehouses: string; locations: string; taxes: string;
  customers: string; suppliers: string; products: string; administration: string; reference: string;
  legalName: string; commercialName: string; activity: string; address: string; wilaya: string; phone: string;
  email: string; identifiers: string; margin: string; timeout: string; policy: string; code: string;
  nameAr: string; nameFr: string; active: string; actions: string; price: string; purchaseCost: string;
  salePrice: string; zeroMargin: string; belowCost: string; overrideReason: string; role: string;
  displayName: string; language: string; dates: string; period: string; documentType: string; previewNumber: string;
};
const COPY: Record<Locale, Copy> = {
  "ar-DZ": {
    appName: "POSMAN — دفتر العمليات", local: "محلي دون اتصال", preview: "معاينة متصفح — لا يتم حفظ البيانات", loading: "جارٍ التحميل…",
    retry: "إعادة المحاولة", save: "حفظ", create: "إنشاء", cancel: "إلغاء", search: "بحث", empty: "لا توجد سجلات بعد.",
    denied: "ليس لديك الإذن لفتح هذه المساحة.", error: "تعذر إكمال العملية المحلية.", success: "تم حفظ التغيير محليًا.",
    logout: "تسجيل الخروج", setupTitle: "تهيئة POSMAN لأول مرة", setupLead: "إعداد الشركة والسنة الجبائية والمسؤول المحلي في معاملة واحدة.",
    next: "التالي", back: "السابق", finish: "إتمام التهيئة", recoveryKeep: "رمز الاسترجاع يظهر مرة واحدة. خزّنه خارج الجهاز قبل المتابعة.",
    loginTitle: "الدخول المحلي", username: "اسم المستخدم", password: "كلمة المرور", recover: "استرجاع كلمة المرور", newPassword: "كلمة المرور الجديدة",
    recoveryCode: "رمز الاسترجاع", company: "بيانات الشركة", fiscal: "السنة والفترات", pricing: "الضرائب والتسعير", sequences: "تسلسل المستندات",
    users: "المستخدمون", roles: "الأدوار والصلاحيات", families: "العائلات", units: "الوحدات", warehouses: "المستودعات",
    locations: "مواقع المستودع", taxes: "الضرائب", customers: "العملاء", suppliers: "الموردون", products: "المواد",
    administration: "الإدارة", reference: "البيانات المرجعية", legalName: "الاسم القانوني", commercialName: "الاسم التجاري", activity: "النشاط",
    address: "العنوان", wilaya: "الولاية", phone: "الهاتف", email: "البريد الإلكتروني", identifiers: "RC / NIF / NIS / AI / RIB",
    margin: "الهامش الافتراضي", timeout: "مهلة الخمول", policy: "سياسة البيع تحت التكلفة", code: "الرمز", nameAr: "الاسم بالعربية",
    nameFr: "الاسم بالفرنسية", active: "نشط", actions: "الإجراءات", price: "السعر", purchaseCost: "تكلفة الشراء HT", salePrice: "سعر البيع HT",
    zeroMargin: "هامش صفري", belowCost: "سعر البيع أقل من تكلفة الشراء", overrideReason: "سبب التجاوز", role: "الدور", displayName: "الاسم الظاهر",
    language: "اللغة", dates: "التواريخ", period: "الفترة", documentType: "نوع المستند", previewNumber: "المعاينة",
  },
  "fr-DZ": {
    appName: "POSMAN — Registre des opérations", local: "Local hors ligne", preview: "Aperçu navigateur — aucune donnée n'est enregistrée", loading: "Chargement…",
    retry: "Réessayer", save: "Enregistrer", create: "Créer", cancel: "Annuler", search: "Rechercher", empty: "Aucun enregistrement.",
    denied: "Vous n’avez pas l’autorisation d’ouvrir cet espace.", error: "L’opération locale n’a pas pu être terminée.", success: "Modification enregistrée localement.",
    logout: "Déconnexion", setupTitle: "Première configuration de POSMAN", setupLead: "Configurer la société, l’exercice et l’administrateur local dans une seule transaction.",
    next: "Suivant", back: "Précédent", finish: "Terminer la configuration", recoveryKeep: "Le code de récupération n’est affiché qu’une fois. Conservez-le hors de l’appareil.",
    loginTitle: "Connexion locale", username: "Nom d’utilisateur", password: "Mot de passe", recover: "Récupérer le mot de passe", newPassword: "Nouveau mot de passe",
    recoveryCode: "Code de récupération", company: "Société", fiscal: "Exercice et périodes", pricing: "Taxes et tarification", sequences: "Séquences documentaires",
    users: "Utilisateurs", roles: "Rôles et permissions", families: "Familles", units: "Unités", warehouses: "Dépôts",
    locations: "Emplacements", taxes: "Taxes", customers: "Clients", suppliers: "Fournisseurs", products: "Articles",
    administration: "Administration", reference: "Données de référence", legalName: "Raison sociale", commercialName: "Nom commercial", activity: "Activité",
    address: "Adresse", wilaya: "Wilaya", phone: "Téléphone", email: "E-mail", identifiers: "RC / NIF / NIS / AI / RIB",
    margin: "Marge par défaut", timeout: "Délai d’inactivité", policy: "Politique de vente sous le coût", code: "Code", nameAr: "Nom arabe",
    nameFr: "Nom français", active: "Actif", actions: "Actions", price: "Prix", purchaseCost: "Coût d’achat HT", salePrice: "Prix de vente HT",
    zeroMargin: "Marge nulle", belowCost: "Prix de vente inférieur au coût d’achat", overrideReason: "Motif de dérogation", role: "Rôle", displayName: "Nom affiché",
    language: "Langue", dates: "Dates", period: "Période", documentType: "Type de document", previewNumber: "Aperçu",
  },
};

const previewReferences: ReferenceRecord[] = [
  { id: "preview-1", code: "GEN", nameAr: "عام", nameFr: "Général", isActive: true, rowVersion: 1, details: {} },
  { id: "preview-2", code: "STD", nameAr: "قياسي", nameFr: "Standard", isActive: true, rowVersion: 1, details: {} },
];
const previewProducts: ProductView[] = [
  { id: "p1", code: "HUI-001", nameAr: "زيت مائدة 1 لتر", nameFr: "Huile 1 L", unitId: "u1", purchasePriceScaled: 520000, salePriceScaled: 610000, suggestedSalePriceScaled: 620000, belowCostPolicy: "ADMIN_OVERRIDE", isActive: true, rowVersion: 1 },
  { id: "p2", code: "CAF-250", nameAr: "قهوة 250غ", nameFr: "Café 250 g", unitId: "u1", purchasePriceScaled: 480000, salePriceScaled: 480000, suggestedSalePriceScaled: 540000, pricingWarning: "ZERO_MARGIN", belowCostPolicy: "ADMIN_OVERRIDE", isActive: true, rowVersion: 1 },
];

function Field({ label, children, wide = false }: { label: string; children: ReactNode; wide?: boolean }) {
  return <label className={`p5-field${wide ? " p5-field--wide" : ""}`}><span>{label}</span>{children}</label>;
}
function Input(props: React.InputHTMLAttributes<HTMLInputElement>) { return <input className="p5-input" {...props} />; }
function Select(props: React.SelectHTMLAttributes<HTMLSelectElement>) { return <select className="p5-input" {...props} />; }
function Notice({ tone = "info", children }: { tone?: "info" | "success" | "error" | "warning"; children: ReactNode }) { return <div className={`p5-notice p5-notice--${tone}`} role={tone === "error" ? "alert" : "status"}>{children}</div>; }
function Button({ kind = "normal", ...props }: React.ButtonHTMLAttributes<HTMLButtonElement> & { kind?: "normal" | "primary" | "danger" }) { return <button className={`p5-button p5-button--${kind}`} {...props} />; }

function errorCode(error: unknown): string { return error instanceof Phase05GatewayError ? error.code : "OPERATION_FAILED"; }
function scaledFromInput(value: FormDataEntryValue | null): number { return Math.round(Number(value || 0) * 10_000); }
function scaledToInput(value: number): string { return (value / 10_000).toFixed(2); }

function SetupFlow({ status, onDone }: { status: SetupStatus; onDone: () => void }) {
  const { locale } = useI18n(); const c = COPY[locale];
  const gateway = useMemo(resolvePhase05Gateway, []);
  const [step, setStep] = useState(0); const [busy, setBusy] = useState(false); const [error, setError] = useState<string>();
  const [recoveryCode, setRecoveryCode] = useState<string>();
  const [draft, setDraft] = useState({
    companyCode: "POSMAN", legalName: "", nameAr: "", nameFr: "", activityDescription: "", addressText: "", wilayaCode: "16", city: "", postalCode: "", phone: "", email: "",
    tradeRegisterNumber: "", taxIdentifier: "", statisticalIdentifier: "", taxArticleNumber: "", bankRib: "",
    fiscalStartsOn: status.defaultFiscalStartsOn, fiscalEndsOn: status.defaultFiscalEndsOn, defaultMargin: "20", belowCostPolicy: "ADMIN_OVERRIDE" as BelowCostPolicy,
    administratorUsername: "admin", administratorDisplayName: "", administratorPassword: "", administratorPasswordConfirmation: "",
  });
  const set = (key: keyof typeof draft, value: string) => setDraft((current) => ({ ...current, [key]: value }));
  const persistDraft = async () => {
    if (!gateway) return;
    const { administratorPassword: _password, administratorPasswordConfirmation: _confirmation, ...safe } = draft;
    await gateway.call("save_setup_draft", { draftSchemaVersion: 1, data: safe, rowVersion: undefined });
  };
  const next = async () => { setError(undefined); setBusy(true); try { await persistDraft(); setStep((value) => Math.min(value + 1, 3)); } catch (e) { setError(errorCode(e)); } finally { setBusy(false); } };
  const complete = async () => {
    setBusy(true); setError(undefined);
    try {
      if (!gateway) { setRecoveryCode("PREVIEW-ONLY-NOT-A-REAL-CODE"); return; }
      const result = await gateway.call<{ recoveryCode?: string }>("complete_initial_setup", {
        idempotencyKey: crypto.randomUUID(), companyCode: draft.companyCode, nameAr: draft.nameAr, nameFr: draft.nameFr || undefined,
        legalName: draft.legalName, activityDescription: draft.activityDescription, legalForm: undefined,
        tradeRegisterNumber: draft.tradeRegisterNumber || undefined, taxIdentifier: draft.taxIdentifier || undefined,
        statisticalIdentifier: draft.statisticalIdentifier || undefined, taxArticleNumber: draft.taxArticleNumber || undefined,
        bankRib: draft.bankRib || undefined, socialCapitalMinor: undefined, addressText: draft.addressText, wilayaCode: draft.wilayaCode,
        city: draft.city || undefined, postalCode: draft.postalCode || undefined, phone: draft.phone, email: draft.email || undefined,
        language: locale, fiscalStartsOn: draft.fiscalStartsOn, fiscalEndsOn: draft.fiscalEndsOn,
        defaultMarginRateScaled: Math.round(Number(draft.defaultMargin) * 10_000), belowCostPolicy: draft.belowCostPolicy,
        sessionIdleTimeoutMinutes: 15,
        taxes: [{ code: "TVA19", nameAr: "الرسم على القيمة المضافة 19%", nameFr: "TVA 19%", rateScaled: 190000 }], defaultTaxCode: "TVA19",
        warehouseCode: "DEP-PRINCIPAL", warehouseNameAr: "المستودع الرئيسي", warehouseNameFr: "Dépôt principal",
        administratorUsername: draft.administratorUsername, administratorDisplayName: draft.administratorDisplayName,
        administratorPassword: draft.administratorPassword, administratorPasswordConfirmation: draft.administratorPasswordConfirmation,
      });
      setRecoveryCode(result.recoveryCode);
    } catch (e) { setError(errorCode(e)); } finally { setBusy(false); }
  };
  if (recoveryCode) return <AuthFrame><h1>{c.recoveryCode}</h1><Notice tone="warning">{c.recoveryKeep}</Notice><output className="p5-recovery-code">{recoveryCode}</output><Button kind="primary" onClick={onDone}>{c.loginTitle}</Button></AuthFrame>;
  return <AuthFrame>
    <header><p className="p5-eyebrow">01 — 04</p><h1>{c.setupTitle}</h1><p>{c.setupLead}</p></header>
    <ol className="p5-steps" aria-label={c.setupTitle}>{[c.company, c.fiscal, c.pricing, c.users].map((label, index) => <li className={index === step ? "is-current" : index < step ? "is-done" : ""} key={label}>{label}</li>)}</ol>
    {error ? <Notice tone="error">{c.error} <code>{error}</code></Notice> : null}
    <div className="p5-form-grid">
      {step === 0 ? <>
        <Field label={c.code}><Input value={draft.companyCode} onChange={(e) => set("companyCode", e.target.value)} required /></Field>
        <Field label={c.legalName}><Input value={draft.legalName} onChange={(e) => set("legalName", e.target.value)} required /></Field>
        <Field label={c.nameAr}><Input value={draft.nameAr} onChange={(e) => set("nameAr", e.target.value)} required /></Field>
        <Field label={c.nameFr}><Input value={draft.nameFr} onChange={(e) => set("nameFr", e.target.value)} /></Field>
        <Field label={c.activity} wide><Input value={draft.activityDescription} onChange={(e) => set("activityDescription", e.target.value)} required /></Field>
        <Field label={c.address} wide><Input value={draft.addressText} onChange={(e) => set("addressText", e.target.value)} required /></Field>
        <Field label={c.wilaya}><Input value={draft.wilayaCode} onChange={(e) => set("wilayaCode", e.target.value)} inputMode="numeric" required /></Field>
        <Field label={c.phone}><Input value={draft.phone} onChange={(e) => set("phone", e.target.value)} required /></Field>
        <Field label={c.email}><Input type="email" value={draft.email} onChange={(e) => set("email", e.target.value)} /></Field>
      </> : null}
      {step === 1 ? <>
        <Field label="RC"><Input value={draft.tradeRegisterNumber} onChange={(e) => set("tradeRegisterNumber", e.target.value)} /></Field>
        <Field label="NIF"><Input value={draft.taxIdentifier} onChange={(e) => set("taxIdentifier", e.target.value)} /></Field>
        <Field label="NIS"><Input value={draft.statisticalIdentifier} onChange={(e) => set("statisticalIdentifier", e.target.value)} /></Field>
        <Field label="AI"><Input value={draft.taxArticleNumber} onChange={(e) => set("taxArticleNumber", e.target.value)} /></Field>
        <Field label="RIB" wide><Input value={draft.bankRib} onChange={(e) => set("bankRib", e.target.value)} /></Field>
        <Field label={`${c.fiscal} — début`}><Input type="date" value={draft.fiscalStartsOn} onChange={(e) => set("fiscalStartsOn", e.target.value)} /></Field>
        <Field label={`${c.fiscal} — fin`}><Input type="date" value={draft.fiscalEndsOn} onChange={(e) => set("fiscalEndsOn", e.target.value)} /></Field>
      </> : null}
      {step === 2 ? <>
        <Field label={`${c.margin} %`}><Input type="number" min="0" max="100" value={draft.defaultMargin} onChange={(e) => set("defaultMargin", e.target.value)} /></Field>
        <Field label={c.policy}><Select value={draft.belowCostPolicy} onChange={(e) => set("belowCostPolicy", e.target.value)}><option value="ADMIN_OVERRIDE">ADMIN_OVERRIDE</option><option value="BLOCK">BLOCK</option><option value="WARNING_ONLY">WARNING_ONLY</option></Select></Field>
        <Notice tone="info">TVA 19% · FAC-YYYY-000001 · BL-YYYY-000001 · BC-YYYY-000001</Notice>
      </> : null}
      {step === 3 ? <>
        <Field label={c.username}><Input autoComplete="username" value={draft.administratorUsername} onChange={(e) => set("administratorUsername", e.target.value)} required /></Field>
        <Field label={c.displayName}><Input value={draft.administratorDisplayName} onChange={(e) => set("administratorDisplayName", e.target.value)} required /></Field>
        <Field label={c.password}><Input type="password" autoComplete="new-password" value={draft.administratorPassword} onChange={(e) => set("administratorPassword", e.target.value)} required /></Field>
        <Field label={`${c.password} — confirmation`}><Input type="password" autoComplete="new-password" value={draft.administratorPasswordConfirmation} onChange={(e) => set("administratorPasswordConfirmation", e.target.value)} required /></Field>
      </> : null}
    </div>
    <footer className="p5-auth-actions">{step > 0 ? <Button onClick={() => setStep((v) => v - 1)}>{c.back}</Button> : <span />}{step < 3 ? <Button kind="primary" disabled={busy} onClick={next}>{c.next}</Button> : <Button kind="primary" disabled={busy} onClick={complete}>{c.finish}</Button>}</footer>
  </AuthFrame>;
}

function AuthFrame({ children }: { children: ReactNode }) { const { locale, setLocale } = useI18n(); return <main className="p5-auth"><div className="p5-auth__language"><button onClick={() => setLocale(locale === "ar-DZ" ? "fr-DZ" : "ar-DZ")}>{locale === "ar-DZ" ? "Français" : "العربية"}</button></div><section className="p5-auth__card">{children}</section></main>; }

function LoginFlow({ onLogin }: { onLogin: (session: SessionView) => void }) {
  const { locale } = useI18n(); const c = COPY[locale]; const gateway = useMemo(resolvePhase05Gateway, []);
  const [mode, setMode] = useState<"login" | "recover">("login"); const [busy, setBusy] = useState(false); const [error, setError] = useState<string>();
  const submit = async (event: FormEvent<HTMLFormElement>) => { event.preventDefault(); setBusy(true); setError(undefined); const form = new FormData(event.currentTarget);
    try {
      if (!gateway) { onLogin({ companyId: "preview", userId: "preview", username: "admin", displayName: "Administrateur", preferredLanguage: locale, permissions: ["*"], locked: false }); return; }
      if (mode === "recover") {
        await gateway.call("recover_admin_password", { username: form.get("username"), recoveryCode: form.get("recoveryCode"), newPassword: form.get("newPassword"), newPasswordConfirmation: form.get("newPasswordConfirmation") });
        setMode("login"); return;
      }
      onLogin(await gateway.call<SessionView>("login", { username: form.get("username"), password: form.get("password") }));
    } catch (e) { setError(errorCode(e)); } finally { setBusy(false); }
  };
  return <AuthFrame><header><p className="p5-eyebrow">{c.local}</p><h1>{mode === "login" ? c.loginTitle : c.recover}</h1></header>{error ? <Notice tone="error">{c.error} <code>{error}</code></Notice> : null}
    <form className="p5-form-grid" onSubmit={submit}><Field label={c.username} wide><Input name="username" autoComplete="username" required /></Field>
      {mode === "login" ? <Field label={c.password} wide><Input name="password" type="password" autoComplete="current-password" required /></Field> : <><Field label={c.recoveryCode} wide><Input name="recoveryCode" autoComplete="one-time-code" required /></Field><Field label={c.newPassword}><Input name="newPassword" type="password" required /></Field><Field label={`${c.newPassword} — confirmation`}><Input name="newPasswordConfirmation" type="password" required /></Field></>}
      <div className="p5-auth-actions p5-field--wide"><Button type="button" onClick={() => setMode(mode === "login" ? "recover" : "login")}>{mode === "login" ? c.recover : c.loginTitle}</Button><Button kind="primary" disabled={busy} type="submit">{mode === "login" ? c.loginTitle : c.save}</Button></div></form></AuthFrame>;
}

const referenceCommands: Record<"families" | "units" | "warehouses" | "locations" | "taxes", { list: any; create: any; update: any; active: any }> = {
  families: { list: "list_product_families", create: "create_product_family", update: "update_product_family", active: "set_product_family_active" },
  units: { list: "list_units", create: "create_unit", update: "update_unit", active: "set_unit_active" },
  warehouses: { list: "list_warehouses", create: "create_warehouse", update: "update_warehouse", active: "set_warehouse_active" },
  locations: { list: "list_warehouse_locations", create: "create_warehouse_location", update: "update_warehouse_location", active: "set_warehouse_location_active" },
  taxes: { list: "list_tax_rates", create: "create_tax_rate", update: "update_tax_rate", active: "set_tax_rate_active" },
};

function ReferenceScreen({ kind }: { kind: keyof typeof referenceCommands }) {
  const { locale } = useI18n(); const c = COPY[locale]; const gateway = useMemo(resolvePhase05Gateway, []); const commands = referenceCommands[kind];
  const [records, setRecords] = useState<ReferenceRecord[]>([]); const [state, setState] = useState<LoadState>("loading"); const [message, setMessage] = useState<string>(); const [search, setSearch] = useState("");
  const load = useCallback(async () => { setState("loading"); try { const items = gateway ? (await gateway.call<Page<ReferenceRecord>>(commands.list, { search, page: 1, pageSize: 50, includeInactive: true })).items : previewReferences; setRecords(items); setState(items.length ? "ready" : "empty"); } catch (e) { setState(errorCode(e) === "PERMISSION_DENIED" ? "denied" : "error"); } }, [commands.list, gateway, search]);
  useEffect(() => { void load(); }, [load]);
  const create = async (event: FormEvent<HTMLFormElement>) => { event.preventDefault(); const form = new FormData(event.currentTarget); try { if (gateway) await gateway.call(commands.create, { code: form.get("code"), nameAr: form.get("nameAr"), nameFr: form.get("nameFr") || undefined, numericValue: kind === "taxes" ? scaledFromInput(form.get("numericValue")) : undefined, kind: kind === "units" ? "COUNT" : undefined, parentId: kind === "locations" ? form.get("parentId") || undefined : undefined, relatedId: undefined, addressText: undefined, flag: undefined }); setMessage(c.success); event.currentTarget.reset(); await load(); } catch (e) { setMessage(`${c.error} (${errorCode(e)})`); } };
  return <LedgerPage title={c[kind]} actions={<div className="p5-search"><Input aria-label={c.search} placeholder={c.search} value={search} onChange={(e) => setSearch(e.target.value)} /><Button onClick={load}>{c.search}</Button></div>}>
    {message ? <Notice tone={message === c.success ? "success" : "error"}>{message}</Notice> : null}
    <form className="p5-inline-form" onSubmit={create}><Input name="code" placeholder={c.code} required /><Input name="nameAr" placeholder={c.nameAr} required /><Input name="nameFr" placeholder={c.nameFr} />{kind === "taxes" ? <Input name="numericValue" type="number" step="0.01" placeholder="19.00" required /> : null}{kind === "locations" ? <Input name="parentId" placeholder="warehouseId" required /> : null}<Button kind="primary" type="submit">{c.create}</Button></form>
    <StateBoundary state={state} onRetry={load}><table className="p5-table"><thead><tr><th>{c.code}</th><th>{c.nameAr}</th><th>{c.nameFr}</th><th>{c.active}</th></tr></thead><tbody>{records.map((record) => <tr key={record.id}><td><code>{record.code}</code></td><td>{record.nameAr}</td><td>{record.nameFr || "—"}</td><td>{record.isActive ? "✓" : "—"}</td></tr>)}</tbody></table></StateBoundary>
  </LedgerPage>;
}

function ProductsScreen() {
  const { locale } = useI18n(); const c = COPY[locale]; const gateway = useMemo(resolvePhase05Gateway, []); const [items, setItems] = useState<ProductView[]>([]); const [state, setState] = useState<LoadState>("loading"); const [message, setMessage] = useState<string>();
  const load = useCallback(async () => { setState("loading"); try { const products = gateway ? (await gateway.call<Page<ProductView>>("list_products", { page: 1, pageSize: 100, includeInactive: true })).items : previewProducts; setItems(products); setState(products.length ? "ready" : "empty"); } catch (e) { setState(errorCode(e) === "PERMISSION_DENIED" ? "denied" : "error"); } }, [gateway]);
  useEffect(() => { void load(); }, [load]);
  const create = async (event: FormEvent<HTMLFormElement>) => { event.preventDefault(); const f = new FormData(event.currentTarget); const purchase = scaledFromInput(f.get("purchase")); const sale = scaledFromInput(f.get("sale")); try { if (gateway) await gateway.call("create_product", { code: f.get("code"), nameAr: f.get("nameAr"), nameFr: f.get("nameFr") || undefined, barcode: undefined, unitId: f.get("unitId"), productFamilyId: undefined, defaultTaxRateId: undefined, defaultPurchasePriceScaled: purchase, manualSalePriceScaled: sale, belowCostOverrideReason: f.get("reason") || undefined, marginRateScaled: undefined, minimumStockScaled: 0, productKind: "STOCK_ITEM" }); setMessage(c.success); event.currentTarget.reset(); await load(); } catch (e) { setMessage(`${c.error} (${errorCode(e)})`); } };
  return <LedgerPage title={c.products}><form className="p5-product-form" onSubmit={create}><Field label={c.code}><Input name="code" required /></Field><Field label={c.nameAr}><Input name="nameAr" required /></Field><Field label={c.nameFr}><Input name="nameFr" /></Field><Field label="unitId"><Input name="unitId" required /></Field><Field label={c.purchaseCost}><Input name="purchase" type="number" min="0" step="0.0001" required /></Field><Field label={c.salePrice}><Input name="sale" type="number" min="0" step="0.0001" required /></Field><Field label={c.overrideReason} wide><Input name="reason" /></Field><Button kind="primary" type="submit">{c.create}</Button></form>{message ? <Notice tone={message === c.success ? "success" : "error"}>{message}</Notice> : null}<StateBoundary state={state} onRetry={load}><table className="p5-table"><thead><tr><th>{c.code}</th><th>{c.products}</th><th>{c.purchaseCost}</th><th>{c.salePrice}</th><th>{c.policy}</th></tr></thead><tbody>{items.map((p) => <tr key={p.id} className={p.pricingWarning ? "has-warning" : ""}><td><code>{p.code}</code></td><td>{locale === "ar-DZ" ? p.nameAr : p.nameFr || p.nameAr}</td><td>{scaledToInput(p.purchasePriceScaled)}</td><td>{scaledToInput(p.salePriceScaled)}</td><td>{p.pricingWarning ? <strong className="p5-red">{p.pricingWarning === "BELOW_COST" ? c.belowCost : c.zeroMargin}</strong> : p.belowCostPolicy}</td></tr>)}</tbody></table></StateBoundary></LedgerPage>;
}

function PartnersScreen({ supplier }: { supplier: boolean }) {
  const { locale } = useI18n(); const c = COPY[locale]; const gateway = useMemo(resolvePhase05Gateway, []); const [items, setItems] = useState<PartnerView[]>([]); const [state, setState] = useState<LoadState>("loading");
  const load = useCallback(async () => { setState("loading"); try { const all = gateway ? (await gateway.call<Page<PartnerView>>("list_partners", { page: 1, pageSize: 100, includeInactive: true })).items : [{ id: "partner-preview", code: supplier ? "FRN-001" : "CLI-001", legalName: "SARL Atlas", displayNameAr: supplier ? "مورد الأطلس" : "عميل الأطلس", displayNameFr: supplier ? "Fournisseur Atlas" : "Client Atlas", isCustomer: !supplier, isSupplier: supplier, isActive: true, rowVersion: 1 }]; const filtered = all.filter((p) => supplier ? p.isSupplier : p.isCustomer); setItems(filtered); setState(filtered.length ? "ready" : "empty"); } catch (e) { setState(errorCode(e) === "PERMISSION_DENIED" ? "denied" : "error"); } }, [gateway, supplier]);
  useEffect(() => { void load(); }, [load]);
  const create = async (event: FormEvent<HTMLFormElement>) => { event.preventDefault(); const f = new FormData(event.currentTarget); if (gateway) await gateway.call("create_partner", { code: f.get("code"), legalName: f.get("legalName"), displayNameAr: f.get("nameAr"), displayNameFr: f.get("nameFr") || undefined, isCustomer: !supplier, isSupplier: supplier, legalForm: undefined, activityDescription: undefined, taxIdentifier: undefined, tradeRegisterNumber: undefined, statisticalIdentifier: undefined, taxArticleNumber: undefined, paymentTermId: undefined }); event.currentTarget.reset(); await load(); };
  return <LedgerPage title={supplier ? c.suppliers : c.customers}><form className="p5-inline-form" onSubmit={(e) => void create(e)}><Input name="code" placeholder={c.code} required /><Input name="legalName" placeholder={c.legalName} required /><Input name="nameAr" placeholder={c.nameAr} required /><Input name="nameFr" placeholder={c.nameFr} /><Button kind="primary" type="submit">{c.create}</Button></form><StateBoundary state={state} onRetry={load}><table className="p5-table"><thead><tr><th>{c.code}</th><th>{c.legalName}</th><th>{c.commercialName}</th><th>{c.active}</th></tr></thead><tbody>{items.map((p) => <tr key={p.id}><td><code>{p.code}</code></td><td>{p.legalName}</td><td>{locale === "ar-DZ" ? p.displayNameAr : p.displayNameFr || p.displayNameAr}</td><td>{p.isActive ? "✓" : "—"}</td></tr>)}</tbody></table></StateBoundary></LedgerPage>;
}

function CompanyScreen({ pricingOnly = false }: { pricingOnly?: boolean }) {
  const { locale } = useI18n(); const c = COPY[locale]; const gateway = useMemo(resolvePhase05Gateway, []); const [profile, setProfile] = useState<CompanyProfile>(); const [state, setState] = useState<LoadState>("loading"); const [message, setMessage] = useState<string>();
  const load = useCallback(async () => { setState("loading"); try { const value = gateway ? await gateway.call<CompanyProfile>("get_company_profile") : { id: "preview", code: "POSMAN", legalName: "SARL Atlas Commerce", nameAr: "مؤسسة الأطلس للتجارة", nameFr: "Atlas Commerce", activityDescription: "Commerce", addressText: "Alger", wilayaCode: "16", phone: "0550 00 00 00", defaultMarginRateScaled: 200000, belowCostPolicy: "ADMIN_OVERRIDE" as const, sessionIdleTimeoutMinutes: 15, rowVersion: 1 }; setProfile(value); setState("ready"); } catch (e) { setState(errorCode(e) === "PERMISSION_DENIED" ? "denied" : "error"); } }, [gateway]); useEffect(() => { void load(); }, [load]);
  const save = async (event: FormEvent<HTMLFormElement>) => { event.preventDefault(); if (!profile) return; const f = new FormData(event.currentTarget); try { if (gateway) setProfile(await gateway.call<CompanyProfile>("update_company_profile", { legalName: f.get("legalName") || profile.legalName, nameAr: f.get("nameAr") || profile.nameAr, nameFr: f.get("nameFr") || undefined, activityDescription: f.get("activity") || undefined, legalForm: profile.legalForm, tradeRegisterNumber: profile.tradeRegisterNumber, taxIdentifier: profile.taxIdentifier, statisticalIdentifier: profile.statisticalIdentifier, taxArticleNumber: profile.taxArticleNumber, bankRib: profile.bankRib, socialCapitalMinor: profile.socialCapitalMinor, addressText: f.get("address") || profile.addressText || "", wilayaCode: f.get("wilaya") || profile.wilayaCode || "16", city: profile.city, postalCode: profile.postalCode, phone: f.get("phone") || profile.phone || "", email: f.get("email") || undefined, defaultMarginRateScaled: scaledFromInput(f.get("margin")), belowCostPolicy: f.get("policy"), sessionIdleTimeoutMinutes: Number(f.get("timeout")), rowVersion: profile.rowVersion })); setMessage(c.success); } catch (e) { setMessage(`${c.error} (${errorCode(e)})`); } };
  return <LedgerPage title={pricingOnly ? c.pricing : c.company}>{message ? <Notice tone={message === c.success ? "success" : "error"}>{message}</Notice> : null}<StateBoundary state={state} onRetry={load}>{profile ? <form className="p5-form-grid" onSubmit={save}>{!pricingOnly ? <><Field label={c.legalName}><Input name="legalName" defaultValue={profile.legalName} /></Field><Field label={c.nameAr}><Input name="nameAr" defaultValue={profile.nameAr} /></Field><Field label={c.nameFr}><Input name="nameFr" defaultValue={profile.nameFr} /></Field><Field label={c.activity}><Input name="activity" defaultValue={profile.activityDescription} /></Field><Field label={c.address} wide><Input name="address" defaultValue={profile.addressText} /></Field><Field label={c.wilaya}><Input name="wilaya" defaultValue={profile.wilayaCode} /></Field><Field label={c.phone}><Input name="phone" defaultValue={profile.phone} /></Field><Field label={c.email}><Input name="email" type="email" defaultValue={profile.email} /></Field></> : null}<Field label={`${c.margin} %`}><Input name="margin" type="number" min="0" max="100" step="0.01" defaultValue={profile.defaultMarginRateScaled / 10000} /></Field><Field label={c.policy}><Select name="policy" defaultValue={profile.belowCostPolicy}><option value="ADMIN_OVERRIDE">ADMIN_OVERRIDE</option><option value="BLOCK">BLOCK</option><option value="WARNING_ONLY">WARNING_ONLY</option></Select></Field><Field label={`${c.timeout} (min)`}><Input name="timeout" type="number" min="5" max="120" defaultValue={profile.sessionIdleTimeoutMinutes} /></Field><div className="p5-field--wide"><Button kind="primary" type="submit">{c.save}</Button></div></form> : null}</StateBoundary></LedgerPage>;
}

function FiscalScreen() { const { locale } = useI18n(); const c = COPY[locale]; const gateway = useMemo(resolvePhase05Gateway, []); const [data, setData] = useState<FiscalSetup>(); const [state, setState] = useState<LoadState>("loading"); const load = useCallback(async () => { try { setState("loading"); const value = gateway ? await gateway.call<FiscalSetup>("get_fiscal_setup") : { fiscalYearId: "fy", code: "2026", startsOn: "2026-01-01", endsOn: "2026-12-31", periods: Array.from({ length: 12 }, (_, i) => ({ periodNumber: i + 1, name: String(i + 1).padStart(2, "0"), startsOn: `2026-${String(i + 1).padStart(2, "0")}-01`, endsOn: `2026-${String(i + 1).padStart(2, "0")}-28`, status: "OPEN" })), rowVersion: 1, inUse: false }; setData(value); setState("ready"); } catch (e) { setState(errorCode(e) === "PERMISSION_DENIED" ? "denied" : "error"); } }, [gateway]); useEffect(() => { void load(); }, [load]); return <LedgerPage title={c.fiscal}><StateBoundary state={state} onRetry={load}>{data ? <><div className="p5-metric-strip"><span>{data.code}</span><strong>{data.startsOn} → {data.endsOn}</strong></div><table className="p5-table"><thead><tr><th>{c.period}</th><th>{c.dates}</th><th>Status</th></tr></thead><tbody>{data.periods.map((p) => <tr key={p.periodNumber}><td>{p.periodNumber} · {p.name}</td><td>{p.startsOn} — {p.endsOn}</td><td>{p.status}</td></tr>)}</tbody></table></> : null}</StateBoundary></LedgerPage>; }
function SequenceScreen() { const { locale } = useI18n(); const c = COPY[locale]; const gateway = useMemo(resolvePhase05Gateway, []); const [items, setItems] = useState<DocumentSequenceView[]>([]); const [state, setState] = useState<LoadState>("loading"); const load = useCallback(async () => { try { const values = gateway ? await gateway.call<DocumentSequenceView[]>("list_document_sequences") : [{ id: "1", documentType: "SALES_INVOICE", prefix: "FAC", nextNumber: 1, paddingWidth: 6, preview: "FAC-2026-000001", rowVersion: 1 }, { id: "2", documentType: "DELIVERY_NOTE", prefix: "BL", nextNumber: 1, paddingWidth: 6, preview: "BL-2026-000001", rowVersion: 1 }, { id: "3", documentType: "PURCHASE_ORDER", prefix: "BC", nextNumber: 1, paddingWidth: 6, preview: "BC-2026-000001", rowVersion: 1 }]; setItems(values); setState(values.length ? "ready" : "empty"); } catch (e) { setState(errorCode(e) === "PERMISSION_DENIED" ? "denied" : "error"); } }, [gateway]); useEffect(() => { void load(); }, [load]); return <LedgerPage title={c.sequences}><StateBoundary state={state} onRetry={load}><table className="p5-table"><thead><tr><th>{c.documentType}</th><th>Prefix</th><th>{c.previewNumber}</th></tr></thead><tbody>{items.map((s) => <tr key={s.id}><td>{s.documentType}</td><td><code>{s.prefix}</code></td><td><strong>{s.preview}</strong></td></tr>)}</tbody></table></StateBoundary></LedgerPage>; }
function UsersScreen({ rolesOnly = false }: { rolesOnly?: boolean }) { const { locale } = useI18n(); const c = COPY[locale]; const gateway = useMemo(resolvePhase05Gateway, []); const [items, setItems] = useState<(UserView | RoleView)[]>([]); const [state, setState] = useState<LoadState>("loading"); const load = useCallback(async () => { try { const values = rolesOnly ? (gateway ? await gateway.call<RoleView[]>("list_roles") : [{ id: "r1", code: "SYSTEM_ADMINISTRATOR", nameAr: "مسؤول النظام", nameFr: "Administrateur système", isSystem: true, isActive: true, permissionCodes: ["*"], rowVersion: 1 }]) : (gateway ? (await gateway.call<Page<UserView>>("list_users", { page: 1, pageSize: 100, includeInactive: true })).items : [{ id: "u1", username: "admin", displayName: "Administrateur", preferredLanguage: locale, isActive: true, roleIds: ["r1"], rowVersion: 1 }]); setItems(values); setState(values.length ? "ready" : "empty"); } catch (e) { setState(errorCode(e) === "PERMISSION_DENIED" ? "denied" : "error"); } }, [gateway, locale, rolesOnly]); useEffect(() => { void load(); }, [load]); return <LedgerPage title={rolesOnly ? c.roles : c.users}><StateBoundary state={state} onRetry={load}><table className="p5-table"><thead><tr><th>{rolesOnly ? c.code : c.username}</th><th>{rolesOnly ? c.role : c.displayName}</th><th>{rolesOnly ? "Permissions" : c.language}</th><th>{c.active}</th></tr></thead><tbody>{items.map((item) => rolesOnly ? <tr key={item.id}><td><code>{(item as RoleView).code}</code></td><td>{locale === "ar-DZ" ? (item as RoleView).nameAr : (item as RoleView).nameFr || (item as RoleView).nameAr}</td><td>{(item as RoleView).permissionCodes.length}</td><td>{(item as RoleView).isActive ? "✓" : "—"}</td></tr> : <tr key={item.id}><td><code>{(item as UserView).username}</code></td><td>{(item as UserView).displayName}</td><td>{(item as UserView).preferredLanguage}</td><td>{(item as UserView).isActive ? "✓" : "—"}</td></tr>)}</tbody></table></StateBoundary></LedgerPage>; }

function StateBoundary({ state, onRetry, children }: { state: LoadState; onRetry: () => void; children: ReactNode }) { const { locale } = useI18n(); const c = COPY[locale]; if (state === "loading") return <div className="p5-state" aria-busy="true">{c.loading}</div>; if (state === "denied") return <div className="p5-state"><strong>{c.denied}</strong></div>; if (state === "error") return <div className="p5-state"><strong>{c.error}</strong><Button onClick={onRetry}>{c.retry}</Button></div>; if (state === "empty") return <div className="p5-state">{c.empty}</div>; return <>{children}</>; }
function LedgerPage({ title, actions, children }: { title: string; actions?: ReactNode; children: ReactNode }) { return <section className="p5-ledger" aria-labelledby="p5-title"><header className="p5-ledger__header"><div><p className="p5-eyebrow">POSMAN / PHASE 05</p><h1 id="p5-title">{title}</h1></div>{actions}</header><div className="p5-ledger__body">{children}</div></section>; }

const sections: Array<{ label: keyof Copy; items: Array<[ScreenId, keyof Copy]> }> = [
  { label: "administration", items: [["company", "company"], ["fiscal", "fiscal"], ["pricing", "pricing"], ["sequences", "sequences"], ["users", "users"], ["roles", "roles"]] },
  { label: "reference", items: [["families", "families"], ["units", "units"], ["warehouses", "warehouses"], ["locations", "locations"], ["taxes", "taxes"], ["customers", "customers"], ["suppliers", "suppliers"], ["products", "products"]] },
];
function Workspace({ session, onLogout }: { session: SessionView; onLogout: () => void }) { const { locale, setLocale } = useI18n(); const c = COPY[locale]; const [screen, setScreen] = useState<ScreenId>("company"); const gateway = useMemo(resolvePhase05Gateway, []); const content = screen === "company" ? <CompanyScreen /> : screen === "pricing" ? <CompanyScreen pricingOnly /> : screen === "fiscal" ? <FiscalScreen /> : screen === "sequences" ? <SequenceScreen /> : screen === "users" ? <UsersScreen /> : screen === "roles" ? <UsersScreen rolesOnly /> : screen === "products" ? <ProductsScreen /> : screen === "customers" ? <PartnersScreen supplier={false} /> : screen === "suppliers" ? <PartnersScreen supplier /> : <ReferenceScreen kind={screen as keyof typeof referenceCommands} />; const logout = async () => { if (gateway) await gateway.call("logout"); onLogout(); };
  return <div className="p5-shell"><a className="skip-link" href="#p5-main">Skip</a><header className="p5-topbar"><div><strong>{c.appName}</strong><span>{c.local}</span></div><div className="p5-topbar__actions"><span>{session.displayName}</span><button onClick={() => setLocale(locale === "ar-DZ" ? "fr-DZ" : "ar-DZ")}>{locale === "ar-DZ" ? "Français" : "العربية"}</button><button onClick={() => void logout()}>{c.logout}</button></div></header><aside className="p5-nav" aria-label="Phase 05"><div className="p5-brand">PM<span>05</span></div>{sections.map((section) => <section key={section.label}><h2>{c[section.label]}</h2>{section.items.map(([id, label]) => <button className={screen === id ? "is-active" : ""} onClick={() => setScreen(id)} key={id}>{c[label]}</button>)}</section>)}</aside><main id="p5-main" className="p5-main">{!gateway ? <Notice tone="warning">{c.preview}</Notice> : null}{content}</main></div>;
}

export function Phase05App() {
  const gateway = useMemo(resolvePhase05Gateway, []); const [boot, setBoot] = useState<"loading" | "setup" | "login" | "app" | "error">("loading"); const [status, setStatus] = useState<SetupStatus>(); const [session, setSession] = useState<SessionView>();
  const initialize = useCallback(async () => { try { if (!gateway) { setBoot("login"); return; } const setup = await gateway.getSetupStatus(); setStatus(setup); if (setup.setupRequired) { setBoot("setup"); return; } try { const current = await gateway.getCurrentSession(); setSession(current); setBoot("app"); } catch { setBoot("login"); } } catch { setBoot("error"); } }, [gateway]); useEffect(() => { void initialize(); }, [initialize]);
  if (boot === "loading") return <div className="p5-boot" aria-busy="true">POSMAN · 0005</div>;
  if (boot === "error") return <AuthFrame><Notice tone="error">Local runtime unavailable.</Notice><Button onClick={initialize}>Retry</Button></AuthFrame>;
  if (boot === "setup" && status) return <SetupFlow status={status} onDone={() => setBoot("login")} />;
  if (boot === "login") return <LoginFlow onLogin={(value) => { setSession(value); setBoot("app"); }} />;
  return session ? <Workspace session={session} onLogout={() => { setSession(undefined); setBoot("login"); }} /> : null;
}
