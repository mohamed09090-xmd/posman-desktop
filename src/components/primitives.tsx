import type {
  ButtonHTMLAttributes,
  InputHTMLAttributes,
  ReactNode,
  SelectHTMLAttributes,
} from "react";

export type ButtonVariant = "primary" | "secondary" | "quiet" | "danger";

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  loading?: boolean;
}

export function Button({
  variant = "secondary",
  loading = false,
  disabled,
  children,
  className = "",
  ...props
}: ButtonProps) {
  return (
    <button
      {...props}
      className={`button button--${variant} ${className}`.trim()}
      disabled={disabled || loading}
      aria-busy={loading || undefined}
    >
      {loading ? <span className="button__spinner" aria-hidden="true" /> : null}
      <span>{children}</span>
    </button>
  );
}

export interface FieldProps {
  id: string;
  label: string;
  hint?: string;
  error?: string;
  required?: boolean;
  children: ReactNode;
  className?: string;
}

export function Field({ id, label, hint, error, required, children, className = "" }: FieldProps) {
  const descriptionId = error ? `${id}-error` : hint ? `${id}-hint` : undefined;
  return (
    <div className={`field ${error ? "field--error" : ""} ${className}`.trim()}>
      <label className="field__label" htmlFor={id}>
        {label}
        {required ? <span className="field__required" aria-hidden="true"> *</span> : null}
      </label>
      {children}
      {error ? (
        <p className="field__message field__message--error" id={descriptionId}>
          <span aria-hidden="true">!</span> {error}
        </p>
      ) : hint ? (
        <p className="field__message" id={descriptionId}>{hint}</p>
      ) : null}
    </div>
  );
}

export function Input({ className = "", ...props }: InputHTMLAttributes<HTMLInputElement>) {
  return <input {...props} className={`input ${className}`.trim()} />;
}

export function Select({ className = "", children, ...props }: SelectHTMLAttributes<HTMLSelectElement>) {
  return (
    <select {...props} className={`select ${className}`.trim()}>
      {children}
    </select>
  );
}

export type NoticeTone = "info" | "success" | "error" | "warning";

export interface InlineNoticeProps {
  title: string;
  children: ReactNode;
  tone?: NoticeTone;
  live?: boolean;
}

const noticeSymbols: Record<NoticeTone, string> = {
  info: "i",
  success: "✓",
  error: "!",
  warning: "△",
};

export function InlineNotice({ title, children, tone = "info", live = false }: InlineNoticeProps) {
  return (
    <section
      className={`notice notice--${tone}`}
      aria-live={live ? "polite" : undefined}
      aria-atomic={live || undefined}
    >
      <span className="notice__symbol" aria-hidden="true">{noticeSymbols[tone]}</span>
      <div>
        <h3>{title}</h3>
        <div className="notice__body">{children}</div>
      </div>
    </section>
  );
}

export interface StateProps {
  title?: string;
  children?: ReactNode;
}

export function EmptyState({ title, children }: StateProps) {
  return (
    <div className="state state--empty" role="status">
      <span className="state__mark" aria-hidden="true">—</span>
      {title ? <h3>{title}</h3> : null}
      {children ? <div>{children}</div> : null}
    </div>
  );
}

export function LoadingState({ children }: StateProps) {
  return (
    <div className="state state--loading" role="status" aria-live="polite">
      <span className="loading-bars" aria-hidden="true"><i /><i /><i /></span>
      <span>{children}</span>
    </div>
  );
}
