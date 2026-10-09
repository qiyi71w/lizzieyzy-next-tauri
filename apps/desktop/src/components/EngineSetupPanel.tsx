import { useEffect, useRef, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { checkEngineAssets, loadEngineProfilesSettings, reorderEngineProfilesSettings, saveEngineProfilesSettings } from "../api/backend";
import type { AssetCheckDto, EngineBackendDto, EngineFailureDto, EngineProfileDto, EngineProfileRecordDto, ForegroundEngineSnapshotDto } from "../domain/types";
import { profileHasPendingChanges, runFromSnapshot, verifiedEngineCapabilitiesLabel } from "../domain/foregroundEngine";
import { t } from "../i18n/resources";
import { EngineResourceDetails } from "./EngineResourceDetails";
import { ModelInventoryPanel } from "./ModelInventoryPanel";
import { EngineDiagnosticsPanel } from "./EngineDiagnosticsPanel";

type Props = {
  disabled?: boolean;
  engineSnapshot?: ForegroundEngineSnapshotDto | null;
  engineFailure?: EngineFailureDto | null;
  onProfilesChange?: (profiles: EngineProfileRecordDto[]) => void;
};

export function EngineSetupPanel({ disabled = false, engineSnapshot = null, engineFailure = null, onProfilesChange }: Props) {
  const [profiles, setProfiles] = useState<EngineProfileRecordDto[]>([]);
  const [selectedProfileId, setSelectedProfileId] = useState("default");
  const [profileName, setProfileName] = useState("Local KataGo");
  const [enginePath, setEnginePath] = useState("");
  const [adapterKind, setAdapterKind] = useState<EngineBackendDto>("kata_go_analysis");
  const [argv, setArgv] = useState<string[]>([]);
  const [modelPath, setModelPath] = useState("");
  const [configPath, setConfigPath] = useState("");
  const [workingDir, setWorkingDir] = useState("");
  const [maxVisits, setMaxVisits] = useState("800");
  const [profileStatus, setProfileStatus] = useState("Loading profile...");
  const [assetChecks, setAssetChecks] = useState<AssetCheckDto[]>([]);
  const [autoloadProfileId, setAutoloadProfileId] = useState<string | null>(null);
  const [catalogBusy, setCatalogBusy] = useState(false);
  const catalogWritePending = useRef(false);
  const [orderStatus, setOrderStatus] = useState("");
  const modelOperation = useRef(false);
  const [modelBusy, setModelBusy] = useState(false);
  const editingProfile = profiles.find((record) => record.id === selectedProfileId)?.profile;

  function beginModelOperation() {
    if (disabled || catalogWritePending.current || modelOperation.current) return false;
    modelOperation.current = true;
    setModelBusy(true);
    return true;
  }

  function endModelOperation() {
    modelOperation.current = false;
    setModelBusy(false);
  }

  const visits = Number(maxVisits);
  const canSave = profiles.length > 0 && profileName.trim().length > 0
    && (adapterKind === "generic_gtp" || (Number.isInteger(visits) && visits > 0 && visits <= 0xffffffff));
  const candidateId = engineSnapshot?.lifecycle.state === "switching" ? engineSnapshot.lifecycle.candidate.profile_id : null;
  const activeId = runFromSnapshot(engineSnapshot)?.profile_id;
  const canDeleteProfile = selectedProfileId !== "default" && profiles.length > 1
    && selectedProfileId !== activeId && selectedProfileId !== candidateId;
  const snapshot = engineSnapshot ?? {
    revision: 0,
    lifecycle: { state: "no_engine" as const },
    continuous: { enabled: null, phase: "loading" as const }
  };
  const run = runFromSnapshot(snapshot);
  const savedRecord = profiles.find((profile) => profile.id === run?.profile_id)?.profile;
  const pendingChanges = Boolean(savedRecord && run && profileHasPendingChanges(savedRecord, snapshot));

  useEffect(() => {
    let isMounted = true;
    loadEngineProfilesSettings()
      .then((settings) => {
        if (!isMounted) return;
        const selected = settings.profiles.find((profile) => profile.id === settings.selected_profile_id) ?? settings.profiles[0];
        setProfiles(settings.profiles);
        onProfilesChange?.(settings.profiles);
        setSelectedProfileId(selected?.id ?? "default");
        setAutoloadProfileId(settings.autoload_profile_id ?? null);
        if (!selected) {
          setProfileStatus("No configured profile.");
          return;
        }
        applyProfileRecord(selected);
        setProfileStatus(settings.profiles.length > 1 ? "Profiles loaded." : "Profile loaded.");
      })
      .catch((error: unknown) => {
        if (isMounted) setProfileStatus(`Load failed: ${errorMessage(error)}`);
      });
    return () => {
      isMounted = false;
    };
  }, [onProfilesChange]);

  function applyProfileRecord(record: EngineProfileRecordDto) {
    setProfileName(record.profile.name);
    setEnginePath(record.profile.program);
    setAdapterKind(record.profile.adapter_kind);
    setArgv([...record.profile.argv]);
    setModelPath(record.profile.adapter_kind !== "generic_gtp" ? record.profile.settings.model_path ?? "" : "");
    setConfigPath(record.profile.adapter_kind !== "generic_gtp" ? record.profile.settings.config_path ?? "" : "");
    setWorkingDir(record.profile.working_dir ?? "");
    setMaxVisits(record.profile.adapter_kind !== "generic_gtp" ? String(record.profile.settings.max_visits) : "800");
    setAssetChecks([]);
  }

  function buildProfile(): EngineProfileDto {
    const previous = profiles.find((record) => record.id === selectedProfileId)?.profile;
    const common = { name: profileName, program: enginePath, argv: [...argv], working_dir: optionalPath(workingDir, previous?.working_dir) };
    return adapterKind === "generic_gtp"
      ? { ...common, adapter_kind: "generic_gtp", settings: {} }
      : {
        ...common, adapter_kind: adapterKind,
        settings: {
          model_path: optionalPath(modelPath, previous && previous.adapter_kind !== "generic_gtp" ? previous.settings.model_path : null),
          config_path: optionalPath(configPath, previous && previous.adapter_kind !== "generic_gtp" ? previous.settings.config_path : null),
          max_visits: visits
        }
      };
  }

  function buildProfileRecord(id = selectedProfileId): EngineProfileRecordDto {
    return {
      id,
      profile: buildProfile()
    };
  }

  function updatePath(setter: (value: string) => void, value: string, message?: string) {
    setter(value);
    if (assetChecks.length > 0) {
      setAssetChecks([]);
      setProfileStatus("Path changed. Check assets again.");
    } else if (message) {
      setProfileStatus(message);
    }
  }

  async function persistProfiles(
    nextProfiles: EngineProfileRecordDto[],
    selectedId: string,
    autoloadId: string | null,
    successMessage: string
  ) {
    if (catalogWritePending.current || modelOperation.current) throw new Error(t("engineOrder.busy"));
    catalogWritePending.current = true;
    setCatalogBusy(true);
    try {
      const saved = await saveEngineProfilesSettings({
        version: 1,
        selected_profile_id: selectedId,
        autoload_profile_id: autoloadId,
        profiles: nextProfiles
      });
      const selected = saved.profiles.find((profile) => profile.id === saved.selected_profile_id) ?? saved.profiles[0];
      setProfiles(saved.profiles);
      onProfilesChange?.(saved.profiles);
      setSelectedProfileId(saved.selected_profile_id);
      setAutoloadProfileId(saved.autoload_profile_id ?? null);
      if (selected) applyProfileRecord(selected);
      setProfileStatus(successMessage);
    } finally {
      catalogWritePending.current = false;
      setCatalogBusy(false);
    }
  }

  async function handleReorder(profileId: string, direction: "top" | "up" | "down" | "bottom") {
    if (disabled || catalogWritePending.current || modelOperation.current) return;
    const index = profiles.findIndex((record) => record.id === profileId);
    if (index < 0) return;
    const destination = direction === "top" ? 0 : direction === "bottom" ? profiles.length - 1
      : direction === "up" ? Math.max(0, index - 1) : Math.min(profiles.length - 1, index + 1);
    if (index === destination) return;
    const ids = profiles.map((record) => record.id);
    const ordered = [...ids];
    ordered.splice(index, 1);
    ordered.splice(destination, 0, profileId);
    catalogWritePending.current = true;
    setCatalogBusy(true);
    setOrderStatus(t("engineOrder.saving"));
    try {
      const saved = await reorderEngineProfilesSettings({ expected_profile_ids: ids, profile_ids: ordered });
      setProfiles(saved.profiles);
      onProfilesChange?.(saved.profiles);
      setOrderStatus(t("engineOrder.saved"));
    } catch (error) {
      setOrderStatus(`${t("engineOrder.failed")} ${errorMessage(error)}`);
    } finally {
      catalogWritePending.current = false;
      setCatalogBusy(false);
    }
  }

  async function handleSelectProfile(profileId: string) {
    const profile = profiles.find((item) => item.id === profileId);
    if (!profile) return;
    try {
      await persistProfiles(profiles, profileId, autoloadProfileId, `Selected ${profile.profile.name}.`);
    } catch (error) {
      setProfileStatus(`Select failed: ${errorMessage(error)}`);
    }
  }

  async function handleAddProfile() {
    const id = `profile-${Date.now().toString(36)}`;
    const nextProfile: EngineProfileRecordDto = {
      ...buildProfileRecord(id),
      profile: {
        ...buildProfile(),
        name: nextProfileName(profiles)
      }
    };
    try {
      await persistProfiles([...profiles.map((profile) => profile.id === selectedProfileId ? buildProfileRecord(profile.id) : profile), nextProfile], id, autoloadProfileId, "Profile added.");
    } catch (error) {
      setProfileStatus(`Add failed: ${errorMessage(error)}`);
    }
  }

  async function handleDeleteProfile() {
    if (!canDeleteProfile) return;
    const nextProfiles = profiles.filter((profile) => profile.id !== selectedProfileId);
    const nextSelected = nextProfiles.find((profile) => profile.id === "default")?.id ?? nextProfiles[0]?.id ?? "default";
    try {
      await persistProfiles(nextProfiles, nextSelected, autoloadProfileId === selectedProfileId ? null : autoloadProfileId, "Profile deleted.");
    } catch (error) {
      setProfileStatus(`Delete failed: ${errorMessage(error)}`);
    }
  }

  async function handleAutoloadToggle(checked: boolean) {
    if (profiles.length === 0) return;
    const nextAutoload = checked
      ? selectedProfileId
      : autoloadProfileId === selectedProfileId
        ? null
        : autoloadProfileId;
    try {
      await persistProfiles(
        profiles,
        selectedProfileId,
        nextAutoload,
        checked ? "Autoload Default saved." : "Autoload Default cleared."
      );
    } catch (error) {
      setProfileStatus(`Autoload Default failed: ${errorMessage(error)}`);
    }
  }

  async function handlePickPath(label: string, currentValue: string, directory: boolean, setter: (value: string) => void) {
    if (modelOperation.current) return;
    try {
      const selected = await open({
        title: `Select ${label}`,
        directory,
        multiple: false,
        defaultPath: currentValue.trim() || undefined
      });
      const selectedPath = Array.isArray(selected) ? selected[0] : selected;
      if (modelOperation.current) return;
      if (!selectedPath) {
        setProfileStatus(`${label} selection canceled.`);
        return;
      }
      updatePath(setter, selectedPath, `${label} selected. Check assets again.`);
    } catch (error) {
      setProfileStatus(`Native picker unavailable: ${errorMessage(error)}`);
    }
  }

  async function handleSaveProfile() {
    if (!canSave) return;
    try {
      const currentRecord = buildProfileRecord(selectedProfileId);
      const nextProfiles = profiles.some((profile) => profile.id === selectedProfileId)
        ? profiles.map((profile) => profile.id === selectedProfileId ? currentRecord : profile)
        : [...profiles, currentRecord];
      await persistProfiles(nextProfiles, selectedProfileId, autoloadProfileId, "Profile saved.");
      setProfileStatus("Profile saved.");
    } catch (error) {
      setProfileStatus(`Save failed: ${errorMessage(error)}`);
    }
  }

  async function handleCheckAssets() {
    try {
      const checks = await checkEngineAssets(buildProfile());
      setAssetChecks(checks);
      setProfileStatus(assetStatus(checks));
    } catch (error) {
      setProfileStatus(`Check failed: ${errorMessage(error)}`);
    }
  }

  async function handleReloadProfiles() {
    if (catalogWritePending.current || modelOperation.current) return;
    catalogWritePending.current = true;
    setCatalogBusy(true);
    try {
      const settings = await loadEngineProfilesSettings();
      const selected = settings.profiles.find((profile) => profile.id === settings.selected_profile_id);
      setProfiles(settings.profiles);
      onProfilesChange?.(settings.profiles);
      setSelectedProfileId(settings.selected_profile_id);
      setAutoloadProfileId(settings.autoload_profile_id);
      if (selected) applyProfileRecord(selected);
      setProfileStatus("Profiles reloaded.");
    } catch (error) {
      setProfileStatus(`Reload failed: ${errorMessage(error)}`);
    } finally {
      catalogWritePending.current = false;
      setCatalogBusy(false);
    }
  }

  return (
    <section className="engine-setup-panel" aria-label="引擎设置" data-focus-owner="engine" tabIndex={-1}>
      <EngineDiagnosticsPanel engineSnapshot={snapshot} disabled={disabled} />
      <fieldset disabled={disabled || modelBusy} style={{ border: 0, padding: 0, minWidth: 0 }}>
      <div className="engine-run-row">
        <label>
          <span>配置</span>
          <select value={selectedProfileId} disabled={catalogBusy} onChange={(event) => void handleSelectProfile(event.target.value)}>
            {profiles.map((profile) => (
              <option key={profile.id} value={profile.id}>{profile.profile.name}</option>
            ))}
          </select>
        </label>
        <label>
          <span>名称</span>
          <input value={profileName} onChange={(event) => setProfileName(event.target.value)} placeholder="本地 KataGo" />
        </label>
        <button type="button" onClick={() => void handleAddProfile()} disabled={!canSave || catalogBusy}>新增</button>
        <button type="button" onClick={() => void handleDeleteProfile()} disabled={!canDeleteProfile || catalogBusy}>删除</button>
        <label>
          <input
            type="checkbox"
            aria-label="Autoload Default"
            checked={autoloadProfileId === selectedProfileId}
            disabled={profiles.length === 0 || catalogBusy}
            onChange={(event) => void handleAutoloadToggle(event.target.checked)}
          />
          <span>启动时自动加载</span>
        </label>
      </div>
      <div className="engine-profile-order" aria-busy={catalogBusy}>
        <h3>{t("engineOrder.title")}</h3>
        <p className="message">{t("engineOrder.hint")}</p>
        <ul aria-label={t("engineOrder.title")}>
          {profiles.map((record, index) => <li key={record.id} data-profile-order-id={record.id}>
            <strong>{record.profile.name}</strong>
            <div className="engine-profile-order-actions">
              {(["top", "up", "down", "bottom"] as const).map((direction) => <button key={direction} type="button"
                aria-label={`${t(`engineOrder.${direction}`)} ${record.profile.name}`}
                disabled={disabled || catalogBusy || ((direction === "top" || direction === "up") ? index === 0 : index === profiles.length - 1)}
                onClick={() => void handleReorder(record.id, direction)}>{t(`engineOrder.${direction}`)}</button>)}
            </div>
          </li>)}
        </ul>
        {orderStatus ? <p className="message" role="status">{orderStatus}</p> : null}
      </div>
      <div className="engine-grid">
        <label>
          <span>适配器</span>
          <select value={adapterKind} onChange={(event) => { setAdapterKind(event.target.value as EngineBackendDto); setAssetChecks([]); }}>
            <option value="kata_go_analysis">KataGoAnalysis</option>
            <option value="kata_go_gtp">{t("engineRules.protocol")}</option>
            <option value="generic_gtp">GenericGtp</option>
          </select>
        </label>
        <label>
          <span>引擎</span>
          <div className="path-input-row">
            <input value={enginePath} onChange={(event) => updatePath(setEnginePath, event.target.value)} placeholder="/path/to/katago" aria-invalid={isKnownMissing(assetChecks, "engine binary")} title={pathCheckTitle(assetChecks, "engine binary")} />
            <button type="button" className="path-picker-button" onClick={() => void handlePickPath("引擎", enginePath, false, setEnginePath)}>浏览</button>
          </div>
        </label>
        {adapterKind !== "generic_gtp" ? <>
        <label>
          <span>模型</span>
          <div className="path-input-row">
            <input data-search-target="engine.model-path" value={modelPath} onChange={(event) => updatePath(setModelPath, event.target.value)} placeholder="/path/to/model.bin.gz" aria-invalid={isKnownMissing(assetChecks, "model")} title={pathCheckTitle(assetChecks, "model")} />
            <button type="button" className="path-picker-button" onClick={() => void handlePickPath("模型", modelPath, false, setModelPath)}>浏览</button>
          </div>
        </label>
        <label>
          <span>配置文件</span>
          <div className="path-input-row">
            <input data-search-target="engine.config-path" value={configPath} onChange={(event) => updatePath(setConfigPath, event.target.value)} placeholder="/path/to/analysis.cfg" aria-invalid={isKnownMissing(assetChecks, "config")} title={pathCheckTitle(assetChecks, "config")} />
            <button type="button" className="path-picker-button" onClick={() => void handlePickPath("配置文件", configPath, false, setConfigPath)}>浏览</button>
          </div>
        </label>
        </> : null}
        <label>
          <span>工作目录</span>
          <div className="path-input-row">
            <input value={workingDir} onChange={(event) => updatePath(setWorkingDir, event.target.value)} placeholder="可选" aria-invalid={isKnownMissing(assetChecks, "working directory")} title={pathCheckTitle(assetChecks, "working directory")} />
            <button type="button" className="path-picker-button" onClick={() => void handlePickPath("工作目录", workingDir, true, setWorkingDir)}>浏览</button>
          </div>
        </label>
      </div>
      {adapterKind !== "generic_gtp" && <ModelInventoryPanel key={selectedProfileId}
        modelPath={modelPath} workingDir={workingDir}
        savedModelPath={editingProfile && editingProfile.adapter_kind !== "generic_gtp" ? editingProfile.settings.model_path ?? "" : ""}
        disabled={disabled || catalogBusy} beginOperation={beginModelOperation} endOperation={endModelOperation}
        onSelect={(path) => updatePath(setModelPath, path)} />}
      <div className="engine-grid" aria-label="启动参数">
        {argv.map((argument, index) => (
          <label key={index}>
            <span>参数 {index + 1}</span>
            <div className="path-input-row">
              <input aria-label={`参数 ${index + 1}`} value={argument} onChange={(event) => setArgv((current) => current.map((value, item) => item === index ? event.target.value : value))} />
              <button type="button" className="path-picker-button" aria-label={`删除参数 ${index + 1}`} onClick={() => setArgv((current) => current.filter((_, item) => item !== index))}>删除</button>
            </div>
          </label>
        ))}
        <button type="button" onClick={() => setArgv((current) => [...current, ""])}>新增参数</button>
      </div>
      <p className="message">每项是一个原样传递的参数；空值、空格和中文不会拆分或经 shell 解释。</p>
      <p className="message">已保存配置与当前草稿的能力均待 run 验证。保存只更新目录；不会改变当前 run 的已验证能力。</p>
      <p className="message">{adapterKind === "generic_gtp"
        ? "静态 adapter 上限：GenericGtp 不提供 rich-analysis；落子是否可用及精确局面范围以当前 run 的资格验证为准。"
        : adapterKind === "kata_go_gtp" ? t("engineRules.boundary")
        : "静态 adapter 上限：KataGoAnalysis 可提供单点、连续、整谱/task、候选/PV、胜率/分数、ownership/policy、visits 限制与协议取消；实际能力以当前 run 验证结果为准。"}</p>
      {adapterKind === "generic_gtp" ? <p className="message" role="status">启动后验证 GTP v2、引擎名称、版本与命令列表；保存配置不会验证协议或改变当前 run。</p> : null}
      <p className="message" aria-label="当前 run 能力">{verifiedEngineCapabilitiesLabel(snapshot)}</p>
      <EngineResourceDetails run={run} />
      {engineFailure && <section role="alert" className="message engine-resource-message">
        <h4>{t("engineResource.failure")}: {engineFailure.kind}</h4>
        <p>{t("engineResource.profile")}: {engineFailure.profile_id} · {t("engineResource.run")}: {engineFailure.run_id}</p>
        <p>{engineFailure.message}</p>
        {engineFailure.diagnostic_summary && <p>{engineFailure.diagnostic_summary}</p>}
        <p>{t("engineResource.repair")}</p>
      </section>}
      <div className="engine-run-row">
        {adapterKind === "kata_go_analysis" ?
        <label>
          <span>最大计算量</span>
          <input type="number" min={1} step={1} value={maxVisits} onChange={(event) => setMaxVisits(event.target.value)} />
        </label>
        : null}
        <button onClick={() => void handleSaveProfile()} disabled={!canSave || catalogBusy}>保存配置</button>
        <button type="button" onClick={() => void handleReloadProfiles()} disabled={catalogBusy}>重新加载配置</button>
        <button onClick={() => void handleCheckAssets()} disabled={disabled}>检查资源</button>
      </div>
      {pendingChanges ? <p className="message" role="status">存在待应用更改。只有显式 Restart 才会替换当前 Foreground Engine Run。</p> : null}
      <p className="message">{profileStatus}</p>
      {assetChecks.length > 0 && (
        <p className="message">
          {assetChecks.map((check) => `${check.exists ? "有" : "缺"} ${check.label}${check.path ? `: ${check.path}` : ""}`).join(" | ")}
        </p>
      )}
      </fieldset>
    </section>
  );
}

function optionalPath(value: string, previous: string | null | undefined): string | null {
  return value.length > 0 || previous === "" ? value : null;
}

function assetStatus(checks: AssetCheckDto[]): string {
  const missingRequired = checks.filter((check) => check.required && !check.exists);
  if (missingRequired.length === 0) return "Assets ready.";
  return `Missing required: ${missingRequired.map((check) => `${check.label}${check.path ? ` (${check.path})` : ""}`).join(", ")}.`;
}

function isKnownMissing(checks: AssetCheckDto[], label: string): boolean {
  return checks.some((check) => check.label === label && check.required && !check.exists);
}

function pathCheckTitle(checks: AssetCheckDto[], label: string): string | undefined {
  const check = checks.find((item) => item.label === label);
  if (!check) return undefined;
  return check.exists ? `Resolved path: ${check.path}` : `Missing required ${label}${check.path ? `: ${check.path}` : ""}`;
}

function nextProfileName(profiles: EngineProfileRecordDto[]): string {
  const existing = new Set(profiles.map((profile) => profile.profile.name));
  let index = profiles.length + 1;
  let name = `KataGo Profile ${index}`;
  while (existing.has(name)) {
    index += 1;
    name = `KataGo Profile ${index}`;
  }
  return name;
}

function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (error && typeof error === "object" && "message" in error && typeof (error as { message: unknown }).message === "string") {
    return (error as { message: string }).message;
  }
  return String(error);
}
