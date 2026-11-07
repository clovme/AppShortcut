# WinKit 工具箱

Windows 桌面应用：把常用程序加入清单，一键在**桌面**和**开始菜单**生成快捷方式。新加入的程序默认两个位置都创建，也可以按行关掉其中一个、只建桌面或只建开始菜单；开始菜单默认不分组、直接放 `Programs` 根目录，需要时再按行设置分组；支持单个创建，也可以取消勾选排除某个程序。应用还内置了进程管理，可以按 WebView2、node 等进程名筛选，批量结束同名多实例进程；以及按方案维护环境变量：先建 JAVA / Go 这样的方案，按 `key: value` 添加变量，程序启动后按注册表检测是否已设置，支持写入、Path 追加、编辑和删除系统中的变量。进程管理是默认主页，快捷方式页和环境变量页从标题栏一键切换。

技术栈：Tauri 2 + Rust + Vue 3 + TypeScript + Vite。

## 名字

产品名定为 **WinKit 工具箱**，可执行文件为 `WinKit.exe`。`Win` 指 Windows，`Kit` 是工具箱：快捷方式、进程管理、环境变量三件事都装在一个窗口里，不再是只做快捷方式的单一工具。包标识符现在是 `WinKit`，数据目录仍沿用 `%APPDATA%\AppShortcut`，避免升级后找不到原来的清单和方案。

## 图标

仓库根目录的 `app-icon.svg` 是唯一图标源：1024 × 1024、透明圆角方形、Windows 蓝渐变底、白色 `W` 字标、右下角白色快捷方式箭头徽标。不要再手改 `src-tauri\icons` 下的 PNG / ICO / ICNS；要改图标只改这个 SVG，然后运行：

```powershell
npm run tauri:icon
```

这个脚本调用 Tauri CLI，会把 `src-tauri\icons` 下的 `32x32.png`、`64x64.png`、`128x128.png`、`128x128@2x.png`、`icon.png`、`icon.ico`、`icon.icns` 以及 `Square*Logo.png`、`StoreLogo.png` 全部重新生成。Tauri CLI 还会顺手生成 `android` / `ios` 两个移动端目录，Windows 项目用不到，生成后删掉即可。图标生成后必须重新构建 exe（`cargo build --release`），`WinKit.exe` 的资源图标才会真正换掉。

## 当前进度

