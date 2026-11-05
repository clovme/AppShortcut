use std::ffi::c_void;
use std::mem::size_of;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::Manager;
use windows::core::{w, GUID, Interface, PCWSTR};
use windows::Win32::Foundation::{
    GetLastError, SetLastError, ERROR_ALREADY_EXISTS, HWND, SIZE, WIN32_ERROR,
};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, BITMAP, BITMAPINFO, BI_RGB,
    DIB_RGB_COLORS, HBITMAP, HGDIOBJ,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, IBindCtx, IPersistFile,
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, STGM_READ,
};
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::Shell::Common::COMDLG_FILTERSPEC;
use windows::Win32::UI::Shell::{
    FileOpenDialog, FOLDERID_CommonPrograms, FOLDERID_Desktop, FOLDERID_Programs,
    FOLDERID_PublicDesktop, FOS_ALLOWMULTISELECT, FOS_FILEMUSTEXIST, FOS_PATHMUSTEXIST,
    IFileOpenDialog, IShellItem, IShellItemImageFactory, IShellLinkW, KF_FLAG_DEFAULT,
    SHCreateItemFromParsingName, SHGetKnownFolderPath, SIGDN_FILESYSPATH, SIIGBF,
    SIIGBF_BIGGERSIZEOK, SIIGBF_ICONONLY, ShellLink, FILEOPENDIALOGOPTIONS,
};
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, SetForegroundWindow, ShowWindow, SW_RESTORE,
};

const ERROR_CANCELLED_CODE: u32 = 0x8007_04C7;
const LIBRARY_FILE_NAME: &str = "shortcuts.json";
const SINGLE_INSTANCE_MUTEX: &str = "Local\\AppShortcut_SingleInstance";
const WINDOW_TITLE: &str = "快捷方式管家";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutItem {
    name: String,
    path: String,
    targets: Vec<String>,
    scope: String,
    #[serde(default)]
    group: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateResult {
    program: String,
    target: String,
    status: &'static str,
    message: Option<String>,
    link_path: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutState {
    program: String,
    program_path: String,
    target: String,
    status: &'static str,
    message: Option<String>,
    link_path: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearResult {
    deleted: usize,
    failed: usize,
    errors: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PickedProgram {
    name: String,
    path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgramIcon {
    path: String,
    icon: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryFile {
    path: String,
    content: String,
    portable: bool,
}

fn exe_dir() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|error| format!("无法定位程序目录：{error}"))?;
    exe.parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "无法定位程序目录".to_string())
}

fn portable_library_path() -> Result<PathBuf, String> {
    Ok(exe_dir()?.join(LIBRARY_FILE_NAME))
}

fn fallback_library_path() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join("AppShortcut").join(LIBRARY_FILE_NAME)
}

fn read_library_at(path: &Path) -> Result<Option<String>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(path)
        .map_err(|error| format!("读取清单 {} 失败：{error}", path.display()))?;
    if serde_json::from_str::<serde_json::Value>(&text).is_err() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        let backup = path.with_extension(format!("invalid-{stamp}.json"));
        let _ = std::fs::rename(path, &backup);
        return Ok(None);
    }
    Ok(Some(text))
}

fn write_library_at(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("无法创建目录 {}：{error}", parent.display()))?;
    }
    let temp = path.with_extension("json.tmp");
    std::fs::write(&temp, content)
        .map_err(|error| format!("写入清单临时文件失败：{error}"))?;
    std::fs::rename(&temp, path).map_err(|error| format!("保存清单失败：{error}"))?;
    Ok(())
}

fn load_library_blocking() -> Result<LibraryFile, String> {
    let portable_path = portable_library_path()?;
    if let Some(content) = read_library_at(&portable_path)? {
        return Ok(LibraryFile {
            path: portable_path.display().to_string(),
            content,
            portable: true,
        });
    }

    let fallback = fallback_library_path();
    if let Some(content) = read_library_at(&fallback)? {
        return Ok(LibraryFile {
            path: fallback.display().to_string(),
            content,
            portable: false,
        });
    }

    Ok(LibraryFile {
        path: portable_path.display().to_string(),
        content: String::new(),
        portable: true,
    })
}

