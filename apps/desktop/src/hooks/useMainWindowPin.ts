import { useEffect, useRef, useState } from "react";
import { loadMainWindowPin, setMainWindowPin } from "../api/mainWindowPin";
import type { MainWindowPinStatusDto } from "../domain/types";

export function useMainWindowPin(native: boolean, ready: boolean, frozen: boolean): MainWindowPinControl {
  const [status, setStatus] = useState<MainWindowPinStatusDto | null>(null);
  const [pending, setPending] = useState(false);
  const inFlight = useRef(false);
  const revision = useRef(0);
  const [failure, setFailure] = useState<string | null>(null);
  useEffect(() => {
    if (!native || !ready) return;
    let active = true;
    const initialRevision = revision.current;
    loadMainWindowPin().then((value) => { if (active && revision.current === initialRevision) setStatus(value); })
      .catch((error: unknown) => { if (active && revision.current === initialRevision) setFailure(String(error)); });
    return () => { active = false; };
  }, [native, ready]);

  async function apply(value: boolean | null) {
    if (!native || !ready || frozen || inFlight.current) return;
    inFlight.current = true;
    revision.current += 1;
    setPending(true);
    setFailure(null);
    try {
      setStatus(await setMainWindowPin(value));
    } catch (error) {
      // Transport failure cannot establish the native or durable outcome.
      setStatus(null);
      setFailure(String(error));
    } finally {
      inFlight.current = false;
      setPending(false);
    }
  }
  return {
    status,
    disabled: !native || !ready || frozen || pending || status === null,
    retryDisabled: !native || !ready || frozen || pending,
    message: !native ? "窗口置顶仅原生桌面可用。" : pending ? "正在应用窗口置顶…" : failure ?? status?.error ?? "",
    apply
  };
}

export type MainWindowPinControl = {
  status: MainWindowPinStatusDto | null;
  disabled: boolean;
  retryDisabled: boolean;
  message: string;
  apply: (value: boolean | null) => Promise<void>;
};
