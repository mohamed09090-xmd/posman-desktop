import {
  createContext,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from "react";
import { resolveRuntimeStatusGateway } from "../../platform/tauri/runtime-environment";
import type { RuntimeStatusGateway } from "../../platform/tauri/runtime-status";
import {
  RuntimeStatusController,
  type RuntimeViewState,
} from "./runtime-state";

interface RuntimeStatusContextValue {
  state: RuntimeViewState;
  retry: () => boolean;
}

const RuntimeStatusContext = createContext<RuntimeStatusContextValue | null>(null);

export function RuntimeStatusProvider({
  children,
  gateway,
}: {
  children: ReactNode;
  gateway?: RuntimeStatusGateway | null;
}) {
  const [controller] = useState(
    () => new RuntimeStatusController(gateway === undefined ? resolveRuntimeStatusGateway() : gateway),
  );
  const [state, setState] = useState(controller.getSnapshot);

  useEffect(() => {
    const unsubscribe = controller.subscribe(() => setState(controller.getSnapshot()));
    controller.activate();
    return () => {
      unsubscribe();
      controller.deactivate();
    };
  }, [controller]);

  return (
    <RuntimeStatusContext.Provider value={{ state, retry: controller.retry }}>
      {children}
    </RuntimeStatusContext.Provider>
  );
}

export function useRuntimeStatus(): RuntimeStatusContextValue {
  const value = useContext(RuntimeStatusContext);
  if (!value) {
    throw new Error("useRuntimeStatus must be used within RuntimeStatusProvider");
  }
  return value;
}
