import type { KeyboardEvent, ReactNode } from "react";
import { Button } from "./primitives";

export type StatusTone = "confirmed" | "pending" | "shortage" | "draft" | "posted" | "available" | "low" | "out";

const statusSymbols: Record<StatusTone, string> = {
  confirmed: "✓",
  pending: "…",
  shortage: "!",
  draft: "○",
  posted: "◆",
  available: "✓",
  low: "△",
  out: "×",
};

export function StatusStamp({ tone, children }: { tone: StatusTone; children: ReactNode }) {
  return (
    <span className={`status-stamp status-stamp--${tone}`}>
      <span aria-hidden="true">{statusSymbols[tone]}</span>
      <span>{children}</span>
    </span>
  );
}

export interface DataGridColumn<Row> {
  key: string;
  header: ReactNode;
  render: (row: Row) => ReactNode;
  align?: "start" | "center" | "end";
  width?: string;
}

export interface DataGridProps<Row extends { id: string }> {
  caption: string;
  columns: readonly DataGridColumn<Row>[];
  rows: readonly Row[];
  selectedId?: string | null;
  onRowSelect?: (row: Row) => void;
  empty?: ReactNode;
  density?: "dense" | "comfortable";
  testId?: string;
}

export function DataGrid<Row extends { id: string }>({
  caption,
  columns,
  rows,
  selectedId,
  onRowSelect,
  empty,
  density = "dense",
  testId,
}: DataGridProps<Row>) {
  if (rows.length === 0 && empty) {
    return <>{empty}</>;
  }

  return (
    <div className={`data-grid data-grid--${density}`} data-testid={testId}>
      <table>
        <caption className="sr-only">{caption}</caption>
        <thead>
          <tr>
            {columns.map((column) => (
              <th
                key={column.key}
                scope="col"
                className={`align-${column.align ?? "start"}`}
                style={column.width ? { inlineSize: column.width } : undefined}
              >
                {column.header}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => {
            const selected = row.id === selectedId;
            return (
              <tr
                key={row.id}
                className={selected ? "is-selected" : undefined}
                aria-selected={onRowSelect ? selected : undefined}
                tabIndex={onRowSelect ? 0 : undefined}
                onClick={onRowSelect ? () => onRowSelect(row) : undefined}
                onKeyDown={onRowSelect ? (event: KeyboardEvent<HTMLTableRowElement>) => {
                  if (event.key === "Enter" || event.key === " ") {
                    event.preventDefault();
                    onRowSelect(row);
                  }
                } : undefined}
              >
                {columns.map((column) => (
                  <td key={column.key} className={`align-${column.align ?? "start"}`}>
                    {column.render(row)}
                  </td>
                ))}
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

export function DocumentCanvas({ children, label }: { children: ReactNode; label: string }) {
  return <article className="document-canvas" aria-label={label}>{children}</article>;
}

export interface ProcessStep {
  id: string;
  label: string;
  reference: string;
  state: "completed" | "current" | "pending";
  stateLabel: string;
}

export function ProcessStrip({ steps, label }: { steps: readonly ProcessStep[]; label: string }) {
  const symbols = { completed: "✓", current: "●", pending: "○" } as const;
  return (
    <ol className="process-strip" aria-label={label}>
      {steps.map((step) => (
        <li key={step.id} className={`process-step process-step--${step.state}`} aria-current={step.state === "current" ? "step" : undefined}>
          <span className="process-step__marker" aria-hidden="true">{symbols[step.state]}</span>
          <div>
            <strong>{step.label}</strong>
            <span>{step.reference}</span>
            <small>{step.stateLabel}</small>
          </div>
        </li>
      ))}
    </ol>
  );
}

export function DetailDrawer({
  title,
  closeLabel,
  onClose,
  children,
}: {
  title: string;
  closeLabel: string;
  onClose: () => void;
  children: ReactNode;
}) {
  return (
    <aside className="detail-drawer" aria-labelledby="detail-drawer-title" data-testid="product-drawer">
      <header className="detail-drawer__header">
        <div>
          <span className="eyebrow">POSMAN</span>
          <h2 id="detail-drawer-title">{title}</h2>
        </div>
        <Button variant="quiet" type="button" onClick={onClose} aria-label={closeLabel}>×</Button>
      </header>
      <div className="detail-drawer__body">{children}</div>
    </aside>
  );
}

export function ActionDock({ children, label }: { children: ReactNode; label: string }) {
  return <div className="action-dock" role="group" aria-label={label}>{children}</div>;
}
