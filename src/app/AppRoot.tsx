import { useEffect, useState } from "react";
import { I18nProvider, useI18n } from "../i18n/I18nProvider";
import { Phase05App } from "../features/phase05/Phase05App";
import { Phase06Workspace } from "../features/phase06/Phase06Workspace";
import { Phase07Workspace } from "../features/phase07/Phase07Workspace";
import { Phase08Workspace } from "../features/phase08/Phase08Workspace";
import "../styles/tokens.css";
import "../styles/ui-foundation.css";

type Workspace = "phase05" | "phase06" | "phase07" | "phase08";

function resolveWorkspace(): Workspace {
  if (window.location.hash === "#phase08") return "phase08";
  if (window.location.hash === "#phase07") return "phase07";
  if (window.location.hash === "#phase06") return "phase06";
  return "phase05";
}

function DesktopWorkspaces() {
  const { locale } = useI18n();
  const [workspace, setWorkspace] = useState<Workspace>(resolveWorkspace);

  useEffect(() => {
    const onHashChange = () => setWorkspace(resolveWorkspace());
    window.addEventListener("hashchange", onHashChange);
    return () => window.removeEventListener("hashchange", onHashChange);
  }, []);

  const navigate = (target: Workspace) => {
    window.location.hash = target;
    setWorkspace(target);
  };

  const labels = locale === "ar-DZ"
    ? { phase05: "الإدارة والبيانات المرجعية", phase06: "المخزون والمشتريات", phase07: "المبيعات", phase08: "المحاسبة والدفعات" }
    : { phase05: "Administration et références", phase06: "Stock et achats", phase07: "Ventes", phase08: "Comptabilité et paiements" };

  return <>
    <nav className="workspace-switcher" aria-label={locale === "ar-DZ" ? "مساحات POSMAN" : "Espaces POSMAN"}>
      {(["phase05", "phase06", "phase07", "phase08"] as const).map((item) => (
        <button type="button" key={item} aria-current={workspace === item ? "page" : undefined} onClick={() => navigate(item)}>
          {labels[item]}
        </button>
      ))}
    </nav>
    {workspace === "phase08" ? <Phase08Workspace /> : workspace === "phase07" ? <Phase07Workspace /> : workspace === "phase06" ? <Phase06Workspace /> : <Phase05App />}
  </>;
}

export function AppRoot() {
  return <I18nProvider><DesktopWorkspaces /></I18nProvider>;
}
