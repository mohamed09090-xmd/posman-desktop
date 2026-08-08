import type { ReactNode } from "react";

import type { Phase09Copy } from "./copy";

export type OperationStatus =
  | { kind: "idle" }
  | { kind: "loading" }
  | { kind: "success"; message: string }
  | { kind: "error"; message: string; code?: string };

export function PermissionBoundary({
  allowed,
  copy,
  children,
}: {
  allowed: boolean;
  copy: Phase09Copy;
  children: ReactNode;
}) {
  if (!allowed) {
    return (
      <section className="phase09-state" role="status" tabIndex={0}>
        <strong>{copy.denied}</strong>
      </section>
    );
  }
  return <>{children}</>;
}

export function OperationNotice({
  status,
  copy,
}: {
  status: OperationStatus;
  copy: Phase09Copy;
}) {
  if (status.kind === "idle") {
    return null;
  }
  if (status.kind === "loading") {
    return (
      <p className="phase09-notice" role="status" aria-live="polite">
        <span className="phase09-progress" aria-hidden="true" />
        {copy.loading}
      </p>
    );
  }
  return (
    <p
      className={`phase09-notice phase09-notice--${status.kind}`}
      role={status.kind === "error" ? "alert" : "status"}
      aria-live="polite"
    >
      {status.kind === "error" && status.code ? (
        <code>{status.code}</code>
      ) : null}
      {status.message}
    </p>
  );
}

export function EmptyState({ copy }: { copy: Phase09Copy }) {
  return (
    <p className="phase09-state" role="status">
      {copy.empty}
    </p>
  );
}

export function IntegrityBadge({ state }: { state: string }) {
  const normalized = state.toUpperCase();
  const safe = normalized === "VERIFIED" || normalized === "OK";
  return (
    <span
      className={`phase09-integrity phase09-integrity--${safe ? "safe" : "warning"}`}
    >
      {state}
    </span>
  );
}

export function hasPermission(
  permissions: readonly string[],
  permission: string,
): boolean {
  return permissions.includes("*") || permissions.includes(permission);
}
