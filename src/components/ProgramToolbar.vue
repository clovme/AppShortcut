<script setup lang="ts">
defineProps<{ total: number; included: number; search: string; picking: boolean }>();

const emit = defineEmits<{
  (e: "update:search", value: string): void;
  (e: "add"): void;
}>();

function onSearchInput(event: Event) {
  emit("update:search", (event.target as HTMLInputElement).value);
}
</script>

<template>
  <div class="toolbar">
    <div class="toolbar-title">
      <h2>程序清单</h2>
      <span v-if="total" class="count">{{ total }}</span>
      <span class="toolbar-sub">已勾选 {{ included }} 个</span>
    </div>

    <div class="toolbar-actions">
      <label class="search">
        <svg viewBox="0 0 20 20" width="16" height="16" aria-hidden="true">
          <circle cx="8.6" cy="8.6" r="5.4" stroke="currentColor" stroke-width="1.6" fill="none" />
          <path d="M12.7 12.7 16.6 16.6" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
        </svg>
        <input
          :value="search"
          type="search"
          placeholder="搜索程序名称或路径"
          @input="onSearchInput"
        />
      </label>

      <button class="btn btn-add" type="button" :disabled="picking" @click="emit('add')">
        <i v-if="picking" class="spinner" />
        <svg v-else viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">
          <path d="M8 2.7v10.6M2.7 8h10.6" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" />
        </svg>
        {{ picking ? "选择中…" : "添加程序" }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 16px 18px 12px;
  flex: none;
}

.toolbar-title {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.toolbar-title h2 {
  font-size: 15px;
  font-weight: 700;
  color: var(--text);
}

.count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 22px;
  height: 20px;
  padding: 0 6px;
  border-radius: 6px;
  background: var(--accent-soft);
  color: var(--accent-strong);
  font-size: 12px;
  font-weight: 700;
}

.toolbar-sub {
  font-size: 12px;
  color: var(--text-3);
}

.toolbar-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: none;
}

.search {
  display: flex;
  align-items: center;
  gap: 7px;
  width: 230px;
  height: 34px;
  padding: 0 10px;
  border-radius: 9px;
  border: 1px solid var(--line);
  background: var(--surface-muted);
  color: var(--text-3);
  transition: border-color 0.16s ease, background 0.16s ease, box-shadow 0.16s ease;
}

.search:focus-within {
  border-color: var(--accent);
  background: #ffffff;
  box-shadow: 0 0 0 3px rgba(15, 108, 189, 0.12);
  color: var(--accent);
}

.search input {
  width: 100%;
  min-width: 0;
  border: none;
  outline: none;
  background: transparent;
  font-size: 12.5px;
  color: var(--text);
  user-select: text;
}

.search input::placeholder {
  color: var(--text-3);
}

.btn-add {
  height: 34px;
  padding: 0 13px;
  border-color: #bcd8ef;
  background: var(--accent-soft);
  color: var(--accent-strong);
  font-size: 13px;
}

.btn-add:hover:not(:disabled) {
  border-color: var(--accent);
  background: #dceafa;
}
</style>
