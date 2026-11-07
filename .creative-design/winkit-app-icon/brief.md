# WinKit 工具箱 · 应用图标 Brief

## 主题

为 Windows 桌面工具「WinKit 工具箱 / WinKit.exe」设计一枚可作图标源的方形 SVG，并据此生成 Tauri 所需的 PNG / ICO / ICNS / Square*Logo 全套图标。

## 核心对象

- 产品：WinKit 工具箱（快捷方式、进程管理、环境变量三块能力）。
- 品牌基调：Windows 蓝、干净、可信、工具感；不使用 indigo / purple。
- 现有视觉来源：`AppHeader.vue` 品牌 mark（蓝色圆角方形 `#2f8fe0 → #0a579b`、白色列表条、右下白圆蓝箭头）与 `global.css` 主色 `#0f6cbd`。

## 设计决定

- 图形：蓝色圆角方形底板 + 白色粗笔画 `W` 字标 + 右下角白色圆形快捷方式箭头徽标。
- 语义：`W` 直接对应 WinKit；箭头徽标承接现有标题栏品牌 mark 的「快捷方式」语义；三块业务不逐一刻画，避免小尺寸糊合。
- 构图：W 占主要视觉面积，徽标只作次级识别；四周保留安全区，16 / 32 / 48 px 仍可辨认。
- 色彩：底色沿用标题栏蓝渐变；W 为近白冷色；箭头为 `#0f6cbd`。颜色角色不超过三种。
- 小尺寸策略：不添加细线、纹理、文字和小装饰；W 笔画足够粗，徽标只保留圆形与简洁箭头。

## 交付

- `app-icon.svg`：1024 × 1024，透明画布，圆角方形外透明。
- 由 `tauri icon` 生成 `src-tauri/icons` 全套图标。
- 重新构建 `WinKit.exe`，让 PE 资源图标同步更新。

## 验收

- 方形、透明角、无锯齿；16 / 32 / 48 / 128 px 下 W 和箭头仍可识别。
- 与标题栏品牌蓝一致；不使用 indigo / purple。
- `icon.ico`、`icon.icns`、`32x32.png`、`128x128.png`、`Square*Logo.png` 均实际生成且来自同一 SVG。
- exe 重新构建成功，资源图标随 `icon.ico` 更新。
