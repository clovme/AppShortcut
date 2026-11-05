import type { ProgramPreset, TargetKind } from "../types";

/** 常用程序清单：只用于浏览器预览时的示例数据，桌面应用中由系统文件选择窗口真实添加 */
export const PROGRAM_PRESETS: ProgramPreset[] = [
  {
    name: "Visual Studio Code",
    path: "C:\\Users\\Administrator\\AppData\\Local\\Programs\\Microsoft VS Code\\Code.exe",
    color: "#2563eb",
  },
  {
    name: "微信",
    path: "C:\\Program Files\\Tencent\\WeChat\\WeChat.exe",
    color: "#16a34a",
  },
  {
    name: "Google Chrome",
    path: "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
    color: "#ea580c",
  },
  {
    name: "QQ",
    path: "C:\\Program Files\\Tencent\\QQNT\\QQ.exe",
    color: "#0284c7",
  },
  {
    name: "钉钉",
    path: "C:\\Program Files (x86)\\DingDing\\main\\current\\DingTalk.exe",
    color: "#1d4ed8",
  },
  {
    name: "网易云音乐",
    path: "C:\\Program Files (x86)\\NetEase\\CloudMusic\\cloudmusic.exe",
    color: "#b91c1c",
  },
  {
    name: "Notepad++",
    path: "C:\\Program Files\\Notepad++\\notepad++.exe",
    color: "#65a30d",
  },
  {
    name: "7-Zip",
    path: "C:\\Program Files\\7-Zip\\7zFM.exe",
    color: "#475569",
  },
  {
    name: "OBS Studio",
    path: "C:\\Program Files\\obs-studio\\bin\\64bit\\obs64.exe",
    color: "#0f766e",
  },
  {
    name: "PotPlayer",
    path: "C:\\Program Files\\DAUM\\PotPlayer\\PotPlayerMini64.exe",
    color: "#c2410c",
  },
  {
    name: "Figma",
    path: "C:\\Users\\Administrator\\AppData\\Local\\Figma\\Figma.exe",
    color: "#0e7490",
  },
  {
    name: "Telegram",
    path: "C:\\Users\\Administrator\\AppData\\Roaming\\Telegram Desktop\\Telegram.exe",
    color: "#0369a1",
  },
];

export interface SeedProgram extends ProgramPreset {
  included: boolean;
  createdTargets: TargetKind[];
}

/** 浏览器预览用的示例清单；桌面应用从空清单开始，由用户自己选择程序 */
export const SEED_PROGRAMS: SeedProgram[] = [
  {
    ...PROGRAM_PRESETS[0],
    included: true,
    createdTargets: ["desktop"],
  },
  {
    ...PROGRAM_PRESETS[1],
    included: true,
    createdTargets: ["desktop", "startmenu"],
  },
  {
    ...PROGRAM_PRESETS[2],
    included: true,
    createdTargets: [],
  },
  {
    ...PROGRAM_PRESETS[3],
    included: true,
    createdTargets: ["startmenu"],
  },
  {
    ...PROGRAM_PRESETS[4],
    included: true,
    createdTargets: [],
  },
  {
    ...PROGRAM_PRESETS[5],
    included: true,
    createdTargets: [],
  },
  {
    ...PROGRAM_PRESETS[6],
    included: true,
    createdTargets: [],
  },
  {
    ...PROGRAM_PRESETS[7],
    included: false,
    createdTargets: [],
  },
  {
    ...PROGRAM_PRESETS[10],
    included: true,
    createdTargets: [],
  },
  {
    ...PROGRAM_PRESETS[11],
    included: false,
    createdTargets: [],
  },
];

const COLOR_POOL = ["#2563eb", "#0f766e", "#b45309", "#1d4ed8", "#0e7490", "#b42318", "#65a30d"];

/** 从可执行文件路径推断默认程序名 */
export function nameFromPath(path: string): string {
  const file = path.split(/[\\/]/).pop() ?? "";
  return file.replace(/\.(exe|lnk|bat|cmd)$/i, "") || file;
}

/** 根据名称稳定分配一个图标底色 */
export function colorForName(name: string): string {
  let hash = 0;
  for (let i = 0; i < name.length; i += 1) {
    hash = (hash * 31 + name.charCodeAt(i)) % 997;
  }
  return COLOR_POOL[hash % COLOR_POOL.length];
}
