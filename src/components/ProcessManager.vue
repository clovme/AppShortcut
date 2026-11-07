<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import type { ProcessInfo } from "../types";
import { isTauri, killProcesses, listProcesses } from "../services/shortcutService";

type ProcessNameOption = { key: string; name: string; count: number };
type NoticeTone = "ok" | "warn";

const props = withDefaults(defineProps<{ embedded?: boolean }>(), { embedded: false });
const emit = defineEmits<{ close: [] }>();

const processes = ref<ProcessInfo[]>([]);
const currentPid = ref(0);
const query = ref("");
const activeName = ref("all");
const nameListRef = ref<HTMLElement | null>(null);
const selected = ref<number[]>([]);
const loading = ref(false);
const killing = ref(false);
const error = ref("");
const notice = ref("");
const noticeTone = ref<NoticeTone>("ok");

const selectedSet = computed(() => new Set(selected.value));
const normalizedQuery = computed(() => query.value.trim().toLowerCase());

function matchesQuery(process: ProcessInfo, keyword: string) {
  if (!keyword) return true;
  const name = process.name.toLowerCase();
  const path = (process.path ?? "").toLowerCase();
  const portKeyword = keyword.startsWith(":") ? keyword.slice(1) : keyword;
  return (
    name.includes(keyword) ||
    path.includes(keyword) ||
    String(process.pid) === keyword ||
    String(process.parentPid) === keyword ||
    (process.ports ?? []).some((port) => String(port) === portKeyword)
  );
}

function formatPorts(ports: number[] | undefined) {
  if (!ports?.length) return "—";
  if (ports.length <= 4) return ports.join(", ");
  return `${ports.slice(0, 4).join(", ")} +${ports.length - 4}`;
}

const nameOptions = computed<ProcessNameOption[]>(() => {
  const keyword = normalizedQuery.value;
  const matched = processes.value.filter((process) => matchesQuery(process, keyword));
  const groups = new Map<string, ProcessNameOption>();
  for (const process of matched) {
    const key = process.name.toLowerCase();
    const existing = groups.get(key);
    if (existing) {
      existing.count += 1;
    } else {
      groups.set(key, { key, name: process.name, count: 1 });
    }
  }
  const names = [...groups.values()].sort(
    (left, right) =>
      right.count - left.count ||
      left.name.localeCompare(right.name, "en", { sensitivity: "base" })
  );
  return [{ key: "all", name: "全部", count: matched.length }, ...names];
});

const filtered = computed(() => {
  const keyword = normalizedQuery.value;
  return processes.value.filter((process) => {
    if (activeName.value !== "all" && process.name.toLowerCase() !== activeName.value) {
      return false;
    }
    return matchesQuery(process, keyword);
  });
});

const selectableFiltered = computed(() => filtered.value.filter((process) => !process.protected));
const selectedInfos = computed(() =>
  processes.value.filter((process) => selectedSet.value.has(process.pid))
);
const allFilteredSelected = computed(
  () =>
    selectableFiltered.value.length > 0 &&
    selectableFiltered.value.every((process) => selectedSet.value.has(process.pid))
);
const someFilteredSelected = computed(
  () =>
    !allFilteredSelected.value &&
    selectableFiltered.value.some((process) => selectedSet.value.has(process.pid))
);
const protectedCount = computed(() => processes.value.filter((process) => process.protected).length);

watch([query, activeName], () => {
  selected.value = [];
  notice.value = "";
});

watch(activeName, () => {
  scrollActiveNameIntoView();
});

watch(nameOptions, (options) => {
  if (!options.some((option) => option.key === activeName.value)) {
    activeName.value = "all";
  } else {
    scrollActiveNameIntoView();
  }
});

