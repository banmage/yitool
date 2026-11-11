import { invoke } from "@tauri-apps/api/core";
import type {
  CastRequest,
  CastingPayload,
  DrawSelection,
  EvidenceEntry,
  FormOptions,
  HistoryDetail,
  Profile,
  RunSummary,
  Setting,
} from "./types";

export function getFormOptions(): Promise<FormOptions> {
  return invoke<FormOptions>("get_form_options");
}

export function drawColors(): Promise<DrawSelection> {
  return invoke<DrawSelection>("draw_colors");
}

export function castMeihua(request: CastRequest): Promise<CastingPayload> {
  return invoke<CastingPayload>("cast_meihua", { request });
}

export function listProfiles(): Promise<Profile[]> {
  return invoke<Profile[]>("profiles_list");
}

export function createProfile(displayName: string): Promise<Profile> {
  return invoke<Profile>("profile_create", { displayName });
}

export function renameProfile(
  profileId: string,
  displayName: string,
): Promise<void> {
  return invoke("profile_rename", { profileId, displayName });
}

export function deleteProfile(profileId: string): Promise<void> {
  return invoke("profile_delete", { profileId });
}

export function getSettings(profileId: string): Promise<Setting> {
  return invoke<Setting>("settings_get", { profileId });
}

export function saveSettings(setting: Setting): Promise<void> {
  return invoke("settings_save", { setting });
}

export function listHistory(profileId: string): Promise<RunSummary[]> {
  return invoke<RunSummary[]>("history_list", { profileId });
}

export function getHistory(
  profileId: string,
  runId: string,
): Promise<HistoryDetail | null> {
  return invoke<HistoryDetail | null>("history_get", { profileId, runId });
}

export function deleteHistory(
  profileId: string,
  runId: string,
): Promise<void> {
  return invoke("history_delete", { profileId, runId });
}

export function resolveEvidence(href: string): Promise<EvidenceEntry> {
  if (!/^yitool:\/\/(rule|source|resource)\/[A-Za-z0-9][A-Za-z0-9_.-]*$/.test(href)) {
    return Promise.reject(new Error("依据链接格式无效"));
  }
  return invoke<EvidenceEntry>("resolve_evidence", { href });
}

export function errorMessage(error: unknown): string {
  if (typeof error === "string") {
    return error;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return "操作失败，请检查本地资源与输入后重试";
}