- 界面完成：**默认打开进程管理页**，标题栏可在「进程管理 / 环境变量 / 快捷方式」之间切换；快捷方式页有程序清单、每程序桌面 / 开始菜单位置切换、每程序自定义开始菜单分组、范围选择、快捷方式改名、排除 / 单建 / 一键创建、手动刷新、真实状态、进度反馈、真实程序图标；进程页可按名称筛选、按端口查找占用进程并批量结束进程；环境变量页改成**方案式管理**：左侧方案列表（JAVA / Go 等），每个方案左侧有启用勾选框、默认启用；右侧按 `key: value` 添加变量，默认系统变量、可切用户变量，启动后按注册表真实检测“缺失 / 待设置 N 项 / 已设置 / 值不同”；支持整组方案启用 / 停用、单条写入、全部写入、Path 分段追加、删系统变量但不删方案项，以及方案的新建、改名、删除和搜索
- 管理员权限：构建时通过 `src-tauri/app.manifest` + `src-tauri/build.rs` 给 `WinKit.exe` 嵌入 `requestedExecutionLevel="requireAdministrator"`，双击启动会先弹 UAC；同意后可以直接写系统环境变量、结束高权限进程，不需要再右键“以管理员身份运行”
- 图标：`app-icon.svg` 重新设计为 WinKit 蓝底 `W` + 快捷方式箭头，`npm run tauri:icon` 已重新生成 `src-tauri\icons` 全套 PNG / ICO / ICNS / Square*Logo；release exe 重新构建后资源图标已替换
- 后端完成：系统原生文件选择窗口、`.lnk` 写入、目录解析、按 TargetPath 判断已存在、自定义分组文件夹、单例模式、真实 `.lnk` 状态检查、按每个程序自己的创建位置清除、清单文件读写、进程枚举与批量结束、用户 / 系统环境变量注册表读写、环境变量方案文件读写、原生文件夹选择窗口
- 已通过：`vue-tsc` 类型检查、Vite 生产构建、`cargo check`、18 个 Rust 单元测试（真实写出 `.lnk`、同名同目标跳过、同名异目标冲突、自定义分组目录、真实状态判定与清除、exe 图标抽取、清单读写与损坏备份、环境变量方案文件往返与坏 JSON 隔离、进程保护规则、快照包含当前进程、网络字节序端口解码、真实监听端口映射到当前进程、空结束请求、用户环境变量读取 / 写入 / 删除 / 改名、非法变量名拒绝）
- 实机验证：单例模式（连续启动两次只有 1 个进程）；NotepadTest 写入 `Programs\RustRover家族\NotepadTest.lnk` 后状态变为“已存在”；点击“清除创建标记与快捷方式”后 `.lnk` 与空分组目录都被删除，状态回到“未创建”；伪造 JSON 里的创建标记会在启动时被磁盘真实状态纠正。
- 实机验证（每个程序单独位置）：TargetDesktopTest 只勾桌面、TargetStartTest 只勾开始菜单，一次创建后桌面只出现前者、`Programs` 根目录只出现后者，交叉位置没有文件；清除后两个 `.lnk` 都被删除，状态回到“未创建”。
- 实机验证（手动刷新）：RefreshTest 双建后手动删掉桌面 `.lnk`，界面仍显示“已存在”；点“刷新”（F5 / Ctrl+R 同样触发）后变为“部分已存在”、待创建 1 个，并提示“检测到 1 / 2 个快捷方式已存在”。
- 实机验证（进程管理）：200 个进程里筛出 5 个 `node.exe`，勾选 3 个测试进程后一次结束，界面提示“已结束 3 个进程”，Vite 开发服务未被误杀；“结束选中及子进程”验证了 node 父子进程一起结束；WebView2 筛选 18 个，其中 6 个本应用 WebView2 组件被标记为受保护且不可勾选。
- 实机验证（全宽与进程名列表）：进程页面板宽度 1136px，与 1180px 窗口的可用区域一致；181 个进程自动去重成 59 类，列表按数量显示 `svchost.exe 71`、`Doubao.exe 16`、`msedgewebview2.exe 6`、`node.exe 4` 等；点 `node.exe` 只显示 4 行，勾选其中 2 个一次性结束，列表里的 `node.exe` 数量与表格同时从 4 变 2，选中的名称仍保持在可视区域。
- 实机验证（搜索联动与滚动条）：搜索 `doubao` 后，左侧名称列表从 62 类收窄到 5 类、“全部”数量从 184 变 21，表格同步只剩 21 行；清空搜索后名称和数量全部恢复。左侧名称列表与右侧进程列表都使用同一套细滚动条样式。
- 实机验证（端口）：启动测试 HTTP 服务监听 `127.0.0.1:18080`，进程页里对应 `node.exe`（PID 2892）的端口列显示 `18080`；搜 `18080` / `:18080` 都只剩该进程和 `node.exe` 这一类，搜 `135` 命中监听 135 的 `svchost.exe`。应用自己的 WebView2 调试端口 `9333`、Vite 开发服务的 `1420` 也能按 PID 正确显示。
- 实机验证（页面切换）：应用启动直接进入进程管理页，加载真实进程列表；点标题栏「快捷方式」切到程序清单，清单数据正常显示；再切回「进程管理」会重新读取进程。进程页按 F5 只刷新进程列表，不重载页面。
- 实机验证（环境变量方案）：新建 `TESTJAVA` 方案，添加两条用户变量后立即写入注册表，状态从“缺失”变为“已设置”；删除其中一条**系统变量**后，注册表值消失、方案项保留并显示“缺失”，证明状态来自真实注册表而不是 JSON；再从方案项编辑值会同步更新系统变量；点 `×` 只移除方案项、点“删除方案”只删方案，都不动系统变量。另用真实用户 `Path` 验证追加模式：写入 `D:\AppShortcutPathTest\bin` 后注册表保留原有 227 个字符并追加到 254 个字符，状态显示“已设置”，测试后已按备份精确还原。并且把 `Path` 的四种写法（无分号、前后都有分号、只有前导分号、只有尾随分号）逐一轮换，都按 `;` 拆成路径项比较，状态都稳定显示“已设置”，不会因为空分段或首尾分号变成不同状态。测试方案与测试变量均已清理，`HKCU:\Environment` 仍只有原有的 `GOPATH` / `Path` / `TEMP` / `TMP`；测试方案删除后，用户自建的 `Java` 方案（`JAVA_HOME` / `CLASSPATH` / `Path`）保留。左侧方案的启用勾选框也实机验证：取消 `Java` 勾选后左侧状态变为“未启用”、右侧出现“方案未启用”标签、“全部写入”按钮禁用，再勾回后恢复“全部已设置”，勾选状态以方案级 `enabled` 写入方案文件。值输入框右侧的文件夹按钮也已实机打开原生“选择文件夹”窗口，并确认取消后能正常返回。

