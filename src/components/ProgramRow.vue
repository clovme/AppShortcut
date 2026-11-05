<script setup lang="ts">
import { computed, nextTick, ref } from "vue";
import type { CreateTargets, ProgramItem, TargetKind, UserScope } from "../types";

const props = defineProps<{
  item: ProgramItem;
  groupOptions: string[];
  icon?: string;
}>();

const emit = defineEmits<{
  (e: "toggle", value: boolean): void;
  (e: "scope", value: UserScope): void;
  (e: "targets", value: CreateTargets): void;
  (e: "rename", value: string): void;
  (e: "group", value: string): void;
  (e: "create"): void;
  (e: "remove"): void;
}>();

const editingName = ref(false);
const draftName = ref("");
const nameInput = ref<HTMLInputElement | null>(null);

const editingGroup = ref(false);
const manualGroup = ref(false);
const draftGroup = ref("");
const groupInput = ref<HTMLInputElement | null>(null);
const groupSelect = ref<HTMLSelectElement | null>(null);

function startRename() {
  draftName.value = props.item.name;
  editingName.value = true;
  void nextTick(() => {
    nameInput.value?.focus();
    nameInput.value?.select();
  });
}

function commitName() {
  if (!editingName.value) return;
  const next = draftName.value.trim();
  editingName.value = false;
  if (!next || next === props.item.name) return;
  emit("rename", next);
}

function cancelRename() {
  editingName.value = false;
  draftName.value = props.item.name;
}

const groupChoices = computed(() => {
  const list = props.groupOptions.filter((group) => group !== props.item.group);
  if (props.item.group) list.unshift(props.item.group);
  return list;
});

function startGroupEdit() {
  draftGroup.value = props.item.group;
  editingGroup.value = true;
  manualGroup.value = groupChoices.value.length === 0;
  void nextTick(() => {
    if (manualGroup.value) {
      groupInput.value?.focus();
      groupInput.value?.select();
    } else {
      groupSelect.value?.focus();
    }
  });
}

function startManualGroup() {
  manualGroup.value = true;
  draftGroup.value = "";
  void nextTick(() => {
    groupInput.value?.focus();
  });
}

function onGroupSelect(event: Event) {
  const next = (event.target as HTMLSelectElement).value;
  if (next === "__new_group__") {
    startManualGroup();
    return;
  }
  editingGroup.value = false;
  manualGroup.value = false;
  if (next !== props.item.group) emit("group", next);
}

function onGroupBlur() {
  // 选“新建分组…”后焦点会转到输入框，这次失焦不取消编辑
  if (manualGroup.value) return;
  cancelGroup();
}

function commitGroup() {
  if (!editingGroup.value) return;
  editingGroup.value = false;
  manualGroup.value = false;
  const next = draftGroup.value.trim();
  // 手动输入只用于新建分组；留空视为取消，避免误清掉已有分组
  if (!next || next === props.item.group) return;
  emit("group", next);
}

function cancelGroup() {
  editingGroup.value = false;
  manualGroup.value = false;
  draftGroup.value = props.item.group;
}

function clearGroup() {
  emit("group", "");
}

const activeTargets = computed<TargetKind[]>(() => {
  const list: TargetKind[] = [];
  if (props.item.targets.desktop) list.push("desktop");
  if (props.item.targets.startmenu) list.push("startmenu");
  return list;
});

function toggleTarget(kind: TargetKind) {
  const next: CreateTargets = { ...props.item.targets, [kind]: !props.item.targets[kind] };
  if (kind === "startmenu" && props.item.targets.startmenu) cancelGroup();
  emit("targets", next);
}

const pending = computed(() =>
  activeTargets.value.filter((target) => !props.item.createdTargets.includes(target))
);

const initial = computed(() => {
  const name = props.item.name.trim();
  const first = name.slice(0, 1);
  return /^[a-z]/i.test(first) ? first.toUpperCase() : first || "?";
});

const tileStyle = computed(() => ({
  backgroundColor: `${props.item.color}1a`,
  color: props.item.color,
  boxShadow: `inset 0 0 0 1px ${props.item.color}33`,
}));