fn save_library_blocking(content: &str) -> Result<LibraryFile, String> {
    serde_json::from_str::<serde_json::Value>(content)
        .map_err(|error| format!("清单内容不是合法 JSON：{error}"))?;

    let portable_path = portable_library_path()?;
    let portable_error = match write_library_at(&portable_path, content) {
        Ok(()) => {
            return Ok(LibraryFile {
                path: portable_path.display().to_string(),
                content: content.to_string(),
                portable: true,
            });
        }
        Err(error) => error,
    };

    let fallback = fallback_library_path();
    write_library_at(&fallback, content).map_err(|error| {
        format!(
            "程序目录写入失败（{portable_error}），备用目录 {} 也写入失败（{error}）",
            fallback.display()
        )
    })?;
    Ok(LibraryFile {
        path: fallback.display().to_string(),
        content: content.to_string(),
        portable: false,
    })
}

#[tauri::command]
fn load_library() -> Result<LibraryFile, String> {
    load_library_blocking()
}

#[tauri::command]
fn save_library(content: String) -> Result<LibraryFile, String> {
    save_library_blocking(&content)
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn sanitize_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|character| {
            if matches!(character, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
                '_'
            } else {
                character
            }
        })
        .collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        "快捷方式".to_string()
    } else {
        trimmed.chars().take(120).collect()
    }
}

fn known_folder(id: &GUID) -> Result<PathBuf, String> {
    unsafe {
        let raw = SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None)
            .map_err(|error| format!("无法获取系统目录：{error}"))?;
        let text = raw
            .to_string()
            .map_err(|error| format!("目录路径解析失败：{error}"))?;
        CoTaskMemFree(Some(raw.0 as *const c_void));
        Ok(PathBuf::from(text))
    }
}

fn sanitize_group(group: &str) -> String {
    if group.trim().is_empty() {
        return String::new();
    }
    let cleaned = sanitize_name(group);
    let trimmed = cleaned.trim().trim_matches('.').trim();
    if trimmed.is_empty() {
        String::new()
    } else {
        trimmed.chars().take(60).collect()
    }
}

fn folder_for(target: &str, scope: &str, group: &str) -> Result<PathBuf, String> {
    let id: &GUID = match (target, scope) {
        ("desktop", "all") => &FOLDERID_PublicDesktop,
        ("desktop", _) => &FOLDERID_Desktop,
        ("startmenu", "all") => &FOLDERID_CommonPrograms,
        ("startmenu", _) => &FOLDERID_Programs,
        _ => return Err(format!("未知的创建位置：{target}")),
    };
    let base = known_folder(id)?;
    if target == "startmenu" {
        let folder = sanitize_group(group);
        if folder.is_empty() {
            Ok(base)
        } else {
            Ok(base.join(folder))
        }
    } else {
        Ok(base)
    }
}

fn pick_programs_blocking(owner: Option<HWND>) -> Result<Vec<PickedProgram>, String> {
    unsafe {
        let init = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let result = pick_programs_inner(owner);
        if init.is_ok() {
            CoUninitialize();
        }
        result
    }
}

unsafe fn pick_programs_inner(owner: Option<HWND>) -> Result<Vec<PickedProgram>, String> {
    unsafe {
        let dialog: IFileOpenDialog = CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)
            .map_err(|error| format!("无法打开文件选择窗口：{error}"))?;

        let options = FILEOPENDIALOGOPTIONS(
            FOS_ALLOWMULTISELECT.0 | FOS_FILEMUSTEXIST.0 | FOS_PATHMUSTEXIST.0,
        );
        dialog
            .SetOptions(options)
            .map_err(|error| format!("文件选择窗口初始化失败：{error}"))?;
        let _ = dialog.SetTitle(w!("选择要创建快捷方式的程序"));
        let filters = [COMDLG_FILTERSPEC {
            pszName: w!("程序 (*.exe)"),
            pszSpec: w!("*.exe"),
        }];
        let _ = dialog.SetFileTypes(&filters);
        let _ = dialog.SetFileTypeIndex(1);

        if let Err(error) = dialog.Show(owner) {
            if error.code().0 as u32 == ERROR_CANCELLED_CODE {
                return Ok(Vec::new());
            }
            return Err(format!("文件选择窗口打开失败：{error}"));
        }

        let results = dialog
            .GetResults()
            .map_err(|error| format!("读取选择结果失败：{error}"))?;
        let count = results
            .GetCount()
            .map_err(|error| format!("读取选择结果失败：{error}"))?;

        let mut picked = Vec::new();
        for index in 0..count {
            let item = results
                .GetItemAt(index)
                .map_err(|error| format!("读取选择结果失败：{error}"))?;
            let raw = item
                .GetDisplayName(SIGDN_FILESYSPATH)
                .map_err(|error| format!("读取文件路径失败：{error}"))?;
            let path = raw.to_string().ok();
            CoTaskMemFree(Some(raw.0 as *const c_void));

            if let Some(path) = path {
                if path.is_empty() {
                    continue;
                }
                let name = Path::new(&path)
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("程序")
                    .to_string();
                picked.push(PickedProgram { name, path });
            }
        }
        Ok(picked)
    }
}

