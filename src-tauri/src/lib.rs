mod compress;
mod context_menu;
mod settings;

use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_notification::NotificationExt;

#[cfg(target_os = "macos")]
extern "C" {
    fn registerTinyImageService();
    fn setActivationPolicyAccessory();
}

// ── 启动文件队列 ───────────────────────────────────────────────
use std::sync::Mutex;
static STARTUP_FILES: Mutex<Vec<(String, bool)>> = Mutex::new(vec![]);
static FRONTEND_READY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static IS_BACKGROUND: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

// ── 后台压缩计数器 ────────────────────────────────────────────
// 用于多文件后台压缩：等所有任务结束后再通知并退出
static BG_PENDING: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);
static BG_RESULTS: Mutex<(u32, u32)> = Mutex::new((0, 0)); // (success, error)

// ── 后台压缩辅助函数 ──────────────────────────────────────────
// 为每个文件启动一个 spawn_blocking 任务；全部完成后发通知，
// 若是纯后台模式（IS_BACKGROUND）则退出 app。
fn spawn_bg_compress(app: AppHandle, files: Vec<String>) {
    let settings = settings::load();
    for file in files {
        let prev = BG_PENDING.fetch_add(1, Ordering::SeqCst);
        // 新的一批次开始（从 0 变为 1），重置累计结果，避免跨批次数字累加
        if prev == 0 {
            if let Ok(mut g) = BG_RESULTS.lock() {
                *g = (0, 0);
            }
        }
        let handle = app.clone();
        let f = file.clone();
        let s = settings.clone();
        tauri::async_runtime::spawn(async move {
            let handle2 = handle.clone();
            let res = tokio::task::spawn_blocking(move || {
                compress::compress_image(&f, &s, &handle2)
            })
            .await
            .unwrap_or_else(|_| Err(anyhow::anyhow!("panic")));

            {
                let mut g = BG_RESULTS.lock().unwrap_or_else(|e| e.into_inner());
                if res.is_ok() {
                    g.0 += 1;
                } else {
                    g.1 += 1;
                }
            }

            let rem = BG_PENDING.fetch_sub(1, Ordering::SeqCst) - 1;
            if rem == 0 {
                // 等待 500ms，让 MultiSelectModel=Player 的后续调用通过单实例路由过来
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                if BG_PENDING.load(Ordering::SeqCst) > 0 {
                    return; // 有新任务加入，不退出
                }

                let (ok, err) = *BG_RESULTS.lock().unwrap_or_else(|e| e.into_inner());
                // 后台模式强制使用系统通知（窗口隐藏，dialog 无法显示）
                let message = if err == 0 {
                    format!("成功压缩 {} 张图片", ok)
                } else {
                    format!("压缩完成：{} 成功，{} 失败", ok, err)
                };
                handle
                    .notification()
                    .builder()
                    .title("TinyImage")
                    .body(&message)
                    .show()
                    .ok();

                if IS_BACKGROUND.load(Ordering::SeqCst) {
                    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                    handle.exit(0);
                }
            }
        });
    }
}

// ── 普通命令 ──────────────────────────────────────────────────

