# 快捷方式管家 · AppShortcut

Windows 桌面应用：把常用程序加入清单，一键在**桌面**和**开始菜单**生成快捷方式。新加入的程序默认两个位置都创建，也可以按行关掉其中一个、只建桌面或只建开始菜单；开始菜单默认不分组、直接放 `Programs` 根目录，需要时再按行设置分组；支持单个创建，也可以取消勾选排除某个程序。

技术栈：Tauri 2 + Rust + Vue 3 + TypeScript + Vite。

## 当前进度

- 界面完成：程序清单、每程序桌面 / 开始菜单位置切换、每程序自定义开始菜单分组、范围选择、快捷方式改名、排除 / 单建 / 一键创建、手动刷新、真实状态、进度反馈、真实程序图标
- 后端完成：系统原生文件选择窗口、`.lnk` 写入、目录解析、按 TargetPath 判断已存在、自定义分组文件夹、单例模式、真实 `.lnk` 状态检查、按每个程序自己的创建位置清除、清单文件读写
- 已通过：`vue-tsc` 类型检查、Vite 生产构建、`cargo check`、8 个 Rust 单元测试（真实写出 `.lnk`、同名同目标跳过、同名异目标冲突、自定义分组目录、真实状态判定与清除、exe 图标抽取、清单读写与损坏备份）
- 实机验证：单例模式（连续启动两次只有 1 个进程）；NotepadTest 写入 `Programs\RustRover家族\NotepadTest.lnk` 后状态变为“已存在”；点击“清除创建标记与快捷方式”后 `.lnk` 与空分组目录都被删除，状态回到“未创建”；伪造 JSON 里的创建标记会在启动时被磁盘真实状态纠正。
- 实机验证（每个程序单独位置）：TargetDesktopTest 只勾桌面、TargetStartTest 只勾开始菜单，一次创建后桌面只出现前者、`Programs` 根目录只出现后者，交叉位置没有文件；清除后两个 `.lnk` 都被删除，状态回到“未创建”。
- 实机验证（手动刷新）：RefreshTest 双建后手动删掉桌面 `.lnk`，界面仍显示“已存在”；点“刷新”（F5 / Ctrl+R 同样触发）后变为“部分已存在”、待创建 1 个，并提示“检测到 1 / 2 个快捷方式已存在”。

## 功能说明

| 能力 | 说明 |
| --- | --- |
| 添加程序 | 点“添加程序”弹出 **Windows 原生文件选择窗口**（Rust `IFileOpenDialog`），可一次多选 `.exe`，不再手填路径 |
| 程序清单 | 勾选即参与创建，取消勾选即“排除”；支持搜索名称 / 路径、全选 / 全不选；每行显示从 exe 抽取的真实图标，取不到时回退首字色块 |
| 快捷方式名 | 每行名称后的铅笔可改名（也可双击名称）；改完立即生效，`.lnk` 用这个新名字；改名会清掉该行旧状态，避免误以为已完成 |
| 创建位置 | **每个程序单独设置**，新加入默认「桌面 + 开始菜单」。行内两个小标签可直接点：点掉「桌面」就只建开始菜单，点掉「开始菜单」就只建桌面；每个程序至少保留一个位置，不会所有程序一刀切 |
| 开始菜单分组 | **默认不分组**，快捷方式直接放 `Programs` 根目录，不会给每个程序都建一个文件夹。需要分组时，点行内目标位置后的「＋」：清单里还没有分组就直接输入；**已有分组时先出现下拉，可直接选已有名字，不会手输打错**；要新分组选「新建分组…」再输入。设置后该行才显示分组胶囊，点胶囊也是同一个下拉；选「不分组」或点右侧 `×` 可取消。分组只影响开始菜单，桌面不受影响；只建桌面时这一列不显示 |
| 用户范围 | **每个程序单独选择**“当前用户 / 所有用户”；切换范围会清掉旧标记，按新范围重新检查 / 创建 |
| 已存在判定 | **以磁盘上真实 `.lnk` 为准**：启动、加入程序、改名、换范围 / 分组、创建前后都会重新读取同名 `.lnk` 的 TargetPath。与当前程序真实路径一致才显示“已存在”；同名但指向别的程序显示冲突原因，不会静默当作已存在。需要替换时打开“覆盖同名快捷方式” |
| 手动刷新 | 点工具栏「刷新」，或按 **F5 / Ctrl+R**，重新读盘检测所有程序的快捷方式，适合手动删除了 `.lnk`、在资源管理器里新建了快捷方式之后使用；按钮会显示刷新中，完成后提示检测到多少个已存在。浏览器预览无法读盘，只提示去桌面应用刷新 |
| 桌面交互 | 关闭右键菜单、界面文字选择和图标拖拽，避免像浏览器页面；搜索框、快捷方式改名框、分组输入框仍可正常选择、编辑和粘贴。F5 / Ctrl+R 会被拦截并转为刷新，不会重载 WebView |
| 一键创建 | 串行创建所有未创建项，显示“正在创建 n / m”和逐行状态；创建完成后重新读盘，状态自然变成“已存在” |
| 单个创建 | 每行的“创建”按钮只处理该程序；全部存在后按钮变“已完成” |
| 清单持久化 | 清单、每个程序的创建位置、范围、分组、开关项保存在 **exe 同目录的 `shortcuts.json`**，重启后仍在 |
| 清除已创建 | 点击“清除创建标记与快捷方式”会先确认，再按**每个程序自己设置的创建位置**，删除同名且 TargetPath 一致的 `.lnk`；指向其他程序的同名文件不会删。删除后空的分组目录会一并移除，状态立即重新读盘变为“未创建” |
| 单例模式 | 重复启动只保留一个实例：第二个进程检测到命名互斥体后，会还原并前置已有窗口，然后退出，不再开第二个窗口 |
| 结果反馈 | 已存在 / 部分已存在 / 创建失败（悬停看失败原因） |

