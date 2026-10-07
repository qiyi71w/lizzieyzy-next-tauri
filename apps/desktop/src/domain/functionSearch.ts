import type { ResourceKey } from "../i18n/resources";
import { t } from "../i18n/resources";

export type FunctionSearchAction = {
  id: string;
  label: ResourceKey;
  keywords: readonly string[];
  shortcut?: string;
  disabledReason?: ResourceKey;
  execute: () => void;
};

/** Source keywords stay independent of the effective display translation. */
export function searchFunctions(catalog: readonly FunctionSearchAction[], query: string): FunctionSearchAction[] {
  const needles = query.trim().toLocaleLowerCase().split(/\s+/).filter(Boolean);
  return catalog.filter((entry) => {
    const haystack = [t(entry.label), entry.id, entry.shortcut ?? "", ...entry.keywords].join(" ").toLocaleLowerCase();
    return needles.every((needle) => haystack.includes(needle));
  });
}