const statusKind = computed(() => {
  if (props.item.failed) return "failed";
  if (props.item.creating) return "progress";
  if (!props.item.included) return "excluded";
  if (!activeTargets.value.length) return "idle";
  const done = activeTargets.value.filter((target) =>
    props.item.createdTargets.includes(target)
  ).length;
  if (!done) return "idle";
  if (done < activeTargets.value.length) return "partial";
  const allSkipped = activeTargets.value.every((target) =>
    props.item.skippedTargets.includes(target)
  );
  return allSkipped ? "skipped" : "created";
});

const statusText = computed(() => {
  switch (statusKind.value) {
    case "failed":
      return "创建失败";
    case "progress":
      return "创建中";
    case "excluded":
      return "已排除";
    case "created":
      return "已创建";
    case "partial":
      return "部分已存在";
    case "skipped":
      return "已存在";
    default:
      return "未创建";
  }
});

const statusTitle = computed(() =>
  props.item.errorMessage ??
  (statusKind.value === "skipped"
    ? "磁盘上已存在同名且指向该程序的快捷方式"
    : "")
);

const actionText = computed(() => {
  if (props.item.creating) return "创建中";
  return pending.value.length ? "创建" : "已完成";
});

const canCreate = computed(
  () => props.item.included && Boolean(pending.value.length) && !props.item.creating
);

const scopeTitle = computed(() => {
  const group = props.item.group ? `「${props.item.group}」分组` : "开始菜单根目录";
  return props.item.scope === "all"
    ? `写入所有用户目录（开始菜单为公共开始菜单的${group}），可能需要管理员权限`
    : `只写入当前用户目录（开始菜单为${group}），无需管理员权限`;
});

function onToggle(event: Event) {
  emit("toggle", (event.target as HTMLInputElement).checked);
}

function onScopeChange(event: Event) {
  emit("scope", (event.target as HTMLSelectElement).value as UserScope);
}
</script>

