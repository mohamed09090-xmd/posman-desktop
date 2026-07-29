import { useMemo, useState } from "react";
import {
  ActionDock,
  DataGrid,
  DetailDrawer,
  DocumentCanvas,
  ProcessStrip,
  StatusStamp,
  type DataGridColumn,
  type ProcessStep,
  type StatusTone,
} from "../../components/operational";
import {
  Button,
  EmptyState,
  Field,
  InlineNotice,
  Input,
  LoadingState,
  Select,
} from "../../components/primitives";
import { useI18n } from "../../i18n/I18nProvider";
import {
  filterProducts,
  invoiceLineFixtures,
  invoiceTotals,
  openingStockFixtures,
  openingStockTotals,
  productFixtures,
  todayOperationFixtures,
  type InvoiceLineFixture,
  type OpeningStockLineFixture,
  type ProductFixture,
  type TodayOperationFixture,
} from "./fixtures";

function localized(locale: "ar-DZ" | "fr-DZ", ar: string, fr: string): string {
  return locale === "ar-DZ" ? ar : fr;
}

function StatusLabel({ tone }: { tone: StatusTone }) {
  const { t } = useI18n();
  return <StatusStamp tone={tone}>{t(`status.${tone}` as Parameters<typeof t>[0])}</StatusStamp>;
}

export function TodayScreen({ onNavigate }: { onNavigate: (destination: "invoice" | "products" | "opening-stock") => void }) {
  const { locale, t } = useI18n();
  const columns: readonly DataGridColumn<TodayOperationFixture>[] = [
    {
      key: "task",
      header: t("today.column.task"),
      render: (row) => <strong>{localized(locale, row.taskAr, row.taskFr)}</strong>,
    },
    { key: "reference", header: t("today.column.reference"), render: (row) => <code>{row.reference}</code> },
    { key: "owner", header: t("today.column.owner"), render: (row) => localized(locale, row.ownerAr, row.ownerFr) },
    { key: "time", header: t("today.column.time"), render: (row) => <time>{row.time}</time>, align: "end" },
    { key: "status", header: t("today.column.status"), render: (row) => <StatusLabel tone={row.status} />, align: "end" },
  ];

  const groups = [
    ["intervention", "today.intervention"],
    ["delivery", "today.readyDelivery"],
    ["invoice", "today.readyInvoice"],
    ["unposted", "today.unposted"],
    ["stock", "today.lowStock"],
    ["recent", "today.recent"],
  ] as const;

  return (
    <div className="today-ledger" data-testid="today-screen">
      <section className="shortcut-row" aria-labelledby="shortcut-title">
        <div>
          <span className="section-number">00</span>
          <h2 id="shortcut-title">{t("today.shortcuts")}</h2>
        </div>
        <div className="shortcut-row__actions">
          <Button variant="primary" onClick={() => onNavigate("invoice")}>{t("today.sell")}</Button>
          <Button onClick={() => onNavigate("opening-stock")}>{t("today.buy")}</Button>
          <Button variant="quiet" onClick={() => onNavigate("products")}>{t("today.count")}</Button>
        </div>
      </section>

      {groups.map(([group, labelKey], index) => {
        const rows = todayOperationFixtures.filter((operation) => operation.group === group);
        return (
          <section className="ledger-section" key={group} aria-labelledby={`today-${group}`}>
            <header className="ledger-section__header">
              <span className="section-number">{String(index + 1).padStart(2, "0")}</span>
              <h2 id={`today-${group}`}>{t(labelKey)}</h2>
              <span className="ledger-section__count">{rows.length}</span>
            </header>
            <DataGrid caption={t(labelKey)} columns={columns} rows={rows} />
          </section>
        );
      })}
    </div>
  );
}