unsafe fn write_link(exe: &Path, link_path: &Path, description: &str) -> Result<(), String> {
    unsafe {
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)
            .map_err(|error| format!("创建快捷方式对象失败：{error}"))?;

        let exe_wide = wide(&exe.to_string_lossy());
        link.SetPath(PCWSTR::from_raw(exe_wide.as_ptr()))
            .map_err(|error| format!("设置程序路径失败：{error}"))?;

        if let Some(parent) = exe.parent() {
            let working_dir = wide(&parent.to_string_lossy());
            let _ = link.SetWorkingDirectory(PCWSTR::from_raw(working_dir.as_ptr()));
        }

        let description_wide = wide(description);
        let _ = link.SetDescription(PCWSTR::from_raw(description_wide.as_ptr()));
        let _ = link.SetIconLocation(PCWSTR::from_raw(exe_wide.as_ptr()), 0);

        let persist: IPersistFile = link
            .cast()
            .map_err(|error| format!("获取快捷方式保存接口失败：{error}"))?;
        let link_wide = wide(&link_path.to_string_lossy());
        persist
            .Save(PCWSTR::from_raw(link_wide.as_ptr()), true)
            .map_err(|error| format!("写入快捷方式失败：{error}"))
    }
}

fn normalize_path(path: &Path) -> String {
    let resolved = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    resolved
        .to_string_lossy()
        .replace('/', "\\")
        .to_lowercase()
}

fn same_target(shortcut_target: &Path, exe: &Path) -> bool {
    normalize_path(shortcut_target) == normalize_path(exe)
}

unsafe fn read_link_target(link_path: &Path) -> Option<PathBuf> {
    unsafe {
        let init = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let target = read_link_target_inner(link_path);
        if init.is_ok() {
            CoUninitialize();
        }
        target
    }
}

unsafe fn read_link_target_inner(link_path: &Path) -> Option<PathBuf> {
    unsafe {
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).ok()?;
        let persist: IPersistFile = link.cast().ok()?;
        let path_wide = wide(&link_path.to_string_lossy());
        persist
            .Load(PCWSTR::from_raw(path_wide.as_ptr()), STGM_READ)
            .ok()?;

        let mut buffer = [0u16; 1024];
        link.GetPath(&mut buffer, std::ptr::null_mut(), 0).ok()?;
        let end = buffer.iter().position(|value| *value == 0).unwrap_or(buffer.len());
        let target = String::from_utf16_lossy(&buffer[..end]);
        if target.trim().is_empty() {
            None
        } else {
            Some(PathBuf::from(target))
        }
    }
}

