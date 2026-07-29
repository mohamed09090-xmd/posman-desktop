import { useMemo, useState } from "react";
import {
  CommandBar,
  Workspace,
  WorkspaceHeader,
  WorkspaceRail,
  firstViewForWorkspace,
  type GalleryView,
  type WorkspaceId,
} from "../components/layout";
import { InlineNotice } from "../components/primitives";
import { I18nProvider, useI18n } from "../i18n/I18nProvider";
import {
  InvoiceScreen,
  OpeningStockScreen,
  PlaceholderScreen,
  ProductsScreen,
  SalesCycleScreen,
  StatesGalleryScreen,
  TodayScreen,
} from "../features/ui-gallery/screens";
import "../styles/tokens.css";
import "../styles/ui-foundation.css";

const workspaceCopy = {
  today: ["workspace.today.title", "workspace.today.subtitle"],
  sales: ["workspace.sales.title", "workspace.sales.subtitle"],
  purchases: ["workspace.purchases.title", "workspace.purchases.subtitle"],
  inventory: ["workspace.inventory.title", "workspace.inventory.subtitle"],
  accounting: ["workspace.accounting.title", "workspace.accounting.subtitle"],
  reports: ["workspace.reports.title", "workspace.reports.subtitle"],
  admin: ["workspace.admin.title", "workspace.admin.subtitle"],
} as const;

function AppGallery() {
  const { t } = useI18n();
  const [workspace, setWorkspace] = useState<WorkspaceId>("today");
  const [view, setView] = useState<GalleryView>("today");
  const [feedback, setFeedback] = useState(false);

  const [titleKey, subtitleKey] = workspaceCopy[workspace];
  const screen = useMemo(() => {
    const showFeedback = () => setFeedback(true);
    switch (view) {
      case "today":
        return <TodayScreen onNavigate={(destination) => {
          if (destination === "invoice") {
            setWorkspace("sales");
            setView("invoice");
          } else {
            setWorkspace("inventory");
            setView(destination);
          }
        }} />;
      case "invoice":
        return <InvoiceScreen onDemoAction={showFeedback} />;
      case "sales-cycle":
        return <SalesCycleScreen />;
      case "products":
        return <ProductsScreen />;
      case "opening-stock":
        return <OpeningStockScreen onDemoAction={showFeedback} />;
      case "states":
        return <StatesGalleryScreen onDemoAction={showFeedback} />;
      case "placeholder":
        return <PlaceholderScreen onBack={() => {
          setWorkspace("today");
          setView("today");
        }} />;
    }
  }, [view]);

  const selectWorkspace = (nextWorkspace: WorkspaceId) => {
    setWorkspace(nextWorkspace);
    setView(firstViewForWorkspace(nextWorkspace));
    setFeedback(false);
  };

  return (
    <div className="app-frame">
      <a className="skip-link" href="#main-content">{t("app.skipToContent")}</a>
      <CommandBar workspace={workspace} onDemoAction={() => setFeedback(true)} />
      <WorkspaceRail active={workspace} onSelect={selectWorkspace} />
      <Workspace>
        <WorkspaceHeader
          workspace={workspace}
          view={view}
          title={t(titleKey)}
          subtitle={t(subtitleKey)}
          onViewSelect={(nextView) => {
            setView(nextView);
            setFeedback(false);
          }}
        />
        <div className="workspace__content">
          {feedback ? (
            <InlineNotice title={t("notice.successTitle")} tone="success" live>
              <p>{t("command.demoFeedback")}</p>
            </InlineNotice>
          ) : null}
          {screen}
        </div>
        <p className="fixture-boundary">{t("app.galleryDescription")}</p>
      </Workspace>
    </div>
  );
}

export function AppRoot() {
  return <I18nProvider><AppGallery /></I18nProvider>;
}