## 功能说明

| 能力 | 说明 |
| --- | --- |
| 页面布局 | **进程管理是默认主页**，启动后直接显示进程列表；标题栏「环境变量」切到环境变量管理，点「快捷方式」切到程序清单，再点「进程管理」切回。三个页面都是铺满窗口的整页布局，不是弹窗 |
| 添加程序 | 点“添加程序”弹出 **Windows 原生文件选择窗口**（Rust `IFileOpenDialog`），可一次多选 `.exe`，不再手填路径 |
| 程序清单 | 勾选即参与创建，取消勾选即“排除”；支持搜索名称 / 路径、全选 / 全不选；每行显示从 exe 抽取的真实图标，取不到时回退首字色块 |
| 快捷方式名 | 每行名称后的铅笔可改名（也可双击名称）；改完立即生效，`.lnk` 用这个新名字；改名会清掉该行旧状态，避免误以为已完成 |
| 创建位置 | **每个程序单独设置**，新加入默认「桌面 + 开始菜单」。行内两个小标签可直接点：点掉「桌面」就只建开始菜单，点掉「开始菜单」就只建桌面；每个程序至少保留一个位置，不会所有程序一刀切 |
| 开始菜单分组 | **默认不分组**，快捷方式直接放 `Programs` 根目录，不会给每个程序都建一个文件夹。需要分组时，点行内目标位置后的「＋」：清单里还没有分组就直接输入；**已有分组时先出现下拉，可直接选已有名字，不会手输打错**；要新分组选「新建分组…」再输入。设置后该行才显示分组胶囊，点胶囊也是同一个下拉；选「不分组」或点右侧 `×` 可取消。分组只影响开始菜单，桌面不受影响；只建桌面时这一列不显示 |
| 用户范围 | **每个程序单独选择**“当前用户 / 所有用户”；切换范围会清掉旧标记，按新范围重新检查 / 创建 |
| 已存在判定 | **以磁盘上真实 `.lnk` 为准**：启动、加入程序、改名、换范围 / 分组、创建前后都会重新读取同名 `.lnk` 的 TargetPath。与当前程序真实路径一致才显示“已存在”；同名但指向别的程序显示冲突原因，不会静默当作已存在。需要替换时打开“覆盖同名快捷方式” |
| 手动刷新 | 在快捷方式页，点工具栏「刷新」或按 **F5 / Ctrl+R**，重新读盘检测所有程序的快捷方式，适合手动删除了 `.lnk`、在资源管理器里新建了快捷方式之后使用；按钮会显示刷新中，完成后提示检测到多少个已存在。浏览器预览无法读盘，只提示去桌面应用刷新 |
| 桌面交互 | 关闭右键菜单、界面文字选择和图标拖拽，避免像浏览器页面；搜索框、快捷方式改名框、分组输入框仍可正常选择、编辑和粘贴。F5 / Ctrl+R 会被拦截并转为刷新，不会重载 WebView：进程页刷新进程，快捷方式页刷新快捷方式 |
| 进程管理 | **默认主页**，列出 PID、父 PID、进程名、路径、端口、工作集内存和线程数；左侧「进程名称」列表按进程名自动去重并带数量，支持一键选某一类进程、按名称 / 路径 / PID / 端口搜索，按当前筛选全选后批量结束。支持“仅结束选中”和“结束选中及子进程”，结束时二次确认；本应用自身、本应用 WebView2 组件和 Windows 关键进程会被标记为受保护，不能勾选 |
| 环境变量管理 | 标题栏第二个页面，按**方案**管理“准备设置、但系统里可能还没有”的变量。左侧新建 `JAVA` / `Go` 等方案，每个方案 item 最左边有**启用勾选框**，默认勾选；取消勾选表示整个方案都不参与写入，方案项和系统里已有的变量都不动。右侧按 `key: value` 添加变量；每条变量单独选系统 / 用户范围（默认系统），值可直接输入或调 Windows 文件夹选择框。启动和刷新时按变量名去注册表检测真实状态，不保存“已设置”标记；支持单条写入、全部写入、Path 分段追加、编辑、删除系统变量（方案项保留）和从方案移除（不动系统变量） |
| 一键创建 | 串行创建所有未创建项，显示“正在创建 n / m”和逐行状态；创建完成后重新读盘，状态自然变成“已存在” |
| 单个创建 | 每行的“创建”按钮只处理该程序；全部存在后按钮变“已完成” |
| 清单持久化 | 清单、每个程序的创建位置、范围、分组、开关项保存在 **exe 同目录的 `WinKit.json`**，重启后仍在 |
| 清除已创建 | 点击“清除创建标记与快捷方式”会先确认，再按**每个程序自己设置的创建位置**，删除同名且 TargetPath 一致的 `.lnk`；指向其他程序的同名文件不会删。删除后空的分组目录会一并移除，状态立即重新读盘变为“未创建” |
| 单例模式 | 重复启动只保留一个实例：第二个进程检测到命名互斥体后，会还原并前置已有窗口，然后退出，不再开第二个窗口 |
| 结果反馈 | 已存在 / 部分已存在 / 创建失败（悬停看失败原因） |

