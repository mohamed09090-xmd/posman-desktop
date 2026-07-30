import { invoke, isTauri } from "@tauri-apps/api/core";
import {
  createRuntimeStatusGateway,
  type InvokeFunction,
  type RuntimeStatusGateway,
} from "./runtime-status";

declare global {
  interface Window {
    __POSMAN_DEV_RUNTIME_INVOKER__?: InvokeFunction;
  }
}

export function resolveRuntimeStatusGateway(): RuntimeStatusGateway | null {
  if (import.meta.env.DEV && typeof window !== "undefined") {
    const developmentInvoker = window.__POSMAN_DEV_RUNTIME_INVOKER__;
    if (typeof developmentInvoker === "function") {
      return createRuntimeStatusGateway(developmentInvoker);
    }
  }

  if (!isTauri()) {
    return null;
  }

  return createRuntimeStatusGateway(invoke);
}
