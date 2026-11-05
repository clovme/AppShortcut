import { computed, reactive, ref, watch } from "vue";
import type {
  CreateTargets,
  LibraryOptions,
  PickedProgram,
  ProgramItem,
  ShortcutRequestItem,
  ShortcutResult,
  TargetKind,
  ToastMessage,
  UserScope,
} from "../types";
import { DEFAULT_START_MENU_GROUP, LEGACY_START_MENU_GROUP, DEFAULT_CREATE_TARGETS } from "../types";
import { SEED_PROGRAMS, colorForName } from "../data/programs";
import {
  TARGET_LABELS,
  clearShortcuts,
  createShortcuts,
  inspectShortcuts,
  isTauri,
  loadLibrary,
  loadProgramIcons,
  saveLibrary,
} from "../services/shortcutService";

const STORAGE_KEY = "appshortcut.library.v1";

interface PersistedState {
  programs: ProgramItem[];
  options: LibraryOptions;
}

interface ApplyOutcome {
  created: number;
  skipped: number;
  failed: number;
  error: string | null;
}

function isTargetKind(value: unknown): value is TargetKind {
  return value === "desktop" || value === "startmenu";
}

function normalizeScope(value: unknown): UserScope {
  return value === "all" ? "all" : "current";
}

function normalizeGroup(value: unknown): string {
  if (typeof value !== "string") return DEFAULT_START_MENU_GROUP;
  const next = value.trim();
  if (!next || next === LEGACY_START_MENU_GROUP) return DEFAULT_START_MENU_GROUP;
  return next;
}

/** 每个程序至少保留一个创建位置；字段缺失或两个都关闭时回落到默认值 */
function normalizeTargets(value: unknown, fallback: CreateTargets = DEFAULT_CREATE_TARGETS): CreateTargets {
  const source = value && typeof value === "object" ? (value as Partial<CreateTargets>) : null;
  const desktop =
    typeof source?.desktop === "boolean" ? source.desktop : fallback.desktop;
  const startmenu =
    typeof source?.startmenu === "boolean" ? source.startmenu : fallback.startmenu;
  if (!desktop && !startmenu) return { ...DEFAULT_CREATE_TARGETS };
  return { desktop, startmenu };
}

function createItem(
  source: { name: string; path: string; color: string },
  extras: {
    included?: boolean;
    scope?: UserScope;
    group?: string;
    targets?: CreateTargets;
    createdTargets?: TargetKind[];
    skippedTargets?: TargetKind[];
  } = {}
): ProgramItem {
  return {
    id: `p-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 7)}`,
    name: source.name,
    path: source.path,
    color: source.color,
    included: extras.included ?? true,
    scope: extras.scope ?? "current",
    group: normalizeGroup(extras.group),
    targets: normalizeTargets(extras.targets),
    createdTargets: extras.createdTargets ?? [],
    skippedTargets: extras.skippedTargets ?? [],
    creating: false,
    failed: false,
    errorMessage: null,
  };
}

function seedPrograms(): ProgramItem[] {
  return SEED_PROGRAMS.map((program, index) => ({
    ...createItem(program, {
      included: program.included,
      createdTargets: [...program.createdTargets],
    }),
    id: `seed-${index}`,
  }));
}

function normalizeProgram(
  raw: unknown,
  index: number,
  legacyTargets: CreateTargets
): ProgramItem | null {
  if (!raw || typeof raw !== "object") return null;
  const value = raw as Partial<ProgramItem>;
  if (typeof value.name !== "string" || typeof value.path !== "string") return null;
  const createdTargets = Array.isArray(value.createdTargets)
    ? value.createdTargets.filter(isTargetKind)
    : [];
  const skippedTargets = Array.isArray(value.skippedTargets)
    ? value.skippedTargets.filter(isTargetKind)
    : [];
  return {
    id: typeof value.id === "string" ? value.id : `item-${index}`,
    name: value.name,
    path: value.path,
    color: typeof value.color === "string" ? value.color : colorForName(value.name),
    included: value.included !== false,
    scope: normalizeScope(value.scope),
    group: normalizeGroup(value.group),
    targets: normalizeTargets(value.targets, legacyTargets),
    createdTargets,
    skippedTargets,
    creating: false,
    failed: false,
    errorMessage: null,
  };
}

