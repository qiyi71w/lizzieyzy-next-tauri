import type { EngineRunDto } from "../domain/types";
import { t } from "../i18n/resources";

export function EngineResourceDetails({ run }: { run: EngineRunDto | null }) {
  const identity = run?.qualified_resource;
  const unknown = t("engineResource.unknown");
  return <details className="message engine-resource-message">
    <summary>{t("engineResource.title")}</summary>
    {!identity ? <p>{t("engineResource.unverified")}</p> : <>
      <p>{t("engineResource.qualified")}</p>
      <p>{t("engineResource.run")}: {run?.run_id} · {t("engineResource.profile")}: {run?.profile_id}</p>
      <dl>
        <dt>{t("engineResource.origin")}</dt><dd>{identity.origin === "local_unknown" ? t("engineResource.localUnknown") : identity.origin}</dd>
        <dt>{t("engineResource.version")}</dt><dd>{identity.version ?? unknown}</dd>
        <dt>{t("engineResource.source")}</dt><dd>{identity.source_commit ?? unknown}</dd>
        <dt>{t("engineResource.backend")}</dt><dd>{identity.backend ?? unknown}</dd>
        <dt>{t("engineResource.revision")}</dt><dd>{identity.profile_revision}</dd>
      </dl>
      <ul>{identity.resources.map((resource, index) => <li key={`${resource.component}-${index}`}>
        {resource.component}: {resource.resolved_path}<br />
        SHA256: {resource.sha256} · {resource.bytes} B
      </li>)}</ul>
      <p>{t("engineResource.paths")}</p>
      {!identity.static_zlib_exemption && <p>{t("engineResource.noExemption")}</p>}
    </>}
  </details>;
}
