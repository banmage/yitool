import "@testing-library/jest-dom/vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { cleanup } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App";
import {
  castMeihua,
  drawColors,
  getFormOptions,
  getHistory,
  getSettings,
  listHistory,
  listProfiles,
  resolveEvidence,
} from "./api";
import type {
  CastingPayload,
  FormOptions,
  HistoryDetail,
  Profile,
  Setting,
} from "./types";

vi.mock("./api", () => ({
  castMeihua: vi.fn(),
  createProfile: vi.fn(),
  deleteHistory: vi.fn(),
  deleteProfile: vi.fn(),
  drawColors: vi.fn(),
  getFormOptions: vi.fn(),
  getHistory: vi.fn(),
  getSettings: vi.fn(),
  listHistory: vi.fn(),
  listProfiles: vi.fn(),
  renameProfile: vi.fn(),
  resolveEvidence: vi.fn(),
  saveSettings: vi.fn(),
  errorMessage: (error: unknown) =>
    error instanceof Error ? error.message : String(error),
}));

const formOptions: FormOptions = {
  colors: Array.from({ length: 9 }, (_, index) => ({
    colorId: index + 1,
    name: `颜色${index + 1}`,
    element: "火",
  })),
  zodiacs: Array.from({ length: 12 }, (_, index) => ({
    zodiacId: index + 1,
    name: `生肖${index + 1}`,
    branch: "子",
  })),
};

const profile: Profile = {
  profile_id: "default",
  display_name: "默认档案",
  created_at: "2026-11-11T12:00:00+08:00",
  is_default: true,
};

const settings: Setting = {
  profile_id: "default",
  mode: "FIXED",
  input_mode: "NINE_COLOR",
  aux_mode: "ZODIAC",
  day_bound: "ZICHU",
  timezone: "Asia/Shanghai",
  ab_mode: "INDEPENDENT",
  animation_enabled: true,
  privacy_redact: true,
};

const report = "# 完整报告\n\n免责声明：仅供学习与娱乐。";
const payload: CastingPayload = {
  runId: "MEIHUA-TEST",
  createdAt: "2026-11-06T12:00:00Z",
  mode: "FIXED",
  scheme: "NINE_COLOR",
  cal: {
    solar: "2026-11-06T12:00:00+08:00",
    timezone: "Asia/Shanghai",
    day_bound: "ZICHU",
    effective_date: "2026-11-06",
    lunar: { year: 2026, month: 9, day: 27, leap: false },
    jieqi: { current: "霜降", next: "立冬", next_at: "2026-11-07T00:00:00+08:00" },
    ganzhi: { year: "丙午", month: "戊戌", day: "丁酉", hour: "丙午" },
    xun: { xun_head: "甲午", kong_branches: ["辰", "巳"] },
    status: "VERIFIED",
    disclaimer: "历法参考。",
  },
  result: {
    scheme: "NINE_COLOR",
    original: { name: "本卦", upper: "乾", lower: "坤", binary_bottom_up: "000111" },
    mutual: { name: "互卦", upper: "坎", lower: "离", binary_bottom_up: "101010" },
    changed: { name: "变卦", upper: "震", lower: "巽", binary_bottom_up: "011100" },
    moving_line: 3,
    ti: "上卦",
    yong: "下卦",
    ti_element: "金",
    yong_element: "土",
    wangshuai: [{ trigram: "乾", element: "金", relation: "旺" }],
    jishen: {
      element: "火",
      weak_positions: [],
      rule_level: "L3",
      note: "学习参考。",
    },
    steps: [
      {
        step_index: 1,
        step_id: "MEIHUA-STEP-1",
        engine: "MEIHUA",
        rule_ids: ["MEIHUA-QIGUA-001"],
        data_refs: ["COLOR-NUM-02"],
        source_ref: "SRC-MEIHUA-001",
        rule_level: "L3",
        formula_summary: "九色起卦步骤",
        output: { result: "本卦" },
        status: "PENDING",
      },
    ],
    status: "PENDING",
    formal_allowed: false,
    disclaimer: "仅供学习与娱乐。",
  },
  versions: {
    srs: "SRS-v0.5",
    ars: "ARS-v0.2",
    ir: "IR-v0.1",
    cal: "CAL-v0.2",
    rule_set: "RULE-v0.2",
    data: "DATA-v0.2",
    cal_data: "CAL-DATA-v0.2",
    tzdata: "2025b",
    seed_algo: "SEED-v2",
    rng: "RNG-v1",
    app: "APP-v0.2",
    db_schema: "DB-v0.3",
  },
  seed: {
    mode: "NINE_COLOR",
    baseSeed: "aa",
    drawSeed: "bb",
    snapshotDigest: "cc",
    runSeed: "dd",
    colorId: 1,
    auxId: 3,
    traditional: null,
  },
  stored: true,
  reportMarkdown: report,
};

