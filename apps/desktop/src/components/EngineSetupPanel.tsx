import { useEffect, useState } from "react";
import { createPortal } from "react-dom";
import { open } from "@tauri-apps/plugin-dialog";
import { checkEngineAssets, loadEngineProfilesSettings, saveEngineProfilesSettings } from "../api/backend";
import type { AssetCheckDto, EngineProfileDto, EngineProfileRecordDto } from "../domain/types";

type EngineCommands = {
  analyzeOnce: () => void;
  analyzeGame: () => void;
  canRun: boolean;
  engineLabel: string;
};

type Props = {
  disabled?: boolean;
  onRun: (profile: EngineProfileDto, maxVisits: number) => void | Promise<void>;
  onAnalyzeGame: (profile: EngineProfileDto, maxVisits: number) => void | Promise<void>;
  onCancelAnalysis?: () => void | Promise<void>;
  analysisProgress?: { completed: number; expected: number; turn: number; responseJsonl: string } | null;
  activeJobId?: string | null;
  toolbarTarget?: HTMLElement | null;
  onBindCommands?: (commands: EngineCommands) => void;
};

export function EngineSetupPanel({ disabled = false, onRun, onAnalyzeGame, onCancelAnalysis, analysisProgress = null, activeJobId = null, toolbarTarget = null, onBindCommands }: Props) {
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

  const visits = Number(maxVisits);
  const isAnalysisActive = activeJobId !== null;
  const missingRequiredAssets = assetChecks.filter((check) => check.required && !check.exists);
  const hasKnownMissingRequiredAssets = missingRequiredAssets.length > 0;
  const progressLabel = analysisProgress
    ? `${analysisProgress.completed}/${analysisProgress.expected || "?"} 个局面，第 ${analysisProgress.turn} 手`
    : isAnalysisActive
      ? "正在起分析…"
      : "";
  const progressPercent = analysisProgress && analysisProgress.expected > 0
    ? Math.min(100, Math.round((analysisProgress.completed / analysisProgress.expected) * 100))
    : 0;
  const canRun =
    !disabled &&
    enginePath.trim().length > 0 &&
    modelPath.trim().length > 0 &&
    configPath.trim().length > 0 &&
    Number.isFinite(visits) &&
    visits > 0 &&
    !hasKnownMissingRequiredAssets;
  const canSave = profileName.trim().length > 0 && Number.isFinite(visits) && visits > 0;

  useEffect(() => {
    onBindCommands?.({
      analyzeOnce: handleRun,
      analyzeGame: handleAnalyzeGame,
      canRun,
      engineLabel: enginePath.trim() ? (profileName.trim() || "已加载引擎") : "未加载引擎"
    });
  }, [canRun, enginePath, profileName, onBindCommands, visits, disabled, hasKnownMissingRequiredAssets]);
  const canDeleteProfile = selectedProfileId !== "default" && profiles.length > 1;

  useEffect(() => {
    let isMounted = true;
    loadEngineProfilesSettings()
      .then((settings) => {
        if (!isMounted) return;
        const selected = settings.profiles.find((profile) => profile.id === settings.selected_profile_id) ?? settings.profiles[0];
        setProfiles(settings.profiles);
        setSelectedProfileId(selected?.id ?? "default");
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

  async function persistProfiles(nextProfiles: EngineProfileRecordDto[], selectedId: string, successMessage: string) {
    const saved = await saveEngineProfilesSettings({ selected_profile_id: selectedId, profiles: nextProfiles });
    const selected = saved.profiles.find((profile) => profile.id === saved.selected_profile_id) ?? saved.profiles[0];
    setProfiles(saved.profiles);
    setSelectedProfileId(saved.selected_profile_id);
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
      await saveEngineProfilesSettings({ selected_profile_id: profileId, profiles });
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
      await persistProfiles([...profiles.map((profile) => profile.id === selectedProfileId ? buildProfileRecord(profile.id) : profile), nextProfile], id, "Profile added.");
    } catch (error) {
      setProfileStatus(`Add failed: ${errorMessage(error)}`);
    }
  }

  async function handleDeleteProfile() {
    if (!canDeleteProfile) return;
    const nextProfiles = profiles.filter((profile) => profile.id !== selectedProfileId);
    const nextSelected = nextProfiles.find((profile) => profile.id === "default")?.id ?? nextProfiles[0]?.id ?? "default";
    try {
      await persistProfiles(nextProfiles, nextSelected, "Profile deleted.");
    } catch (error) {
      setProfileStatus(`Delete failed: ${errorMessage(error)}`);
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

  function handleRun() {
    if (hasKnownMissingRequiredAssets) {
      setProfileStatus(assetStatus(assetChecks));
      return;
    }
    if (!canRun) return;
    void onRun(buildProfile(), Math.floor(visits));
  }

  function handleAnalyzeGame() {
    if (hasKnownMissingRequiredAssets) {
      setProfileStatus(assetStatus(assetChecks));
      return;
    }
    if (!canRun) return;
    void onAnalyzeGame(buildProfile(), Math.floor(visits));
  }

  async function handleSaveProfile() {
    if (!canSave) return;
    try {
      const currentRecord = buildProfileRecord(selectedProfileId);
      const nextProfiles = profiles.some((profile) => profile.id === selectedProfileId)
        ? profiles.map((profile) => profile.id === selectedProfileId ? currentRecord : profile)
        : [...profiles, currentRecord];
      await persistProfiles(nextProfiles, selectedProfileId, "Profile saved.");
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

  const runActions = (
    <>
      <button className="primary" onClick={handleRun} disabled={!canRun}>{disabled ? "分析中…" : "分析此手"}</button>
      <button onClick={handleAnalyzeGame} disabled={!canRun}>{disabled ? "分析中…" : "分析全局"}</button>
      {isAnalysisActive && <button onClick={() => void onCancelAnalysis?.()} disabled={!onCancelAnalysis}>取消</button>}
      {(isAnalysisActive || analysisProgress) && (
        <div className="analysis-progress" aria-live="polite">
          <div className="analysis-progress-track">
            <span style={{ width: `${progressPercent}%` }} />
          </div>
          <span>{progressLabel}</span>
        </div>
      )}
    </>
  );

  return (
    <section className="engine-setup-panel" aria-label="KataGo 引擎">
      {toolbarTarget ? createPortal(runActions, toolbarTarget) : <div className="engine-run-row">{runActions}</div>}
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
  return error instanceof Error ? error.message : String(error);
}