function scrollActiveNameIntoView() {
  void nextTick(() => {
    const list = nameListRef.value;
    const item = list?.querySelector<HTMLElement>(".name-item.active");
    if (!list || !item) return;
    const listRect = list.getBoundingClientRect();
    const itemRect = item.getBoundingClientRect();
    if (itemRect.top < listRect.top) {
      list.scrollTop -= listRect.top - itemRect.top + 6;
    } else if (itemRect.bottom > listRect.bottom) {
      list.scrollTop += itemRect.bottom - listRect.bottom + 6;
    }
  });
}

function refresh() {
  notice.value = "";
  void load();
}

function setNameFilter(key: string) {
  if (activeName.value !== key) activeName.value = key;
}

function toggleRow(pid: number) {
  if (selectedSet.value.has(pid)) {
    selected.value = selected.value.filter((item) => item !== pid);
  } else {
    selected.value = [...selected.value, pid];
  }
}

function toggleAllFiltered() {
  if (allFilteredSelected.value) {
    const visible = new Set(selectableFiltered.value.map((process) => process.pid));
    selected.value = selected.value.filter((pid) => !visible.has(pid));
    return;
  }
  const merged = new Set(selected.value);
  for (const process of selectableFiltered.value) merged.add(process.pid);
  selected.value = [...merged];
}

function formatMemory(kb: number): string {
  if (!kb) return "—";
  if (kb >= 1024 * 1024) return `${(kb / 1024 / 1024).toFixed(2)} GB`;
  if (kb >= 1024) return `${(kb / 1024).toFixed(1)} MB`;
  return `${kb} KB`;
}

async function load() {
  error.value = "";
  if (!isTauri()) {
    error.value = "浏览器预览无法读取真实进程，请在桌面应用里使用进程管理";
    processes.value = [];
    currentPid.value = 0;
    return;
  }
  loading.value = true;
  try {
    const snapshot = await listProcesses();
    processes.value = snapshot.processes;
    currentPid.value = snapshot.currentPid;
    const alive = new Set(
      snapshot.processes.filter((process) => !process.protected).map((process) => process.pid)
    );
    selected.value = selected.value.filter((pid) => alive.has(pid));
  } catch (reason) {
    error.value = reason instanceof Error ? reason.message : String(reason);
  } finally {
    loading.value = false;
  }
}

async function runKill(includeTree: boolean) {
  if (killing.value || !selectedInfos.value.length) return;
  const targets = selectedInfos.value;
  const preview = targets
    .slice(0, 6)
    .map((process) => `${process.name} (PID ${process.pid})`)
    .join("、");
  const extra = targets.length > 6 ? ` 等 ${targets.length} 个` : "";
  const treeText = includeTree ? "及其全部子进程" : "";
  const confirmed = window.confirm(
    `将结束 ${targets.length} 个进程${treeText}：\n${preview}${extra}\n\n进程里未保存的数据可能丢失，确定继续吗？`
  );
  if (!confirmed) return;

  killing.value = true;
  notice.value = "";
  try {
    const result = await killProcesses(
      targets.map((process) => process.pid),
      includeTree
    );
    const parts = [`已结束 ${result.killed} 个进程`];
    if (result.failed) parts.push(`${result.failed} 个失败`);
    noticeTone.value = result.failed ? "warn" : "ok";
    notice.value = result.errors.length
      ? `${parts.join("，")}：${result.errors.slice(0, 2).join("；")}${result.errors.length > 2 ? " 等" : ""}`
      : parts.join("，");
    selected.value = [];
    const targetPids = targets.map((process) => process.pid);
    await new Promise((resolve) => window.setTimeout(resolve, 600));
    await load();
    // TerminateProcess 返回后，系统偶尔还会在快照里保留几百毫秒的退出中进程；
    // 如果目标 PID 仍在列表里，再补一次读取，避免界面显示“已结束”却还挂着行。
    if (processes.value.some((process) => targetPids.includes(process.pid))) {
      window.setTimeout(() => {
        if (!killing.value) void load();
      }, 900);
    }
  } catch (reason) {
    noticeTone.value = "warn";
    notice.value = reason instanceof Error ? reason.message : String(reason);
  } finally {
    killing.value = false;
  }
}

