import type { WorkspaceShares } from "./projection";

type Snapshot = {
  shares: WorkspaceShares | null;
  status: "saved" | "pending" | "saving" | "unsaved";
  error: string | null;
  ready: boolean;
  frozen: boolean;
};

/** One layout writer: drafts are immediate; only successful writes advance the durable revision. */
export class WorkspacePersistence {
  snapshot: Snapshot = { shares: null, status: "saved", error: null, ready: false, frozen: false };
  private revision = 0;
  private durableRevision = 0;
  private timer: ReturnType<typeof setTimeout> | null = null;
  private active: Promise<void> | null = null;
  private listeners = new Set<() => void>();

  constructor(private readonly save: (shares: WorkspaceShares | null) => Promise<unknown>) {}

  subscribe = (listener: () => void) => {
    this.listeners.add(listener);
    return () => { this.listeners.delete(listener); };
  };
  getSnapshot = () => this.snapshot;

  private publish(patch: Partial<Snapshot>) {
    this.snapshot = { ...this.snapshot, ...patch };
    this.listeners.forEach((listener) => listener());
  }

  load(shares: WorkspaceShares | null) {
    if (!this.snapshot.ready) this.publish({ shares, ready: true });
  }

  freeze(frozen: boolean) { this.publish({ frozen }); }

  update = (shares: WorkspaceShares | null) => {
    if (!this.snapshot.ready || this.snapshot.frozen) return;
    this.revision++;
    this.publish({ shares, status: "pending", error: null });
    this.clearTimer();
    this.timer = setTimeout(() => {
      this.timer = null;
      void this.write();
    }, 500);
  };

  private clearTimer() {
    clearTimeout(this.timer ?? undefined);
    this.timer = null;
  }

  private write(): Promise<void> {
    if (this.active) return this.active;
    if (this.revision === this.durableRevision) return Promise.resolve();
    const revision = this.revision;
    const shares = this.snapshot.shares;
    this.publish({ status: "saving", error: null });
    this.active = Promise.resolve().then(() => this.save(shares)).then(() => {
      this.durableRevision = revision;
      if (revision === this.revision) this.publish({ status: "saved", error: null });
    }, (error: unknown) => {
      if (revision === this.revision) {
        this.publish({ status: "unsaved", error: error instanceof Error ? error.message : String(error) });
      }
    }).finally(() => {
      this.active = null;
      // A settled request owns neither the newer draft nor its debounce deadline.
      if (revision !== this.revision && this.timer === null) void this.write();
    });
    return this.active;
  }

  retry = () => {
    this.clearTimer();
    return this.write();
  };

  /** Shared pre-departure/final-native-exit fence; timeout never starts a competing writer. */
  async flush(): Promise<void> {
    this.clearTimer();
    const drain = async () => {
      while (this.durableRevision !== this.revision) {
        await this.write();
        this.clearTimer();
        if (this.snapshot.status === "unsaved") throw new Error(this.snapshot.error ?? "Layout save failed.");
      }
    };
    let timeout: ReturnType<typeof setTimeout> | undefined;
    try {
      await Promise.race([
        drain(),
        new Promise<never>((_, reject) => {
          timeout = setTimeout(() => reject(new Error("Layout save timed out after 5 seconds.")), 5000);
        })
      ]);
    } finally {
      clearTimeout(timeout);
    }
  }

  dispose() { this.clearTimer(); }
}
