import { useI18n } from "../../i18n/I18nProvider";
import { useRuntimeStatus } from "./RuntimeStatusProvider";

function StatusMark({ tone }: { tone: "neutral" | "ready" | "warning" | "error" }) {
  return (
    <svg
      className={`runtime-status__mark runtime-status__mark--${tone}`}
      viewBox="0 0 12 12"
      aria-hidden="true"
      focusable="false"
    >
      <circle cx="6" cy="6" r="4.25" fill="currentColor" />
    </svg>
  );
}

function PrimaryStatusText({ children }: { children: string }) {
  return (
    <strong className="runtime-status__primary" data-testid="runtime-status-primary">
      {children}
    </strong>
  );
}

export function RuntimeStatusIndicator() {
  const { state } = useRuntimeStatus();
  const { formatNumber, t } = useI18n();

  if (state.kind === "ready") {
    return (
      <div
        className="runtime-status runtime-status--ready"
        data-testid="runtime-status-ready"
        role="status"
        aria-live="polite"
      >
        <StatusMark tone="ready" />
        <span className="runtime-status__copy">
          <PrimaryStatusText>{t("runtime.ready")}</PrimaryStatusText>
          <small>
            {t("runtime.schemaVersion")}: <bdi>{state.status.schemaVersion}</bdi>
            <span aria-hidden="true"> · </span>
            {t("runtime.migrationCount")}: {formatNumber(state.status.migrationCount, 0)}
          </small>
        </span>
      </div>
    );
  }

  if (state.kind === "preview") {
    return (
      <div
        className="runtime-status runtime-status--preview"
        data-testid="runtime-status-preview"
        role="status"
        aria-live="polite"
      >
        <StatusMark tone="warning" />
        <PrimaryStatusText>{t("runtime.preview")}</PrimaryStatusText>
      </div>
    );
  }

  if (state.kind === "error") {
    return (
      <div className="runtime-status runtime-status--error" data-testid="runtime-status-error">
        <StatusMark tone="error" />
        <PrimaryStatusText>{t("runtime.errorTitle")}</PrimaryStatusText>
      </div>
    );
  }

  return (
    <div
      className="runtime-status runtime-status--initializing"
      data-testid="runtime-status-initializing"
      role="status"
      aria-live="polite"
    >
      <StatusMark tone="neutral" />
      <PrimaryStatusText>
        {state.retrying ? t("runtime.retrying") : t("runtime.initializing")}
      </PrimaryStatusText>
    </div>
  );
}

export function RuntimeStatusNotice() {
  const { retry, state } = useRuntimeStatus();
  const { t } = useI18n();

  if (state.kind === "initializing" && state.retrying) {
    return (
      <section
        className="runtime-notice runtime-notice--retrying"
        data-testid="runtime-retrying-notice"
        role="status"
        aria-live="polite"
      >
        <div>
          <strong>{t("runtime.errorTitle")}</strong>
          <p>{t("runtime.retrying")}</p>
        </div>
        <button type="button" disabled>{t("runtime.retrying")}</button>
      </section>
    );
  }

  if (state.kind !== "error") {
    return null;
  }

  return (
    <section
      className="runtime-notice runtime-notice--error"
      data-testid="runtime-error-notice"
      data-error-code={state.code}
      role="alert"
    >
      <div>
        <strong>{t("runtime.errorTitle")}</strong>
        <p>{t("runtime.errorGeneric")}</p>
      </div>
      <button type="button" onClick={retry} data-testid="runtime-retry">
        {t("runtime.retry")}
      </button>
    </section>
  );
}
