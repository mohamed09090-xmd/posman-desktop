import { invoke } from "@tauri-apps/api/core";

import { Phase09GatewayError, type SafeError } from "./contracts";

type JsonRecord = Record<string, unknown>;

function isRecord(value: unknown): value is JsonRecord {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isSafeError(value: unknown): value is SafeError {
  return (
    isRecord(value) &&
    typeof value.code === "string" &&
    typeof value.message === "string" &&
    typeof value.retryable === "boolean"
  );
}

export function normalizePhase09Error(error: unknown): Phase09GatewayError {
  if (error instanceof Phase09GatewayError) {
    return error;
  }
  if (isSafeError(error)) {
    return new Phase09GatewayError(error);
  }
  if (isRecord(error) && isSafeError(error.error)) {
    return new Phase09GatewayError(error.error);
  }
  return new Phase09GatewayError({
    code: "INTERNAL_ERROR",
    message: "POSMAN could not complete the local operation.",
    retryable: true,
  });
}

export async function invokePhase09<T>(
  command: string,
  payload?: JsonRecord,
): Promise<T> {
  try {
    return await invoke<T>(command, payload);
  } catch (error: unknown) {
    throw normalizePhase09Error(error);
  }
}

export class RequestGate {
  private generation = 0;

  begin(): number {
    this.generation += 1;
    return this.generation;
  }

  isCurrent(generation: number): boolean {
    return generation === this.generation;
  }

  invalidate(): void {
    this.generation += 1;
  }
}

export function requireString(value: unknown, field: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new Phase09GatewayError({
      code: "INVALID_RESPONSE",
      message: `POSMAN returned an invalid ${field}.`,
      retryable: true,
    });
  }
  return value;
}

export function requireArray<T>(value: unknown, field: string): T[] {
  if (!Array.isArray(value)) {
    throw new Phase09GatewayError({
      code: "INVALID_RESPONSE",
      message: `POSMAN returned an invalid ${field}.`,
      retryable: true,
    });
  }
  return value as T[];
}

export function requireObject<T extends JsonRecord>(
  value: unknown,
  field: string,
): T {
  if (!isRecord(value)) {
    throw new Phase09GatewayError({
      code: "INVALID_RESPONSE",
      message: `POSMAN returned an invalid ${field}.`,
      retryable: true,
    });
  }
  return value as T;
}