桌面应用启动时优先读取 `WinKit.json`；读不到才显示空清单。浏览器预览（`npm run dev`）只加载示例数据，存 localStorage，不会碰本地文件；预览下的刷新按钮只会提示“请在桌面应用里刷新”。

`createdTargets` / `skippedTargets` 只是磁盘检查结果的缓存：应用启动、每行切换桌面 / 开始菜单、切换范围 / 分组、手动刷新、创建和清除之后都会重新读真实 `.lnk`。手改 JSON 伪造“已创建”不会让界面显示已存在。

每个程序的创建位置保存在 `programs[].targets` 里（`{ "desktop": true, "startmenu": false }` 这类结构），至少有一个为 `true`。旧清单里只有顶层 `targets` 的全局开关，加载时会作为这些程序缺失字段时的默认值迁移，不会影响已加入程序的位置。

分组下拉的候选来自当前清单里已经用过的非空分组名，不会去猜磁盘上其他文件夹；没有候选时直接输入新名字。旧版本清单里若存过 `group: "默认分组"`，加载时按“不分组”迁移，不会再生成这个文件夹。

## 进程管理

应用启动后直接进入进程管理页，用来处理 WebView2、`node.exe` 这类一次开很多个、在任务管理器里逐个点太慢的进程。标题栏右上角可在「进程管理 / 环境变量 / 快捷方式」之间切换；进程页按 F5 / Ctrl+R 只重新读取进程，不会重载页面。

- 列表数据：进程名、PID、父 PID、端口、可执行文件路径、工作集内存、线程数；按内存从大到小排序，受保护进程排在最后；**整页铺满窗口宽度**，不缩在中间
- 端口：显示进程正在**监听 / 绑定**的 TCP 与 UDP 端口（IPv4 + IPv6，已去重）；一个进程有多个端口时用逗号分隔，超过 4 个显示前 4 个并加 `+N`，鼠标悬停看完整列表。没有端口的进程显示 `—`。端口由 `GetExtendedTcpTable` / `GetExtendedUdpTable` 按 PID 归属，不依赖 `netstat` 文本解析
- 筛选：左侧「进程名称」列表，首项是「全部」，其余按进程名**自动去重**生成并带实例数量，按数量降序排列；刷新或结束进程后会跟着更新。搜索框支持进程名 / 路径 / PID / 父 PID / 端口（如 `8080` 或 `:8080`），**搜索会同时收窄左侧名称列表和对应数量**；选中某个名称后再搜索，是在该名称范围内叠加
- 批量选择：表头复选框全选当前筛选结果；受保护进程不会被“全选”带上。勾选后底部显示已选数量
- 结束方式：`仅结束选中` 只结束勾选的那几个 PID；`结束选中及子进程` 先结束子进程、再结束父进程，适合关掉 WebView2 浏览器进程或带 worker 的 node
- 二次确认：结束前弹确认框，列出前 6 个进程名和 PID；确认后才调用 `TerminateProcess`
- 受保护进程：本应用自身、本应用拉起的 WebView2 组件（浏览器进程及其 renderer）、PID 0/4、`smss.exe` / `csrss.exe` / `wininit.exe` / `services.exe` / `lsass.exe` / `winlogon.exe`，以及 Windows 安全组件，界面显示黄色锁标签且复选框禁用；Rust 侧在真正结束前会再检查一次，不能靠构造请求绕过
- 结束后自动重新读取进程列表，并在底部显示“已结束 N 个进程”；有失败时把失败原因带出来
- 浏览器预览没有真实进程，进程页只显示提示，不会调用系统接口

结束别人的进程可能需要管理员权限：普通权限下打开高权限进程的句柄会失败，界面会显示“无法打开进程 / 结束失败”，不会静默当作成功。

## 环境变量管理

