import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { checkEngineAssets, loadEngineProfilesSettings, saveEngineProfilesSettings } from "../api/backend";
import type { AssetCheckDto, EngineProfileDto, EngineProfileRecordDto, ForegroundEngineSnapshotDto } from "../domain/types";
import { profileHasPendingChanges, runFromSnapshot } from "../domain/foregroundEngine";

type Props = {
  disabled?: boolean;
  engineSnapshot?: ForegroundEngineSnapshotDto | null;
};

export function EngineSetupPanel({ disabled = false, engineSnapshot = null }: Props) {
  const [profiles, setProfiles] = useState<EngineProfileRecordDto[]>([]);
  const [selectedProfileId, setSelectedProfileId] = useState("default");
  const [profileName, setProfileName] = useState("Local KataGo");
  const [enginePath, setEnginePath] = useState("");
  const [modelPath, setModelPath] = useState("");
  const [configPath, setConfigPath] = useState("");
  const [workingDir, setWorkingDir] = useState("");
  const [maxVisits, setMaxVisits] = useState("800");
  const [profileStatus, setProfileStatus] = useState("Loading profile...");
  const [assetChecks, setAssetChecks] = useState<AssetCheckDto[]>([]);
  const [autoloadProfileId, setAutoloadProfileId] = useState<string | null>(null);

  const visits = Number(maxVisits);
  const canSave = profileName.trim().length > 0 && Number.isFinite(visits) && visits > 0;
  const canDeleteProfile = selectedProfileId !== "default" && profiles.length > 1;
  const snapshot = engineSnapshot ?? { revision: 0, lifecycle: { state: "no_engine" as const } };
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
        setSelectedProfileId(selected?.id ?? "default");
        setAutoloadProfileId(settings.autoload_profile_id ?? null);
        if (!selected) {
          setProfileStatus("Default profile ready.");
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
  }, []);

  function applyProfileRecord(record: EngineProfileRecordDto) {
    setProfileName(record.profile.name);
    setEnginePath(record.profile.engine_path);
    setModelPath(record.profile.model_path ?? "");
    setConfigPath(record.profile.config_path ?? "");
    setWorkingDir(record.profile.working_dir ?? "");
    setMaxVisits(String(record.max_visits));
    setAssetChecks([]);
  }

  function buildProfile(): EngineProfileDto {
    return {
      name: profileName.trim() || "Local KataGo",
      engine_path: enginePath.trim(),
      model_path: optionalPath(modelPath),
      config_path: optionalPath(configPath),
      working_dir: optionalPath(workingDir),
      backend: "kata_go_analysis"
    };
  }

  function buildProfileRecord(id = selectedProfileId): EngineProfileRecordDto {
    return {
      id,
      profile: buildProfile(),
      max_visits: Math.floor(visits)
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
    const saved = await saveEngineProfilesSettings({
      selected_profile_id: selectedId,
      autoload_profile_id: autoloadId,
      profiles: nextProfiles
    });
    const selected = saved.profiles.find((profile) => profile.id === saved.selected_profile_id) ?? saved.profiles[0];
    setProfiles(saved.profiles);
    setSelectedProfileId(saved.selected_profile_id);
    setAutoloadProfileId(saved.autoload_profile_id ?? null);
    if (selected) applyProfileRecord(selected);
    setProfileStatus(successMessage);
  }

  async function handleSelectProfile(profileId: string) {
    const profile = profiles.find((item) => item.id === profileId);
    if (!profile) return;
    setSelectedProfileId(profileId);
    applyProfileRecord(profile);
    setProfileStatus(`Selected ${profile.profile.name}.`);
    try {
      await saveEngineProfilesSettings({
        selected_profile_id: profileId,
        autoload_profile_id: autoloadProfileId,
        profiles
      });
    } catch (error) {
      setProfileStatus(`Select saved locally but persistence failed: ${errorMessage(error)}`);
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
    try {
      const selected = await open({
        title: `Select ${label}`,
        directory,
        multiple: false,
        defaultPath: currentValue.trim() || undefined
      });
      const selectedPath = Array.isArray(selected) ? selected[0] : selected;
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

  return (
    <section className="engine-setup-panel" aria-label="引擎设置">
      <div className="engine-run-row">
        <label>
          <span>配置</span>
          <select value={selectedProfileId} onChange={(event) => void handleSelectProfile(event.target.value)}>
            {profiles.map((profile) => (
              <option key={profile.id} value={profile.id}>{profile.profile.name}</option>
            ))}
          </select>
        </label>
        <label>
          <span>名称</span>
          <input value={profileName} onChange={(event) => setProfileName(event.target.value)} placeholder="本地 KataGo" />
        </label>
        <button type="button" onClick={() => void handleAddProfile()} disabled={!canSave}>新增</button>
        <button type="button" onClick={() => void handleDeleteProfile()} disabled={!canDeleteProfile}>删除</button>
        <label>
          <input
            type="checkbox"
            aria-label="Autoload Default"
            checked={autoloadProfileId === selectedProfileId}
            disabled={profiles.length === 0}
            onChange={(event) => void handleAutoloadToggle(event.target.checked)}
          />
          <span>启动时自动加载</span>
        </label>
      </div>
      <div className="engine-grid">
        <label>
          <span>引擎</span>
          <div className="path-input-row">
            <input value={enginePath} onChange={(event) => updatePath(setEnginePath, event.target.value)} placeholder="/path/to/katago" aria-invalid={isKnownMissing(assetChecks, "engine binary")} title={pathCheckTitle(assetChecks, "engine binary")} />
            <button type="button" className="path-picker-button" onClick={() => void handlePickPath("引擎", enginePath, false, setEnginePath)}>浏览</button>
          </div>
        </label>
        <label>
          <span>模型</span>
          <div className="path-input-row">
            <input value={modelPath} onChange={(event) => updatePath(setModelPath, event.target.value)} placeholder="/path/to/model.bin.gz" aria-invalid={isKnownMissing(assetChecks, "model")} title={pathCheckTitle(assetChecks, "model")} />
            <button type="button" className="path-picker-button" onClick={() => void handlePickPath("模型", modelPath, false, setModelPath)}>浏览</button>
          </div>
        </label>
        <label>
          <span>配置文件</span>
          <div className="path-input-row">
            <input value={configPath} onChange={(event) => updatePath(setConfigPath, event.target.value)} placeholder="/path/to/analysis.cfg" aria-invalid={isKnownMissing(assetChecks, "config")} title={pathCheckTitle(assetChecks, "config")} />
            <button type="button" className="path-picker-button" onClick={() => void handlePickPath("配置文件", configPath, false, setConfigPath)}>浏览</button>
          </div>
        </label>
        <label>
          <span>工作目录</span>
          <div className="path-input-row">
            <input value={workingDir} onChange={(event) => updatePath(setWorkingDir, event.target.value)} placeholder="可选" aria-invalid={isKnownMissing(assetChecks, "working directory")} title={pathCheckTitle(assetChecks, "working directory")} />
            <button type="button" className="path-picker-button" onClick={() => void handlePickPath("工作目录", workingDir, true, setWorkingDir)}>浏览</button>
          </div>
        </label>
      </div>
      <div className="engine-run-row">
        <label>
          <span>最大计算量</span>
          <input type="number" min={1} step={1} value={maxVisits} onChange={(event) => setMaxVisits(event.target.value)} />
        </label>
        <button onClick={() => void handleSaveProfile()} disabled={!canSave}>保存配置</button>
        <button onClick={() => void handleCheckAssets()} disabled={disabled}>检查资源</button>
      </div>
      {pendingChanges ? <p className="message" role="status">存在待应用更改。只有显式 Restart 才会替换当前 Foreground Engine Run。</p> : null}
      <p className="message">{profileStatus}</p>
      {assetChecks.length > 0 && (
        <p className="message">
          {assetChecks.map((check) => `${check.exists ? "有" : "缺"} ${check.label}${check.path ? `: ${check.path}` : ""}`).join(" | ")}
        </p>
      )}
    </section>
  );
}

function optionalPath(value: string): string | null {
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
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
