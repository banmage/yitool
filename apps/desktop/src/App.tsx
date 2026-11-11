import {
  useEffect,
  useRef,
  useState,
  type FormEvent,
} from "react";
import {
  castMeihua,
  createProfile,
  deleteHistory,
  deleteProfile,
  drawColors,
  errorMessage,
  getFormOptions,
  getHistory,
  getSettings,
  listHistory,
  listProfiles,
  renameProfile,
  resolveEvidence,
  saveSettings,
} from "./api";
import type {
  CalResult,
  CastMode,
  CastingPayload,
  DrawSelection,
  EvidenceEntry,
  FormOptions,
  HistoryDetail,
  InputScheme,
  MeihuaResult,
  Profile,
  RunSummary,
  Setting,
  StepLog,
} from "./types";

const DISCLAIMER =
  "本工具仅供个人学习、文化研究与娱乐，不构成任何决策、医疗、法律或投资建议；请勿据此替代专业意见。";

const NINE_COLOR_NOTICE =
  "九色为本系统自定义扩展象数（L3），不是传统梅花易数原典标准。";

const BAZI_REPRO_NOTICE =
  "本次不含生辰。若启用生辰，仅本次会话有效，重算需重新输入，不承诺脱离原始输入长期复现。";

function localDateTime(): string {
  const now = new Date();
  now.setMinutes(now.getMinutes() - now.getTimezoneOffset());
  return now.toISOString().slice(0, 16);
}

function errorText(value: unknown): string {
  return errorMessage(value);
}