<template>
  <div
    class="program-row"
    :class="{ 'is-excluded': !item.included, 'is-creating': item.creating }"
  >
    <label class="check" :title="item.included ? '取消勾选即排除该程序' : '勾选后加入创建'">
      <input type="checkbox" :checked="item.included" @change="onToggle" />
      <span class="check-box">
        <svg viewBox="0 0 12 12" width="11" height="11" aria-hidden="true">
          <path d="M2 6.2 4.7 9 10 3.2" stroke="currentColor" stroke-width="1.8" fill="none" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </span>
    </label>

    <div class="program-main">
      <div class="tile" :class="{ 'has-icon': icon }" :style="icon ? undefined : tileStyle">
        <img v-if="icon" class="tile-image" :src="icon" alt="" />
        <template v-else>{{ initial }}</template>
      </div>
      <div class="program-text">
        <div class="program-name">
          <input
            v-if="editingName"
            ref="nameInput"
            v-model="draftName"
            class="name-input"
            maxlength="120"
            aria-label="快捷方式名称"
            @keydown.enter.prevent="commitName"
            @keydown.esc.prevent="cancelRename"
            @blur="commitName"
          />
          <template v-else>
            <span class="name" :title="`${item.name} · 双击可改名`" @dblclick="startRename">
              {{ item.name }}
            </span>
            <button class="btn-rename" type="button" title="修改快捷方式名称" @click="startRename">
              <svg viewBox="0 0 12 12" width="11" height="11" aria-hidden="true">
                <path
                  d="M2 9.9 2.4 7.4 7.8 2 10 4.2 4.6 9.6 2 9.9Z"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.2"
                  stroke-linejoin="round"
                />
              </svg>
            </button>
          </template>
          <span v-if="!item.included" class="excluded-tag">已排除</span>
        </div>
        <div class="program-path" :title="item.path">{{ item.path }}</div>
      </div>
    </div>

    <div class="program-targets">
      <button
        class="target-chip"
        :class="{ off: !item.targets.desktop }"
        type="button"
        :aria-pressed="item.targets.desktop"
        title="点击切换：是否创建桌面快捷方式（每个程序至少保留一个位置）"
        @click="toggleTarget('desktop')"
      >
        桌面
      </button>
      <button
        class="target-chip"
        :class="{ off: !item.targets.startmenu }"
        type="button"
        :aria-pressed="item.targets.startmenu"
        title="点击切换：是否创建开始菜单快捷方式（每个程序至少保留一个位置）"
        @click="toggleTarget('startmenu')"
      >
        开始菜单
      </button>

      <template v-if="item.targets.startmenu">
        <div v-if="!editingGroup && item.group" class="group-line">
          <button
            class="group-chip"
            type="button"
            :title="`开始菜单分组：${item.group}，点击修改`"
            @click="startGroupEdit"
          >
            <svg viewBox="0 0 12 12" width="10" height="10" aria-hidden="true">
              <path d="M1.6 3.1h3l1 1.2h4.8v4.6H1.6Z" fill="none" stroke="currentColor" stroke-width="1.1" stroke-linejoin="round" />
            </svg>
            <span class="group-name">{{ item.group }}</span>
          </button>
          <button class="group-clear" type="button" title="取消分组，直接放到开始菜单根目录" @click="clearGroup">
            <svg viewBox="0 0 10 10" width="8" height="8" aria-hidden="true">
              <path d="M2 2l6 6M8 2 2 8" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
            </svg>
          </button>
        </div>

        <button
          v-else-if="!editingGroup"
          class="group-add"
          type="button"
          title="为这个程序设置开始菜单分组（默认不分组）"
          aria-label="设置开始菜单分组"
          @click="startGroupEdit"
        >
          <svg viewBox="0 0 14 14" width="12" height="12" aria-hidden="true">
            <path d="M1.5 3.2h3.1l.9 1.2h6.9v5.8H1.5Z" fill="none" stroke="currentColor" stroke-width="1.1" stroke-linejoin="round" />
            <path d="M8.2 5.7v3.4M6.5 7.4h3.4" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" />
          </svg>
        </button>

        <template v-else-if="!manualGroup && groupChoices.length">
          <select
            ref="groupSelect"
            class="group-select"
            :value="item.group"
            aria-label="选择开始菜单分组"
            @change="onGroupSelect"
            @blur="onGroupBlur"
            @keydown.esc.prevent="cancelGroup"
          >
            <option value="">不分组</option>
            <option v-for="group in groupChoices" :key="group" :value="group">{{ group }}</option>
            <option value="__new_group__">新建分组…</option>
          </select>
        </template>

        <input
          v-else
          ref="groupInput"
          v-model="draftGroup"
          class="group-input"
          maxlength="60"
          placeholder="输入新分组名"
          aria-label="开始菜单分组名称"
          @keydown.enter.prevent="commitGroup"
          @keydown.esc.prevent="cancelGroup"
          @blur="commitGroup"
        />
      </template>
    </div>

    <label class="scope-field" :class="{ 'is-all': item.scope === 'all' }" :title="scopeTitle">
      <select class="scope-select" :value="item.scope" @change="onScopeChange">
        <option value="current">当前用户</option>
        <option value="all">所有用户</option>
      </select>
    </label>

    <div class="program-status">
      <span class="status" :class="statusKind" :title="statusTitle">
        <i v-if="item.creating" class="spinner" />
        <i v-else class="dot" />
        {{ statusText }}
      </span>
    </div>

    <div class="program-actions">
      <button class="btn-create" type="button" :disabled="!canCreate" @click="emit('create')">
        {{ actionText }}
      </button>
      <button class="btn-icon" type="button" title="从清单移除" @click="emit('remove')">
        <svg viewBox="0 0 14 14" width="12" height="12" aria-hidden="true">
          <path d="M3 3l8 8M11 3 3 11" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" />
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.program-row {
  display: grid;
  grid-template-columns: 20px minmax(0, 1fr) 132px 86px 86px 104px;
  align-items: center;
  gap: 12px;
  padding: 11px 18px;
  border-radius: 10px;
  transition: background 0.14s ease, opacity 0.14s ease;
}

.program-row:hover {
  background: var(--surface-muted);
}

.program-row.is-excluded {
  opacity: 0.56;
}

.program-row.is-creating {
  background: #f4f9ff;
}