const historyDetail: HistoryDetail = {
  run: {
    run_id: "MEIHUA-TEST",
    profile_id: "default",
    created_at: "2026-11-06T12:00:00Z",
    mode: "FIXED",
    engine_scope: "MEIHUA",
    random_snapshot_digest: "cc",
    versions: payload.versions,
    disclaimer_present: true,
  },
  report: {
    run_id: "MEIHUA-TEST",
    scheme: "NINE_COLOR",
    status: "PENDING",
    original_name: "本卦",
    changed_name: "变卦",
    moving_line: 3,
    formal_allowed: false,
    cal_json: JSON.stringify(payload.cal),
    result_json: JSON.stringify(payload.result),
    report_markdown: report,
  },
  steps: payload.result.steps,
  notes: [],
};

describe("Meihua desktop workflow", () => {
  beforeEach(() => {
    vi.mocked(getFormOptions).mockResolvedValue(formOptions);
    vi.mocked(listProfiles).mockResolvedValue([profile]);
    vi.mocked(getSettings).mockResolvedValue(settings);
    vi.mocked(listHistory).mockResolvedValue([]);
    vi.mocked(castMeihua).mockResolvedValue(payload);
  });

  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  it("does not submit until the verified selectors load", async () => {
    render(<App />);
    await screen.findByRole("combobox", { name: "九色" });
    expect(screen.getByRole("combobox", { name: "生肖" })).toBeEnabled();
    expect(castMeihua).not.toHaveBeenCalled();
  });

  it("does not submit an invalid local date", async () => {
    render(<App />);
    await screen.findByRole("combobox", { name: "九色" });
    fireEvent.change(screen.getByLabelText("起卦本地时间"), {
      target: { value: "" },
    });
    fireEvent.click(screen.getByRole("button", { name: "开始推演" }));

    expect(castMeihua).not.toHaveBeenCalled();
  });

  it("submits traditional numbers with the traditional scheme", async () => {
    render(<App />);
    await screen.findByRole("combobox", { name: "九色" });
    fireEvent.click(screen.getByRole("tab", { name: "传统数字" }));
    fireEvent.change(screen.getByLabelText("上卦数字 U0"), {
      target: { value: "9" },
    });
    fireEvent.change(screen.getByLabelText("下卦数字 D0"), {
      target: { value: "16" },
    });
    fireEvent.click(screen.getByRole("button", { name: "开始推演" }));

    await waitFor(() =>
      expect(castMeihua).toHaveBeenCalledWith(
        expect.objectContaining({
          scheme: "TRADITIONAL_NUMBER",
          traditional: [9, 16],
          profileId: "default",
        }),
      ),
    );
  });

  it("fills nine-color inputs from a CSPRNG draw", async () => {
    vi.mocked(drawColors).mockResolvedValue({ colorId: 5, auxId: 8 });
    render(<App />);
    await screen.findByRole("combobox", { name: "九色" });
    fireEvent.click(screen.getByRole("button", { name: "随机抽卡" }));

    await screen.findByText(/已抽到九色 5、生肖 8/);
    expect(screen.getByRole("combobox", { name: "九色" })).toHaveValue("5");
    expect(screen.getByRole("combobox", { name: "生肖" })).toHaveValue("8");
  });

  it("displays pending status and reuses the exact report for copy", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    const createObjectURL = vi.fn((_blob: Blob) => "blob:report");
    const revokeObjectURL = vi.fn();
    Object.defineProperty(URL, "createObjectURL", {
      configurable: true,
      value: createObjectURL,
    });
    Object.defineProperty(URL, "revokeObjectURL", {
      configurable: true,
      value: revokeObjectURL,
    });
    const anchorClick = vi
      .spyOn(HTMLAnchorElement.prototype, "click")
      .mockImplementation(() => undefined);
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText },
    });

    render(<App />);
    await screen.findByRole("combobox", { name: "九色" });
    fireEvent.click(screen.getByRole("button", { name: "开始推演" }));

    expect(
      await screen.findByText("待校核 · 不得作为正式结论"),
    ).toBeInTheDocument();
    fireEvent.click(
      screen.getByRole("button", { name: "查看完整过程与导出" }),
    );
    const reportDialog = await screen.findByRole("dialog");
    expect(reportDialog.querySelector("pre")?.textContent).toBe(report);
    const closeButton = screen.getByRole("button", { name: "关闭报告" });
    expect(document.activeElement).toBe(closeButton);
    fireEvent.keyDown(document, { key: "Tab", shiftKey: true });
    expect(document.activeElement).toBe(
      screen.getByRole("button", { name: "下载 .md" }),
    );
    fireEvent.keyDown(document, { key: "Tab" });
    expect(document.activeElement).toBe(closeButton);
    fireEvent.click(screen.getByRole("button", { name: "复制 Markdown" }));

    await waitFor(() => expect(writeText).toHaveBeenCalledWith(report));
    expect(screen.getByText("已复制完整报告。")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "下载 .md" }));
    await waitFor(() => expect(anchorClick).toHaveBeenCalledOnce());
    const downloadedBlob = createObjectURL.mock.calls[0][0];
    const downloadedText = await new Promise<string>((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => resolve(String(reader.result));
      reader.onerror = () => reject(reader.error);
      reader.readAsText(downloadedBlob);
    });
    expect(downloadedText).toBe(report);
    fireEvent.keyDown(document, { key: "Escape" });
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    expect(document.activeElement).toBe(
      screen.getByRole("button", { name: "查看完整过程与导出" }),
    );
  });

  it("opens only resolved local evidence and shows its pending status", async () => {
    vi.mocked(resolveEvidence).mockResolvedValue({
      kind: "RULE",
      id: "MEIHUA-QIGUA-001",
      title: "梅花起卦",
      href: "yitool://rule/MEIHUA-QIGUA-001",
      anchor: "evidence-rule-MEIHUA-QIGUA-001",
      path: null,
      version: "MEIHUA-CORE-v0.2",
      status: "PENDING",
      source_ref: "SRC-MEIHUA-001",
      summary: "规则摘要",
      fields: [["层级", "L3"]],
    });

    render(<App />);
    await screen.findByRole("combobox", { name: "九色" });
    fireEvent.click(screen.getByRole("button", { name: "开始推演" }));
    fireEvent.click(
      await screen.findByRole("button", {
        name: "查看规则 MEIHUA-QIGUA-001",
      }),
    );

    expect(await screen.findByText("梅花起卦")).toBeInTheDocument();
    expect(screen.getByText("此依据尚待校勘/校核。")).toBeInTheDocument();
    expect(resolveEvidence).toHaveBeenCalledWith(
      "yitool://rule/MEIHUA-QIGUA-001",
    );
  });

  it("loads a saved history detail without recomputing", async () => {
    vi.mocked(listHistory).mockResolvedValue([
      {
        run_id: "MEIHUA-TEST",
        profile_id: "default",
        created_at: "2026-11-06T12:00:00Z",
        mode: "FIXED",
        scheme: "NINE_COLOR",
        status: "PENDING",
        original_name: "本卦",
        changed_name: "变卦",
        moving_line: 3,
        formal_allowed: false,
      },
    ]);
    vi.mocked(getHistory).mockResolvedValue(historyDetail);

    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "打开复盘" }));

    const dialog = await screen.findByRole("dialog");
    expect(dialog.querySelector("pre")?.textContent).toBe(report);
    expect(getHistory).toHaveBeenCalledWith("default", "MEIHUA-TEST");
    expect(castMeihua).not.toHaveBeenCalled();
  });
});
