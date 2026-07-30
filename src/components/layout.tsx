import type { ReactNode } from "react";
import type { MessageKey } from "../i18n/dictionaries";
import { useI18n } from "../i18n/I18nProvider";
import { Button } from "./primitives";

export type WorkspaceId = "today" | "sales" | "purchases" | "inventory" | "accounting" | "reports" | "admin";
export type GalleryView = "today" | "invoice" | "sales-cycle" | "products" | "opening-stock" | "states" | "placeholder";

export interface WorkspaceItem {
  id: WorkspaceId;
  labelKey: MessageKey;
  index: string;
}

export const workspaceItems: readonly WorkspaceItem[] = [
  { id: "today", labelKey: "nav.today", index: "01" },
  { id: "sales", labelKey: "nav.sales", index: "02" },
  { id: "purchases", labelKey: "nav.purchases", index: "03" },
  { id: "inventory", labelKey: "nav.inventory", index: "04" },
  { id: "accounting", labelKey: "nav.accounting", index: "05" },
  { id: "reports", labelKey: "nav.reports", index: "06" },
  { id: "admin", labelKey: "nav.admin", index: "07" },
] as const;

const createLabels: Record<WorkspaceId, MessageKey> = {
  today: "command.createSale",
  sales: "command.createSale",
  purchases: "command.createPurchase",
  inventory: "command.createStock",
  accounting: "command.createEntry",
  reports: "command.createReport",
  admin: "command.createAdmin",
};

export function CommandBar({
  workspace,
  onDemoAction,
  runtimeStatus,
}: {
  workspace: WorkspaceId;
  onDemoAction: () => void;
  runtimeStatus: ReactNode;
}) {
  const { locale, setLocale, t } = useI18n();
  const workspaceItem = workspaceItems.find((item) => item.id === workspace) ?? workspaceItems[0];
  return (
    <header className="command-bar" aria-label={t("app.productName")}>
      <div className="command-bar__identity">
        <strong>PM</strong>
        <div>
          <span>{t("app.productName")}</span>
          <small>{t("app.workspacePrefix")} {t(workspaceItem.labelKey)}</small>
        </div>
      </div>
      <div className="command-bar__search">
        <label className="sr-only" htmlFor="global-search">{t("command.searchLabel")}</label>
        <svg
          className="command-bar__search-mark"
          viewBox="0 0 20 20"
          aria-hidden="true"
          focusable="false"
        >
          <circle cx="8.5" cy="8.5" r="5.25" fill="none" stroke="currentColor" strokeWidth="1.5" />
          <path d="m12.4 12.4 4.1 4.1" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
        </svg>
        <input
          id="global-search"
          type="search"
          placeholder={t("command.searchPlaceholder")}
          aria-describedby="global-search-hint"
        />
        <small id="global-search-hint" className="sr-only">{t("command.searchHint")}</small>
        <kbd>Ctrl K</kbd>
      </div>
      <div className="command-bar__actions">
        <Button variant="primary" type="button" onClick={onDemoAction}>
          <span aria-hidden="true">＋</span> {t(createLabels[workspace])}
        </Button>
        <button
          className="language-switch"
          type="button"
          onClick={() => setLocale(locale === "ar-DZ" ? "fr-DZ" : "ar-DZ")}
          aria-label={t("command.language")}
          data-testid="language-switch"
        >
          <span aria-hidden="true">文</span>
          {locale === "ar-DZ" ? t("command.switchToFrench") : t("command.switchToArabic")}
        </button>
        <div className="command-bar__company runtime-status-host">
          <span className="command-bar__company-name">{t("app.demoCompany")}</span>
          {runtimeStatus}
        </div>
      </div>
    </header>
  );
}

export function WorkspaceRail({
  active,
  onSelect,
}: {
  active: WorkspaceId;
  onSelect: (workspace: WorkspaceId) => void;
}) {
  const { t } = useI18n();
  return (
    <nav className="workspace-rail" aria-label={t("nav.label")}>
      <div className="workspace-rail__spine" aria-hidden="true" />
      <ol>
        {workspaceItems.map((item) => {
          const isActive = item.id === active;
          return (
            <li key={item.id}>
              <button
                type="button"
                className={isActive ? "is-active" : undefined}
                aria-current={isActive ? "page" : undefined}
                onClick={() => onSelect(item.id)}
                data-workspace={item.id}
              >
                <span className="workspace-rail__index">{item.index}</span>
                <span className="workspace-rail__label">{t(item.labelKey)}</span>
                {isActive ? <span className="sr-only"> — {t("nav.active")}</span> : null}
              </button>
            </li>
          );
        })}
      </ol>
    </nav>
  );
}

export interface SubnavItem {
  id: GalleryView;
  labelKey: MessageKey;
}

const workspaceSubnav: Partial<Record<WorkspaceId, readonly SubnavItem[]>> = {
  today: [{ id: "today", labelKey: "subnav.today" }],
  sales: [
    { id: "invoice", labelKey: "subnav.invoice" },
    { id: "sales-cycle", labelKey: "subnav.salesCycle" },
  ],
  inventory: [
    { id: "products", labelKey: "subnav.products" },
    { id: "opening-stock", labelKey: "subnav.openingStock" },
  ],
  admin: [{ id: "states", labelKey: "subnav.states" }],
};

export function firstViewForWorkspace(workspace: WorkspaceId): GalleryView {
  return workspaceSubnav[workspace]?.[0]?.id ?? "placeholder";
}

export function WorkspaceHeader({
  workspace,
  view,
  title,
  subtitle,
  onViewSelect,
}: {
  workspace: WorkspaceId;
  view: GalleryView;
  title: string;
  subtitle: string;
  onViewSelect: (view: GalleryView) => void;
}) {
  const { t } = useI18n();
  const items = workspaceSubnav[workspace] ?? [];
  return (
    <header className="workspace-header">
      <div className="workspace-header__title">
        <span className="eyebrow">{t("app.fixtureBadge")}</span>
        <h1 id={view === "products" ? "products-heading" : undefined}>{title}</h1>
        <p>{subtitle}</p>
      </div>
      {items.length > 1 ? (
        <nav className="workspace-tabs" aria-label={t("subnav.label")}>
          {items.map((item) => (
            <button
              type="button"
              key={item.id}
              className={view === item.id ? "is-active" : undefined}
              aria-current={view === item.id ? "page" : undefined}
              onClick={() => onViewSelect(item.id)}
              data-view={item.id}
            >
              {t(item.labelKey)}
            </button>
          ))}
        </nav>
      ) : null}
    </header>
  );
}

export function Workspace({ children }: { children: ReactNode }) {
  return <main id="main-content" className="workspace" tabIndex={-1}>{children}</main>;
}
