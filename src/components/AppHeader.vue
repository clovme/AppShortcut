<script setup lang="ts">
defineProps<{ included: number; total: number }>();
</script>

<template>
  <header class="app-header">
    <div class="brand">
      <div class="brand-mark">
        <svg viewBox="0 0 34 34" width="34" height="34" aria-hidden="true">
          <defs>
            <linearGradient id="brandGradient" x1="0" y1="0" x2="1" y2="1">
              <stop offset="0" stop-color="#2f8fe0" />
              <stop offset="1" stop-color="#0a579b" />
            </linearGradient>
          </defs>
          <rect x="1.5" y="1.5" width="27" height="27" rx="8.5" fill="url(#brandGradient)" />
          <rect x="7" y="8" width="16" height="3" rx="1.5" fill="#ffffff" opacity="0.92" />
          <rect x="7" y="13.5" width="11" height="3" rx="1.5" fill="#ffffff" opacity="0.62" />
          <rect x="7" y="19" width="13.5" height="3" rx="1.5" fill="#ffffff" opacity="0.62" />
          <circle cx="25.5" cy="25.5" r="7.5" fill="#ffffff" stroke="#e2e8f0" stroke-width="1" />
          <path
            d="M21.9 25.5h7.2m0 0-2.5-2.5m2.5 2.5-2.5 2.5"
            stroke="#0f6cbd"
            stroke-width="1.9"
            stroke-linecap="round"
            stroke-linejoin="round"
            fill="none"
          />
        </svg>
      </div>
      <div class="brand-copy">
        <h1>
          快捷方式管家
          <span class="brand-en">AppShortcut</span>
        </h1>
        <p>选好程序，桌面与开始菜单快捷方式一键生成</p>
      </div>
    </div>

    <div class="header-meta">
      <span class="chip">Windows · 当前用户</span>
      <span class="chip chip-accent">已加入 {{ included }} / {{ total }}</span>
    </div>
  </header>
</template>

<style scoped>
.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  padding: 22px 24px 18px;
  flex: none;
}

.brand {
  display: flex;
  align-items: center;
  gap: 14px;
  min-width: 0;
}

.brand-mark {
  flex: none;
  width: 46px;
  height: 46px;
  border-radius: 14px;
  display: grid;
  place-items: center;
  background: #ffffff;
  border: 1px solid var(--line);
  box-shadow: 0 8px 18px -12px rgba(15, 108, 189, 0.6);
}

.brand-copy {
  min-width: 0;
}

.brand-copy h1 {
  display: flex;
  align-items: baseline;
  gap: 10px;
  font-size: 21px;
  font-weight: 700;
  letter-spacing: -0.01em;
  color: var(--text);
}

.brand-en {
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--text-3);
}

.brand-copy p {
  margin-top: 3px;
  font-size: 13px;
  color: var(--text-2);
}

.header-meta {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: none;
}

.chip {
  display: inline-flex;
  align-items: center;
  height: 28px;
  padding: 0 12px;
  border-radius: 999px;
  border: 1px solid var(--line);
  background: rgba(255, 255, 255, 0.7);
  color: var(--text-2);
  font-size: 12px;
  font-weight: 500;
}

.chip-accent {
  border-color: #bcd8ef;
  background: var(--accent-soft);
  color: var(--accent-strong);
  font-weight: 600;
}
</style>