标题栏第二个页面。这里管理的是“要设置、但系统里可能还没有”的变量，而不是把注册表里已有的变量再抄一张表。典型用法是刚装完系统，先建一个 `JAVA` 方案，把 `JAVA_HOME`、`CLASSPATH`、`Path` 作为几行 `key: value` 加进去；程序启动或刷新时按变量名去注册表检测，缺哪条就写哪条。

- 方案：左侧是方案列表，可新建、改名（点方案名后的铅笔）、删除；方案名自定义，比如 `JAVA`、`Go`。方案只保存 `key: value`、范围和类型，不保存“已设置”状态
- 方案行：左侧每个方案最左边是**启用勾选框**，默认勾选；勾选表示整份方案参与写入，取消后左侧状态显示“未启用”、右侧打出“方案未启用”标签，“全部写入”禁用并跳过整个方案。方案项和系统变量都不会因为取消勾选被删除。右侧变量行只有变量名、值、范围、状态、操作，不再单独设启用开关，避免一个方案只写进去一半
- 变量行：变量名、值、范围、状态、操作。新变量默认**系统变量**，行内下拉可切成“用户变量”；用户变量不需要管理员权限，系统变量需要
- 值输入：可以直接输入，也可以点值输入框右侧的文件夹按钮，调 Windows 原生文件夹选择框；选中的路径会填进值里
- 真实状态：启动、刷新、写入、删除后都会重新读 `HKCU\Environment` 和 `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment`，按变量名（忽略大小写）判断。状态包括“缺失”“待设置 N 项”“已设置”“值不同”“读取失败”，不是 JSON 里的缓存。`Path` 不再按整行文本判断，而是按 `;` 拆成路径项逐项判断：所有要设置的路径项都在现有 Path 里就是“已设置”，缺几项显示“待设置 N 项”
- Path 分段：变量名是 `Path` 时，比较和写入都先按 `;` 拆段、去掉首尾空白和空段、忽略大小写。`Path=%JAVA_HOME%\bin;%JAVA_HOME%\jre\bin`、带前导 `;`、带尾随 `;`、前后都有 `;` 这四种写法是同一组路径，不会显示成四种状态，也不会写进空分段。勾选“追加”时不覆盖整条 Path，只把缺少的路径项补到末尾；已有项不会重复加入。`Path` 以外的变量按整串值比较
- 写入：每行有“写入 / 更新”按钮；右上角“全部写入”会串行处理当前方案的全部变量，方案被取消勾选时按钮禁用，不会写进去任何一项；已经“已设置”的行跳过。写入成功后会广播 `WM_SETTINGCHANGE`，新启动的程序才能拿到新值
- 编辑：点行尾铅笔改方案项的名称 / 值 / 范围 / 展开 / 追加；保存后方案项和系统变量一起更新，旧变量名会按“先写新名、再删旧名”处理
- 删除：行尾垃圾桶删除的是**系统中的环境变量**，方案项会保留并显示“缺失”，可以再点“写入”重新创建；行尾 `×` 只从方案里移除这一项，不动系统变量；右上角“删除方案”只删方案，不动任何系统变量
- 搜索：左侧搜索框按方案名或变量名过滤，方案数量显示为“匹配数 / 总数”
- 刷新：点右上角“刷新检测”或按 F5 / Ctrl+R，重新读两个注册表范围；在注册表或系统属性里改过之后也能同步
- 类型：每条变量可勾选“展开”。勾选后写 `REG_EXPAND_SZ`，值里的 `%USERPROFILE%`、`%JAVA_HOME%` 等引用由 Windows 展开；不勾选写 `REG_SZ`
- 权限：构建产物 `WinKit.exe` 已带管理员清单，双击会先弹 UAC，同意后可直接写系统变量；用户变量本来就不需要管理员。若运行的是没有清单的旧产物，普通权限下写系统变量会失败，页面顶部会提示“当前不是管理员，系统变量暂时写不进去”，方案仍会保存，右键 `WinKit.exe` 以管理员身份运行后再点“全部写入”即可

方案文件保存位置：**`WinKit.exe` 同目录的 `environment-profiles.json`**（开发时是 `src-tauri\target\debug\environment-profiles.json`）；目录不可写时回落到 `%APPDATA%\AppShortcut\environment-profiles.json`。结构为 `{"version":1,"profiles":[{"id","name","note","enabled","variables":[{"id","name","value","scope","expand","append"}]}]}`，写入用临时文件 + 重命名；坏 JSON 会改名为 `invalid-<时间戳>.json` 备份后按空方案启动。