function App() {
  const [options, setOptions] = useState<FormOptions | null>(null);
  const [optionsError, setOptionsError] = useState("");
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [profileId, setProfileId] = useState("");
  const [settings, setSettings] = useState<Setting | null>(null);
  const [settingsError, setSettingsError] = useState("");
  const [scheme, setScheme] = useState<InputScheme>("NINE_COLOR");
  const [mode, setMode] = useState<CastMode>("FIXED");
  const [colorId, setColorId] = useState("");
  const [auxId, setAuxId] = useState("");
  const [traditionalU, setTraditionalU] = useState("9");
  const [traditionalD, setTraditionalD] = useState("16");
  const [datetime, setDatetime] = useState(localDateTime);
  const [timezone, setTimezone] = useState("Asia/Shanghai");
  const [dayBound, setDayBound] = useState<"ZICHU" | "ZIZHENG">("ZICHU");
  const [result, setResult] = useState<CastingPayload | null>(null);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [drawMessage, setDrawMessage] = useState("");
  const [history, setHistory] = useState<RunSummary[]>([]);
  const [historyError, setHistoryError] = useState("");
  const [reportOpen, setReportOpen] = useState(false);
  const [evidence, setEvidence] = useState<EvidenceEntry | null>(null);
  const [evidenceError, setEvidenceError] = useState("");
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [profileOpen, setProfileOpen] = useState(false);
  const [newProfileName, setNewProfileName] = useState("");
  const [shellMessage, setShellMessage] = useState("");
  const [copyMessage, setCopyMessage] = useState("");
  const previousFocus = useRef<HTMLElement | null>(null);

  useEffect(() => {
    let active = true;
    Promise.all([getFormOptions(), listProfiles()])
      .then(([loadedOptions, loadedProfiles]) => {
        if (!active) return;
        setOptions(loadedOptions);
        setColorId(String(loadedOptions.colors[0]?.colorId ?? ""));
        setAuxId(String(loadedOptions.zodiacs[0]?.zodiacId ?? ""));
        setProfiles(loadedProfiles);
        const preferred =
          loadedProfiles.find((profile) => profile.is_default) ?? loadedProfiles[0];
        setProfileId(preferred?.profile_id ?? "");
      })
      .catch((cause: unknown) => {
        if (active) setOptionsError(errorText(cause));
      });
    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    if (!profileId) return;
    let active = true;
    Promise.all([getSettings(profileId), listHistory(profileId)])
      .then(([loadedSettings, loadedHistory]) => {
        if (!active) return;
        setSettings(loadedSettings);
        applySettings(loadedSettings);
        setHistory(loadedHistory);
        setSettingsError("");
        setHistoryError("");
      })
      .catch((cause: unknown) => {
        if (active) setSettingsError(errorText(cause));
      });
    return () => {
      active = false;
    };
  }, [profileId]);

  function applySettings(value: Setting) {
    setMode(value.mode);
    setScheme(value.input_mode);
    setDayBound(value.day_bound);
    setTimezone(value.timezone);
  }

  useEffect(() => {
    if (!reportOpen && !evidence && !settingsOpen && !profileOpen) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        closeDialogs();
        return;
      }
      if (event.key === "Tab") {
        const dialog = document.querySelector<HTMLElement>('[role="dialog"]');
        const focusable = dialog?.querySelectorAll<HTMLElement>(
          'button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
        );
        if (!focusable?.length) {
          event.preventDefault();
          return;
        }
        const first = focusable[0];
        const last = focusable[focusable.length - 1];
        if (event.shiftKey && document.activeElement === first) {
          event.preventDefault();
          last.focus();
        } else if (!event.shiftKey && document.activeElement === last) {
          event.preventDefault();
          first.focus();
        }
      }
    };
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [reportOpen, evidence, settingsOpen, profileOpen]);

  useEffect(() => {
    if (reportOpen || evidence || settingsOpen || profileOpen) {
      document.querySelector<HTMLElement>("[data-dialog-close]")?.focus();
    } else if (previousFocus.current) {
      previousFocus.current.focus();
      previousFocus.current = null;
    }
  }, [reportOpen, evidence, settingsOpen, profileOpen]);

  function rememberFocus(trigger?: HTMLElement) {
    previousFocus.current =
      trigger ??
      (document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null);
  }

  function closeDialogs() {
    setReportOpen(false);
    setEvidence(null);
    setEvidenceError("");
    setSettingsOpen(false);
    setProfileOpen(false);
    setCopyMessage("");
  }

  async function refreshHistory() {
    if (!profileId) return;
    try {
      setHistory(await listHistory(profileId));
      setHistoryError("");
    } catch (cause: unknown) {
      setHistoryError(errorText(cause));
    }
  }

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError("");
    setCopyMessage("");
    setDrawMessage("");
    if (!options) {
      setError(optionsError || "九色与生肖资源尚未准备就绪");
      return;
    }
    if (!profileId) {
      setError("请先选择或创建本地档案");
      return;
    }
    if (!datetime || !timezone.trim()) {
      setError("请完成起卦时间与时区输入");
      return;
    }

    let requestColor: number | undefined;
    let requestAux: number | undefined;
    let traditional: [number, number] | undefined;
    if (scheme === "NINE_COLOR") {
      if (!colorId || !auxId) {
        setError("请选择九色与生肖");
        return;
      }
      requestColor = Number(colorId);
      requestAux = Number(auxId);
    } else {
      const u = Number(traditionalU);
      const d = Number(traditionalD);
      if (!Number.isInteger(u) || !Number.isInteger(d) || u <= 0 || d <= 0) {
        setError("传统数字必须为两个正整数");
        return;
      }
      traditional = [u, d];
    }

    setBusy(true);
    setResult(null);
    try {
      const payload = await castMeihua({
        mode,
        scheme,
        colorId: requestColor,
        auxId: requestAux,
        traditional,
        profileId,
        datetime,
        timezone: timezone.trim(),
        dayBound,
      });
      setResult(payload);
      await refreshHistory();
    } catch (cause: unknown) {
      setError(errorText(cause));
    } finally {
      setBusy(false);
    }
  }

  async function randomDraw() {
    setDrawMessage("");
    try {
      const draw: DrawSelection = await drawColors();
      setScheme("NINE_COLOR");
      setColorId(String(draw.colorId));
      setAuxId(String(draw.auxId));
      setDrawMessage(
        `已抽到九色 ${draw.colorId}、生肖 ${draw.auxId}；结果将写入本次输入快照。`,
      );
    } catch (cause: unknown) {
      setError(errorText(cause));
    }
  }

  async function openEvidence(href: string, trigger: HTMLElement) {
    rememberFocus(trigger);
    setEvidence(null);
    setEvidenceError("");
    try {
      setEvidence(await resolveEvidence(href));
    } catch (cause: unknown) {
      setEvidenceError(errorText(cause));
    }
  }

  async function copyReport() {
    if (!result) return;
    try {
      if (!navigator.clipboard?.writeText) {
        throw new Error("此环境未提供系统剪贴板；请在下方文本中手动选择并复制");
      }
      await navigator.clipboard.writeText(result.reportMarkdown);
      setCopyMessage("已复制完整报告。");
    } catch (cause: unknown) {
      setCopyMessage(
        `复制失败：${errorText(cause)}。报告文本仍可手动选择。`,
      );
    }
  }

  function downloadReport() {
    if (!result) return;
    try {
      const blob = new Blob([result.reportMarkdown], {
        type: "text/markdown;charset=utf-8",
      });
      const url = URL.createObjectURL(blob);
      const anchor = document.createElement("a");
      anchor.href = url;
      anchor.download = `${result.runId}.md`;
      anchor.click();
      window.setTimeout(() => URL.revokeObjectURL(url), 30_000);
      setCopyMessage("已请求下载 Markdown 报告。");
    } catch (cause: unknown) {
      setCopyMessage(
        `下载失败：${errorText(cause)}。报告文本仍可手动选择。`,
      );
    }
  }

  async function openHistory(run: RunSummary) {
    setError("");
    try {
      const detail = await getHistory(profileId, run.run_id);
      if (!detail) {
        setError("历史记录不可用或已被删除");
        return;
      }
      const payload = historyToPayload(detail);
      setResult(payload);
      setReportOpen(true);
    } catch (cause: unknown) {
      setError(errorText(cause));
    }
  }

  async function removeHistory(run: RunSummary) {
    if (!window.confirm(`删除历史记录 ${run.run_id}？此操作不可撤销。`)) {
      return;
    }
    try {
      await deleteHistory(profileId, run.run_id);
      await refreshHistory();
      if (result?.runId === run.run_id) setResult(null);
    } catch (cause: unknown) {
      setHistoryError(errorText(cause));
    }
  }

  async function saveCurrentSettings(next: Setting) {
    try {
      await saveSettings(next);
      setSettings(next);
      applySettings(next);
      setSettingsError("");
      setSettingsOpen(false);
      await refreshHistory();
    } catch (cause: unknown) {
      setSettingsError(errorText(cause));
    }
  }

  async function addProfile() {
    if (!newProfileName.trim()) {
      setShellMessage("请输入档案名称");
      return;
    }
    try {
      const created = await createProfile(newProfileName.trim());
      setProfiles(await listProfiles());
      setProfileId(created.profile_id);
      setNewProfileName("");
      setShellMessage(`已创建档案：${created.display_name}`);
    } catch (cause: unknown) {
      setShellMessage(errorText(cause));
    }
  }

  async function renameCurrentProfile(profile: Profile) {
    const next = window.prompt("新的档案名称", profile.display_name);
    if (!next?.trim()) return;
    try {
      await renameProfile(profile.profile_id, next.trim());
      setProfiles(await listProfiles());
      setShellMessage("档案已重命名");
    } catch (cause: unknown) {
      setShellMessage(errorText(cause));
    }
  }

  async function removeProfile(profile: Profile) {
    if (profile.is_default) {
      setShellMessage("默认档案不可删除");
      return;
    }
    if (!window.confirm(`删除档案“${profile.display_name}”及其全部历史？`)) {
      return;
    }
    try {
      await deleteProfile(profile.profile_id);
      const rest = await listProfiles();
      setProfiles(rest);
      setProfileId(rest.find((item) => item.is_default)?.profile_id ?? rest[0]?.profile_id ?? "");
      setResult(null);
      setShellMessage("档案已删除；导出文件与系统备份不受本操作控制");
    } catch (cause: unknown) {
      setShellMessage(errorText(cause));
    }
  }

  const activeError = optionsError || error;
  const currentProfile = profiles.find((profile) => profile.profile_id === profileId);

  return (
    <main className="app-shell">
      <header className="app-header">
        <div>
          <p className="eyebrow">离线 · 本地 · 可溯源</p>
          <h1>梅花易数学习</h1>
          <p className="subtitle">阶段 1 · {scheme === "NINE_COLOR" ? "九色" : "传统数字"}单引擎</p>
        </div>
        <div className="header-actions">
          <label className="compact-field">
            档案
            <select
              value={profileId}
              onChange={(event) => {
                setProfileId(event.target.value);
                setResult(null);
              }}
              aria-label="当前档案"
            >
              {profiles.map((profile) => (
                <option key={profile.profile_id} value={profile.profile_id}>
                  {profile.display_name}
                </option>
              ))}
            </select>
          </label>
          <button
            type="button"
            className="secondary-button"
            onClick={(event) => {
              rememberFocus(event.currentTarget);
              setProfileOpen(true);
            }}
          >
            档案与设置
          </button>
          <span className="offline-badge">不连接网络</span>
        </div>
      </header>

      <aside className="notice" role="note">
        <strong>学习与娱乐用途</strong>
        <p>{DISCLAIMER}</p>
        <p className="muted-text">{NINE_COLOR_NOTICE}</p>
        <p className="muted-text">{BAZI_REPRO_NOTICE}</p>
      </aside>

      {shellMessage && (
        <p className="info-message" role="status">
          {shellMessage}
        </p>
      )}

      <section className="panel" aria-labelledby="cast-heading">
        <div className="section-heading">
          <div>
            <p className="eyebrow">输入</p>
            <h2 id="cast-heading">梅花起卦</h2>
            <p className="muted-text">
              {scheme === "NINE_COLOR" ? NINE_COLOR_NOTICE : "传统数字模式用于学习对照，规则层级为 L2。"}
            </p>
          </div>
          <span className="resource-status">
            {options ? "本地资源校验通过" : "正在校验本地资源"}
          </span>
        </div>

        <div className="mode-tabs" role="tablist" aria-label="输入模式">
          <button
            type="button"
            role="tab"
            aria-selected={scheme === "NINE_COLOR"}
            className={scheme === "NINE_COLOR" ? "tab active" : "tab"}
            onClick={() => setScheme("NINE_COLOR")}
          >
            九色模式
          </button>
          <button
            type="button"
            role="tab"
            aria-selected={scheme === "TRADITIONAL_NUMBER"}
            className={scheme === "TRADITIONAL_NUMBER" ? "tab active" : "tab"}
            onClick={() => setScheme("TRADITIONAL_NUMBER")}
          >
            传统数字
          </button>
        </div>

        <form className="input-grid" onSubmit={submit}>
          {scheme === "NINE_COLOR" ? (
            <>
              <label>
                九色
                <select
                  value={colorId}
                  onChange={(event) => setColorId(event.target.value)}
                  required
                  disabled={!options || busy}
                >
                  {(options?.colors ?? []).map((color) => (
                    <option key={color.colorId} value={color.colorId}>
                      {color.name} · {color.element}
                    </option>
                  ))}
                </select>
              </label>
              <label>
                生肖
                <select
                  value={auxId}
                  onChange={(event) => setAuxId(event.target.value)}
                  required
                  disabled={!options || busy}
                >
                  {(options?.zodiacs ?? []).map((zodiac) => (
                    <option key={zodiac.zodiacId} value={zodiac.zodiacId}>
                      {zodiac.name} · {zodiac.branch}
                    </option>
                  ))}
                </select>
              </label>
              <div className="form-actions inline-actions">
                <button
                  type="button"
                  className="secondary-button"
                  onClick={randomDraw}
                  disabled={busy}
                >
                  随机抽卡
                </button>
                {drawMessage && <p className="muted-text">{drawMessage}</p>}
              </div>
            </>
          ) : (
            <>
              <label>
                上卦数字 U0
                <input
                  type="number"
                  min={1}
                  step={1}
                  value={traditionalU}
                  onChange={(event) => setTraditionalU(event.target.value)}
                  required
                  disabled={busy}
                />
              </label>
              <label>
                下卦数字 D0
                <input
                  type="number"
                  min={1}
                  step={1}
                  value={traditionalD}
                  onChange={(event) => setTraditionalD(event.target.value)}
                  required
                  disabled={busy}
                />
              </label>
            </>
          )}
          <label>
            推演模式
            <select
              value={mode}
              onChange={(event) => setMode(event.target.value as CastMode)}
              disabled={busy}
            >
              <option value="FIXED">固定模式（可复现）</option>
              <option value="CAST">摇卦模式（保存快照）</option>
            </select>
          </label>
          <label>
            起卦本地时间
            <input
              type="datetime-local"
              value={datetime}
              onChange={(event) => setDatetime(event.target.value)}
              required
              disabled={busy}
            />
          </label>
          <label>
            IANA 时区
            <input
              type="text"
              value={timezone}
              onChange={(event) => setTimezone(event.target.value)}
              placeholder="Asia/Shanghai"
              autoComplete="off"
              required
              disabled={busy}
            />
          </label>
          <label>
            日界
            <select
              value={dayBound}
              onChange={(event) =>
                setDayBound(event.target.value as "ZICHU" | "ZIZHENG")
              }
              disabled={busy}
            >
              <option value="ZICHU">子初换日（23:00）</option>
              <option value="ZIZHENG">子正换日（00:00）</option>
            </select>
          </label>
          <div className="form-actions">
            <button type="submit" className="primary-button" disabled={busy || !options}>
              {busy ? "推演中…" : "开始推演"}
            </button>
          </div>
        </form>
        {activeError && (
          <p className="error-message" role="alert">
            {activeError}
          </p>
        )}
      </section>

      {result && (
        <ResultView
          payload={result}
          onOpenReport={(trigger) => {
            rememberFocus(trigger);
            setCopyMessage("");
            setReportOpen(true);
          }}
          onOpenEvidence={openEvidence}
        />
      )}

      <section className="panel history-panel" aria-labelledby="history-heading">
        <div className="section-heading">
          <div>
            <p className="eyebrow">本地历史</p>
            <h2 id="history-heading">推演记录</h2>
            <p className="muted-text">
              当前档案：{currentProfile?.display_name ?? "未选择"}；只保存非敏感记录。
            </p>
          </div>
          <button type="button" className="secondary-button" onClick={refreshHistory}>
            刷新
          </button>
        </div>
        {historyError && (
          <p className="error-message" role="alert">
            {historyError}
          </p>
        )}
        {history.length === 0 ? (
          <p className="muted-text">当前档案还没有历史记录。</p>
        ) : (
          <ul className="history-list">
            {history.map((run) => (
              <li key={run.run_id} className="history-item">
                <div>
                  <p className="eyebrow">
                    {run.created_at} · {run.mode} · {run.scheme}
                  </p>
                  <h3>
                    {run.original_name} → {run.changed_name} · 第 {run.moving_line} 爻
                  </h3>
                  <p className="muted-text">
                    {run.run_id} · {run.status}
                    {run.formal_allowed ? "" : " · 待校核"}
                  </p>
                </div>
                <div className="history-actions">
                  <button
                    type="button"
                    className="text-button"
                    onClick={() => openHistory(run)}
                  >
                    打开复盘
                  </button>
                  <button
                    type="button"
                    className="text-button danger"
                    onClick={() => removeHistory(run)}
                  >
                    删除
                  </button>
                </div>
              </li>
            ))}
          </ul>
        )}
      </section>

      {reportOpen && result && (
        <section
          className="dialog-backdrop"
          role="presentation"
          onMouseDown={(event) => {
            if (event.target === event.currentTarget) closeDialogs();
          }}
        >
          <div
            className="dialog-panel report-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="report-title"
          >
            <div className="dialog-heading">
              <div>
                <p className="eyebrow">复制、下载与窗口展示使用同一份文本</p>
                <h2 id="report-title">完整推演过程</h2>
              </div>
              <button
                type="button"
                className="icon-button"
                aria-label="关闭报告"
                data-dialog-close
                onClick={closeDialogs}
              >
                ×
              </button>
            </div>
            <div className="dialog-actions">
              <button type="button" className="secondary-button" onClick={copyReport}>
                复制 Markdown
              </button>
              <button type="button" className="secondary-button" onClick={downloadReport}>
                下载 .md
              </button>
            </div>
            <p className="muted-text">
              下载默认文件名为 {result.runId}.md；内容与当前窗口逐字一致。
            </p>
            {copyMessage && (
              <p className="info-message" role="status">
                {copyMessage}
              </p>
            )}
            <pre className="report-text">
              {result.reportMarkdown}
            </pre>
          </div>
        </section>
      )}

      {evidence && (
        <section
          className="dialog-backdrop"
          role="presentation"
          onMouseDown={(event) => {
            if (event.target === event.currentTarget) closeDialogs();
          }}
        >
          <div
            className="dialog-panel evidence-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="evidence-title"
          >
            <div className="dialog-heading">
              <div>
                <p className="eyebrow">只读本地依据</p>
                <h2 id="evidence-title">{evidence.title}</h2>
              </div>
              <button
                type="button"
                className="icon-button"
                aria-label="关闭依据详情"
                data-dialog-close
                onClick={closeDialogs}
              >
                ×
              </button>
            </div>
            <EvidenceDetails entry={evidence} />
          </div>
        </section>
      )}

      {evidenceError && !evidence && (
        <section className="dialog-backdrop" role="presentation">
          <div
            className="dialog-panel evidence-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="evidence-error-title"
          >
            <div className="dialog-heading">
              <div>
                <p className="eyebrow">依据读取失败</p>
                <h2 id="evidence-error-title">链接不可用/依据缺失</h2>
              </div>
              <button
                type="button"
                className="icon-button"
                aria-label="关闭依据详情"
                data-dialog-close
                onClick={closeDialogs}
              >
                ×
              </button>
            </div>
            <p className="error-message" role="alert">
              {evidenceError}
            </p>
          </div>
        </section>
      )}

      {settingsOpen && settings && (
        <SettingsDialog
          setting={settings}
          error={settingsError}
          onClose={closeDialogs}
          onSave={saveCurrentSettings}
        />
      )}

      {profileOpen && (
        <section
          className="dialog-backdrop"
          role="presentation"
          onMouseDown={(event) => {
            if (event.target === event.currentTarget) closeDialogs();
          }}
        >
          <div
            className="dialog-panel settings-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="profiles-title"
          >
            <div className="dialog-heading">
              <div>
                <p className="eyebrow">单库多档案 · 本地隔离</p>
                <h2 id="profiles-title">档案与设置</h2>
              </div>
              <button
                type="button"
                className="icon-button"
                aria-label="关闭档案与设置"
                data-dialog-close
                onClick={closeDialogs}
              >
                ×
              </button>
            </div>
            <ul className="profile-list">
              {profiles.map((profile) => (
                <li key={profile.profile_id} className="profile-item">
                  <div>
                    <strong>{profile.display_name}</strong>
                    {profile.is_default && <span className="muted-text"> · 默认</span>}
                    <p className="muted-text">{profile.profile_id}</p>
                  </div>
                  <div className="history-actions">
                    <button
                      type="button"
                      className="text-button"
                      onClick={() => setProfileId(profile.profile_id)}
                    >
                      切换
                    </button>
                    <button
                      type="button"
                      className="text-button"
                      onClick={() => renameCurrentProfile(profile)}
                    >
                      重命名
                    </button>
                    <button
                      type="button"
                      className="text-button danger"
                      disabled={profile.is_default}
                      onClick={() => removeProfile(profile)}
                    >
                      删除
                    </button>
                  </div>
                </li>
              ))}
            </ul>
            <div className="inline-form">
              <label>
                新档案名称
                <input
                  type="text"
                  maxLength={50}
                  value={newProfileName}
                  onChange={(event) => setNewProfileName(event.target.value)}
                  placeholder="例如：家人 B"
                />
              </label>
              <button type="button" className="primary-button" onClick={addProfile}>
                创建档案
              </button>
            </div>
            {shellMessage && (
              <p className="info-message" role="status">
                {shellMessage}
              </p>
            )}
            <div className="dialog-actions">
              <button
                type="button"
                className="secondary-button"
                onClick={() => {
                  closeDialogs();
                  setSettingsOpen(true);
                }}
              >
                打开当前档案设置
              </button>
            </div>
          </div>
        </section>
      )}

      <footer className="app-footer">
        <p>{DISCLAIMER}</p>
        <p>
          历史与设置仅保存在本机 SQLite；不保存生辰或问事原文。应用内删除不控制导出文件与操作系统备份。
        </p>
      </footer>
    </main>
  );
}