#[tauri::command]
fn init_window(window: tauri::Window) {
    if !IS_BACKGROUND.load(Ordering::SeqCst) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
fn get_startup_files() -> Vec<(String, bool)> {
    FRONTEND_READY.store(true, Ordering::SeqCst);
    STARTUP_FILES.lock().map(|mut v| v.drain(..).collect()).unwrap_or_default()
}

// ── 设置命令 ──────────────────────────────────────────────────

#[tauri::command]
fn load_settings() -> settings::AppSettings {
    settings::load()
}

#[tauri::command]
fn save_settings(settings: settings::AppSettings) -> Result<(), String> {
    settings::save(&settings).map_err(|e| e.to_string())
}

// ── 图片预览命令 ──────────────────────────────────────────────

#[tauri::command]
fn get_image_preview(path: String) -> Result<String, String> {
    use base64::Engine;
    let data = std::fs::read(&path).map_err(|e| e.to_string())?;
    let ext = std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpg")
        .to_lowercase();
    let mime = match ext.as_str() {
        "png" => "image/png",
        "webp" => "image/webp",
        _ => "image/jpeg",
    };
    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
    Ok(format!("data:{};base64,{}", mime, b64))
}

/// 将 tinyimage://compress?file=path1&file=path2 中的文件路径解析出来
fn parse_files_from_url(url: &str) -> Vec<String> {
    let query = match url.find('?') {
        Some(i) => &url[i + 1..],
        None => return vec![],
    };
    let mut files = Vec::new();
    for param in query.split('&') {
        if let Some(encoded) = param.strip_prefix("file=") {
            let decoded = percent_decode(encoded);
            if !decoded.is_empty() {
                files.push(decoded);
            }
        }
    }
    filter_image_args(files)
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (
                (bytes[i + 1] as char).to_digit(16),
                (bytes[i + 2] as char).to_digit(16),
            ) {
                out.push((h * 16 + l) as u8);
                i += 3;
                continue;
            }
        } else if bytes[i] == b'+' {
            out.push(b' ');
            i += 1;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

// ── 压缩命令 ──────────────────────────────────────────────────

#[tauri::command]
async fn compress_image(
    app: AppHandle,
    file_path: String,
    settings: settings::AppSettings,
) -> Result<compress::CompressResult, String> {
    tokio::task::spawn_blocking(move || {
        compress::compress_image(&file_path, &settings, &app)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

// ── 通知命令 ──────────────────────────────────────────────────

#[tauri::command]
fn notify_result(
    app: AppHandle,
    settings: settings::AppSettings,
    success_count: u32,
    error_count: u32,
) -> Result<(), String> {
    use settings::NotifyMode;

    let total = success_count + error_count;
    if total == 0 {
        return Ok(());
    }

    let message = if error_count == 0 {
        format!("成功压缩 {} 张图片", success_count)
    } else {
        format!(
            "压缩完成：{} 成功，{} 失败",
            success_count, error_count
        )
    };

    match settings.notify_mode {
        NotifyMode::Silent => {}

        NotifyMode::Dialog => {
            app.emit("show-result-dialog", &message)
                .map_err(|e| e.to_string())?;
        }

        NotifyMode::Notification => {
            app.notification()
                .builder()
                .title("TinyImage")
                .body(&message)
                .show()
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

// ── 右键菜单命令 ──────────────────────────────────────────────

#[tauri::command]
fn register_context_menu() -> Result<(), String> {
    context_menu::register().map_err(|e| e.to_string())
}

#[tauri::command]
fn unregister_context_menu() -> Result<(), String> {
    context_menu::unregister().map_err(|e| e.to_string())
}

// ── 工具函数 ──────────────────────────────────────────────────

fn filter_image_args(args: Vec<String>) -> Vec<String> {
    let image_exts = ["png", "jpg", "jpeg", "webp"];
    args.into_iter()
        .filter(|a| {
            let lower = a.to_lowercase();
            image_exts.iter().any(|ext| lower.ends_with(ext))
        })
        .collect()
}

/// 解析命令行参数，返回 (is_compress_mode, file_paths)
fn parse_args(raw: Vec<String>) -> (bool, Vec<String>) {
    let is_compress = raw.iter().any(|a| a == "--compress");
    let files = filter_image_args(
        raw.into_iter().filter(|a| a != "--compress").collect(),
    );
    (is_compress, files)
}

// ── Tauri 入口 ────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // 单实例插件必须第一个注册
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            // argv 是第二个实例的完整 std::env::args()，skip(1) 去掉程序路径
            let (is_compress, files) = parse_args(argv.into_iter().skip(1).collect());

            if is_compress && !files.is_empty() {
                // 右键"用TinyImage压缩"：在当前进程后台压缩，不影响 UI。
                // 若当前进程从未向用户展示过窗口（由 NSServices 静默启动的中间进程），
                // 则压缩完成后自动退出，避免留下隐藏的僵尸进程。
                let was_launched_silently = !FRONTEND_READY.load(Ordering::SeqCst)
                    && !app
                        .get_webview_window("main")
                        .and_then(|w| w.is_visible().ok())
                        .unwrap_or(false);
                if was_launched_silently {
                    IS_BACKGROUND.store(true, Ordering::SeqCst);
                    #[cfg(target_os = "macos")]
                    unsafe { setActivationPolicyAccessory() };
                }
                spawn_bg_compress(app.clone(), files);
            } else if !files.is_empty() {
                // 打开方式：仅添加文件到列表，不自动压缩
                if FRONTEND_READY.load(Ordering::SeqCst) {
                    app.emit("add-files", &files).ok();
                } else {
                    if let Ok(mut guard) = STARTUP_FILES.lock() {
                        for f in &files {
                            guard.push((f.clone(), false));
                        }
                    }
                }
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            } else {
                // 无文件参数（直接启动第二个实例），只把已有窗口带到前台
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            // 注册原生 NSServices 处理器（macOS）
            #[cfg(target_os = "macos")]
            unsafe {
                registerTinyImageService();
            }

            // 清理旧版 Automator workflow 残留
            #[cfg(target_os = "macos")]
            context_menu::cleanup_legacy_workflows();

            // ── 解析启动参数 ──────────────────────────────────
            // skip(1) 去掉 argv[0]（程序路径），再 skip(1) 去掉 Tauri 内部注入的 URL scheme 参数
            let raw: Vec<String> = std::env::args().skip(1).collect();
            let (is_compress, file_args) = parse_args(raw);

            if is_compress {
                // 右键"用TinyImage压缩"首次启动：纯后台模式，压缩完退出
                IS_BACKGROUND.store(true, Ordering::SeqCst);
                #[cfg(target_os = "macos")]
                unsafe { setActivationPolicyAccessory() };
                if !file_args.is_empty() {
                    spawn_bg_compress(app.handle().clone(), file_args);
                } else {
                    // --compress 但没有文件，直接退出
                    app.handle().exit(0);
                }
            } else if !file_args.is_empty() {
                // 通过"打开方式"启动：显示窗口，仅添加文件，不自动压缩
                if let Ok(mut guard) = STARTUP_FILES.lock() {
                    for f in file_args {
                        guard.push((f, false));
                    }
                }
            }

            // ── macOS deep-link 处理 ──────────────────────────
            let handle = app.handle().clone();
            app.deep_link().on_open_url(move |event| {
                for url in event.urls() {
                    // ── macOS "打开方式" 传来的 file:// URL ──────────
                    // 直接提取本地路径，仅添加到列表，不自动压缩
                    if url.scheme() == "file" {
                        if let Ok(path) = url.to_file_path() {
                            let path_str = path.to_string_lossy().to_string();
                            let valid = filter_image_args(vec![path_str]);
                            if valid.is_empty() {
                                continue;
                            }
                            if !FRONTEND_READY.load(Ordering::SeqCst) {
                                if let Ok(mut guard) = STARTUP_FILES.lock() {
                                    for f in &valid {
                                        guard.push((f.clone(), false));
                                    }
                                }
                            } else {
                                handle.emit("add-files", &valid).ok();
                            }
                            if let Some(window) = handle.get_webview_window("main") {
                                let _ = window.unminimize();
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        continue;
                    }

                    // ── tinyimage:// 自定义协议（右键压缩 / NSServices）
                    let u = url.as_str();
                    let is_bg = u.contains("background=1");

                    let mut was_hidden = true;
                    if let Some(window) = handle.get_webview_window("main") {
                        if let Ok(v) = window.is_visible() {
                            was_hidden = !v;
                        }
                    }

                    let files = parse_files_from_url(u);
                    if files.is_empty() {
                        continue;
                    }

                    if is_bg {
                        // 服务菜单请求：始终后台压缩，不弹出窗口。
                        // 若 app 是因本次请求才启动的（窗口从未显示），则压缩后退出并隐藏 Dock 图标。
                        if was_hidden {
                            IS_BACKGROUND.store(true, Ordering::SeqCst);
                            #[cfg(target_os = "macos")]
                            unsafe { setActivationPolicyAccessory() };
                        }
                        spawn_bg_compress(handle.clone(), files);
                        continue;
                    }

                    // 前台模式：显示 UI，把文件发给前端并自动压缩
                    if !FRONTEND_READY.load(Ordering::SeqCst) {
                        if let Ok(mut guard) = STARTUP_FILES.lock() {
                            for f in &files {
                                guard.push((f.clone(), true));
                            }
                        }
                    }
                    handle.emit("compress-files", &files).ok();
                    if let Some(window) = handle.get_webview_window("main") {
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_settings,
            save_settings,
            get_image_preview,
            compress_image,
            notify_result,
            register_context_menu,
            unregister_context_menu,
            get_startup_files,
            init_window,
        ])
        .build(tauri::generate_context!())
        .expect("构建 TinyImage 时出错")
        .run(|app, event| {
            // RunEvent::Opened 仅在 macOS 存在，专用于"打开方式"和图标拖拽
            // Windows 不编译此分支，避免 E0599 编译错误
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Opened { urls } = &event {
                let files: Vec<String> = urls
                    .iter()
                    .filter(|u| u.scheme() == "file")
                    .filter_map(|u| u.to_file_path().ok())
                    .map(|p| p.to_string_lossy().into_owned())
                    .collect();
                let valid = filter_image_args(files);
                if !valid.is_empty() {
                    if !FRONTEND_READY.load(Ordering::SeqCst) {
                        if let Ok(mut guard) = STARTUP_FILES.lock() {
                            for f in &valid {
                                guard.push((f.clone(), false));
                            }
                        }
                    } else {
                        app.emit("add-files", &valid).ok();
                    }
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
            let _ = (app, event);
        });
}
