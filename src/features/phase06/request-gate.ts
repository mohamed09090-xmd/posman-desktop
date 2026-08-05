export type RequestSnapshot<T> =
  | { state: "loading"; value: T | null }
  | { state: "ready"; value: T }
  | { state: "empty"; value: T }
  | { state: "error"; value: T | null };

/**
 * Suppresses stale and post-unmount updates for local IPC reads.
 * A later run always supersedes an earlier one.
 */
export class LatestRequestGate<T> {
  private active = false;
  private generation = 0;

  activate(): void {
    this.active = true;
  }

  deactivate(): void {
    this.active = false;
    this.generation += 1;
  }

  async run(
    loader: () => Promise<T>,
    isEmpty: (value: T) => boolean,
    publish: (snapshot: RequestSnapshot<T>) => void,
  ): Promise<void> {
    const token = ++this.generation;
    if (this.active) publish({ state: "loading", value: null });
    try {
      const value = await loader();
      if (!this.active || token !== this.generation) return;
      publish({ state: isEmpty(value) ? "empty" : "ready", value });
    } catch {
      if (!this.active || token !== this.generation) return;
      publish({ state: "error", value: null });
    }
  }
}