## 清单文件

保存位置：**`WinKit.exe` 所在目录下的 `WinKit.json`**（开发时是 `src-tauri\target\debug\WinKit.json`）。

- 写入用“临时文件 + 重命名”，避免写到一半损坏
- 清单结构为 `version: 2`，每个程序在 `programs[].targets` 里保存自己的桌面 / 开始菜单位置；旧 `version: 1` 的全局 `targets` 会作为默认值迁移
- 程序目录不可写时（例如装进 `Program Files` 又没提权），自动回落到 `%APPDATA%\AppShortcut\WinKit.json`，界面底部会显示实际路径
- 首次从旧版本升级时，会把浏览器 localStorage 里的旧清单自动迁移进该文件
- 如果目录里只有旧版 `shortcuts.json`，首次读取会自动改名为 `WinKit.json`，不会丢数据
- 文件损坏会先改名为 `WinKit.invalid-<时间戳>.json` 备份，再按空清单启动，不会静默清空

## 快捷方式写到哪里

| 目标 | 当前用户 | 所有用户 |
| --- | --- | --- |
| 桌面 | `%USERPROFILE%\Desktop`（`FOLDERID_Desktop`） | `%PUBLIC%\Desktop`（`FOLDERID_PublicDesktop`） |
| 开始菜单 | `%APPDATA%\Microsoft\Windows\Start Menu\Programs\<分组名>`（`FOLDERID_Programs` + 分组目录） | `%ProgramData%\Microsoft\Windows\Start Menu\Programs\<分组名>`（`FOLDERID_CommonPrograms` + 分组目录） |

`<分组名>` 来自每一行自己的“开始菜单分组”，例如 `RustRover家族`；留空时不建子目录，直接放在 `Programs` 根目录。分组名会由 Rust 清洗非法字符、去掉首尾点和空格，最长 60 字，避免 `..` 之类的路径穿越。

“当前用户”目录无需管理员权限；“所有用户”写公共目录，普通账户通常可写，但被组策略 / ACL 限制时会写入失败，界面会显示具体错误，不会静默失败。

## 单例模式

应用启动时先创建命名互斥体 `Local\AppShortcut_SingleInstance`。如果已存在，说明第一个实例还在运行：

1. 用窗口标题「WinKit 工具箱」找到已有窗口；
2. `ShowWindow(SW_RESTORE)` 还原（最小化时也能回到前台）；
3. `SetForegroundWindow` 前置；
4. 第二个进程直接退出，不创建第二个窗口。

互斥体在进程退出时由系统回收，不需要额外清理。

## 运行

```bash
npm install          # 首次
npm run dev          # 只跑前端，浏览器打开 http://localhost:1420（演示数据，不写文件）
npm run tauri:dev    # 桌面窗口（需要 Rust 工具链）
npm run tauri:build  # 只构建 WinKit.exe（不生成 MSI / NSIS）
```

`src-tauri\tauri.conf.json` 的 `bundle.active` 已设为 `false`；`npm run tauri:build` 只产出 `src-tauri\target\release\WinKit.exe`，不会生成 `msi` / `nsis` 安装包。以后需要安装包时，再把 `active` 改回 `true`，并在 `bundle.targets` 里指定 `msi` / `nsis`。

如果 `npm run tauri:dev` 报 `program not found: cargo`，把 `%USERPROFILE%\.cargo\bin` 加进当前终端 PATH 再启动。

## 管理员权限

`src-tauri/app.manifest` 里声明：

```xml
<requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
```

`src-tauri/build.rs` 把这个清单交给 `tauri_build::WindowsAttributes::app_manifest`，通过 Windows 资源编译进可执行文件。因此 `cargo build`、`npm run tauri:dev`、`npm run tauri:build` 产出的 `WinKit.exe` 都带管理员清单，双击启动时 Windows 会先弹 UAC 确认框；同意后进程直接是管理员，写系统环境变量、结束高权限进程都不再需要右键“以管理员身份运行”。清单里同时保留了 Common Controls v6，原生文件 / 文件夹选择框的控件样式不受影响。

## 目录结构