function parseState(raw: string | null): PersistedState | null {
  if (!raw) return null;
  try {
    const parsed = JSON.parse(raw) as Partial<PersistedState> & {
      targets?: Partial<CreateTargets>;
    };
    if (!Array.isArray(parsed.programs)) return null;
    // 旧清单只有全局创建位置；新清单按每个程序保存，缺失时用旧值兜底
    const legacyTargets = normalizeTargets(parsed.targets);
    const programs = parsed.programs
      .map((program, index) => normalizeProgram(program, index, legacyTargets))
      .filter((program): program is ProgramItem => program !== null);
    if (!programs.length) return null;
    return {
      programs,
      options: {
        overwrite: parsed.options?.overwrite === true,
      },
    };
  } catch {
    return null;
  }
}

function applyResults(program: ProgramItem, results: ShortcutResult[]): ApplyOutcome {
  const outcome: ApplyOutcome = { created: 0, skipped: 0, failed: 0, error: null };

  results.forEach((result) => {
    if (result.status === "failed") {
      outcome.failed += 1;
      outcome.error = result.message ?? "创建失败";
      return;
    }
    if (!program.createdTargets.includes(result.target)) {
      program.createdTargets.push(result.target);
    }
    if (result.status === "skipped") {
      outcome.skipped += 1;
      if (!program.skippedTargets.includes(result.target)) {
        program.skippedTargets.push(result.target);
      }
    } else {
      outcome.created += 1;
      program.skippedTargets = program.skippedTargets.filter(
        (target) => target !== result.target
      );
    }
  });

  program.failed = outcome.failed > 0;
  program.errorMessage = outcome.error;
  return outcome;
}

function summaryText(outcome: ApplyOutcome): string {
  const parts: string[] = [];
  if (outcome.created) parts.push(`已创建 ${outcome.created} 个`);
  if (outcome.skipped) parts.push(`跳过 ${outcome.skipped} 个（已存在）`);
  if (outcome.failed) parts.push(`失败 ${outcome.failed} 个`);
  return parts.join("，");
}