function historyToPayload(detail: HistoryDetail): CastingPayload {
  const result = JSON.parse(detail.report.result_json) as MeihuaResult;
  const cal = JSON.parse(detail.report.cal_json) as CalResult;
  return {
    runId: detail.run.run_id,
    createdAt: detail.run.created_at,
    mode: detail.run.mode,
    scheme: detail.report.scheme,
    cal,
    result,
    versions: detail.run.versions,
    reportMarkdown: detail.report.report_markdown,
    stored: true,
  };
}

function SettingsDialog({
  setting,
  error,
  onClose,
  onSave,
}: {
  setting: Setting;
  error: string;
  onClose: () => void;
  onSave: (setting: Setting) => void;
}) {
  const [draft, setDraft] = useState(setting);
  return (
    <section
      className="dialog-backdrop"
      role="presentation"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) onClose();
      }}
    >
      <div
        className="dialog-panel settings-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="settings-title"
      >
        <div className="dialog-heading">
          <div>
            <p className="eyebrow">按档案隔离</p>
            <h2 id="settings-title">当前档案设置</h2>
          </div>
          <button
            type="button"
            className="icon-button"
            aria-label="关闭设置"
            data-dialog-close
            onClick={onClose}
          >
            ×
          </button>
        </div>
        <div className="input-grid settings-grid">
          <label>
            默认输入模式
            <select
              value={draft.input_mode}
              onChange={(event) =>
                setDraft({
                  ...draft,
                  input_mode: event.target.value as InputScheme,
                })
              }
            >
              <option value="NINE_COLOR">九色</option>
              <option value="TRADITIONAL_NUMBER">传统数字</option>
            </select>
          </label>
          <label>
            默认推演模式
            <select
              value={draft.mode}
              onChange={(event) =>
                setDraft({ ...draft, mode: event.target.value as CastMode })
              }
            >
              <option value="FIXED">固定模式</option>
              <option value="CAST">摇卦模式</option>
            </select>
          </label>
          <label>
            日界
            <select
              value={draft.day_bound}
              onChange={(event) =>
                setDraft({
                  ...draft,
                  day_bound: event.target.value as "ZICHU" | "ZIZHENG",
                })
              }
            >
              <option value="ZICHU">子初换日（23:00）</option>
              <option value="ZIZHENG">子正换日（00:00）</option>
            </select>
          </label>
          <label>
            时区
            <input
              type="text"
              value={draft.timezone}
              onChange={(event) =>
                setDraft({ ...draft, timezone: event.target.value })
              }
            />
          </label>
          <label className="checkbox-field">
            <input
              type="checkbox"
              checked={draft.animation_enabled}
              onChange={(event) =>
                setDraft({ ...draft, animation_enabled: event.target.checked })
              }
            />
            启用动画
          </label>
          <label className="checkbox-field">
            <input
              type="checkbox"
              checked={draft.privacy_redact}
              onChange={(event) =>
                setDraft({ ...draft, privacy_redact: event.target.checked })
              }
            />
            隐私脱敏提示
          </label>
        </div>
        {error && (
          <p className="error-message" role="alert">
            {error}
          </p>
        )}
        <div className="dialog-actions">
          <button type="button" className="secondary-button" onClick={onClose}>
            取消
          </button>
          <button type="button" className="primary-button" onClick={() => onSave(draft)}>
            保存设置
          </button>
        </div>
      </div>
    </section>
  );
}