unsafe fn hbitmap_to_data_url(bitmap: HBITMAP) -> Option<String> {
    unsafe {
        let mut info: BITMAP = std::mem::zeroed();
        let got_object = GetObjectW(
            HGDIOBJ(bitmap.0),
            size_of::<BITMAP>() as i32,
            Some((&mut info as *mut BITMAP).cast()),
        );
        if got_object == 0 {
            return None;
        }
        let width = info.bmWidth;
        let height = info.bmHeight.abs();
        if width <= 0 || height <= 0 || width > 512 || height > 512 {
            return None;
        }

        let mut header: BITMAPINFO = std::mem::zeroed();
        header.bmiHeader.biSize = size_of::<windows::Win32::Graphics::Gdi::BITMAPINFOHEADER>() as u32;
        header.bmiHeader.biWidth = width;
        header.bmiHeader.biHeight = -height;
        header.bmiHeader.biPlanes = 1;
        header.bmiHeader.biBitCount = 32;
        header.bmiHeader.biCompression = BI_RGB.0;

        let mut pixels = vec![0u8; (width * height * 4) as usize];
        let memory_dc = CreateCompatibleDC(None);
        let dib_result = GetDIBits(
            memory_dc,
            bitmap,
            0,
            height as u32,
            Some(pixels.as_mut_ptr().cast()),
            &mut header,
            DIB_RGB_COLORS,
        );
        let _ = DeleteDC(memory_dc);
        if dib_result == 0 {
            return None;
        }

        let all_transparent = pixels.chunks_exact(4).all(|pixel| pixel[3] == 0);
        for pixel in pixels.chunks_exact_mut(4) {
            pixel.swap(0, 2);
            if all_transparent {
                pixel[3] = 255;
            }
        }

        let mut encoded = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut encoded, width as u32, height as u32);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().ok()?;
            writer.write_image_data(&pixels).ok()?;
            writer.finish().ok()?;
        }
        Some(format!(
            "data:image/png;base64,{}",
            base64_encode(&encoded)
        ))
    }
}

unsafe fn icon_for_path(path: &str) -> Option<String> {
    unsafe {
        let wide_path = wide(path);
        let item: IShellItem = SHCreateItemFromParsingName(
            PCWSTR::from_raw(wide_path.as_ptr()),
            None::<&IBindCtx>,
        )
        .ok()?;
        let factory: IShellItemImageFactory = item.cast().ok()?;
        let bitmap = factory
            .GetImage(
                SIZE {
                    cx: 64,
                    cy: 64,
                },
                SIIGBF(SIIGBF_ICONONLY.0 | SIIGBF_BIGGERSIZEOK.0),
            )
            .ok()?;
        let icon = hbitmap_to_data_url(bitmap);
        let _ = DeleteObject(HGDIOBJ(bitmap.0));
        icon
    }
}

fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        output.push(TABLE[(first >> 2) as usize] as char);
        output.push(TABLE[(((first & 0b11) << 4) | (second >> 4)) as usize] as char);
        if chunk.len() > 1 {
            output.push(TABLE[(((second & 0b1111) << 2) | (third >> 6)) as usize] as char);
        } else {
            output.push('=');
        }
        if chunk.len() > 2 {
            output.push(TABLE[(third & 0b11_1111) as usize] as char);
        } else {
            output.push('=');
        }
    }
    output
}

#[tauri::command]
async fn program_icons(paths: Vec<String>) -> Vec<ProgramIcon> {
    tauri::async_runtime::spawn_blocking(move || unsafe {
        let init = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let mut icons = Vec::with_capacity(paths.len());
        for path in paths {
            let icon = icon_for_path(&path);
            icons.push(ProgramIcon { path, icon });
        }
        if init.is_ok() {
            CoUninitialize();
        }
        icons
    })
    .await
    .unwrap_or_default()
}

fn create_one(item: &ShortcutItem, target: &str, overwrite: bool) -> CreateResult {
    let folder = match folder_for(target, &item.scope, &item.group) {
        Ok(folder) => folder,
        Err(error) => {
            return CreateResult {
                program: item.name.clone(),
                target: target.to_string(),
                status: "failed",
                message: Some(error),
                link_path: None,
            };
        }
    };
    create_one_in_folder(item, target, &folder, overwrite)
}