function closeFromOverlay() {
  if (!props.embedded) emit("close");
}

function onAppRefresh() {
  refresh();
}

function onKeydown(event: KeyboardEvent) {
  if (props.embedded) return;
  if (event.key === "Escape") {
    event.preventDefault();
    emit("close");
  }
}

onMounted(() => {
  void load();
  window.addEventListener("appshortcut:refresh", onAppRefresh);
  if (!props.embedded) window.addEventListener("keydown", onKeydown);
});

onBeforeUnmount(() => {
  window.removeEventListener("appshortcut:refresh", onAppRefresh);
  window.removeEventListener("keydown", onKeydown);
});
</script>

<template>
  <Teleport to="body" :disabled="embedded">
    <div :class="embedded ? 'process-page' : 'process-overlay'" @mousedown.self="closeFromOverlay">
      <section
        :class="embedded ? 'process-surface' : 'process-modal'"
        :role="embedded ? 'region' : 'dialog'"
        :aria-modal="embedded ? undefined : 'true'"
        aria-labelledby="process-title"
      >
        <header class="process-head">
          <div class="process-title-wrap">
            <div class="process-mark" aria-hidden="true">
              <svg viewBox="0 0 28 28" width="28" height="28">
                <rect x="3" y="5" width="22" height="18" rx="4" fill="none" stroke="currentColor" stroke-width="1.7" />
                <path d="M3 10h22M6 7.7h.1M8.6 7.7h.1M11.2 7.7h.1" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" />
                <path d="M7 14h7M7 17.5h10M17 14h4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" opacity="0.7" />
              </svg>
            </div>
            <div>
              <h2 id="process-title">
                进程管理
                <span class="process-count">共 {{ processes.length }} 个</span>
              </h2>
              <p>WebView2、node 这类同名多实例进程，筛选后可以一次结束</p>
            </div>
          </div>
          <button v-if="!embedded" class="close-btn" type="button" title="关闭（Esc）" @click="emit('close')">
            <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
              <path d="m4 4 8 8m0-8-8 8" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" />
            </svg>
          </button>
        </header>

        <div class="process-toolbar">
          <div class="process-search">
            <svg viewBox="0 0 18 18" width="15" height="15" aria-hidden="true">
              <circle cx="7.6" cy="7.6" r="4.6" fill="none" stroke="currentColor" stroke-width="1.6" />
              <path d="m11.3 11.3 3.5 3.5" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
            </svg>
            <input v-model="query" type="text" placeholder="搜索进程名、路径、PID 或端口" spellcheck="false" />
            <button v-if="query" class="search-clear" type="button" title="清空搜索" @click="query = ''">
              <svg viewBox="0 0 14 14" width="12" height="12" aria-hidden="true">
                <path d="m3.5 3.5 7 7m0-7-7 7" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
              </svg>
            </button>
          </div>

          <span class="toolbar-summary">当前显示 {{ filtered.length }} 个</span>

          <button class="refresh-btn" type="button" :disabled="loading || killing" title="重新读取进程列表" @click="refresh">
            <span v-if="loading" class="spinner" />
            <svg v-else viewBox="0 0 18 18" width="15" height="15" aria-hidden="true">
              <path d="M14.2 7.2A5.5 5.5 0 1 0 14.5 10" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
              <path d="M14.3 3.8v3.7h-3.7" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
            </svg>
            {{ loading ? "读取中…" : "刷新" }}
          </button>
        </div>

        <div class="process-body">
          <aside class="name-list" aria-label="按进程名称筛选">
            <div class="name-list-head">
              <span>进程名称</span>
              <span>{{ nameOptions.length - 1 }} 类</span>
            </div>
            <div ref="nameListRef" class="name-list-scroll scroll-area">
              <button
                v-for="option in nameOptions"
                :key="option.key"
                type="button"
                class="name-item"
                :class="{ active: activeName === option.key }"
                :title="option.name"
                @click="setNameFilter(option.key)"
              >
                <span class="name-item-label">{{ option.name }}</span>
                <span class="name-item-count">{{ option.count }}</span>
              </button>
            </div>
          </aside>

          <div class="process-table">
            <div class="process-table-head">
          <label class="check select-all-check" :class="{ disabled: !selectableFiltered.length }" title="全选 / 全不选当前筛选结果">
            <input
              type="checkbox"
              :checked="allFilteredSelected"
              :disabled="!selectableFiltered.length"
              @change="toggleAllFiltered"
            />
            <span class="check-box" :class="{ partial: someFilteredSelected }">
              <svg v-if="someFilteredSelected" viewBox="0 0 12 12" width="11" height="11" aria-hidden="true">
                <path d="M2.5 6h7" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
              </svg>
              <svg v-else viewBox="0 0 12 12" width="11" height="11" aria-hidden="true">
                <path d="M2 6.2 4.7 9 10 3.2" stroke="currentColor" stroke-width="1.8" fill="none" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
            </span>
          </label>
          <span>进程</span>
          <span class="num-head">PID</span>
          <span class="num-head">父 PID</span>
          <span class="num-head" title="TCP 监听 + UDP 绑定的端口（IPv4 / IPv6），多个用逗号分隔">端口</span>
          <span class="num-head">内存</span>
          <span class="num-head">线程</span>
        </div>

        <div class="process-list scroll-area">
          <div v-if="error" class="process-empty">
            <div class="empty-icon" aria-hidden="true">
              <svg viewBox="0 0 32 32" width="32" height="32">
                <rect x="5" y="7" width="22" height="17" rx="4" fill="none" stroke="currentColor" stroke-width="1.7" />
                <path d="M10 14h12M10 18h8" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" />
              </svg>
            </div>
            <h3>暂时读不到进程</h3>
            <p>{{ error }}</p>
          </div>

          <div v-else-if="loading && !processes.length" class="process-empty">
            <span class="spinner large" />
            <p>正在读取进程列表…</p>
          </div>

          <div v-else-if="!filtered.length" class="process-empty">
            <h3>没有匹配的进程</h3>
            <p>换个关键词，或清空搜索；也可以把左侧名称切回“全部”。</p>
          </div>

          <div v-else class="process-rows">
            <div
              v-for="process in filtered"
              :key="process.pid"
              class="process-row"
              :class="{ protected: process.protected, picked: selectedSet.has(process.pid) }"
            >
              <label class="check row-check" :class="{ disabled: process.protected }" :title="process.protected ? process.protectedReason ?? '受保护' : '选择该进程'">
                <input
                  type="checkbox"
                  :checked="selectedSet.has(process.pid)"
                  :disabled="process.protected"
                  @change="toggleRow(process.pid)"
                />
                <span class="check-box">
                  <svg viewBox="0 0 12 12" width="11" height="11" aria-hidden="true">
                    <path d="M2 6.2 4.7 9 10 3.2" stroke="currentColor" stroke-width="1.8" fill="none" stroke-linecap="round" stroke-linejoin="round" />
                  </svg>
                </span>
              </label>

              <div class="process-main">
                <div class="process-name-line">
                  <span class="process-name">{{ process.name }}</span>
                  <span v-if="process.pid === currentPid" class="self-badge">本应用</span>
                  <span v-else-if="process.protected" class="lock-badge">
                    <svg viewBox="0 0 12 14" width="10" height="12" aria-hidden="true">
                      <rect x="2" y="6" width="8" height="6.5" rx="1.5" fill="none" stroke="currentColor" stroke-width="1.3" />
                      <path d="M4 6V4.6a2 2 0 0 1 4 0V6" fill="none" stroke="currentColor" stroke-width="1.3" />
                    </svg>
                    {{ process.protectedReason }}
                  </span>
                </div>
                <span class="process-path" :title="process.path ?? ''">{{ process.path ?? "路径不可读取" }}</span>
              </div>

              <span class="pid-cell">{{ process.pid }}</span>
              <span class="pid-cell parent-cell">{{ process.parentPid }}</span>
              <span
                class="port-cell"
                :class="{ empty: !(process.ports ?? []).length }"
                :title="(process.ports ?? []).join(', ')"
              >{{ formatPorts(process.ports) }}</span>
              <span class="memory-cell">{{ formatMemory(process.memoryKb) }}</span>
              <span class="thread-cell">{{ process.threadCount }}</span>
            </div>
          </div>
        </div>
        </div>
        </div>

        <footer class="process-footer">
          <div class="process-summary">
            <span class="selection-count">已选 {{ selectedInfos.length }} 个</span>
            <span v-if="protectedCount" class="protected-hint">· {{ protectedCount }} 个受保护进程不可选</span>
            <span v-if="notice" class="process-notice" :class="noticeTone" :title="notice">{{ notice }}</span>
          </div>
          <div class="process-actions">
            <button class="btn danger-ghost" type="button" :disabled="!selectedInfos.length || killing" @click="runKill(false)">
              {{ killing ? "处理中…" : "仅结束选中" }}
            </button>
            <button class="btn danger-solid" type="button" :disabled="!selectedInfos.length || killing" @click="runKill(true)">
              结束选中及子进程
            </button>
          </div>
        </footer>
      </section>
    </div>
  </Teleport>