function ResultView({
  payload,
  onOpenReport,
  onOpenEvidence,
}: {
  payload: CastingPayload;
  onOpenReport: (trigger: HTMLElement) => void;
  onOpenEvidence: (href: string, trigger: HTMLElement) => void;
}) {
  const { result, cal, versions } = payload;
  return (
    <section className="result-area" aria-labelledby="result-heading">
      <div className="section-heading">
        <div>
          <p className="eyebrow">
            {result.scheme === "NINE_COLOR" ? "九色" : "传统数字"} · {payload.mode} ·{" "}
            {payload.runId}
          </p>
          <h2 id="result-heading">推演结果</h2>
        </div>
        <button
          type="button"
          className="primary-button"
          onClick={(event) => onOpenReport(event.currentTarget)}
        >
          查看完整过程与导出
        </button>
      </div>

      {!result.formal_allowed && (
        <aside className="pending-banner" role="status">
          <strong>待校核 · 不得作为正式结论</strong>
          <p>
            当前规则集或数据未冻结。本结果仅作学习参考，不会升级为正式结论。
          </p>
        </aside>
      )}
      <div className="hexagram-grid">
        <HexagramCard title="本卦" hexagram={result.original} />
        <HexagramCard title="互卦" hexagram={result.mutual} />
        <HexagramCard title="变卦" hexagram={result.changed} />
      </div>

      <div className="panel result-panel">
        <h3>体用与旺衰</h3>
        <dl className="result-facts">
          <div>
            <dt>动爻</dt>
            <dd>第 {result.moving_line} 爻</dd>
          </div>
          <div>
            <dt>体</dt>
            <dd>
              {result.ti} · {result.ti_element}
            </dd>
          </div>
          <div>
            <dt>用</dt>
            <dd>
              {result.yong} · {result.yong_element}
            </dd>
          </div>
          <div>
            <dt>忌神五行</dt>
            <dd>{result.jishen.element}</dd>
          </div>
        </dl>
        <ul className="plain-list">
          {result.wangshuai.map((entry) => (
            <li key={`${entry.trigram}-${entry.element}`}>
              {entry.trigram}：{entry.relation}（{entry.element}）
            </li>
          ))}
        </ul>
        {result.jishen.weak_positions.length > 0 && (
          <p>
            弱势方位：
            {result.jishen.weak_positions
              .map((position) => `${position.trigram} ${position.direction}`)
              .join("、")}
          </p>
        )}
        {result.scheme === "NINE_COLOR" && (
          <p className="muted-text">{NINE_COLOR_NOTICE}</p>
        )}
      </div>

      {payload.seed && (
        <div className="panel seed-panel">
          <h3>随机与快照</h3>
          <dl className="result-facts seed-facts">
            <div>
              <dt>推演模式</dt>
              <dd>{payload.seed.mode}</dd>
            </div>
            <div>
              <dt>基础种子</dt>
              <dd>
                <code>{payload.seed.baseSeed}</code>
              </dd>
            </div>
            <div>
              <dt>抽卡子种子</dt>
              <dd>
                <code>{payload.seed.drawSeed}</code>
              </dd>
            </div>
            <div>
              <dt>输入快照摘要</dt>
              <dd>
                <code>{payload.seed.snapshotDigest}</code>
              </dd>
            </div>
            <div>
              <dt>推演种子</dt>
              <dd>
                <code>{payload.seed.runSeed}</code>
              </dd>
            </div>
          </dl>
        </div>
      )}

      <div className="panel calendar-panel">
        <h3>历法上下文</h3>
        <p>
          {cal.solar} · 农历 {cal.lunar.year} 年{cal.lunar.leap ? "闰" : ""}
          {cal.lunar.month} 月 {cal.lunar.day} 日 · {cal.timezone} · {cal.day_bound}
        </p>
        <p>
          年月日时：{cal.ganzhi.year} · {cal.ganzhi.month} · {cal.ganzhi.day} ·{" "}
          {cal.ganzhi.hour}
        </p>
        <p>
          当前节气：{cal.jieqi.current} · 下个节气：{cal.jieqi.next}（
          {cal.jieqi.next_at}）
        </p>
      </div>

      <div className="panel trace-panel">
        <div className="section-heading">
          <div>
            <p className="eyebrow">可追溯步骤</p>
            <h3>推演过程与依据</h3>
          </div>
          <StatusBadge status={result.status} />
        </div>
        <ol className="step-list">
          {result.steps.map((step) => (
            <StepCard
              key={step.step_id}
              step={step}
              onOpenEvidence={onOpenEvidence}
            />
          ))}
        </ol>
      </div>

      <div className="panel versions-panel">
        <h3>版本信息</h3>
        <dl className="versions-grid">
          {Object.entries(versions).map(([name, version]) => (
            <div key={name}>
              <dt>{name}</dt>
              <dd>{version}</dd>
            </div>
          ))}
        </dl>
      </div>
      <p className="result-disclaimer" role="note">
        免责声明：{result.disclaimer || DISCLAIMER}
      </p>
    </section>
  );
}