fn create_one_in_folder(
    item: &ShortcutItem,
    target: &str,
    folder: &Path,
    overwrite: bool,
) -> CreateResult {
    let result = |status: &'static str, message: Option<String>, link: Option<PathBuf>| CreateResult {
        program: item.name.clone(),
        target: target.to_string(),
        status,
        message,
        link_path: link.map(|path| path.to_string_lossy().into_owned()),
    };

    if let Err(error) = std::fs::create_dir_all(folder) {
        return result(
            "failed",
            Some(format!("无法创建目标目录 {}：{error}", folder.display())),
            None,
        );
    }

    let exe = Path::new(&item.path);
    if !exe.is_file() {
        return result(
            "failed",
            Some(format!("程序文件不存在：{}", item.path)),
            None,
        );
    }

    let name = sanitize_name(&item.name);
    let link_path = shortcut_path(folder, &item.name);

    if link_path.exists() {
        let existing = unsafe { read_link_target(&link_path) };
        let matches_target = existing
            .as_deref()
            .map(|existing_target| same_target(existing_target, exe))
            .unwrap_or(false);

        if matches_target {
            if !overwrite {
                return result(
                    "skipped",
                    Some("同名快捷方式已存在且指向同一程序，已跳过".to_string()),
                    Some(link_path),
                );
            }
        } else if !overwrite {
            let detail = existing
                .map(|existing_target| format!("，现有快捷方式指向 {}", existing_target.display()))
                .unwrap_or_else(|| "，无法读取现有快捷方式的目标".to_string());
            return result(
                "failed",
                Some(format!(
                    "同名快捷方式已存在但目标不一致{detail}；开启“覆盖同名快捷方式”后可替换"
                )),
                Some(link_path),
            );
        }
    }

    match unsafe { write_link(exe, &link_path, &name) } {
        Ok(()) => result("created", None, Some(link_path)),
        Err(error) => result("failed", Some(error), Some(link_path)),
    }
}

fn shortcut_path(folder: &Path, name: &str) -> PathBuf {
    folder.join(format!("{}.lnk", sanitize_name(name)))
}

fn inspect_one_in_folder(item: &ShortcutItem, target: &str, folder: &Path) -> ShortcutState {
    let state = |status: &'static str, message: Option<String>, link: Option<PathBuf>| ShortcutState {
        program: item.name.clone(),
        program_path: item.path.clone(),
        target: target.to_string(),
        status,
        message,
        link_path: link.map(|path| path.to_string_lossy().into_owned()),
    };

    let exe = Path::new(&item.path);
    let link_path = shortcut_path(folder, &item.name);
    if !link_path.is_file() {
        return state("missing", None, Some(link_path));
    }

    match unsafe { read_link_target(&link_path) } {
        Some(existing) if same_target(&existing, exe) => state(
            "exists",
            Some("已存在指向该程序的快捷方式".to_string()),
            Some(link_path),
        ),
        Some(existing) => state(
            "conflict",
            Some(format!("同名快捷方式指向其他程序：{}", existing.display())),
            Some(link_path),
        ),
        None => state(
            "conflict",
            Some("同名快捷方式已存在，但无法读取它的目标路径".to_string()),
            Some(link_path),
        ),
    }
}

fn inspect_shortcuts_blocking(items: Vec<ShortcutItem>) -> Vec<ShortcutState> {
    let mut states = Vec::new();
    for item in &items {
        for target in &item.targets {
            let folder = match folder_for(target, &item.scope, &item.group) {
                Ok(folder) => folder,
                Err(error) => {
                    states.push(ShortcutState {
                        program: item.name.clone(),
                        program_path: item.path.clone(),
                        target: target.clone(),
                        status: "error",
                        message: Some(error),
                        link_path: None,
                    });
                    continue;
                }
            };
            states.push(inspect_one_in_folder(item, target, &folder));
        }
    }
    states
}

fn clear_one_in_folder(item: &ShortcutItem, folder: &Path) -> Result<bool, String> {
    let exe = Path::new(&item.path);
    let link_path = shortcut_path(folder, &item.name);
    if !link_path.is_file() {
        return Ok(false);
    }

    match unsafe { read_link_target(&link_path) } {
        Some(existing) if same_target(&existing, exe) => {
            std::fs::remove_file(&link_path)
                .map_err(|error| format!("删除快捷方式 {} 失败：{error}", link_path.display()))?;
            Ok(true)
        }
        Some(existing) => Err(format!(
            "{} 指向其他程序（{}），未删除",
            link_path.display(),
            existing.display()
        )),
        None => Err(format!(
            "{} 的目标路径无法读取，未删除",
            link_path.display()
        )),
    }
}