</template>

<style scoped>
.process-overlay {
  position: fixed;
  inset: 0;
  z-index: 70;
  display: grid;
  place-items: center;
  padding: 24px;
  background: rgba(12, 22, 34, 0.44);
  backdrop-filter: blur(2px);
  animation: overlay-in 0.16s ease;
}

.process-page {
  flex: 1;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  display: flex;
}

.process-modal,
.process-surface {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border-radius: 16px;
  background: var(--surface);
}

.process-modal {
  width: min(940px, calc(100vw - 48px));
  height: min(700px, calc(100vh - 48px));
  border: 1px solid rgba(255, 255, 255, 0.7);
  box-shadow: var(--shadow-float);
  animation: modal-in 0.2s ease;
}

.process-surface {
  flex: 1;
  width: 100%;
  min-width: 0;
  min-height: 0;
  border: 1px solid var(--line);
  box-shadow: var(--shadow-card);
}

.process-head {
  flex: none;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 20px;
  padding: 20px 22px 16px;
  border-bottom: 1px solid var(--line);
  background: linear-gradient(180deg, #f7fbff 0%, #ffffff 100%);
}

.process-title-wrap {
  display: flex;
  align-items: center;
  gap: 13px;
  min-width: 0;
}

.process-mark {
  flex: none;
  width: 42px;
  height: 42px;
  display: grid;
  place-items: center;
  border-radius: 12px;
  color: var(--accent);
  background: var(--accent-soft);
  border: 1px solid #cfdfef;
}

.process-head h2 {
  display: flex;
  align-items: baseline;
  gap: 10px;
  font-size: 19px;
  font-weight: 700;
  letter-spacing: -0.01em;
}

.process-count {
  font-size: 11.5px;
  font-weight: 500;
  color: var(--text-3);
}

.process-head p {
  margin-top: 3px;
  font-size: 12.5px;
  color: var(--text-2);
}

.close-btn {
  flex: none;
  width: 32px;
  height: 32px;
  display: grid;
  place-items: center;
  padding: 0;
  border: 1px solid transparent;
  border-radius: 9px;
  color: var(--text-2);
  background: transparent;
}

.close-btn:hover {
  color: var(--text);
  background: #edf3fa;
  border-color: var(--line);
}

.process-toolbar {
  flex: none;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px 22px 12px;
  border-bottom: 1px solid var(--line);
}

.process-search {
  flex: 1;
  min-width: 180px;
  display: flex;
  align-items: center;
  gap: 8px;
  height: 36px;
  padding: 0 10px;
  border: 1px solid var(--line-strong);
  border-radius: 9px;
  background: var(--surface-muted);
  color: var(--text-3);
}

.process-search:focus-within {
  border-color: var(--accent);
  background: #fff;
  box-shadow: 0 0 0 3px rgba(15, 108, 189, 0.1);
}

.process-search input {
  flex: 1;
  min-width: 0;
  height: 100%;
  padding: 0;
  border: 0;
  outline: 0;
  background: transparent;
  font-size: 12.5px;
  color: var(--text);
}

.search-clear {
  flex: none;
  width: 20px;
  height: 20px;
  display: grid;
  place-items: center;
  padding: 0;
  border: 0;
  border-radius: 50%;
  color: var(--text-3);
  background: transparent;
}

.search-clear:hover {
  color: var(--text);
  background: #e6edf5;
}

.toolbar-summary {
  flex: none;
  font-size: 11.5px;
  color: var(--text-3);
  white-space: nowrap;
}

.refresh-btn {
  flex: none;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 36px;
  padding: 0 11px;
  border: 1px solid var(--line-strong);
  border-radius: 9px;
  background: #fff;
  color: var(--text-2);
  font-size: 12.5px;
  font-weight: 600;
}

.refresh-btn:hover:not(:disabled) {
  border-color: #b3c9de;
  color: var(--accent-strong);
  background: #f6fbfe;
}

.refresh-btn:disabled {
  opacity: 0.55;
}

.process-body {
  flex: 1;
  min-height: 0;
  display: flex;
  overflow: hidden;
  background: #fff;
}

.name-list {
  flex: none;
  width: 214px;
  min-height: 0;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--line);
  background: #fbfcfe;
}

