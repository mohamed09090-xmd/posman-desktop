export interface RuntimeStatus {
  databaseReady: boolean;
  schemaVersion: string;
  migrationCount: number;
  foreignKeysEnabled: boolean;
  journalMode: string;
}

export type InvokeFunction = (
  command: string,
  args?: Record<string, unknown>,
) => Promise<unknown>;

export const RUNTIME_STATUS_COMMAND = "get_runtime_status";
export const RUNTIME_STATUS_INVALID_RESPONSE = "RUNTIME_STATUS_INVALID_RESPONSE";
export const RUNTIME_BRIDGE_UNAVAILABLE = "RUNTIME_BRIDGE_UNAVAILABLE";
export const RUNTIME_STATUS_REQUEST_FAILED = "RUNTIME_STATUS_REQUEST_FAILED";
export const RUNTIME_STATUS_NOT_READY = "RUNTIME_STATUS_NOT_READY";
export const RUNTIME_STATUS_UNAVAILABLE = "RUNTIME_STATUS_UNAVAILABLE";

const SAFE_RUNTIME_ERROR_CODES = new Set([
  RUNTIME_STATUS_INVALID_RESPONSE,
  RUNTIME_BRIDGE_UNAVAILABLE,
  RUNTIME_STATUS_REQUEST_FAILED,
  RUNTIME_STATUS_NOT_READY,
  RUNTIME_STATUS_UNAVAILABLE,
]);

export class RuntimeGatewayError extends Error {
  readonly code: string;

  constructor(code: string) {
    super("The local runtime status could not be verified.");
    this.name = "RuntimeGatewayError";
    this.code = code;
  }
}

export interface RuntimeStatusGateway {
  getRuntimeStatus(): Promise<RuntimeStatus>;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isNonEmptyString(value: unknown): value is string {
  return typeof value === "string" && value.trim().length > 0;
}

export function validateRuntimeStatus(payload: unknown): RuntimeStatus {
  if (!isRecord(payload)) {
    throw new RuntimeGatewayError(RUNTIME_STATUS_INVALID_RESPONSE);
  }

  const {
    databaseReady,
    schemaVersion,
    migrationCount,
    foreignKeysEnabled,
    journalMode,
  } = payload;

  if (
    typeof databaseReady !== "boolean" ||
    !isNonEmptyString(schemaVersion) ||
    !Number.isInteger(migrationCount) ||
    (migrationCount as number) < 0 ||
    typeof foreignKeysEnabled !== "boolean" ||
    !isNonEmptyString(journalMode)
  ) {
    throw new RuntimeGatewayError(RUNTIME_STATUS_INVALID_RESPONSE);
  }

  return {
    databaseReady,
    schemaVersion: schemaVersion.trim(),
    migrationCount: migrationCount as number,
    foreignKeysEnabled,
    journalMode: journalMode.trim(),
  };
}

export function normalizeRuntimeError(error: unknown): RuntimeGatewayError {
  if (error instanceof RuntimeGatewayError) {
    return error;
  }

  if (isRecord(error) && isNonEmptyString(error.code)) {
    const code = error.code.trim();
    if (SAFE_RUNTIME_ERROR_CODES.has(code)) {
      return new RuntimeGatewayError(code);
    }
  }

  return new RuntimeGatewayError(RUNTIME_STATUS_REQUEST_FAILED);
}

export function createRuntimeStatusGateway(
  invoker: InvokeFunction,
): RuntimeStatusGateway {
  return {
    async getRuntimeStatus(): Promise<RuntimeStatus> {
      try {
        const payload = await invoker(RUNTIME_STATUS_COMMAND);
        return validateRuntimeStatus(payload);
      } catch (error) {
        throw normalizeRuntimeError(error);
      }
    },
  };
}