function HexagramCard({
  title,
  hexagram,
}: {
  title: string;
  hexagram: CastingPayload["result"]["original"];
}) {
  return (
    <article className="hexagram-card">
      <p className="eyebrow">{title}</p>
      <h3>{hexagram.name}</h3>
      <p>
        上{hexagram.upper} · 下{hexagram.lower}
      </p>
      <code>{hexagram.binary_bottom_up}</code>
    </article>
  );
}

function StepCard({
  step,
  onOpenEvidence,
}: {
  step: StepLog;
  onOpenEvidence: (href: string, trigger: HTMLElement) => void;
}) {
  const links = [
    ...step.rule_ids.map((id) => ({
      id,
      href: `yitool://rule/${id}`,
      kind: "规则",
    })),
    {
      id: step.source_ref,
      href: `yitool://source/${step.source_ref}`,
      kind: "出处",
    },
    ...step.data_refs.map((id) => ({
      id,
      href: `yitool://resource/${id}`,
      kind: "资源",
    })),
  ];

  return (
    <li className="step-card">
      <div className="step-heading">
        <div>
          <p className="eyebrow">
            步骤 {step.step_index} · {step.engine}
          </p>
          <h4>{step.step_id}</h4>
        </div>
        <StatusBadge status={step.status} />
      </div>
      <p>{step.formula_summary}</p>
      <p>
        规则层级：{step.rule_level} · 输出：
        <code>{JSON.stringify(step.output)}</code>
      </p>
      <div className="evidence-links" aria-label="本步骤依据">
        {links.map((link) => (
          <button
            type="button"
            className="text-button"
            key={`${link.kind}-${link.id}`}
            onClick={(event) =>
              onOpenEvidence(link.href, event.currentTarget)
            }
          >
            查看{link.kind} {link.id}
          </button>
        ))}
      </div>
    </li>
  );
}