.name-list-head {
  flex: none;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 10px 13px 8px;
  font-size: 11.5px;
  font-weight: 600;
  color: var(--text-3);
  border-bottom: 1px solid #eef2f6;
}

.name-list-head span:last-child {
  font-size: 10.5px;
  font-weight: 500;
  color: var(--text-3);
}

.name-list-scroll {
  flex: 1;
  min-height: 0;
  overflow-x: hidden;
  overflow-y: auto;
  padding: 5px 0 7px;
}

.name-item {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  height: 34px;
  padding: 0 13px;
  border: 0;
  border-left: 3px solid transparent;
  border-radius: 0;
  background: transparent;
  color: var(--text-2);
  font-size: 12px;
  text-align: left;
  transition: background 0.12s ease, color 0.12s ease;
}

.name-item:hover {
  color: var(--accent-strong);
  background: #f1f7fd;
}

.name-item.active {
  color: var(--accent-strong);
  background: #eaf3fc;
  border-left-color: var(--accent);
  font-weight: 600;
}

.name-item-label {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.name-item-count {
  flex: none;
  min-width: 22px;
  text-align: right;
  font-size: 10.5px;
  font-weight: 500;
  font-variant-numeric: tabular-nums;
  color: var(--text-3);
}

.name-item.active .name-item-count {
  color: var(--accent);
}

.process-table {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.process-table-head {
  flex: none;
  display: grid;
  grid-template-columns: 24px minmax(0, 1fr) 72px 72px 104px 84px 54px;
  align-items: center;
  gap: 12px;
  padding: 8px 22px 9px;
  border-bottom: 1px solid var(--line);
  background: #fff;
  font-size: 11.5px;
  font-weight: 600;
  color: var(--text-3);
}

.num-head {
  text-align: right;
}

.port-cell {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-align: right;
  font-size: 12px;
  font-variant-numeric: tabular-nums;
  color: var(--text-2);
}

.port-cell.empty {
  color: var(--text-3);
}

.process-list {
  flex: 1;
  min-height: 0;
  background: #fff;
}

.process-row {
  display: grid;
  grid-template-columns: 24px minmax(0, 1fr) 72px 72px 104px 84px 54px;
  align-items: center;
  gap: 12px;
  min-height: 50px;
  padding: 7px 22px;
  border-bottom: 1px solid #edf1f6;
  transition: background 0.12s ease;
}

.process-row:hover {
  background: #f7fafd;
}

.process-row.picked {
  background: #f1f7fd;
}

.process-row.protected {
  background: #fbfcfd;
}

.process-row.protected .process-name {
  color: var(--text-2);
}

.check.disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.check.disabled input {
  cursor: not-allowed;
}

.row-check .check-box {
  width: 17px;
  height: 17px;
}

.process-main {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.process-name-line {
  display: flex;
  align-items: center;
  gap: 7px;
  min-width: 0;
}

.process-name {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
}

.self-badge {
  flex: none;
  height: 18px;
  padding: 0 6px;
  border-radius: 5px;
  border: 1px solid #bcd8ef;
  background: var(--accent-soft);
  color: var(--accent-strong);
  font-size: 10px;
  font-weight: 600;
  line-height: 16px;
}

.lock-badge {
  flex: none;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  height: 18px;
  padding: 0 6px;
  border-radius: 5px;
  border: 1px solid #ead8b5;
  background: var(--warn-soft);
  color: var(--warn);
  font-size: 10px;
  font-weight: 600;
  line-height: 16px;
  white-space: nowrap;
}

.process-path {
  display: block;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 11px;
  color: var(--text-3);
}

.pid-cell,
.thread-cell {
  text-align: right;
  font-size: 12px;
  font-variant-numeric: tabular-nums;
  color: var(--text-2);
}

.parent-cell {
  color: var(--text-3);
}

.memory-cell {
  text-align: right;
  font-size: 12px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  color: var(--text);
}

.thread-cell {
  color: var(--text-3);
}

.process-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 7px;
  min-height: 260px;
  padding: 40px 24px;
  text-align: center;
  color: var(--text-2);
}

.process-empty h3 {
  font-size: 14px;
  font-weight: 700;
  color: var(--text);
}

.process-empty p {
  max-width: 440px;
  font-size: 12.5px;
  line-height: 1.6;
  color: var(--text-2);
}

.empty-icon {
  width: 58px;
  height: 58px;
  margin-bottom: 3px;
  display: grid;
  place-items: center;
  border-radius: 15px;
  color: var(--accent);
  background: var(--accent-soft);
}

.spinner.large {
  width: 22px;
  height: 22px;
  border-width: 3px;
  color: var(--accent);
}

.process-footer {
  flex: none;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 22px 15px;
  border-top: 1px solid var(--line);
  background: #fbfcfe;
}

.process-summary {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-3);
  white-space: nowrap;
}

.selection-count {
  font-weight: 600;
  color: var(--text);
}

.protected-hint {
  color: var(--text-3);
}

.process-notice {
  min-width: 0;
  max-width: 330px;
  margin-left: 6px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 600;
}

.process-notice.ok {
  color: var(--success);
}

.process-notice.warn {
  color: var(--danger);
}

.process-actions {
  flex: none;
  display: flex;
  align-items: center;
  gap: 8px;
}

.danger-ghost {
  color: var(--danger);
  border-color: #efc7c3;
  background: #fff;
}

.danger-ghost:hover:not(:disabled) {
  border-color: #e3a8a2;
  background: var(--danger-soft);
  color: var(--danger);
}

.danger-solid {
  color: #fff;
  border-color: var(--danger);
  background: var(--danger);
  box-shadow: 0 10px 20px -12px rgba(180, 35, 24, 0.8);
}

.danger-solid:hover:not(:disabled) {
  border-color: #921d14;
  background: #921d14;
}

@keyframes overlay-in {
  from {
    opacity: 0;
  }
}

@keyframes modal-in {
  from {
    opacity: 0;
    transform: translateY(8px) scale(0.985);
  }
}

@media (max-height: 720px) {
  .process-modal {
    height: calc(100vh - 32px);
  }
  .process-head {
    padding-top: 15px;
    padding-bottom: 12px;
  }
}
</style>
