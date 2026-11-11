export type Status =
  | "OK"
  | "PENDING"
  | "ERROR"
  | "VERIFIED"
  | "FROZEN"
  | "DEPRECATED";

export type InputScheme = "NINE_COLOR" | "TRADITIONAL_NUMBER";
export type CastMode = "FIXED" | "CAST";

export interface FormOptions {
  colors: ColorOption[];
  zodiacs: ZodiacOption[];
}

export interface ColorOption {
  colorId: number;
  name: string;
  element: string;
}

export interface ZodiacOption {
  zodiacId: number;
  name: string;
  branch: string;
}

export interface VersionSet {
  srs: string;
  ars: string;
  ir: string;
  cal: string;
  rule_set: string;
  data: string;
  cal_data: string;
  tzdata: string;
  seed_algo: string;
  rng: string;
  app: string;
  db_schema: string;
}

export interface Hexagram {
  name: string;
  upper: string;
  lower: string;
  binary_bottom_up: string;
}

export interface StepLog {
  step_index: number;
  step_id: string;
  engine: string;
  rule_ids: string[];
  data_refs: string[];
  source_ref: string;
  rule_level: string;
  formula_summary: string;
  output: unknown;
  status: Status;
}

export interface MeihuaResult {
  scheme: string;
  original: Hexagram;
  mutual: Hexagram;
  changed: Hexagram;
  moving_line: number;
  ti: string;
  yong: string;
  ti_element: string;
  yong_element: string;
  wangshuai: Array<{
    trigram: string;
    element: string;
    relation: string;
  }>;
  jishen: {
    element: string;
    weak_positions: Array<{
      trigram: string;
      direction: string;
      element: string;
    }>;
    rule_level: string;
    note: string;
  };
  steps: StepLog[];
  status: Status;
  formal_allowed: boolean;
  disclaimer: string;
}

export interface CalResult {
  solar: string;
  timezone: string;
  day_bound: "ZICHU" | "ZIZHENG";
  effective_date: string;
  lunar: {
    year: number;
    month: number;
    day: number;
    leap: boolean;
  };
  jieqi: {
    current: string;
    next: string;
    next_at: string;
  };
  ganzhi: {
    year: string;
    month: string;
    day: string;
    hour: string;
  };
  xun: {
    xun_head: string;
    kong_branches: [string, string];
  };
  status: Status;
  disclaimer: string;
}

export interface SeedView {
  mode: string;
  baseSeed: string;
  drawSeed: string;
  snapshotDigest: string;
  runSeed: string;
  colorId: number | null;
  auxId: number | null;
  traditional: [number, number] | null;
}

export interface CastRequest {
  mode: CastMode;
  scheme: InputScheme;
  colorId?: number;
  auxId?: number;
  traditional?: [number, number];
  profileId?: string;
  datetime: string;
  timezone: string;
  dayBound: "ZICHU" | "ZIZHENG";
}

export interface CastingPayload {
  runId: string;
  createdAt: string;
  mode: string;
  scheme: string;
  cal: CalResult;
  result: MeihuaResult;
  versions: VersionSet;
  reportMarkdown: string;
  seed?: SeedView;
  stored: boolean;
}

export interface DrawSelection {
  colorId: number;
  auxId: number;
}

export interface Profile {
  profile_id: string;
  display_name: string;
  created_at: string;
  is_default: boolean;
}

export interface Setting {
  profile_id: string;
  mode: CastMode;
  input_mode: InputScheme;
  aux_mode: "ZODIAC" | "MONTH";
  day_bound: "ZICHU" | "ZIZHENG";
  timezone: string;
  ab_mode: string;
  animation_enabled: boolean;
  privacy_redact: boolean;
}

export interface RunSummary {
  run_id: string;
  profile_id: string;
  created_at: string;
  mode: string;
  scheme: string;
  status: string;
  original_name: string;
  changed_name: string;
  moving_line: number;
  formal_allowed: boolean;
}

export interface HistoryDetail {
  run: {
    run_id: string;
    profile_id: string;
    created_at: string;
    mode: string;
    engine_scope: string;
    random_snapshot_digest: string | null;
    versions: VersionSet;
    disclaimer_present: boolean;
  };
  report: {
    run_id: string;
    scheme: string;
    status: string;
    original_name: string;
    changed_name: string;
    moving_line: number;
    formal_allowed: boolean;
    cal_json: string;
    result_json: string;
    report_markdown: string;
  };
  steps: StepLog[];
  notes: Array<{
    note_id: string;
    run_id: string | null;
    body: string;
    created_at: string;
  }>;
}

export interface EvidenceEntry {
  kind: "RULE" | "SOURCE" | "RESOURCE";
  id: string;
  title: string;
  href: string;
  anchor: string;
  path: string | null;
  version: string;
  status: string;
  source_ref: string | null;
  summary: string;
  fields: Array<[string, string]>;
}
