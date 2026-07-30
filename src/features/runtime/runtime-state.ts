export interface RuntimeStatusSnapshot {
  databaseReady: boolean;
  schemaVersion: string;
  migrationCount: number;
  foreignKeysEnabled: boolean;
  journalMode: string;
}

export interface RuntimeStatusGatewayLike {
  getRuntimeStatus(): Promise<RuntimeStatusSnapshot>;
}

export type RuntimeViewState =
  | { kind: "initializing"; retrying: boolean }
  | { kind: "ready"; status: RuntimeStatusSnapshot }
  | { kind: "error"; code: string }
  | { kind: "preview" };

type Listener = () => void;

const REQUEST_FAILED_CODE = "RUNTIME_STATUS_REQUEST_FAILED";
const NOT_READY_CODE = "RUNTIME_STATUS_NOT_READY";

function errorCode(error: unknown): string {
  if (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    typeof error.code === "string" &&
    error.code.trim().length > 0
  ) {
    return error.code.trim();
  }
  return REQUEST_FAILED_CODE;
}

export class RuntimeStatusController {
  private readonly gateway: RuntimeStatusGatewayLike | null;
  private readonly listeners = new Set<Listener>();
  private state: RuntimeViewState;
  private active = false;
  private pending = false;
  private requestSequence = 0;

  constructor(gateway: RuntimeStatusGatewayLike | null) {
    this.gateway = gateway;
    this.state = gateway ? { kind: "initializing", retrying: false } : { kind: "preview" };
  }

  getSnapshot = (): RuntimeViewState => this.state;

  subscribe = (listener: Listener): (() => void) => {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  };

  activate(): void {
    this.active = true;
    if (this.gateway) {
      void this.request(false);
    }
  }

  deactivate(): void {
    this.active = false;
    this.pending = false;
    this.requestSequence += 1;
  }

  retry = (): boolean => {
    if (!this.gateway || this.pending) {
      return false;
    }
    void this.request(true);
    return true;
  };

  private publish(nextState: RuntimeViewState): void {
    this.state = nextState;
    for (const listener of this.listeners) {
      listener();
    }
  }

  private async request(retrying: boolean): Promise<void> {
    if (!this.gateway || this.pending) {
      return;
    }

    this.pending = true;
    const requestId = ++this.requestSequence;
    this.publish({ kind: "initializing", retrying });

    try {
      const status = await this.gateway.getRuntimeStatus();
      if (!this.active || requestId !== this.requestSequence) {
        return;
      }

      if (!status.databaseReady || !status.foreignKeysEnabled) {
        this.publish({ kind: "error", code: NOT_READY_CODE });
        return;
      }

      this.publish({ kind: "ready", status });
    } catch (error) {
      if (!this.active || requestId !== this.requestSequence) {
        return;
      }
      this.publish({ kind: "error", code: errorCode(error) });
    } finally {
      if (requestId === this.requestSequence) {
        this.pending = false;
      }
    }
  }
}