export function ProductsScreen() {
  const { locale, t, formatMoney } = useI18n();
  const [query, setQuery] = useState("");
  const [family, setFamily] = useState("all");
  const [selected, setSelected] = useState<ProductFixture | null>(null);

  const families = useMemo(() => Array.from(new Set(productFixtures.map((product) =>
    localized(locale, product.familyAr, product.familyFr),
  ))), [locale]);
  const filtered = useMemo(() => filterProducts(productFixtures, query, family, locale), [query, family, locale]);

  const columns: readonly DataGridColumn<ProductFixture>[] = [
    {
      key: "code",
      header: t("products.column.code"),
      render: (row) => <div className="code-stack"><code>{row.code}</code><small>{row.barcode}</small></div>,
    },
    { key: "name", header: t("products.column.name"), render: (row) => <strong>{localized(locale, row.nameAr, row.nameFr)}</strong> },
    { key: "family", header: t("products.column.family"), render: (row) => localized(locale, row.familyAr, row.familyFr) },
    { key: "price", header: t("products.column.price"), render: (row) => formatMoney(row.salePriceMinor), align: "end" },
    { key: "onHand", header: t("products.column.onHand"), render: (row) => row.onHand, align: "end" },
    { key: "reserved", header: t("products.column.reserved"), render: (row) => row.reserved, align: "end" },
    { key: "available", header: t("products.column.available"), render: (row) => row.onHand - row.reserved, align: "end" },
    { key: "minimum", header: t("products.column.minimum"), render: (row) => row.minimum, align: "end" },
    { key: "status", header: t("products.column.status"), render: (row) => <StatusLabel tone={row.status} />, align: "end" },
  ];

  return (
    <div className={`product-browser ${selected ? "has-drawer" : ""}`} data-testid="products-screen">
      <section className="product-browser__list" aria-labelledby="products-heading">
        <div className="filter-bar">
          <Field id="product-search" label={t("products.search")}>
            <Input
              id="product-search"
              type="search"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder={t("products.searchPlaceholder")}
              data-testid="product-search"
            />
          </Field>
          <Field id="product-family" label={t("products.family")}>
            <Select id="product-family" value={family} onChange={(event) => setFamily(event.target.value)}>
              <option value="all">{t("products.allFamilies")}</option>
              {families.map((item) => <option value={item} key={item}>{item}</option>)}
            </Select>
          </Field>
          <div className="filter-bar__result" aria-live="polite">
            <strong>{filtered.length}</strong>
            <span>{t("table.rows")}</span>
          </div>
        </div>
        <DataGrid
          caption={t("products.title")}
          columns={columns}
          rows={filtered}
          selectedId={selected?.id}
          onRowSelect={setSelected}
          testId="product-grid"
          empty={<EmptyState title={t("products.emptyTitle")}><p>{t("products.emptyBody")}</p></EmptyState>}
        />
      </section>

      {selected ? (
        <DetailDrawer title={t("drawer.title")} closeLabel={t("drawer.close")} onClose={() => setSelected(null)}>
          <section className="drawer-section">
            <h3>{t("drawer.identity")}</h3>
            <dl>
              <div><dt>{t("drawer.code")}</dt><dd><code>{selected.code}</code></dd></div>
              <div><dt>{t("drawer.barcode")}</dt><dd>{selected.barcode}</dd></div>
              <div><dt>{t("products.column.name")}</dt><dd>{localized(locale, selected.nameAr, selected.nameFr)}</dd></div>
            </dl>
          </section>
          <section className="drawer-section">
            <h3>{t("drawer.pricing")}</h3>
            <dl><div><dt>{t("drawer.salePrice")}</dt><dd>{formatMoney(selected.salePriceMinor)}</dd></div></dl>
          </section>
          <section className="drawer-section">
            <h3>{t("drawer.tax")}</h3>
            <dl><div><dt>{t("drawer.taxRate")}</dt><dd>{selected.taxRateBasisPoints / 100}%</dd></div></dl>
          </section>
          <section className="drawer-section">
            <h3>{t("drawer.stock")}</h3>
            <dl>
              <div><dt>{t("drawer.warehouse")}</dt><dd>{localized(locale, selected.warehouseAr, selected.warehouseFr)}</dd></div>
              <div><dt>{t("products.column.onHand")}</dt><dd>{selected.onHand}</dd></div>
              <div><dt>{t("products.column.reserved")}</dt><dd>{selected.reserved}</dd></div>
              <div><dt>{t("products.column.available")}</dt><dd>{selected.onHand - selected.reserved}</dd></div>
            </dl>
          </section>
          <section className="drawer-section">
            <h3>{t("drawer.lastMovement")}</h3>
            <p>{localized(locale, selected.lastMovementAr, selected.lastMovementFr)}</p>
          </section>
          <Button type="button" variant="primary" onClick={() => undefined}>{t("command.demoFeedback")}</Button>
        </DetailDrawer>
      ) : null}
    </div>
  );
}