桌面应用启动时优先读取 `shortcuts.json`；读不到才显示空清单。浏览器预览（`npm run dev`）只加载示例数据，存 localStorage，不会碰本地文件；预览下的刷新按钮只会提示“请在桌面应用里刷新”。

`createdTargets` / `skippedTargets` 只是磁盘检查结果的缓存：应用启动、每行切换桌面 / 开始菜单、切换范围 / 分组、手动刷新、创建和清除之后都会重新读真实 `.lnk`。手改 JSON 伪造“已创建”不会让界面显示已存在。

每个程序的创建位置保存在 `programs[].targets` 里（`{ "desktop": true, "startmenu": false }` 这类结构），至少有一个为 `true`。旧清单里只有顶层 `targets` 的全局开关，加载时会作为这些程序缺失字段时的默认值迁移，不会影响已加入程序的位置。

分组下拉的候选来自当前清单里已经用过的非空分组名，不会去猜磁盘上其他文件夹；没有候选时直接输入新名字。旧版本清单里若存过 `group: "默认分组"`，加载时按“不分组”迁移，不会再生成这个文件夹。

## 清单文件

保存位置：**`AppShortcut.exe` 所在目录下的 `shortcuts.json`**（开发时是 `src-tauri\target\debug\shortcuts.json`）。

- 写入用“临时文件 + 重命名”，避免写到一半损坏
- 清单结构为 `version: 2`，每个程序在 `programs[].targets` 里保存自己的桌面 / 开始菜单位置；旧 `version: 1` 的全局 `targets` 会作为默认值迁移
- 程序目录不可写时（例如装进 `Program Files` 又没提权），自动回落到 `%APPDATA%\AppShortcut\shortcuts.json`，界面底部会显示实际路径
- 首次从旧版本升级时，会把浏览器 localStorage 里的旧清单自动迁移进该文件
- 文件损坏会先改名为 `shortcuts.invalid-<时间戳>.json` 备份，再按空清单启动，不会静默清空

## 快捷方式写到哪里

| 目标 | 当前用户 | 所有用户 |
| --- | --- | --- |
| 桌面 | `%USERPROFILE%\Desktop`（`FOLDERID_Desktop`） | `%PUBLIC%\Desktop`（`FOLDERID_PublicDesktop`） |
| 开始菜单 | `%APPDATA%\Microsoft\Windows\Start Menu\Programs\<分组名>`（`FOLDERID_Programs` + 分组目录） | `%ProgramData%\Microsoft\Windows\Start Menu\Programs\<分组名>`（`FOLDERID_CommonPrograms` + 分组目录） |

`<分组名>` 来自每一行自己的“开始菜单分组”，例如 `RustRover家族`；留空时不建子目录，直接放在 `Programs` 根目录。分组名会由 Rust 清洗非法字符、去掉首尾点和空格，最长 60 字，避免 `..` 之类的路径穿越。

“当前用户”目录无需管理员权限；“所有用户”写公共目录，普通账户通常可写，但被组策略 / ACL 限制时会写入失败，界面会显示具体错误，不会静默失败。

## 单例模式

应用启动时先创建命名互斥体 `Local\AppShortcut_SingleInstance`。如果已存在，说明第一个实例还在运行：

1. 用窗口标题「快捷方式管家」找到已有窗口；
2. `ShowWindow(SW_RESTORE)` 还原（最小化时也能回到前台）；
3. `SetForegroundWindow` 前置；
4. 第二个进程直接退出，不创建第二个窗口。

互斥体在进程退出时由系统回收，不需要额外清理。

## 运行

```bash
npm install          # 首次
npm run dev          # 只跑前端，浏览器打开 http://localhost:1420（演示数据，不写文件）
npm run tauri:dev    # 桌面窗口（需要 Rust 工具链）
npm run tauri:build  # 打包安装程序
```

如果 `npm run tauri:dev` 报 `program not found: cargo`，把 `%USERPROFILE%\.cargo\bin` 加进当前终端 PATH 再启动。

## 目录结构