```
src/
  App.vue                    页面骨架：默认进程管理页 + 可切换的环境变量 / 快捷方式页
  components/
    AppHeader.vue            标题、进程 / 环境变量 / 快捷方式页面切换与统计
    ProgramToolbar.vue       搜索、添加程序（调系统选择窗口）
    ProgramRow.vue           单行：勾选、名称/路径、创建位置 + 分组、范围、状态、创建 / 移除
    PlanPanel.vue            每程序位置说明、覆盖选项、汇总、主按钮与清除按钮
    ProcessManager.vue       进程管理页：筛选、批量选择、结束进程 / 进程树
    EnvironmentManager.vue   环境变量页：方案列表、key:value 表格、真实状态检测、写入 / 追加 / 删除
    ToastStack.vue           底部提示
  composables/
    useShortcutLibrary.ts    清单状态、真实状态刷新、创建 / 清除队列、计数、持久化、图标加载
  services/
    shortcutService.ts       invoke 封装：pick_programs / pick_folder / create_shortcuts / inspect_shortcuts / clear_shortcuts / list_processes / kill_processes / load_environment_variables / save_environment_variable / delete_environment_variable / load_environment_profiles / save_environment_profiles / load_library / save_library / program_icons
  data/programs.ts           浏览器预览用的示例清单与颜色分配
  types.ts                   共享类型
src-tauri/
  app.manifest              Windows 应用清单：Common Controls v6 + requireAdministrator
  build.rs                  把 app.manifest 交给 tauri-build 嵌入 exe
  src/lib.rs                 Rust 命令：原生程序 / 文件夹选择窗口、目录解析、IShellLink 写入 / 读取、状态检查、清除、图标抽取、进程枚举 / 结束、环境变量注册表读写、环境变量方案文件读写、单例、清单文件读写
  Cargo.toml                 依赖（tauri、windows 0.62、png）
```

## Rust 命令

- `pick_programs()` → `PickedProgram[]`
  用 `IFileOpenDialog` 打开系统选择窗口，多选 `.exe`，通过 `IShellItem::GetDisplayName(SIGDN_FILESYSPATH)` 取真实路径；用户取消返回空数组。
- `pick_folder()` → `string | null`
  用同一个 `IFileOpenDialog`，但加 `FOS_PICKFOLDERS`，只允许选文件夹。返回选中目录的完整路径；取消返回 `null`。用于环境变量方案里“值”旁边的文件夹按钮。
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
  优先读 exe 同目录 `WinKit.json`，其次读 `%APPDATA%\AppShortcut\WinKit.json`；文件损坏会备份后返回空内容。
- `save_library(content)` → `LibraryFile`
  先写 exe 同目录（临时文件 + 重命名），失败自动写 `%APPDATA%\AppShortcut`，并返回实际路径与 `portable` 标记。
- `list_processes()` → `ProcessSnapshot { processes, currentPid }`
  用 `CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS)` 取 PID、父 PID、进程名和线程数；命中进程再用 `OpenProcess` + `QueryFullProcessImageNameW` 补路径，用 `GetProcessMemoryInfo` 补工作集内存。端口用 `GetExtendedTcpTable(TCP_TABLE_OWNER_PID_LISTENER)` 和 `GetExtendedUdpTable(UDP_TABLE_OWNER_PID)` 分别读 IPv4 / IPv6 表，按 `dwOwningPid` 汇总到进程并排序去重，返回 `ports: number[]`。按“可结束的在前、内存从大到小”排序。本应用自身及其 WebView2 子进程树、PID 0/4 和 Windows 关键进程会带 `protected` 与 `protectedReason`。
- `kill_processes(pids, includeTree)` → `KillResult { killed, failed, errors }`
  先重新拍进程快照，`includeTree` 时按子进程 → 父进程的顺序收集整棵树；结束前再次检查保护名单，调用 `OpenProcess(PROCESS_TERMINATE)` + `TerminateProcess`。返回成功数、失败数和逐条错误；不等待进程完全退出，前端结束后立刻重新读列表。
- `load_environment_variables()` → `EnvironmentSnapshot { user, system }`
  分别读取 `HKCU\Environment` 和 `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment`，用 `RegEnumValueW` 枚举 `REG_SZ` / `REG_EXPAND_SZ` 值，返回变量名、值、是否展开；同时用 `RegOpenKeyExW(KEY_SET_VALUE)` 探测当前范围是否可写，系统变量在普通权限下会是只读。单个范围读取失败时只把错误放在该范围的 `error` 里，不影响另一个范围。
- `save_environment_variable(scope, originalName, name, value, expand)` → `EnvironmentVariable`
  用户 / 系统范围二选一；变量名去首尾空格，拒绝空名和 `=`；新增时同名变量已存在会返回错误，改名时只删除旧名称、不覆盖其他变量；`expand` 决定写 `REG_SZ` 还是 `REG_EXPAND_SZ`。写入后广播 `WM_SETTINGCHANGE`。