export function OpeningStockScreen({ onDemoAction }: { onDemoAction: () => void }) {
  const { locale, t, formatMoney, formatDate, formatNumber } = useI18n();
  const columns: readonly DataGridColumn<OpeningStockLineFixture>[] = [
    { key: "product", header: t("opening.column.product"), render: (row) => <strong>{localized(locale, row.productAr, row.productFr)}</strong> },
    { key: "quantity", header: t("opening.column.quantity"), render: (row) => <Input aria-label={`${t("opening.column.quantity")} ${localized(locale, row.productAr, row.productFr)}`} type="number" defaultValue={row.quantity} />, align: "end" },
    { key: "unitCost", header: t("opening.column.unitCost"), render: (row) => <Input aria-label={`${t("opening.column.unitCost")} ${localized(locale, row.productAr, row.productFr)}`} type="text" defaultValue={formatMoney(row.unitCostMinor)} />, align: "end" },
    { key: "total", header: t("opening.column.total"), render: (row) => <strong>{formatMoney(row.totalMinor)}</strong>, align: "end" },
  ];

  return (
    <DocumentCanvas label={t("opening.title")}>
      <header className="document-heading">
        <div>
          <span className="document-kind">OPENING / 2026</span>
          <h2>{t("opening.title")}</h2>
          <p>{t("opening.subtitle")}</p>
        </div>
        <StatusLabel tone="draft" />
      </header>
      <div className="document-fields document-fields--three">
        <Field id="opening-warehouse" label={t("opening.warehouse")}>
          <Select id="opening-warehouse" defaultValue="main"><option value="main">{localized(locale, "المستودع الرئيسي", "Dépôt principal")}</option></Select>
        </Field>
        <Field id="opening-date" label={t("opening.date")}>
          <Input id="opening-date" type="text" defaultValue={formatDate("2026-07-29")} />
        </Field>
        <Field id="opening-reference" label={t("opening.reference")}>
          <Input id="opening-reference" defaultValue="OPEN-2026-0001" />
        </Field>
      </div>
      <DataGrid caption={t("opening.title")} columns={columns} rows={openingStockFixtures} density="comfortable" />
      <div className="document-summary">
        <dl>
          <div><dt>{t("opening.totalQuantity")}</dt><dd>{formatNumber(openingStockTotals.quantity)}</dd></div>
          <div className="document-summary__total"><dt>{t("opening.totalCost")}</dt><dd>{formatMoney(openingStockTotals.costMinor)}</dd></div>
        </dl>
      </div>
      <InlineNotice title={t("opening.warningTitle")} tone="warning"><p>{t("opening.warningBody")}</p></InlineNotice>
      <ActionDock label={t("today.shortcuts")}>
        <span className="action-dock__context">OPEN-2026-0001 · {t("app.fixtureBadge")}</span>
        <div>
          <Button variant="quiet" onClick={onDemoAction}>{t("action.close")}</Button>
          <Button onClick={onDemoAction}>{t("action.validate")}</Button>
          <Button variant="primary" onClick={onDemoAction}>{t("action.saveDraft")}</Button>
        </div>
      </ActionDock>
    </DocumentCanvas>
  );
}

