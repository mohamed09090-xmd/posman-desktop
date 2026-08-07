import { useCallback, useEffect, useMemo, useState } from "react";

import {
  normalizePhase09Error,
  templatesGateway,
  type Phase09Locale,
  type TemplateConfiguration,
  type TemplateDetail,
  type TemplateDraftView,
} from "../../platform/tauri/phase09";
import type { Phase09Copy } from "./copy";
import {
  EmptyState,
  OperationNotice,
  type OperationStatus,
} from "./shared";

const DOCUMENT_TYPES = [
  "SALES_ORDER",
  "DELIVERY_NOTE",
  "SALES_INVOICE",
  "SALES_CREDIT_NOTE",
  "PURCHASE_ORDER",
  "GOODS_RECEIPT",
  "SUPPLIER_INVOICE",
  "PURCHASE_RETURN",
  "CUSTOMER_RECEIPT",
  "SUPPLIER_PAYMENT",
] as const;

function cloneConfiguration(
  configuration: TemplateConfiguration,
): TemplateConfiguration {
  return {
    ...configuration,
    enabledSections: [...configuration.enabledSections],
  };
}

export function TemplatesPanel({
  locale,
  copy,
  canManage,
}: {
  locale: Phase09Locale;
  copy: Phase09Copy;
  canManage: boolean;
}) {
  const [documentType, setDocumentType] = useState<string>("SALES_INVOICE");
  const [detail, setDetail] = useState<TemplateDetail | null>(null);
  const [draft, setDraft] = useState<TemplateDraftView | null>(null);
  const [displayName, setDisplayName] = useState("");
  const [configuration, setConfiguration] =
    useState<TemplateConfiguration | null>(null);
  const [status, setStatus] = useState<OperationStatus>({ kind: "idle" });

  const key = useMemo(
    () => ({ documentType, locale }),
    [documentType, locale],
  );

  const load = useCallback(async () => {
    setStatus({ kind: "loading" });
    try {
      const next = await templatesGateway.get(key);
      setDetail(next);
      setDraft(next.draft);
      setDisplayName(next.draft?.displayName ?? next.summary.displayName);
      setConfiguration(
        next.draft ? cloneConfiguration(next.draft.configuration) : null,
      );
      setStatus({ kind: "idle" });
    } catch (error: unknown) {
      const safe = normalizePhase09Error(error);
      setStatus({ kind: "error", message: safe.message, code: safe.code });
    }
  }, [key]);

  useEffect(() => {
    void load();
  }, [load]);

  async function createDraft() {
    setStatus({ kind: "loading" });
    try {
      const created = await templatesGateway.createDraft({
        ...key,
        displayName: detail?.summary.displayName ?? null,
      });
      setDraft(created);
      setDisplayName(created.displayName);
      setConfiguration(cloneConfiguration(created.configuration));
      setStatus({ kind: "success", message: copy.createDraft });
      await load();
    } catch (error: unknown) {
      const safe = normalizePhase09Error(error);
      setStatus({ kind: "error", message: safe.message, code: safe.code });
    }
  }

  async function saveDraft() {
    if (!draft || !configuration) {
      return;
    }
    setStatus({ kind: "loading" });
    try {
      const updated = await templatesGateway.updateDraft({
        draftId: draft.draftId,
        displayName,
        configuration,
        expectedRowVersion: draft.rowVersion,
      });
      setDraft(updated);
      setConfiguration(cloneConfiguration(updated.configuration));
      setStatus({ kind: "success", message: copy.saveDraft });
      await load();
    } catch (error: unknown) {
      const safe = normalizePhase09Error(error);
      setStatus({ kind: "error", message: safe.message, code: safe.code });
    }
  }

  async function publishDraft() {
    if (!draft || !window.confirm(copy.confirmation)) {
      return;
    }
    setStatus({ kind: "loading" });
    try {
      const version = await templatesGateway.publish({
        draftId: draft.draftId,
        expectedRowVersion: draft.rowVersion,
        confirmed: true,
      });
      setStatus({
        kind: "success",
        message: `${copy.publish} · ${version.contentSha256}`,
      });
      await load();
    } catch (error: unknown) {
      const safe = normalizePhase09Error(error);
      setStatus({ kind: "error", message: safe.message, code: safe.code });
    }
  }

  async function retire(versionId: string, rowVersion: number) {
    if (!window.confirm(copy.confirmation)) {
      return;
    }
    setStatus({ kind: "loading" });
    try {
      await templatesGateway.retire({
        templateVersionId: versionId,
        expectedRowVersion: rowVersion,
        confirmed: true,
      });
      setStatus({ kind: "success", message: copy.retire });
      await load();
    } catch (error: unknown) {
      const safe = normalizePhase09Error(error);
      setStatus({ kind: "error", message: safe.message, code: safe.code });
    }
  }

  function setFlag(
    keyName: keyof Pick<
      TemplateConfiguration,
      | "showLogo"
      | "showCompanyIdentity"
      | "showTradeRegister"
      | "showTaxIdentifier"
      | "showPartnerAddress"
      | "showPaymentInformation"
    >,
    value: boolean,
  ) {
    setConfiguration((current) =>
      current ? { ...current, [keyName]: value } : current,
    );
  }

  return (
    <section className="phase09-panel" aria-labelledby="phase09-templates-title">
      <header className="phase09-panel__header">
        <h2 id="phase09-templates-title">{copy.templates}</h2>
        <label>
          <span>{copy.documentType}</span>
          <select
            value={documentType}
            onChange={(event) => setDocumentType(event.currentTarget.value)}
          >
            {DOCUMENT_TYPES.map((value) => (
              <option key={value} value={value}>
                {value}
              </option>
            ))}
          </select>
        </label>
      </header>
      <OperationNotice status={status} copy={copy} />
      {!detail ? (
        <EmptyState copy={copy} />
      ) : (
        <div className="phase09-template-layout">
          <section className="phase09-card" aria-label={copy.state}>
            <p>
              <strong>{detail.summary.displayName}</strong>
            </p>
            <dl className="phase09-definition-list">
              <div>
                <dt>{copy.state}</dt>
                <dd>
                  <span
                    className={`phase09-template-state phase09-template-state--${detail.summary.state.toLowerCase()}`}
                  >
                    {detail.summary.state}
                  </span>
                </dd>
              </div>
              <div>
                <dt>{copy.hash}</dt>
                <dd>
                  <code title={detail.summary.activeContentSha256 ?? undefined}>
                    {detail.summary.activeContentSha256?.slice(0, 20) ?? "—"}
                  </code>
                </dd>
              </div>
            </dl>
            {!draft ? (
              <button
                type="button"
                className="phase09-button--primary"
                disabled={!canManage}
                onClick={() => void createDraft()}
              >
                {copy.createDraft}
              </button>
            ) : null}
          </section>

          {draft && configuration ? (
            <section className="phase09-card phase09-template-editor">
              <label>
                <span>{copy.templates}</span>
                <input
                  value={displayName}
                  onChange={(event) => setDisplayName(event.currentTarget.value)}
                  maxLength={160}
                />
              </label>
              <label>
                <span>العنوان بالعربية</span>
                <input
                  dir="rtl"
                  value={configuration.documentTitleAr}
                  onChange={(event) =>
                    setConfiguration({
                      ...configuration,
                      documentTitleAr: event.currentTarget.value,
                    })
                  }
                />
              </label>
              <label>
                <span>Titre français</span>
                <input
                  dir="ltr"
                  value={configuration.documentTitleFr}
                  onChange={(event) =>
                    setConfiguration({
                      ...configuration,
                      documentTitleFr: event.currentTarget.value,
                    })
                  }
                />
              </label>
              <div className="phase09-check-grid">
                {(
                  [
                    ["showLogo", "Logo"],
                    ["showCompanyIdentity", "Company identity"],
                    ["showTradeRegister", "Trade register"],
                    ["showTaxIdentifier", "Tax identifier"],
                    ["showPartnerAddress", "Partner address"],
                    ["showPaymentInformation", "Payment information"],
                  ] as const
                ).map(([field, label]) => (
                  <label key={field} className="phase09-checkbox">
                    <input
                      type="checkbox"
                      checked={configuration[field]}
                      onChange={(event) => setFlag(field, event.currentTarget.checked)}
                    />
                    <span>{label}</span>
                  </label>
                ))}
              </div>
              <div className="phase09-form phase09-form--inline">
                <label>
                  <span>Spacing</span>
                  <select
                    value={configuration.spacing}
                    onChange={(event) =>
                      setConfiguration({
                        ...configuration,
                        spacing: event.currentTarget.value as "NORMAL" | "COMPACT",
                      })
                    }
                  >
                    <option value="NORMAL">NORMAL</option>
                    <option value="COMPACT">COMPACT</option>
                  </select>
                </label>
                <label>
                  <span>Orientation</span>
                  <select
                    value={configuration.orientation}
                    onChange={(event) =>
                      setConfiguration({
                        ...configuration,
                        orientation: event.currentTarget.value as
                          | "PORTRAIT"
                          | "LANDSCAPE",
                      })
                    }
                  >
                    <option value="PORTRAIT">PORTRAIT</option>
                    <option value="LANDSCAPE">LANDSCAPE</option>
                  </select>
                </label>
              </div>
              <label>
                <span>التذييل بالعربية</span>
                <textarea
                  dir="rtl"
                  value={configuration.footerTextAr}
                  onChange={(event) =>
                    setConfiguration({
                      ...configuration,
                      footerTextAr: event.currentTarget.value,
                    })
                  }
                />
              </label>
              <label>
                <span>Pied de page français</span>
                <textarea
                  dir="ltr"
                  value={configuration.footerTextFr}
                  onChange={(event) =>
                    setConfiguration({
                      ...configuration,
                      footerTextFr: event.currentTarget.value,
                    })
                  }
                />
              </label>
              <div className="phase09-actions">
                <button
                  type="button"
                  disabled={!canManage}
                  onClick={() => void saveDraft()}
                >
                  {copy.saveDraft}
                </button>
                <button
                  type="button"
                  className="phase09-button--primary"
                  disabled={!canManage}
                  onClick={() => void publishDraft()}
                >
                  {copy.publish}
                </button>
              </div>
            </section>
          ) : null}

          <section className="phase09-card">
            <h3>History</h3>
            {detail.versions.length === 0 ? (
              <EmptyState copy={copy} />
            ) : (
              <ul className="phase09-version-list">
                {detail.versions.map((version) => (
                  <li key={version.versionId}>
                    <div>
                      <strong>v{version.versionNumber}</strong>
                      <span
                        className={`phase09-template-state phase09-template-state--${version.status.toLowerCase()}`}
                      >
                        {version.status}
                      </span>
                    </div>
                    <code title={version.contentSha256}>
                      {version.contentSha256.slice(0, 18)}…
                    </code>
                    {version.status === "PUBLISHED" ? (
                      <button
                        type="button"
                        disabled={!canManage}
                        onClick={() =>
                          void retire(version.versionId, version.rowVersion)
                        }
                      >
                        {copy.retire}
                      </button>
                    ) : null}
                  </li>
                ))}
              </ul>
            )}
          </section>
        </div>
      )}
    </section>
  );
}