- `delete_environment_variable(scope, name)` → `()`
  删除指定范围内的变量；不存在、权限不足等错误会带可读中文原因返回，成功后广播环境变量变更。
- `load_environment_profiles()` → `LibraryFile { path, content, portable }`
  优先读 exe 同目录 `environment-profiles.json`，其次读 `%APPDATA%\AppShortcut\environment-profiles.json`；坏 JSON 会改名为 `invalid-<时间戳>.json` 备份后返回空内容。
- `save_environment_profiles(content)` → `LibraryFile`
  先写 exe 同目录（`.json.tmp` 临时文件 + 重命名），失败自动写 `%APPDATA%\AppShortcut`，并返回实际路径与 `portable` 标记。内容由前端序列化为 `{"version":1,"profiles":[...]}`。

环境变量读写放在 `spawn_blocking` 中，注册表句柄用 `Drop` 及时关闭；系统变量写入失败会明确提示需要管理员权限。

进程枚举和结束放在 `spawn_blocking` 中，避免一次打开几百个进程句柄时卡住 UI。句柄用 `CloseHandle` 及时释放。

COM 调用放在 `spawn_blocking` 的工作线程中，每个操作独立 `CoInitializeEx(COINIT_APARTMENTTHREADED)` / `CoUninitialize`；选择窗口传入主窗口 HWND 作为 owner，避免被主界面遮住。

## 已知限制

- 图标在每次启动 / 新加入程序时从系统 Shell 读取，只存在内存里，不写进 `WinKit.json`；少数没有 Shell 图标的 exe 会回退成首字色块
- “清除创建标记与快捷方式”只处理**当前清单里、当前名称、每个程序当前选中的创建位置**对应的 `.lnk`；改名之前生成的旧名字快捷方式不会被自动找到，需要手动清理
- 刷新按当前名称 / 范围 / 分组 / 创建位置去找同名 `.lnk`；手动把快捷方式改成了别的名字或搬到了别的文件夹，刷新不会把它认成同一个程序
- 已经生成过快捷方式后再关掉某个位置的勾选，旧位置的 `.lnk` 不会自动删除；界面只检查当前选中的位置。需要彻底清掉时，先重新勾上该位置再点清除，或手动删除
- 给已有快捷方式改分组后，旧分组里的 `.lnk` 不会自动搬迁；状态会按新分组重新判断，旧文件需要先清除或手动删除
- 同名但指向别的程序的 `.lnk` 不会被清除，避免误删用户已有文件；需要替换时先开启“覆盖同名快捷方式”再创建
- “所有用户”失败时只提示错误，不自动弹 UAC 提权
- 进程管理不显示命令行、启动时间或 CPU 占用；同名 `node.exe` 只能靠 PID、父 PID、端口、路径和内存区分。结束高权限进程或服务需要以管理员身份运行本应用，否则会返回“无法打开进程 / 结束失败”
- 端口列只统计**监听 / 绑定端口**（TCP LISTEN + UDP OWNER_PID），不统计已建立连接里的临时本地端口；TCP 与 UDP 合并展示、不标注协议，也不显示监听地址（如 `127.0.0.1` / `0.0.0.0` / IPv6）
- `Path` 的“追加”只把缺少的分段追加到末尾，不会删除、重排或去重后排序现有分段；点垃圾桶删 `Path` 会删除整条系统变量，确认框会额外提示这一点
- 环境变量方案不提供 `.reg` 导入 / 导出，也没有内置 JAVA / Go / Python / Node 等一键模板；方案和变量都需要自己新建
- 系统环境变量需要管理员权限：普通权限下写系统变量会失败并保留方案，不会自动弹 UAC 或提权重启；用户变量不需要管理员权限
- 环境变量修改后只广播变更通知，已经打开的程序不会自动刷新自己的环境变量；需要重启程序或重新登录才会读取新值
- 环境变量读取只展示字符串类型的 `REG_SZ` / `REG_EXPAND_SZ`；注册表里其他类型（如 `REG_DWORD`）不会出现在检测结果中，也不会被修改
- 值旁边的文件夹选择只负责把选中路径填进值里，不会判断这个路径是否真的是 JAVA_HOME、Go 安装目录；值的含义仍由用户决定
- 结束进程是破坏性操作：未保存的数据会丢失；应用自身和它的 WebView2 组件被列为受保护，不能从面板里结束，需要关闭窗口时请直接关应用
- exe 目录不可写时清单会落到 `%APPDATA%`，此时不再是“绿色便携”状态；安装版建议放在用户可写目录
