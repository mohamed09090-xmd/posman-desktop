import { useMemo, useState } from "react";
import { useI18n } from "../../i18n/I18nProvider";
import { resolvePhase08Gateway } from "../../platform/tauri/phase08";
import { PHASE08_COPY, PHASE08_SECTIONS, type Phase08Section } from "./copy";
import { type Notice } from "./shared";
import { Accounts, Overview, Rules } from "./panels/setup";
import { Journals } from "./panels/journals";
import { Payments } from "./panels/payments";
import { Ledger, Statements } from "./panels/statements";
import { Periods, Queue } from "./panels/periods";
import "./phase08.css";

export function Phase08Workspace() {
  const { locale, setLocale, formatMoney, formatDate } = useI18n();
  const c = PHASE08_COPY[locale];
  const gateway = useMemo(resolvePhase08Gateway, []);
  const [section, setSection] = useState<Phase08Section>("overview");
  const [notice, setNotice] = useState<Notice>();
  return <section className="p8-workspace" aria-labelledby="p8-title">
    <header className="p8-commandbar">
      <div><p>POSMAN / PHASE 08</p><h1 id="p8-title">{c.title}</h1><span>{c.subtitle}</span></div>
      <div className="p8-command-actions"><div className="p8-language" role="group" aria-label="Language"><button type="button" aria-pressed={locale === "ar-DZ"} onClick={() => setLocale("ar-DZ")}>العربية</button><button type="button" aria-pressed={locale === "fr-DZ"} onClick={() => setLocale("fr-DZ")}>Français</button></div><span className="p8-runtime">{gateway ? c.ready : c.preview}</span></div>
    </header>
    <div className="p8-layout">
      <nav className="p8-rail" aria-label={c.title}>
        {PHASE08_SECTIONS.map((item, index) => <button type="button" key={item} aria-current={section === item ? "page" : undefined} onClick={() => { setSection(item); setNotice(undefined); }}>
          <span aria-hidden="true">{String(index + 1).padStart(2, "0")}</span>{c[item]}
        </button>)}
      </nav>
      <main className="p8-canvas" data-testid={`phase08-${section}`}>
        <header className="p8-section-title"><p>{c.title}</p><h2>{c[section]}</h2></header>
        {notice ? <div className={`p8-notice p8-notice--${notice.tone}`} role={notice.tone === "error" ? "alert" : "status"}>{notice.text}</div> : null}
        {section === "overview" ? <Overview gateway={gateway} c={c} setNotice={setNotice} /> : null}
        {section === "accounts" ? <Accounts gateway={gateway} c={c} setNotice={setNotice} /> : null}
        {section === "rules" ? <Rules gateway={gateway} c={c} setNotice={setNotice} /> : null}
        {section === "journals" ? <Journals gateway={gateway} c={c} setNotice={setNotice} formatMoney={formatMoney} formatDate={formatDate} /> : null}
        {section === "payments" ? <Payments gateway={gateway} c={c} setNotice={setNotice} formatMoney={formatMoney} /> : null}
        {section === "statements" ? <Statements gateway={gateway} c={c} formatMoney={formatMoney} /> : null}
        {section === "ledger" ? <Ledger gateway={gateway} c={c} formatMoney={formatMoney} /> : null}
        {section === "periods" ? <Periods gateway={gateway} c={c} setNotice={setNotice} formatDate={formatDate} /> : null}
        {section === "queue" ? <Queue gateway={gateway} c={c} setNotice={setNotice} /> : null}
      </main>
    </div>
  </section>;
}