fn clear_shortcuts_blocking(items: Vec<ShortcutItem>, targets: Vec<String>) -> ClearResult {
    let mut result = ClearResult {
        deleted: 0,
        failed: 0,
        errors: Vec::new(),
    };

    for item in &items {
        // 新版本每个程序带自己的 targets；旧调用没有时退回外层的 targets
        let item_targets: Vec<String> = if item.targets.is_empty() {
            targets.clone()
        } else {
            item.targets.clone()
        };
        for target in &item_targets {
            let folder = match folder_for(target, &item.scope, &item.group) {
                Ok(folder) => folder,
                Err(error) => {
                    result.failed += 1;
                    result.errors.push(error);
                    continue;
                }
            };

            match clear_one_in_folder(item, &folder) {
                Ok(true) => result.deleted += 1,
                Ok(false) => {}
                Err(error) => {
                    result.failed += 1;
                    result.errors.push(error);
                }
            }

            if target == "startmenu" && !sanitize_group(&item.group).is_empty() {
                let _ = std::fs::remove_dir(&folder);
            }
        }
    }

    result
}

#[tauri::command]
async fn inspect_shortcuts(items: Vec<ShortcutItem>) -> Vec<ShortcutState> {
    tauri::async_runtime::spawn_blocking(move || inspect_shortcuts_blocking(items))
        .await
        .unwrap_or_default()
}

#[tauri::command]
async fn clear_shortcuts(items: Vec<ShortcutItem>, targets: Vec<String>) -> ClearResult {
    tauri::async_runtime::spawn_blocking(move || clear_shortcuts_blocking(items, targets))
        .await
        .unwrap_or_else(|_| ClearResult {
            deleted: 0,
            failed: 1,
            errors: vec!["清理快捷方式任务失败".to_string()],
        })
}

fn create_shortcuts_blocking(items: Vec<ShortcutItem>, overwrite: bool) -> Vec<CreateResult> {
    unsafe {
        let init = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let mut results = Vec::new();
        for item in items {
            for target in &item.targets {
                results.push(create_one(&item, target, overwrite));
            }
        }
        if init.is_ok() {
            CoUninitialize();
        }
        results
    }
}

#[tauri::command]
async fn pick_programs(app: tauri::AppHandle) -> Result<Vec<PickedProgram>, String> {
    // HWND 不是 Send，跨线程只传原始地址，进入工作线程后再还原
    let owner_raw = app
        .get_webview_window("main")
        .and_then(|window| window.hwnd().ok())
        .map(|hwnd| hwnd.0 as isize);
    tauri::async_runtime::spawn_blocking(move || {
        let owner = owner_raw.map(|raw| HWND(raw as *mut c_void));
        pick_programs_blocking(owner)
    })
    .await
    .map_err(|error| format!("选择程序失败：{error}"))?
}

#[tauri::command]
async fn create_shortcuts(
    items: Vec<ShortcutItem>,
    overwrite: bool,
) -> Result<Vec<CreateResult>, String> {
    let results = tauri::async_runtime::spawn_blocking(move || {
        create_shortcuts_blocking(items, overwrite)
    })
    .await
    .map_err(|error| format!("创建快捷方式失败：{error}"))?;
    Ok(results)
}

