import { useEffect, useState, useSyncExternalStore } from "react";
import { updateWorkspaceShares } from "../api/preferences";
import { WorkspacePersistence } from "./persistence";

export function useWorkspace() {
  const [owner] = useState(() => new WorkspacePersistence(updateWorkspaceShares));
  const snapshot = useSyncExternalStore(owner.subscribe, owner.getSnapshot);
  useEffect(() => () => owner.dispose(), [owner]);
  return {
    ...snapshot,
    owner,
    disabled: !snapshot.ready || snapshot.frozen,
    setShares: owner.update,
    restoreDefaults: () => owner.update(null)
  };
}