```
src/
  App.vue                    页面骨架：清单 + 侧栏
  components/
    AppHeader.vue            标题与统计
    ProgramToolbar.vue       搜索、添加程序（调系统选择窗口）
    ProgramRow.vue           单行：勾选、名称/路径、创建位置 + 分组、范围、状态、创建 / 移除
    PlanPanel.vue            每程序位置说明、覆盖选项、汇总、主按钮与清除按钮
    ToastStack.vue           底部提示
  composables/
    useShortcutLibrary.ts    清单状态、真实状态刷新、创建 / 清除队列、计数、持久化、图标加载
  services/
    shortcutService.ts       invoke 封装：pick_programs / create_shortcuts / inspect_shortcuts / clear_shortcuts / load_library / save_library / program_icons
  data/programs.ts           浏览器预览用的示例清单与颜色分配
  types.ts                   共享类型
src-tauri/
  src/lib.rs                 Rust 命令：原生选择窗口、目录解析、IShellLink 写入 / 读取、状态检查、清除、图标抽取、单例、清单文件读写
  Cargo.toml                 依赖（tauri、windows 0.62、png）
```

## Rust 命令

- `pick_programs()` → `PickedProgram[]`
  用 `IFileOpenDialog` 打开系统选择窗口，多选 `.exe`，通过 `IShellItem::GetDisplayName(SIGDN_FILESYSPATH)` 取真实路径；用户取消返回空数组。
- `create_shortcuts(items, overwrite)` → `CreateResult[]`
  每个程序按 `scope` 解析目录；`target = startmenu` 时再拼上该行自己的 `group` 分组目录（空分组 = `Programs` 根目录）。已有同名 `.lnk` 时用 `IShellLinkW::GetPath` 读取它的 TargetPath 并与当前 exe 路径比对（canonicalize + 大小写归一）：一致且 `overwrite = false` 返回 `skipped`；指向别的程序则返回 `failed` 并提示“目标不一致”；`overwrite = true` 时直接替换。写入用 `IShellLinkW` + `IPersistFile::Save`，文件名取程序行上的自定义名称（非法字符由 Rust 清洗）。
  返回逐项结果：`created` / `skipped` / `failed`（附 `message` 与 `linkPath`）。
- `inspect_shortcuts(items)` → `ShortcutState[]`
  按每个程序的名称、路径、范围、分组和当前创建位置，检查同名 `.lnk` 是否存在以及 TargetPath 是否一致，返回 `exists` / `conflict` / `missing` / `error`。这是界面“已存在”状态的唯一来源；JSON 里的 `createdTargets` 只是缓存。
- `clear_shortcuts(items, targets)` → `ClearResult { deleted, failed, errors }`
  每个程序优先使用自己的 `item.targets`（旧调用缺省时退回外层 `targets`），只删除同名且 TargetPath 一致的 `.lnk`；同名但指向其他程序的文件保留并返回错误。删除后尝试移除空的分组目录，不删除 `Programs` 根目录或其他文件。前端随后重新 `inspect_shortcuts`，状态回到“未创建”。
- `program_icons(paths)` → `ProgramIcon[]`
  用 `IShellItemImageFactory::GetImage` 取每个 exe 的 64px 图标，经 GDI 转成 32 位像素后用 `png` 编码为 `data:image/png;base64,...`。单个程序取不到图标时返回 `icon: null`，前端回退首字色块；失败不影响清单和创建。
- `load_library()` → `LibraryFile { path, content, portable }`
  优先读 exe 同目录 `shortcuts.json`，其次读 `%APPDATA%\AppShortcut\shortcuts.json`；文件损坏会备份后返回空内容。
- `save_library(content)` → `LibraryFile`
  先写 exe 同目录（临时文件 + 重命名），失败自动写 `%APPDATA%\AppShortcut`，并返回实际路径与 `portable` 标记。

COM 调用放在 `spawn_blocking` 的工作线程中，每个操作独立 `CoInitializeEx(COINIT_APARTMENTTHREADED)` / `CoUninitialize`；选择窗口传入主窗口 HWND 作为 owner，避免被主界面遮住。

## 已知限制

- 图标在每次启动 / 新加入程序时从系统 Shell 读取，只存在内存里，不写进 `shortcuts.json`；少数没有 Shell 图标的 exe 会回退成首字色块
- “清除创建标记与快捷方式”只处理**当前清单里、当前名称、每个程序当前选中的创建位置**对应的 `.lnk`；改名之前生成的旧名字快捷方式不会被自动找到，需要手动清理
- 刷新按当前名称 / 范围 / 分组 / 创建位置去找同名 `.lnk`；手动把快捷方式改成了别的名字或搬到了别的文件夹，刷新不会把它认成同一个程序
- 已经生成过快捷方式后再关掉某个位置的勾选，旧位置的 `.lnk` 不会自动删除；界面只检查当前选中的位置。需要彻底清掉时，先重新勾上该位置再点清除，或手动删除
- 给已有快捷方式改分组后，旧分组里的 `.lnk` 不会自动搬迁；状态会按新分组重新判断，旧文件需要先清除或手动删除
- 同名但指向别的程序的 `.lnk` 不会被清除，避免误删用户已有文件；需要替换时先开启“覆盖同名快捷方式”再创建
- “所有用户”失败时只提示错误，不自动弹 UAC 提权
- exe 目录不可写时清单会落到 `%APPDATA%`，此时不再是“绿色便携”状态；安装版建议放在用户可写目录