#[cfg(windows)]
fn enforce_single_instance() {
    unsafe {
        SetLastError(WIN32_ERROR(0));
        let mutex_name = wide(SINGLE_INSTANCE_MUTEX);
        let handle = match CreateMutexW(None, true, PCWSTR::from_raw(mutex_name.as_ptr())) {
            Ok(handle) => handle,
            Err(_) => return,
        };
        // HANDLE 没有 Drop 实现，进程退出时由系统回收
        let _handle = handle;

        if GetLastError() != ERROR_ALREADY_EXISTS {
            return;
        }

        let title = wide(WINDOW_TITLE);
        if let Ok(window) = FindWindowW(None, PCWSTR::from_raw(title.as_ptr())) {
            if !window.0.is_null() {
                let _ = ShowWindow(window, SW_RESTORE);
                let _ = SetForegroundWindow(window);
            }
        }
        std::process::exit(0);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(windows)]
    enforce_single_instance();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            pick_programs,
            create_shortcuts,
            load_library,
            save_library,
            program_icons,
            inspect_shortcuts,
            clear_shortcuts
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_shortcut_into_temp_folder() {
        let exe = PathBuf::from(r"C:\Windows\System32\notepad.exe");
        assert!(exe.is_file(), "notepad.exe 应存在于系统中");

        let folder = std::env::temp_dir().join("appshortcut-lnk-test");
        std::fs::create_dir_all(&folder).expect("创建测试目录");
        let link_path = folder.join("Notepad 测试.lnk");
        let _ = std::fs::remove_file(&link_path);

        unsafe {
            let init = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            write_link(&exe, &link_path, "Notepad 测试").expect("写入快捷方式");
            if init.is_ok() {
                CoUninitialize();
            }
        }

        assert!(link_path.is_file(), "快捷方式文件应已生成");
        assert!(std::fs::metadata(&link_path).expect("读取元数据").len() > 0);

        let _ = std::fs::remove_file(&link_path);
        let _ = std::fs::remove_dir(&folder);
    }

    #[test]
    fn sanitizes_invalid_file_name_characters() {
        assert_eq!(sanitize_name("a/b:c*?\"<>|d"), "a_b_c______d");
        assert_eq!(sanitize_name("   "), "快捷方式");
    }

    #[test]
    fn skips_when_existing_shortcut_name_and_target_match() {
        let exe = PathBuf::from(r"C:\Windows\System32\notepad.exe");
        assert!(exe.is_file(), "notepad.exe 应存在于系统中");

        let folder = std::env::temp_dir().join("appshortcut-skip-test");
        std::fs::create_dir_all(&folder).expect("创建测试目录");
        let link_path = folder.join("Skip 测试.lnk");
        let _ = std::fs::remove_file(&link_path);

        let item = ShortcutItem {
            name: "Skip 测试".to_string(),
            path: exe.to_string_lossy().into_owned(),
            targets: vec!["desktop".to_string()],
            scope: "current".to_string(),
            group: "RustRover家族".to_string(),
        };

        unsafe {
            let init = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            write_link(&exe, &link_path, "Skip 测试").expect("写入已有快捷方式");
            if init.is_ok() {
                CoUninitialize();
            }
        }

        let before = std::fs::read(&link_path).expect("读取已有快捷方式");
        let skipped = create_one_in_folder(&item, "desktop", &folder, false);
        assert_eq!(skipped.status, "skipped");
        assert_eq!(
            std::fs::read(&link_path).expect("再次读取"),
            before,
            "同名且目标一致时不应改动已有文件"
        );

        let _ = std::fs::remove_file(&link_path);
        let _ = std::fs::remove_dir(&folder);
    }

    #[test]
    fn rejects_same_name_with_different_target_unless_overwrite() {
        let exe = PathBuf::from(r"C:\Windows\System32\notepad.exe");
        let other = PathBuf::from(r"C:\Windows\System32\cmd.exe");
        assert!(exe.is_file() && other.is_file());

        let folder = std::env::temp_dir().join("appshortcut-conflict-test");
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).expect("创建测试目录");
        let link_path = folder.join("Conflict 测试.lnk");

        let item = ShortcutItem {
            name: "Conflict 测试".to_string(),
            path: exe.to_string_lossy().into_owned(),
            targets: vec!["desktop".to_string()],
            scope: "current".to_string(),
            group: String::new(),
        };

        unsafe {
            let init = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            write_link(&other, &link_path, "Conflict 测试").expect("写入指向 cmd 的快捷方式");
            if init.is_ok() {
                CoUninitialize();
            }
        }

        let before = std::fs::read(&link_path).expect("读取已有快捷方式");
        let rejected = create_one_in_folder(&item, "desktop", &folder, false);
        assert_eq!(rejected.status, "failed");
        assert!(rejected
            .message
            .as_deref()
            .unwrap_or_default()
            .contains("目标不一致"));
        assert_eq!(std::fs::read(&link_path).expect("再次读取"), before);

        unsafe {
            let init = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            let created = create_one_in_folder(&item, "desktop", &folder, true);
            if init.is_ok() {
                CoUninitialize();
            }
            assert_eq!(created.status, "created");
        }

        let target = unsafe {
            let init = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            let target = read_link_target(&link_path);
            if init.is_ok() {
                CoUninitialize();
            }
            target
        };
        assert!(
            target.as_deref().map(|value| same_target(value, &exe)).unwrap_or(false),
            "覆盖后应指向 notepad"
        );

        let _ = std::fs::remove_dir_all(&folder);
    }

    #[test]
    fn start_menu_shortcuts_use_custom_group_folder() {
        let folder = folder_for("startmenu", "current", "RustRover家族")
            .expect("读取当前用户开始菜单");
        assert_eq!(
            folder.file_name().and_then(|name| name.to_str()),
            Some("RustRover家族")
        );
        let desktop = folder_for("desktop", "current", "RustRover家族").expect("读取桌面目录");
        assert_ne!(
            desktop.file_name().and_then(|name| name.to_str()),
            Some("RustRover家族")
        );
        let root = folder_for("startmenu", "current", "").expect("读取开始菜单根目录");
        assert_ne!(
            root.file_name().and_then(|name| name.to_str()),
            Some("RustRover家族")
        );
    }

    #[test]
    fn status_and_clear_use_actual_link_target() {
        let exe = PathBuf::from(r"C:\Windows\System32\notepad.exe");
        let other = PathBuf::from(r"C:\Windows\System32\cmd.exe");
        assert!(exe.is_file() && other.is_file());

        let folder = std::env::temp_dir().join("appshortcut-status-test");
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).expect("创建测试目录");
        let link_path = folder.join("Status 测试.lnk");

        let item = ShortcutItem {
            name: "Status 测试".to_string(),
            path: exe.to_string_lossy().into_owned(),
            targets: vec!["desktop".to_string()],
            scope: "current".to_string(),
            group: String::new(),
        };

        let missing = inspect_one_in_folder(&item, "desktop", &folder);
        assert_eq!(missing.status, "missing");

        unsafe {
            let init = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            write_link(&other, &link_path, "Status 测试").expect("写入指向 cmd 的快捷方式");
            if init.is_ok() {
                CoUninitialize();
            }
        }

        let conflict = inspect_one_in_folder(&item, "desktop", &folder);
        assert_eq!(conflict.status, "conflict");
        assert!(conflict
            .message
            .as_deref()
            .unwrap_or_default()
            .contains("其他程序"));
        assert!(clear_one_in_folder(&item, &folder).is_err(), "冲突文件不应删除");
        assert!(link_path.is_file());

        unsafe {
            let init = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            write_link(&exe, &link_path, "Status 测试").expect("改写成指向 notepad 的快捷方式");
            if init.is_ok() {
                CoUninitialize();
            }
        }

        let exists = inspect_one_in_folder(&item, "desktop", &folder);
        assert_eq!(exists.status, "exists");
        assert!(clear_one_in_folder(&item, &folder).expect("删除匹配快捷方式"));
        assert!(!link_path.is_file(), "清除标记应删除真实快捷方式");

        let _ = std::fs::remove_dir_all(&folder);
    }

    #[test]
    fn extracts_program_icon_as_png_data_url() {
        let icon = unsafe {
            let init = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            let icon = icon_for_path(r"C:\Windows\System32\notepad.exe");
            if init.is_ok() {
                CoUninitialize();
            }
            icon
        };
        let value = icon.expect("notepad 应能取到图标");
        assert!(value.starts_with("data:image/png;base64,"));
        assert!(value.len() > 128, "图标数据不应为空");
    }

    #[test]
    fn library_file_round_trip_and_quarantines_invalid_json() {
        let folder = std::env::temp_dir().join("appshortcut-library-test");
        let _ = std::fs::remove_dir_all(&folder);
        let path = folder.join("shortcuts.json");

        let text = r#"{"version":1,"programs":[]}"#;
        write_library_at(&path, text).expect("写入清单");
        assert_eq!(
            read_library_at(&path).expect("读取清单").as_deref(),
            Some(text)
        );
        assert!(!folder.join("shortcuts.json.tmp").exists());

        std::fs::write(&path, b"not json").expect("写入坏数据");
        assert!(read_library_at(&path).expect("读取坏数据").is_none());
        assert!(!path.exists(), "损坏的清单应被移走");
        let backup = std::fs::read_dir(&folder)
            .expect("读取目录")
            .filter_map(|entry| entry.ok())
            .find(|entry| entry.file_name().to_string_lossy().contains("invalid-"));
        assert!(backup.is_some(), "应保留损坏清单的备份");

        let _ = std::fs::remove_dir_all(&folder);
    }
}
