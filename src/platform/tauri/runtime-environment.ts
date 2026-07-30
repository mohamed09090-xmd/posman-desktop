import { invoke } from "@tauri-apps/api/core";
import {
  createRuntimeStatusGateway,
  type InvokeFunction,
  type RuntimeStatusGateway,
} from "./runtime-status";

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
    __POSMAN_DEV_RUNTIME_INVOKER__?: InvokeFunction;
  }
}

function hasTauriBridge(): boolean {
  return (
    typeof window !== "undefined" &&
    typeof window.__TAURI_INTERNALS__ === "object" &&
    window.__TAURI_INTERNALS__ !== null
  );
}

export function resolveRuntimeStatusGateway(): RuntimeStatusGateway | null {
  if (import.meta.env.DEV && typeof window !== "undefined") {
    const developmentInvoker = window.__POSMAN_DEV_RUNTIME_INVOKER__;
    if (typeof developmentInvoker === "function") {
      return createRuntimeStatusGateway(developmentInvoker);
    }
  }

  if (!hasTauriBridge()) {
    return null;
  }

  return createRuntimeStatusGateway(invoke);
}