export function InvoiceScreen({ onDemoAction }: { onDemoAction: () => void }) {
  const { locale, t, formatMoney, formatDate } = useI18n();
  const columns: readonly DataGridColumn<InvoiceLineFixture>[] = [
    { key: "product", header: t("invoice.column.product"), render: (row) => <strong>{localized(locale, row.productAr, row.productFr)}</strong> },
    { key: "quantity", header: t("invoice.column.quantity"), render: (row) => row.quantity, align: "end" },
    { key: "unitPrice", header: t("invoice.column.unitPrice"), render: (row) => formatMoney(row.unitPriceMinor), align: "end" },
    { key: "discount", header: t("invoice.column.discount"), render: (row) => row.lineDiscountMinor ? `− ${formatMoney(row.lineDiscountMinor)}` : "—", align: "end" },
    { key: "net", header: t("invoice.column.net"), render: (row) => formatMoney(row.netHtMinor), align: "end" },
    { key: "tax", header: t("invoice.column.tax"), render: (row) => formatMoney(row.taxMinor), align: "end" },
    { key: "total", header: t("invoice.column.total"), render: (row) => <strong>{formatMoney(row.totalTtcMinor)}</strong>, align: "end" },
  ];

  return (
    <DocumentCanvas label={t("invoice.title")}>
      <header className="document-heading">
        <div>
          <span className="document-kind">SALE / FAC-2026-0072</span>
          <h2>{t("invoice.title")}</h2>
          <p>{t("invoice.subtitle")}</p>
        </div>
        <StatusLabel tone="draft" />
      </header>
      <div className="document-fields document-fields--four">
        <Field id="invoice-number" label={t("invoice.number")}><Input id="invoice-number" defaultValue="FAC-2026-0072" readOnly /></Field>
        <Field id="invoice-customer" label={t("invoice.customer")}><Input id="invoice-customer" defaultValue={localized(locale, "شركة النور للتوزيع", "SARL Ennour Distribution")} /></Field>
        <Field id="invoice-date" label={t("invoice.date")}><Input id="invoice-date" defaultValue={formatDate("2026-07-29")} /></Field>
        <Field id="invoice-due" label={t("invoice.dueDate")}><Input id="invoice-due" defaultValue={formatDate("2026-08-28")} /></Field>
      </div>
      <DataGrid caption={t("invoice.title")} columns={columns} rows={invoiceLineFixtures} density="comfortable" testId="invoice-grid" />
      <div className="document-summary">
        <dl>
          <div><dt>{t("invoice.subtotal")}</dt><dd>{formatMoney(invoiceTotals.subtotalMinor)}</dd></div>
          <div><dt>{t("invoice.lineDiscount")}</dt><dd>− {formatMoney(invoiceTotals.lineDiscountMinor)}</dd></div>
          <div><dt>{t("invoice.generalDiscount")}</dt><dd>− {formatMoney(invoiceTotals.generalDiscountMinor)}</dd></div>
          <div><dt>{t("invoice.netHt")}</dt><dd>{formatMoney(invoiceTotals.netHtMinor)}</dd></div>
          <div><dt>{t("invoice.tax")}</dt><dd>{formatMoney(invoiceTotals.taxMinor)}</dd></div>
          <div className="document-summary__total"><dt>{t("invoice.ttc")}</dt><dd>{formatMoney(invoiceTotals.totalTtcMinor)}</dd></div>
        </dl>
      </div>
      <InlineNotice title={t("invoice.validationTitle")} tone="info"><p>{t("invoice.validationBody")}</p></InlineNotice>
      <ActionDock label={t("today.shortcuts")}>
        <span className="action-dock__context">FAC-2026-0072 · {t("app.fixtureBadge")}</span>
        <div>
          <Button variant="quiet" onClick={onDemoAction}>{t("action.close")}</Button>
          <Button onClick={onDemoAction}>{t("action.validate")}</Button>
          <Button variant="primary" onClick={onDemoAction}>{t("action.saveDraft")}</Button>
          <Button variant="danger" disabled title={t("command.demoFeedback")}>{t("action.post")}</Button>
        </div>
      </ActionDock>
    </DocumentCanvas>
  );
}

export function SalesCycleScreen() {
  const { t } = useI18n();
  const steps: readonly ProcessStep[] = [
    { id: "order", label: t("cycle.order"), reference: "CMD-2026-0114", state: "completed", stateLabel: t("cycle.completed") },
    { id: "delivery", label: t("cycle.delivery"), reference: "BL-2026-0048", state: "completed", stateLabel: t("cycle.partial") },
    { id: "invoice", label: t("cycle.invoice"), reference: "FAC-2026-0072", state: "current", stateLabel: t("cycle.current") },
    { id: "accounting", label: t("cycle.accounting"), reference: "—", state: "pending", stateLabel: t("cycle.pending") },
  ];
  return (
    <DocumentCanvas label={t("cycle.title")}>
      <header className="document-heading">
        <div><span className="document-kind">TRACE / CMD-2026-0114</span><h2>{t("cycle.title")}</h2><p>{t("cycle.subtitle")}</p></div>
        <StatusLabel tone="pending" />
      </header>
      <ProcessStrip steps={steps} label={t("cycle.title")} />
      <section className="cycle-ledger" aria-labelledby="cycle-trace-title">
        <header><span className="section-number">01</span><h3 id="cycle-trace-title">{t("cycle.traceTitle")}</h3></header>
        <p>{t("cycle.traceBody")}</p>
        <div className="partial-delivery-proof">
          <span>8</span><b aria-hidden="true">＋</b><span>12</span><b aria-hidden="true">＝</b><strong>20</strong>
        </div>
      </section>
    </DocumentCanvas>
  );
}

