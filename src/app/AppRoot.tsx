import { useEffect, useState } from "react";
import { I18nProvider, useI18n } from "../i18n/I18nProvider";
import { Phase05App } from "../features/phase05/Phase05App";
import { Phase06Workspace } from "../features/phase06/Phase06Workspace";
import { Phase07Workspace } from "../features/phase07/Phase07Workspace";
import "../styles/tokens.css";
import "../styles/ui-foundation.css";

function DesktopWorkspaces() {
  const { locale } = useI18n();
  const resolveWorkspace = () => window.location.hash === "#phase07" ? "phase07" : window.location.hash === "#phase06" ? "phase06" : "phase05";
  const [workspace, setWorkspace] = useState<"phase05" | "phase06" | "phase07">(resolveWorkspace);

  useEffect(() => {
    const onHashChange = () => setWorkspace(resolveWorkspace());
    window.addEventListener("hashchange", onHashChange);
    return () => window.removeEventListener("hashchange", onHashChange);
  }, []);

  const navigate = (target: "phase05" | "phase06" | "phase07") => {
    window.location.hash = target;
    setWorkspace(target);
  };

  return <>
    <nav className="workspace-switcher" aria-label={locale === "ar-DZ" ? "مساحات POSMAN" : "Espaces POSMAN"}>
      <button type="button" aria-current={workspace === "phase05" ? "page" : undefined} onClick={() => navigate("phase05")}>
        {locale === "ar-DZ" ? "الإدارة والبيانات المرجعية" : "Administration et références"}
      </button>
      <button type="button" aria-current={workspace === "phase06" ? "page" : undefined} onClick={() => navigate("phase06")}>
        {locale === "ar-DZ" ? "المخزون والمشتريات" : "Stock et achats"}
      </button>
      <button type="button" aria-current={workspace === "phase07" ? "page" : undefined} onClick={() => navigate("phase07")}>
        {locale === "ar-DZ" ? "المبيعات" : "Ventes"}
      </button>
    </nav>
    {workspace === "phase07" ? <Phase07Workspace /> : workspace === "phase06" ? <Phase06Workspace /> : <Phase05App />}
  </>;
}

export function AppRoot() {
  return <I18nProvider><DesktopWorkspaces /></I18nProvider>;
}
