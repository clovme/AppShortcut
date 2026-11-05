import { invoke } from "@tauri-apps/api/core";
import type {
  ClearResult,
  LibraryFilePayload,
  PickedProgram,
  ProgramIcon,
  ShortcutRequestItem,
  ShortcutResult,
  ShortcutState,
  TargetKind,
  UserScope,
} from "../types";

export const TARGET_LABELS: Record<TargetKind, string> = {
  desktop: "桌面",
  startmenu: "开始菜单",
};

export const SCOPE_LABELS: Record<UserScope, string> = {
  current: "当前用户",
  all: "所有用户",
};

/** 是否运行在 Tauri 桌面窗口里（浏览器预览时为 false） */
export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/**
 * 调用 Rust 侧的系统文件选择窗口（IFileOpenDialog），支持一次多选 .exe。
 * 用户取消时返回空数组，不视为错误。
 */
export async function pickPrograms(): Promise<PickedProgram[]> {
  if (!isTauri()) {
    throw new Error("当前是浏览器预览，系统文件选择窗口只会在桌面应用中弹出");
  }
  return invoke<PickedProgram[]>("pick_programs");
}

/**
 * 在桌面 / 开始菜单写入 .lnk。
 * Rust 侧对已存在的同名快捷方式返回 skipped（overwrite 为 true 时才覆盖）。
 */
export async function createShortcuts(
  items: ShortcutRequestItem[],
  overwrite: boolean
): Promise<ShortcutResult[]> {
  if (isTauri()) {
    return invoke<ShortcutResult[]>("create_shortcuts", { items, overwrite });
  }

  // 浏览器预览：模拟耗时与结果，方便先看交互，不写入任何文件
  await new Promise((resolve) => setTimeout(resolve, 420 + Math.random() * 260));
  return items.flatMap((item) =>
    item.targets.map((target) => ({
      program: item.name,
      target,
      status: "created" as const,
      message: null,
      linkPath: null,
    }))
  );
}

/**
 * 读取磁盘上真实的 .lnk 状态：exists / conflict / missing / error。
 * 前端不再用 JSON 里的 createdTargets 判断“已存在”。
 */
export async function inspectShortcuts(
  items: ShortcutRequestItem[]
): Promise<ShortcutState[]> {
  if (!isTauri() || !items.length) return [];
  return invoke<ShortcutState[]>("inspect_shortcuts", { items });
}

/**
 * 清除清单对应的真实快捷方式。Rust 侧只删除同名且 TargetPath 与程序一致的 .lnk，
 * 指向其他程序的文件不会删除。
 */
export async function clearShortcuts(
  items: ShortcutRequestItem[],
  targets: TargetKind[]
): Promise<ClearResult> {
  if (!isTauri()) {
    // 浏览器预览没有真实文件，只清空模拟标记
    return { deleted: 0, failed: 0, errors: [] };
  }
  return invoke<ClearResult>("clear_shortcuts", { items, targets });
}

/**
 * 批量读取程序图标。Rust 侧用 IShellItemImageFactory 取 64px 图标并转成 PNG data URL。
 * 浏览器预览没有本地程序，返回空列表。
 */
export async function loadProgramIcons(paths: string[]): Promise<ProgramIcon[]> {
  if (!isTauri() || !paths.length) return [];
  return invoke<ProgramIcon[]>("program_icons", { paths });
}

/**
 * 读取清单文件。Rust 侧优先读 exe 同目录的 shortcuts.json，
 * 读不到时回落到 %APPDATA%\AppShortcut\shortcuts.json。
 */
export async function loadLibrary(): Promise<LibraryFilePayload | null> {
  if (!isTauri()) return null;
  return invoke<LibraryFilePayload>("load_library");
}

/**
 * 保存清单文件，优先写 exe 同目录；程序目录不可写时 Rust 自动回落。
 * 返回实际写入路径与是否 portable。
 */
export async function saveLibrary(content: string): Promise<LibraryFilePayload> {
  if (!isTauri()) {
    throw new Error("浏览器预览没有本地清单文件");
  }
  return invoke<LibraryFilePayload>("save_library", { content });
}
