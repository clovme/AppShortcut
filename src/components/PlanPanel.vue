<script setup lang="ts">
import { computed } from "vue";
import type { LibraryOptions } from "../types";

const props = defineProps<{
  options: LibraryOptions;
  planned: number;
  remaining: number;
  creating: boolean;
  progressDone: number;
  progressTotal: number;
}>();

const emit = defineEmits<{
  (e: "toggle-option", key: keyof LibraryOptions): void;
  (e: "create-all"): void;
  (e: "clear-marks"): void;
}>();

const createLabel = computed(() => {
  if (props.creating) return `正在创建 ${props.progressDone} / ${props.progressTotal}`;
  if (!props.planned) return "请先勾选程序";
  if (!props.remaining) return "没有待创建的快捷方式";
  return `一键创建 ${props.remaining} 个快捷方式`;
});

const hintText = computed(() => {
  if (!props.planned) return "请先勾选要创建快捷方式的程序";
  if (!props.remaining) return "当前程序设置的快捷方式都已创建完成";
  return "按每个程序行内选中的桌面 / 开始菜单位置创建";
});
</script>

<template>
  <aside class="plan">
    <section class="panel plan-card">
      <div class="section-head">
        <h2>创建位置</h2>
        <span class="section-note">每行单独切换</span>
      </div>

      <div class="target-guide">
        <div class="guide-line">
          <span class="guide-chip">桌面</span>
          <span class="guide-plus">+</span>
          <span class="guide-chip">开始菜单</span>
          <span class="guide-default">默认两者</span>
        </div>
        <p>
          新加入程序默认两个位置都创建。左侧每一行都可以单独关掉「桌面」或「开始菜单」，至少保留一个，不会一刀切。
        </p>
      </div>

      <div class="rule" />

      <div class="section-head">
        <h2>高级选项</h2>
      </div>

      <label class="option-row">
        <span class="option-copy">
          <b>覆盖同名快捷方式</b>
          <small>关闭时仅跳过目标一致的已有快捷方式</small>
        </span>
        <span class="switch" :class="{ on: props.options.overwrite }" />
        <input
          class="sr-only"
          type="checkbox"
          :checked="props.options.overwrite"
          @change="emit('toggle-option', 'overwrite')"
        />
      </label>

      <p class="scope-note">
        每个程序可单独选择「当前用户 / 所有用户」；选「所有用户」会写入公共目录，可能需要管理员权限。
      </p>

      <div class="summary">
        <div class="summary-row">
          <span>创建位置</span>
          <b>按每个程序设置</b>
        </div>

        <div class="summary-hero">
          <span>将创建</span>
          <div class="hero-number">
            <strong>{{ props.remaining }}</strong>
            <em>个快捷方式</em>
          </div>
        </div>
        <p class="summary-hint">{{ hintText }}</p>
      </div>

      <div v-if="props.planned > 0 && !props.remaining && !props.creating" class="done-banner">
        <span class="done-icon">
          <svg viewBox="0 0 14 14" width="12" height="12" aria-hidden="true">
            <path d="M2.4 7.4 5.4 10.3 11.6 3.9" stroke="currentColor" stroke-width="1.9" fill="none" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </span>
        <span class="done-copy">
          <b>全部快捷方式已创建</b>
          <small>{{ props.planned }} 个快捷方式均已生成</small>
        </span>
      </div>

      <button
        v-else
        class="btn btn-primary create-all"
        :class="{ 'is-running': props.creating }"
        type="button"
        :disabled="props.creating || !props.remaining"
        @click="emit('create-all')"
      >
        <i v-if="props.creating" class="spinner" />
        {{ createLabel }}
      </button>

      <button class="btn btn-ghost clear-btn" type="button" @click="emit('clear-marks')">
        清除创建标记与快捷方式
      </button>
    </section>
  </aside>
</template>

<style scoped>
.plan {
  display: flex;
  flex-direction: column;
  width: 336px;
  flex: none;
  min-height: 0;
}