export function useShortcutLibrary() {
  const desktop = isTauri();
  const stored = desktop ? null : parseState(localStorage.getItem(STORAGE_KEY));
  const initialPrograms = stored ? stored.programs : desktop ? [] : seedPrograms();
  const programs = ref<ProgramItem[]>(initialPrograms);
  const options = reactive<LibraryOptions>(stored ? { ...stored.options } : { overwrite: false });
  const search = ref("");
  const creating = ref(false);
  const refreshing = ref(false);
  const progress = reactive({ done: 0, total: 0 });
  const toasts = ref<ToastMessage[]>([]);
  const storagePath = ref("");
  const storagePortable = ref(true);
  const storageReady = ref(false);
  const icons = ref<Record<string, string>>({});
  const iconRequests = new Set<string>();
  let toastSeq = 0;
  let hydrated = !desktop;
  let saveTimer: number | undefined;
  let fallbackNotified = false;
  let statusSeq = 0;

  function applyState(state: PersistedState) {
    programs.value = state.programs;
    Object.assign(options, state.options);
  }

  function serialize(): string {
    return JSON.stringify({
      version: 2,
      programs: programs.value,
      options: { ...options },
    });
  }

  async function persistNow() {
    const snapshot = serialize();
    if (!desktop) {
      try {
        localStorage.setItem(STORAGE_KEY, snapshot);
      } catch {
        /* 浏览器本地存储不可用时静默忽略 */
      }
      return;
    }
    try {
      const file = await saveLibrary(snapshot);
      storagePath.value = file.path;
      storagePortable.value = file.portable;
      if (!file.portable && !fallbackNotified) {
        fallbackNotified = true;
        notify(`程序目录不可写，清单已保存到 ${file.path}`, "info");
      }
    } catch (error) {
      notify(
        `保存清单失败：${error instanceof Error ? error.message : String(error)}`,
        "danger"
      );
    }
  }

  async function hydrate() {
    if (!desktop) {
      storageReady.value = true;
      return;
    }
    try {
      const file = await loadLibrary();
      let state: PersistedState | null = null;
      let migrated = false;
      if (file) {
        storagePath.value = file.path;
        storagePortable.value = file.portable;
        if (file.content) state = parseState(file.content);
      }
      if (!state) {
        // 首次迁移：把旧版本存在浏览器 localStorage 里的清单搬进文件
        state = parseState(localStorage.getItem(STORAGE_KEY));
        migrated = Boolean(state);
      }
      if (state) {
        applyState(state);
        void refreshIcons();
        // JSON 里的创建标记只作历史记录，启动时以磁盘上真实的 .lnk 为准
        void refreshStatuses();
      }
      hydrated = true;
      storageReady.value = true;
      if (migrated) void persistNow();
    } catch (error) {
      hydrated = true;
      storageReady.value = true;
      notify(
        `读取清单失败：${error instanceof Error ? error.message : String(error)}`,
        "danger"
      );
    }
  }

  watch(
    () => JSON.stringify({ programs: programs.value, options }),
    () => {
      if (!hydrated) return;
      if (saveTimer) window.clearTimeout(saveTimer);
      saveTimer = window.setTimeout(() => {
        void persistNow();
      }, 150);
    },
    { flush: "post" }
  );

  void hydrate();

  function notify(text: string, tone: ToastMessage["tone"] = "success") {
    const id = (toastSeq += 1);
    toasts.value.push({ id, text, tone });
    window.setTimeout(() => {
      toasts.value = toasts.value.filter((toast) => toast.id !== id);
    }, 2800);
  }

  function iconKey(path: string): string {
    return path.trim().toLowerCase();
  }

  function iconFor(path: string): string | undefined {
    return icons.value[iconKey(path)];
  }

  /** 批量补齐缺失图标；失败时保留首字色块，不打断主流程。 */
  async function refreshIcons(paths?: string[]) {
    if (!desktop) return;
    const keys: string[] = [];
    const wanted = (paths ?? programs.value.map((program) => program.path)).filter((path) => {
      if (!path) return false;
      const key = iconKey(path);
      if (icons.value[key] || iconRequests.has(key)) return false;
      iconRequests.add(key);
      keys.push(key);
      return true;
    });
    if (!wanted.length) return;
    try {
      const results = await loadProgramIcons(wanted);
      results.forEach((result) => {
        if (result.icon) icons.value[iconKey(result.path)] = result.icon;
      });
    } catch {
      /* 图标读取失败不影响清单与创建 */
    } finally {
      keys.forEach((key) => iconRequests.delete(key));
    }
  }

  function targetsOf(program: ProgramItem): TargetKind[] {
    const list: TargetKind[] = [];
    if (program.targets.desktop) list.push("desktop");
    if (program.targets.startmenu) list.push("startmenu");
    return list;
  }

  function samePath(left: string, right: string): boolean {
    return left.replace(/\\/g, "/").trim().toLowerCase() ===
      right.replace(/\\/g, "/").trim().toLowerCase();
  }

  function requestFor(
    program: ProgramItem,
    targetList: TargetKind[] = targetsOf(program)
  ): ShortcutRequestItem {
    return {
      name: program.name,
      path: program.path,
      scope: program.scope,
      group: program.group,
      targets: targetList,
    };
  }

  /**
   * 以磁盘上的真实 .lnk 重新计算每行状态。
   * 同名且 TargetPath 一致 -> 已存在；同名指向别的程序 -> 冲突；文件不存在 -> 未创建。
   */
  async function refreshStatuses(paths?: string[]) {
    if (!desktop || creating.value) return;
    const selected = programs.value.filter(
      (program) => !paths || paths.some((path) => samePath(path, program.path))
    );
    if (!selected.length) return;

    selected.forEach((program) => {
      program.createdTargets = [];
      program.skippedTargets = [];
      program.failed = false;
      program.errorMessage = null;
    });

    const requests = selected
      .map((program) => requestFor(program))
      .filter((request) => request.targets.length > 0);
    if (!requests.length) return;

    const runId = (statusSeq += 1);
    try {
      const states = await inspectShortcuts(requests);
      if (runId !== statusSeq) return;

      states.forEach((state) => {
        const program = selected.find((item) => samePath(item.path, state.programPath));
        if (!program) return;
        if (state.status === "exists") {
          if (!program.createdTargets.includes(state.target)) {
            program.createdTargets.push(state.target);
          }
          if (!program.skippedTargets.includes(state.target)) {
            program.skippedTargets.push(state.target);
          }
        } else if (state.status === "conflict" || state.status === "error") {
          program.failed = true;
          program.errorMessage = state.message ?? "同名快捷方式状态异常";
        }
      });
    } catch {
      /* 状态检查失败时保留“未创建”，不打断创建流程 */
    }
  }

  /** 手动刷新：外部删除或新增 .lnk 后，重新按磁盘真实状态更新所有行 */
  async function refreshNow() {
    if (refreshing.value) return;
    if (creating.value) {
      notify("正在创建快捷方式，完成后再刷新", "info");
      return;
    }
    if (!programs.value.length) {
      notify("清单里还没有程序", "info");
      return;
    }
    if (!desktop) {
      notify("浏览器预览无法读取磁盘，请在桌面应用里刷新", "info");
      return;
    }

    refreshing.value = true;
    try {
      await refreshStatuses();
      const existing = createdCount.value;
      const planned = plannedCount.value;
      notify(
        existing
          ? `已刷新：检测到 ${existing} / ${planned} 个快捷方式已存在`
          : "已刷新：未检测到已存在的快捷方式",
        "info"
      );
    } finally {
      refreshing.value = false;
    }
  }

  /** 清单里已经用过的分组名，供行内下拉直接选择，避免手输入错 */
  const groupOptions = computed(() => {
    const names = new Set<string>();
    programs.value.forEach((program) => {
      if (program.group) names.add(program.group);
    });
    return [...names].sort((left, right) => left.localeCompare(right, "zh-Hans-CN"));
  });
  const filteredPrograms = computed(() => {
    const keyword = search.value.trim().toLowerCase();
    if (!keyword) return programs.value;
    return programs.value.filter(
      (program) =>
        program.name.toLowerCase().includes(keyword) ||
        program.path.toLowerCase().includes(keyword)
    );
  });

  const includedPrograms = computed(() => programs.value.filter((program) => program.included));
  const includedCount = computed(() => includedPrograms.value.length);
  const excludedCount = computed(() => programs.value.length - includedCount.value);
  const plannedCount = computed(() =>
    includedPrograms.value.reduce((total, program) => total + targetsOf(program).length, 0)
  );
  const createdCount = computed(() =>
    includedPrograms.value.reduce((total, program) => {
      const active = targetsOf(program);
      const done = active.filter((target) =>
        program.createdTargets.includes(target)
      ).length;
      return total + done;
    }, 0)
  );
  const remainingCount = computed(() => Math.max(plannedCount.value - createdCount.value, 0));

  function hasPath(path: string): boolean {
    const key = path.trim().toLowerCase();
    return programs.value.some((program) => program.path.trim().toLowerCase() === key);
  }

  function addPrograms(picked: PickedProgram[]) {
    let added = 0;
    let lastAdded = "";
    const addedPaths: string[] = [];
    picked.forEach((program) => {
      if (!program.path || hasPath(program.path)) return;
      programs.value.unshift(
        createItem({
          name: program.name || program.path.split(/[\\/]/).pop() || "程序",
          path: program.path,
          color: colorForName(program.name || program.path),
        })
      );
      added += 1;
      lastAdded = program.name;
      addedPaths.push(program.path);
    });

    if (added === 0) {
      notify("所选程序已在清单中", "info");
    } else if (added === 1) {
      notify(`已加入「${lastAdded}」`);
    } else {
      notify(`已加入 ${added} 个程序`);
    }
    void refreshIcons();
    void refreshStatuses(addedPaths);
  }

  function removeProgram(id: string) {
    const target = programs.value.find((program) => program.id === id);
    if (!target) return;
    programs.value = programs.value.filter((program) => program.id !== id);
    notify(`已从清单移除「${target.name}」`, "info");
  }

  function setIncluded(id: string, value: boolean) {
    const target = programs.value.find((program) => program.id === id);
    if (target) target.included = value;
  }

  function setAllIncluded(value: boolean) {
    programs.value.forEach((program) => {
      program.included = value;
    });
  }

  function setScope(id: string, scope: UserScope) {
    const target = programs.value.find((program) => program.id === id);
    if (!target || target.scope === scope) return;
    target.scope = scope;
    target.createdTargets = [];
    target.skippedTargets = [];
    target.failed = false;
    target.errorMessage = null;
    void refreshStatuses([target.path]);
  }

  function setTargets(id: string, targets: CreateTargets) {
    const target = programs.value.find((program) => program.id === id);
    if (!target || creating.value || target.creating) return;
    const desktop = Boolean(targets.desktop);
    const startmenu = Boolean(targets.startmenu);
    if (!desktop && !startmenu) {
      notify("每个程序至少要保留一个创建位置", "info");
      return;
    }
    if (target.targets.desktop === desktop && target.targets.startmenu === startmenu) return;
    target.targets = { desktop, startmenu };
    target.createdTargets = [];
    target.skippedTargets = [];
    target.failed = false;
    target.errorMessage = null;
    const text = desktop && startmenu ? "桌面 + 开始菜单" : desktop ? "仅桌面" : "仅开始菜单";
    notify(`「${target.name}」创建位置：${text}`, "info");
    void refreshStatuses([target.path]);
  }

  function setGroup(id: string, group: string) {
    const target = programs.value.find((program) => program.id === id);
    if (!target) return;
    const next = normalizeGroup(group);
    if (next === target.group) return;
    target.group = next;
    target.createdTargets = [];
    target.skippedTargets = [];
    target.failed = false;
    target.errorMessage = null;
    notify(
      next
        ? `开始菜单分组已改为「${next}」`
        : "已取消开始菜单分组，快捷方式将直接放到 Programs 根目录",
      "info"
    );
    void refreshStatuses([target.path]);
  }

  function renameProgram(id: string, name: string) {
    const target = programs.value.find((program) => program.id === id);
    if (!target) return;
    const next = name.trim();
    if (!next) {
      notify("快捷方式名不能为空", "danger");
      return;
    }
    if (next === target.name) return;
    target.name = next;
    target.createdTargets = [];
    target.skippedTargets = [];
    target.failed = false;
    target.errorMessage = null;
    notify(`快捷方式名已改为「${next}」`);
    void refreshStatuses([target.path]);
  }

  function toggleOption(key: keyof LibraryOptions) {
    options[key] = !options[key];
  }

  async function clearCreatedMarks() {
    if (creating.value) return;
    if (!programs.value.length) {
      notify("清单里还没有程序", "info");
      return;
    }
    const clearable = programs.value.filter((program) => targetsOf(program).length > 0);
    const allTargets = Array.from(
      new Set<TargetKind>(clearable.flatMap((program) => targetsOf(program)))
    );
    if (!clearable.length || !allTargets.length) {
      notify("清单里没有可清除的创建位置", "info");
      return;
    }

    const targetText = allTargets.map((target) => TARGET_LABELS[target]).join(" + ");
    const confirmed = window.confirm(
      `将按每个程序各自设置的创建位置，删除清单内程序在「${targetText}」上、同名且指向对应程序的快捷方式（同时按各自的分组和范围）。\n\n指向其他程序的同名快捷方式不会被删除。确定继续？`
    );
    if (!confirmed) return;

    try {
      const result = await clearShortcuts(
        clearable.map((program) => requestFor(program)),
        allTargets
      );
      if (!desktop) {
        programs.value.forEach((program) => {
          program.createdTargets = [];
          program.skippedTargets = [];
          program.failed = false;
          program.errorMessage = null;
        });
      }
      await refreshStatuses();

      if (result.failed) {
        notify(
          `已删除 ${result.deleted} 个快捷方式，${result.failed} 个未能删除：${
            result.errors[0] ?? "未知原因"
          }`,
          "danger"
        );
      } else if (result.deleted) {
        notify(`已删除 ${result.deleted} 个快捷方式，并清除创建标记`);
      } else {
        notify("没有找到可删除的快捷方式，创建标记已清除", "info");
      }
    } catch (error) {
      notify(
        `清除快捷方式失败：${error instanceof Error ? error.message : String(error)}`,
        "danger"
      );
    }
  }

  function statusOf(program: ProgramItem) {
    if (program.failed) return "failed" as const;
    const active = targetsOf(program);
    if (!active.length) return "idle" as const;
    const done = active.filter((target) => program.createdTargets.includes(target)).length;
    if (!done) return "idle" as const;
    if (done < active.length) return "partial" as const;
    const allSkipped = active.every((target) => program.skippedTargets.includes(target));
    return allSkipped ? ("skipped" as const) : ("created" as const);
  }

  function pendingTargetsOf(program: ProgramItem): TargetKind[] {
    return targetsOf(program).filter((target) => !program.createdTargets.includes(target));
  }

  async function createOne(id: string) {
    const program = programs.value.find((item) => item.id === id);
    if (!program || creating.value || program.creating) return;
    const active = targetsOf(program);
    if (!active.length) {
      notify("这个程序还没有选择创建位置", "info");
      return;
    }
    // 创建前先读磁盘，避免用 JSON 里的旧标记判断是否已存在
    await refreshStatuses([program.path]);
    const pending = pendingTargetsOf(program);
    if (!pending.length) {
      notify(`「${program.name}」的快捷方式已存在`, "info");
      return;
    }

    program.creating = true;
    try {
      const results = await createShortcuts(
        [requestFor(program, pending)],
        options.overwrite
      );
      const outcome = applyResults(program, results);
      await refreshStatuses([program.path]);
      const text = summaryText(outcome);
      if (outcome.failed) {
        notify(`「${program.name}」${text || "创建失败"}：${outcome.error}`, "danger");
      } else if (outcome.created) {
        notify(`「${program.name}」${text}`);
      } else {
        notify(`「${program.name}」的快捷方式已存在，已跳过`, "info");
      }
    } catch (error) {
      program.failed = true;
      program.errorMessage = error instanceof Error ? error.message : String(error);
      notify(`创建「${program.name}」失败：${program.errorMessage}`, "danger");
    } finally {
      program.creating = false;
    }
  }

  async function createAll() {
    if (creating.value) return;
    // 以磁盘真实状态重新排队，避免 JSON 旧标记让已删除的快捷方式被误判为已存在
    await refreshStatuses();
    const queue = programs.value.filter(
      (program) => program.included && pendingTargetsOf(program).length > 0
    );
    if (!queue.length) {
      notify("没有待创建的快捷方式", "info");
      return;
    }

    creating.value = true;
    progress.total = queue.reduce(
      (total, program) => total + pendingTargetsOf(program).length,
      0
    );
    progress.done = 0;

    let created = 0;
    let skipped = 0;
    let failed = 0;

    for (const program of queue) {
      const pending = pendingTargetsOf(program);
      if (!pending.length) continue;
      program.creating = true;
      try {
        const results = await createShortcuts(
          [requestFor(program, pending)],
          options.overwrite
        );
        const outcome = applyResults(program, results);
        created += outcome.created;
        skipped += outcome.skipped;
        failed += outcome.failed;
      } catch (error) {
        program.failed = true;
        program.errorMessage = error instanceof Error ? error.message : String(error);
        failed += pending.length;
      } finally {
        program.creating = false;
        progress.done = Math.min(progress.done + pending.length, progress.total);
      }
    }

    creating.value = false;
    await refreshStatuses();

    const parts: string[] = [];
    if (created) parts.push(`已创建 ${created} 个快捷方式`);
    if (skipped) parts.push(`跳过 ${skipped} 个（已存在）`);
    if (failed) parts.push(`失败 ${failed} 个`);
    notify(parts.join("，") || "没有需要创建的快捷方式", failed ? "danger" : skipped && !created ? "info" : "success");
  }

  return {
    programs,
    options,
    search,
    creating,
    refreshing,
    progress,
    toasts,
    storagePath,
    storagePortable,
    storageReady,
    filteredPrograms,
    includedCount,
    excludedCount,
    groupOptions,
    plannedCount,
    remainingCount,
    notify,
    refreshNow,
    addPrograms,
    removeProgram,
    setIncluded,
    setAllIncluded,
    setScope,
    setTargets,
    setGroup,
    renameProgram,
    toggleOption,
    clearCreatedMarks,
    statusOf,
    pendingTargetsOf,
    iconFor,
    refreshIcons,
    createOne,
    createAll,
    TARGET_LABELS,
  };
}
