mod compress;
mod context_menu;
mod settings;

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_notification::NotificationExt;

#[cfg(target_os = "macos")]
extern "C" {
    fn registerTinyImageService();
}

// ── 启动文件队列（Open With 传入的文件） ─────────────────────────
use std::sync::Mutex;
static STARTUP_FILES: Mutex<Vec<(String, bool)>> = Mutex::new(vec![]);
static FRONTEND_READY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static IS_BACKGROUND: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[tauri::command]
fn init_window(window: tauri::Window) {
    if !IS_BACKGROUND.load(std::sync::atomic::Ordering::SeqCst) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
fn get_startup_files() -> Vec<(String, bool)> {
    FRONTEND_READY.store(true, std::sync::atomic::Ordering::SeqCst);
    STARTUP_FILES.lock().map(|mut v| v.drain(..).collect()).unwrap_or_default()
}

// ── 设置命令 ─────────────────────────────────────────────────

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

// ── 压缩命令 ─────────────────────────────────────────────────

#[tauri::command]
async fn compress_image(
    file_path: String,
    settings: settings::AppSettings,
) -> Result<compress::CompressResult, String> {
    // 使用 spawn_blocking 防止阻塞 async 运行时，保证 UI 响应式更新能实时推送
    tokio::task::spawn_blocking(move || {
        compress::compress_image(&file_path, &settings)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

// ── 通知命令 ─────────────────────────────────────────────────

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
            // 通过事件发给前端显示 dialog
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

// ── 右键菜单命令 ─────────────────────────────────────────────

#[tauri::command]
fn register_context_menu() -> Result<(), String> {
    context_menu::register().map_err(|e| e.to_string())
}

#[tauri::command]
fn unregister_context_menu() -> Result<(), String> {
    context_menu::unregister().map_err(|e| e.to_string())
}

// ── Tauri 入口 ────────────────────────────────────────────────

fn filter_image_args(args: Vec<String>) -> Vec<String> {
    let image_exts = ["png", "jpg", "jpeg", "webp"];
    args.into_iter()
        .filter(|a| {
            let lower = a.to_lowercase();
            image_exts.iter().any(|ext| lower.ends_with(ext))
        })
        .collect()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // 注册原生 NSServices 处理器（macOS）
            #[cfg(target_os = "macos")]
            unsafe {
                registerTinyImageService();
            }

            // 清理旧版 Automator workflow 残留
            #[cfg(target_os = "macos")]
            context_menu::cleanup_legacy_workflows();

            // 收集通过命令行参数（如 Windows）传入的文件
            let startup = filter_image_args(
                std::env::args().skip(1).collect::<Vec<_>>(),
            );
            if !startup.is_empty() {
                if let Ok(mut guard) = STARTUP_FILES.lock() {
                    for f in startup {
                        guard.push((f, true));
                    }
                }
            }

            // 注册 tinyimage:// URL scheme 处理器
            // app 未运行时 macOS 会先启动再发送 URL；已运行时直接发给现有实例
            let handle = app.handle().clone();
            app.deep_link().on_open_url(move |event| {
                for url in event.urls() {
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

                    if is_bg && was_hidden {
                        // 纯后台模式：不显示 UI，压缩完直接退出
                        IS_BACKGROUND.store(true, std::sync::atomic::Ordering::SeqCst);
                        let app_handle = handle.clone();
                        tauri::async_runtime::spawn(async move {
                            let settings = settings::load();
                            let mut success = 0;
                            let mut error = 0;
                            for file in files {
                                let f = file.clone();
                                let s = settings.clone();
                                let res = tokio::task::spawn_blocking(move || {
                                    compress::compress_image(&f, &s)
                                }).await.unwrap_or_else(|_| Err(anyhow::anyhow!("Panic")));
                                
                                if res.is_ok() { success += 1; } else { error += 1; }
                            }
                            let _ = notify_result(app_handle.clone(), settings, success, error);
                            // 留一点时间给通知发出
                            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                            app_handle.exit(0);
                        });
                        continue;
                    }

                    // 前台模式：显示 UI，把文件发给前端
                    if !FRONTEND_READY.load(std::sync::atomic::Ordering::SeqCst) {
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
            if let tauri::RunEvent::Opened { urls } = event {
                // macOS Open With & Drag-to-Icon 触发此事件
                let mut files = Vec::new();
                for url in urls {
                    // url 是 file:// 开头的本地路径
                    if url.scheme() == "file" {
                        if let Ok(path) = url.to_file_path() {
                            files.push(path.to_string_lossy().to_string());
                        }
                    }
                }
                let valid_files = filter_image_args(files);
                if !valid_files.is_empty() {
                    if !FRONTEND_READY.load(std::sync::atomic::Ordering::SeqCst) {
                        if let Ok(mut guard) = STARTUP_FILES.lock() {
                            for f in &valid_files {
                                guard.push((f.clone(), false));
                            }
                        }
                    }
                    // 发布仅添加文件的事件
                    let _ = app.emit("add-files", &valid_files);
                    
                    // 确保窗口显示并且激活
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        });
}