.plan-card {
  display: flex;
  flex-direction: column;
  flex: 1;
  gap: 8px;
  padding: 16px;
  min-height: 0;
  overflow: auto;
}

.section-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
  padding: 0 2px;
}

.section-head h2 {
  font-size: 14px;
  font-weight: 700;
  color: var(--text);
}

.section-note {
  font-size: 11.5px;
  color: var(--text-3);
}

.target-guide {
  display: grid;
  gap: 8px;
  padding: 12px;
  border-radius: var(--radius);
  border: 1px solid var(--line);
  background: var(--surface-muted);
}

.guide-line {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.guide-chip {
  display: inline-flex;
  align-items: center;
  height: 22px;
  padding: 0 9px;
  border-radius: 6px;
  border: 1px solid #bcd8ef;
  background: #ffffff;
  color: var(--accent-strong);
  font-size: 11px;
  font-weight: 600;
  white-space: nowrap;
}

.guide-plus {
  color: var(--text-3);
  font-size: 11px;
  font-weight: 700;
}

.guide-default {
  margin-left: auto;
  padding: 2px 7px;
  border-radius: 999px;
  background: #eaf4fd;
  color: #3f6f97;
  font-size: 10.5px;
  white-space: nowrap;
}

.target-guide p {
  margin: 0;
  font-size: 11.5px;
  line-height: 1.55;
  color: var(--text-2);
}

.rule {
  height: 1px;
  background: var(--line);
  margin: 4px 0;
}

.option-row {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 2px 2px 4px;
  cursor: pointer;
}

.option-copy {
  display: grid;
  gap: 2px;
}

.option-copy b {
  font-size: 12.5px;
  font-weight: 600;
  color: var(--text);
}

.option-copy small {
  font-size: 11.5px;
  color: var(--text-3);
}

.summary {
  display: grid;
  gap: 8px;
  margin-top: 4px;
  padding: 14px;
  border-radius: var(--radius);
  background: var(--surface-muted);
  border: 1px solid var(--line);
}

.summary-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12.5px;
  color: var(--text-2);
}

.summary-row b {
  font-weight: 600;
  color: var(--text);
}

.summary-hero {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 10px;
  margin-top: 4px;
  padding-top: 12px;
  border-top: 1px dashed var(--line-strong);
  font-size: 12.5px;
  color: var(--text-2);
}

.hero-number {
  display: flex;
  align-items: baseline;
  gap: 6px;
}

.hero-number strong {
  font-size: 30px;
  line-height: 1;
  font-weight: 700;
  letter-spacing: -0.02em;
  color: var(--accent-strong);
}

.hero-number em {
  font-style: normal;
  font-size: 11.5px;
  color: var(--text-3);
}

.summary-hint {
  font-size: 11px;
  color: var(--text-3);
  text-align: right;
}

.create-all {
  height: 46px;
  font-size: 14px;
  border-radius: 12px;
}

.create-all:disabled {
  opacity: 1;
  background: #dfeaf5;
  border-color: #dfeaf5;
  color: #47617a;
  box-shadow: none;
}

.create-all.is-running:disabled {
  opacity: 1;
  background: var(--accent);
  border-color: var(--accent);
  color: #ffffff;
}

.done-banner {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 9px 12px;
  border-radius: 12px;
  border: 1px solid #bfe3cd;
  background: var(--success-soft);
}

.done-icon {
  flex: none;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  background: var(--success);
  color: #ffffff;
}

.done-copy {
  display: grid;
  gap: 1px;
  color: var(--success);
}

.done-copy b {
  font-size: 12px;
  font-weight: 700;
  line-height: 1.3;
}

.done-copy small {
  font-size: 10.5px;
  line-height: 1.35;
  color: #3f7c57;
}

.clear-btn {
  height: 30px;
  font-size: 12px;
}

.scope-note {
  padding: 0 2px 2px;
  font-size: 11px;
  line-height: 1.5;
  color: var(--text-3);
}
</style>