function EvidenceDetails({ entry }: { entry: EvidenceEntry }) {
  const pending = entry.status.toUpperCase() === "PENDING";
  return (
    <div className="evidence-content">
      <dl className="evidence-facts">
        <div>
          <dt>类型</dt>
          <dd>{entry.kind}</dd>
        </div>
        <div>
          <dt>稳定 ID</dt>
          <dd>{entry.id}</dd>
        </div>
        <div>
          <dt>版本</dt>
          <dd>{entry.version}</dd>
        </div>
        <div>
          <dt>状态</dt>
          <dd>
            <StatusBadge status={entry.status} />
          </dd>
        </div>
        {entry.source_ref && (
          <div>
            <dt>来源 ID</dt>
            <dd>{entry.source_ref}</dd>
          </div>
        )}
      </dl>
      {pending && (
        <p className="pending-banner" role="status">
          此依据尚待校勘/校核。
        </p>
      )}
      <p className="evidence-summary">{entry.summary}</p>
      {entry.fields.length > 0 && (
        <dl className="evidence-fields">
          {entry.fields.map(([label, value]) => (
            <div key={label}>
              <dt>{label}</dt>
              <dd>{value}</dd>
            </div>
          ))}
        </dl>
      )}
      <p className="muted-text">只读本地内容，不访问网络或重新计算。</p>
    </div>
  );
}

function StatusBadge({ status }: { status: string }) {
  const pending = status.toUpperCase() === "PENDING";
  return (
    <span className={`status-badge ${pending ? "pending" : ""}`}>
      {status}
    </span>
  );
}

export default App;
