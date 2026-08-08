import { useCallback, useEffect, useMemo, useState } from "react";
import { I18nProvider, useI18n } from "../i18n/I18nProvider";
import { Phase05App } from "../features/phase05/Phase05App";
import { Phase06Workspace } from "../features/phase06/Phase06Workspace";
import { Phase07Workspace } from "../features/phase07/Phase07Workspace";
import { Phase08Workspace } from "../features/phase08/Phase08Workspace";
import { Phase09Workspace } from "../features/phase09/Phase09Workspace";
import {
  resolvePhase05Gateway,
  type SessionView,
} from "../platform/tauri/phase05";
import "../styles/tokens.css";
import "../styles/ui-foundation.css";

type Workspace = "phase05" | "phase06" | "phase07" | "phase08" | "phase09";

function resolveWorkspace(): Workspace {
  if (window.location.hash === "#phase09") return "phase09";
  if (window.location.hash === "#phase08") return "phase08";
  if (window.location.hash === "#phase07") return "phase07";
  if (window.location.hash === "#phase06") return "phase06";
  return "phase05";
}

function Phase09Route() {
  const { locale, setLocale } = useI18n();
  const gateway = useMemo(resolvePhase05Gateway, []);
  const [session, setSession] = useState<SessionView>();
  const [state, setState] = useState<"loading" | "ready" | "error">("loading");

  const load = useCallback(async () => {
    setState("loading");
    if (!gateway) {
      setSession(undefined);
      setState("error");
      return;
    }
    try {
      setSession(await gateway.getCurrentSession());
      setState("ready");
    } catch {
      setSession(undefined);
      setState("error");
    }
  }, [gateway]);

  useEffect(() => {
    void load();
  }, [load]);

  if (state === "loading") {
    return <main className="phase09-workspace phase09-state" role="status" aria-busy="true">
      {locale === "ar-DZ" ? "جارٍ التحقق من الجلسة المحلية…" : "Vérification de la session locale…"}
    </main>;
  }
  if (!session) {
    return <main className="phase09-workspace phase09-state" role="alert">
      <strong>{locale === "ar-DZ" ? "يجب تسجيل الدخول للوصول إلى مساحة الوثائق." : "Connectez-vous pour accéder à l’espace documents."}</strong>
      <button type="button" onClick={() => void load()}>
        {locale === "ar-DZ" ? "إعادة المحاولة" : "Réessayer"}
      </button>
    </main>;
  }
  return <Phase09Workspace
    locale={locale}
    permissions={session.permissions}
    onLocaleChange={setLocale}
    onRestoreCompleted={() => {
      setSession(undefined);
      window.location.hash = "#phase05";
    }}
  />;
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
    ? { phase05: "الإدارة والبيانات المرجعية", phase06: "المخزون والمشتريات", phase07: "المبيعات", phase08: "المحاسبة والدفعات", phase09: "الوثائق والتقارير والنسخ" }
    : { phase05: "Administration et références", phase06: "Stock et achats", phase07: "Ventes", phase08: "Comptabilité et paiements", phase09: "Documents, rapports et sauvegardes" };

  return <>
    <nav className="workspace-switcher" aria-label={locale === "ar-DZ" ? "مساحات POSMAN" : "Espaces POSMAN"}>
      {(["phase05", "phase06", "phase07", "phase08", "phase09"] as const).map((item) => (
        <button type="button" key={item} aria-current={workspace === item ? "page" : undefined} onClick={() => navigate(item)}>
          {labels[item]}
        </button>
      ))}
    </nav>
    {workspace === "phase09" ? <Phase09Route /> : workspace === "phase08" ? <Phase08Workspace /> : workspace === "phase07" ? <Phase07Workspace /> : workspace === "phase06" ? <Phase06Workspace /> : <Phase05App />}
  </>;
}

export function AppRoot() {
  return <I18nProvider><DesktopWorkspaces /></I18nProvider>;
}