export function StatesGalleryScreen({ onDemoAction }: { onDemoAction: () => void }) {
  const { t } = useI18n();
  return (
    <div className="states-gallery" data-testid="states-gallery">
      <section aria-labelledby="states-buttons"><header><span className="section-number">01</span><h2 id="states-buttons">{t("states.buttons")}</h2></header><div className="component-row"><Button variant="primary" onClick={onDemoAction}>{t("button.primary")}</Button><Button onClick={onDemoAction}>{t("button.secondary")}</Button><Button variant="quiet" onClick={onDemoAction}>{t("button.quiet")}</Button><Button variant="danger" onClick={onDemoAction}>{t("button.danger")}</Button><Button loading>{t("button.loading")}</Button><Button disabled>{t("button.disabled")}</Button></div></section>
      <section aria-labelledby="states-fields"><header><span className="section-number">02</span><h2 id="states-fields">{t("states.fields")}</h2></header><div className="component-grid"><Field id="state-name" label={t("field.name")}><Input id="state-name" placeholder={t("field.namePlaceholder")} /></Field><Field id="state-family" label={t("field.family")}><Select id="state-family"><option>{t("products.allFamilies")}</option></Select></Field><Field id="state-error" label={t("field.name")} error={t("field.error")} required><Input id="state-error" aria-invalid="true" aria-describedby="state-error-error" /></Field></div></section>
      <section aria-labelledby="states-stamps"><header><span className="section-number">03</span><h2 id="states-stamps">{t("states.stamps")}</h2></header><div className="component-row"><StatusLabel tone="confirmed" /><StatusLabel tone="pending" /><StatusLabel tone="shortage" /><StatusLabel tone="draft" /><StatusLabel tone="posted" /></div></section>
      <section aria-labelledby="states-notices"><header><span className="section-number">04</span><h2 id="states-notices">{t("states.notices")}</h2></header><div className="notice-stack"><InlineNotice title={t("notice.infoTitle")}><p>{t("notice.infoBody")}</p></InlineNotice><InlineNotice title={t("notice.successTitle")} tone="success"><p>{t("notice.successBody")}</p></InlineNotice><InlineNotice title={t("notice.errorTitle")} tone="error"><p>{t("notice.errorBody")}</p></InlineNotice></div></section>
      <section aria-labelledby="states-empty"><header><span className="section-number">05</span><h2 id="states-empty">{t("states.emptyLoading")}</h2></header><div className="component-grid"><EmptyState title={t("empty.title")}><p>{t("empty.body")}</p></EmptyState><LoadingState>{t("loading.label")}</LoadingState></div></section>
      <section aria-labelledby="states-density"><header><span className="section-number">06</span><h2 id="states-density">{t("states.density")}</h2></header><div className="density-samples"><div><strong>{t("table.dense")}</strong><span>01 · HUI-001 · 36 · 8 · 28</span></div><div className="is-comfortable"><strong>{t("table.comfortable")}</strong><span>02 · ALI-014 · 14 · 6 · 8</span></div></div></section>
    </div>
  );
}

export function PlaceholderScreen({ onBack }: { onBack: () => void }) {
  const { t } = useI18n();
  return (
    <div className="placeholder-screen">
      <EmptyState title={t("generic.notImplementedTitle")}><p>{t("generic.notImplementedBody")}</p><Button variant="primary" onClick={onBack}>{t("generic.backToToday")}</Button></EmptyState>
    </div>
  );
}