.program-main {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.tile {
  flex: none;
  width: 36px;
  height: 36px;
  border-radius: 10px;
  display: grid;
  place-items: center;
  font-size: 15px;
  font-weight: 700;
  letter-spacing: -0.01em;
}

.tile.has-icon {
  background: transparent;
  box-shadow: none;
}

.tile-image {
  width: 32px;
  height: 32px;
  object-fit: contain;
  image-rendering: auto;
}

.program-text {
  min-width: 0;
  display: grid;
  gap: 2px;
}

.program-name {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.name {
  font-size: 13.5px;
  font-weight: 600;
  color: var(--text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.btn-rename {
  flex: none;
  width: 18px;
  height: 18px;
  padding: 0;
  border: none;
  border-radius: 5px;
  background: transparent;
  color: var(--text-3);
  opacity: 0.4;
  display: grid;
  place-items: center;
  transition: opacity 0.14s ease, background 0.14s ease, color 0.14s ease;
}

.program-row:hover .btn-rename {
  opacity: 1;
}

.btn-rename:hover {
  background: var(--accent-soft);
  color: var(--accent-strong);
}

.name-input {
  flex: 1;
  min-width: 0;
  max-width: 260px;
  height: 24px;
  padding: 0 8px;
  border-radius: 6px;
  border: 1px solid var(--accent);
  background: #ffffff;
  color: var(--text);
  font: inherit;
  font-size: 13px;
  font-weight: 600;
  outline: none;
  box-shadow: 0 0 0 2px rgba(15, 108, 189, 0.12);
}

.excluded-tag {
  flex: none;
  height: 18px;
  padding: 0 6px;
  border-radius: 5px;
  background: #eef1f5;
  color: var(--text-3);
  font-size: 10.5px;
  font-weight: 600;
  line-height: 18px;
}

.program-path {
  font-size: 11.5px;
  color: var(--text-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: "Cascadia Mono", Consolas, "Microsoft YaHei UI", monospace;
}

.program-targets {
  display: flex;
  align-items: center;
  gap: 5px;
  flex-wrap: wrap;
}

.target-chip {
  display: inline-flex;
  align-items: center;
  height: 22px;
  padding: 0 8px;
  border-radius: 6px;
  border: 1px solid #d8e4f0;
  background: #f3f8fd;
  color: var(--accent-strong);
  font: inherit;
  font-size: 11px;
  font-weight: 600;
  white-space: nowrap;
  cursor: pointer;
  transition: border-color 0.16s ease, background 0.16s ease, color 0.16s ease;
}

.target-chip:hover {
  border-color: #a7caea;
  background: #eaf4fd;
}

.target-chip:focus-visible {
  outline: 2px solid rgba(15, 108, 189, 0.28);
  outline-offset: 1px;
}

.target-chip.off {
  border-color: var(--line);
  background: #f7f9fc;
  color: var(--text-3);
  text-decoration: line-through;
  text-decoration-thickness: 1px;
}

.target-chip.off:hover {
  border-color: #c3d2e2;
  background: #f2f6fb;
}

.group-line {
  flex: 1 0 100%;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 4px;
}

.group-chip {
  flex: 1;
  min-width: 0;
  height: 20px;
  padding: 0 6px;
  border-radius: 5px;
  border: 1px dashed #c9d9e8;
  background: #fbfdff;
  color: var(--text-2);
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 10.5px;
  font-weight: 500;
  transition: border-color 0.14s ease, background 0.14s ease, color 0.14s ease;
}

.group-add {
  flex: none;
  width: 20px;
  height: 22px;
  padding: 0;
  border-radius: 5px;
  border: 1px dashed #d5e2ee;
  background: #fbfdff;
  color: var(--text-3);
  opacity: 0.7;
  display: grid;
  place-items: center;
  transition: opacity 0.14s ease, border-color 0.14s ease, background 0.14s ease, color 0.14s ease;
}

.group-add:hover,
.program-row:hover .group-add {
  opacity: 1;
}

.group-add:hover {
  border-color: var(--accent);
  background: var(--accent-soft);
  color: var(--accent-strong);
}

.group-clear {
  flex: none;
  width: 16px;
  height: 18px;
  padding: 0;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--text-3);
  opacity: 0.55;
  display: grid;
  place-items: center;
  transition: opacity 0.14s ease, background 0.14s ease, color 0.14s ease;
}

.group-clear:hover {
  opacity: 1;
  background: var(--danger-soft);
  color: var(--danger);
}

.group-chip:hover {
  border-color: var(--accent);
  background: var(--accent-soft);
  color: var(--accent-strong);
}

.group-chip svg {
  flex: none;
  color: var(--text-3);
}

.group-name {
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.group-select {
  flex: 1 0 100%;
  min-width: 0;
  height: 22px;
  padding: 0 18px 0 6px;
  border-radius: 5px;
  border: 1px solid var(--accent);
  background-color: #ffffff;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 10 10' width='9' height='9'%3E%3Cpath d='M2.4 3.6 5 6.4 7.6 3.6' fill='none' stroke='%230f6cbd' stroke-width='1.3' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 6px center;
  appearance: none;
  -webkit-appearance: none;
  color: var(--text);
  font: inherit;
  font-size: 10.5px;
  font-weight: 500;
  outline: none;
  cursor: pointer;
  box-shadow: 0 0 0 2px rgba(15, 108, 189, 0.12);
}

.group-select:focus {
  border-color: var(--accent);
}

.group-input {
  flex: 1 0 100%;
  min-width: 0;
  height: 22px;
  padding: 0 6px;
  border-radius: 5px;
  border: 1px solid var(--accent);
  background: #ffffff;
  color: var(--text);
  font: inherit;
  font-size: 10.5px;
  outline: none;
  box-shadow: 0 0 0 2px rgba(15, 108, 189, 0.12);
}

.scope-field {
  position: relative;
  display: block;
  min-width: 0;
}

.scope-select {
  appearance: none;
  -webkit-appearance: none;
  width: 100%;
  height: 26px;
  padding: 0 20px 0 8px;
  border-radius: 7px;
  border: 1px solid var(--line);
  background-color: #ffffff;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 10 10' width='10' height='10'%3E%3Cpath d='M2.4 3.6 5 6.4 7.6 3.6' fill='none' stroke='%2364748b' stroke-width='1.3' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 7px center;
  color: var(--text-2);
  font-size: 11.5px;
  font-weight: 600;
  cursor: pointer;
  transition: border-color 0.14s ease, background-color 0.14s ease, color 0.14s ease;
}

.scope-select:hover {
  border-color: var(--line-strong);
  color: var(--text);
}

.scope-select:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 2px rgba(15, 108, 189, 0.12);
}

.scope-field.is-all .scope-select {
  border-color: #e6d2ae;
  background-color: #fffaf1;
  color: #8a5a12;
}

.program-status {
  display: flex;
  align-items: center;
}

.status {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 24px;
  padding: 0 9px;
  border-radius: 999px;
  font-size: 11.5px;
  font-weight: 600;
  white-space: nowrap;
}

.dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
  opacity: 0.65;
}

.status.created {
  background: var(--success-soft);
  color: var(--success);
}

.status.partial {
  background: var(--warn-soft);
  color: var(--warn);
}

.status.progress {
  background: var(--accent-soft);
  color: var(--accent-strong);
}

.status.failed {
  background: var(--danger-soft);
  color: var(--danger);
}

.status.skipped {
  background: #eef3f9;
  color: #47617a;
}

.status.excluded,
.status.idle {
  background: #f1f4f8;
  color: var(--text-3);
}

.program-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 6px;
}

.btn-create {
  height: 28px;
  padding: 0 12px;
  border-radius: 8px;
  border: 1px solid var(--line-strong);
  background: #fff;
  color: var(--text);
  font-size: 12px;
  font-weight: 600;
  transition: background 0.14s ease, border-color 0.14s ease, color 0.14s ease;
}

.btn-create:hover:not(:disabled) {
  border-color: var(--accent);
  color: var(--accent-strong);
  background: var(--accent-soft);
}

.btn-create:disabled {
  color: var(--text-3);
  border-color: var(--line);
  background: #f7f9fc;
}

.btn-icon {
  width: 26px;
  height: 26px;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: var(--text-3);
  display: grid;
  place-items: center;
  transition: background 0.14s ease, color 0.14s ease;
}

.btn-icon:hover {
  background: var(--danger-soft);
  color: var(--danger);
}
</style>
