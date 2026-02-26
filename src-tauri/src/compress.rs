use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter};

use crate::settings::{AppSettings, OutputMode};

// 全局共享 Client：避免每次压缩都重建 TLS 上下文和连接池
static HTTP_CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();

fn client() -> &'static reqwest::blocking::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::blocking::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(30))
            .timeout(std::time::Duration::from_secs(120))
            .pool_max_idle_per_host(4)
            .build()
            .expect("构建 HTTP Client 失败")
    })
}

// ── 数据结构 ───────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct CompressResult {
    pub input_size: u64,
    pub output_size: u64,
    pub output_path: String,
}

#[derive(Debug, Deserialize)]
struct TinyPngOutput {
    url: String,
    size: u64,
}

#[derive(Debug, Deserialize)]
struct TinyPngResponse {
    output: TinyPngOutput,
    #[allow(dead_code)]
    input: TinyPngInput,
}

#[derive(Debug, Deserialize)]
struct TinyPngInput {
    #[allow(dead_code)]
    size: u64,
}

#[derive(Debug, Deserialize)]
struct TinyPngError {
    message: String,
}

// ── 进度事件 ───────────────────────────────────────────────────

#[derive(Serialize, Clone)]
struct ProgressEvent<'a> {
    path: &'a str,
    percent: u8,
    phase: &'a str,
}

fn emit_progress(app: &AppHandle, path: &str, percent: u8, phase: &str) {
    app.emit("compress-progress", &ProgressEvent { path, percent, phase }).ok();
}

// ── 上传进度 Reader ────────────────────────────────────────────
// 包装内存数据，在 reqwest 读取 body 时实时发送上传百分比

struct UploadProgress {
    cursor: std::io::Cursor<Vec<u8>>,
    total: u64,
    app: AppHandle,
    path: String,
    last_pct: u8,
}

impl Read for UploadProgress {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.cursor.read(buf)?;
        if self.total > 0 {
            let pos = self.cursor.position();
            // 上传占总进度的 0-40%
            let pct = (pos as f64 / self.total as f64 * 40.0) as u8;
            if pct > self.last_pct {
                self.last_pct = pct;
                emit_progress(&self.app, &self.path, pct, "uploading");
            }
        }
        Ok(n)
    }
}

// ── 压缩入口 ───────────────────────────────────────────────────

pub fn compress_image(
    file_path: &str,
    settings: &AppSettings,
    app: &AppHandle,
) -> Result<CompressResult> {
    if settings.api_key.is_empty() {
        bail!("API Key 未配置，请在设置中填写 TinyPNG API Key");
    }

    let path = Path::new(file_path);
    if !path.exists() {
        bail!("文件不存在: {}", file_path);
    }

    let input_data = fs::read(path)?;
    let input_size = input_data.len() as u64;

    // ── 上传阶段 (0-40%) ────────────────────────────────────────
    emit_progress(app, file_path, 0, "uploading");

    let body = reqwest::blocking::Body::sized(
        UploadProgress {
            total: input_size,
            cursor: std::io::Cursor::new(input_data),
            app: app.clone(),
            path: file_path.to_string(),
            last_pct: 0,
        },
        input_size,
    );

    let upload_resp = client()
        .post("https://api.tinify.com/shrink")
        .basic_auth("api", Some(&settings.api_key))
        .header("Content-Type", "application/octet-stream")
        .body(body)
        .send()?;

    let status = upload_resp.status();
    if !status.is_success() {
        let err: TinyPngError = upload_resp
            .json()
            .unwrap_or(TinyPngError { message: format!("HTTP 错误: {}", status) });
        bail!("TinyPNG 上传失败: {}", err.message);
    }

    // ── 处理阶段 (40-50%)：等待 TinyPNG 服务端压缩 ─────────────
    emit_progress(app, file_path, 40, "processing");
    let tinify_resp: TinyPngResponse = upload_resp.json()?;

    // ── 下载阶段 (50-99%)：流式下载，实时更新百分比 ─────────────
    emit_progress(app, file_path, 50, "downloading");

    let mut download_resp = client()
        .get(&tinify_resp.output.url)
        .basic_auth("api", Some(&settings.api_key))
        .send()?;

    if !download_resp.status().is_success() {
        bail!("下载压缩文件失败: HTTP {}", download_resp.status());
    }

    let total_bytes = tinify_resp.output.size;
    let mut compressed_data = Vec::with_capacity(total_bytes as usize);
    let mut downloaded = 0u64;
    let mut last_pct = 50u8;
    let mut buf = [0u8; 16_384];

    loop {
        let n = download_resp
            .read(&mut buf)
            .map_err(|e| anyhow!("下载读取失败: {}", e))?;
        if n == 0 {
            break;
        }
        compressed_data.extend_from_slice(&buf[..n]);
        downloaded += n as u64;
        if total_bytes > 0 {
            // 下载占总进度的 50-99%，留 1% 给写文件
            let pct = (50.0 + downloaded as f64 / total_bytes as f64 * 49.0) as u8;
            if pct > last_pct {
                last_pct = pct;
                emit_progress(app, file_path, pct, "downloading");
            }
        }
    }

    if compressed_data.len() < 64 {
        bail!(
            "下载的压缩文件异常（{}字节），请重试",
            compressed_data.len()
        );
    }

    let output_size = compressed_data.len() as u64;
    let output_path = resolve_output_path(path, settings)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // 先写临时文件再原子替换，避免 overwrite 模式下失败时损坏原图
    let tmp_path = output_path.with_extension("__tinytmp__");
    fs::write(&tmp_path, &compressed_data)
        .map_err(|e| anyhow!("写入临时文件失败: {}", e))?;
    fs::rename(&tmp_path, &output_path).map_err(|e| {
        let _ = fs::remove_file(&tmp_path);
        anyhow!("移动文件失败: {}", e)
    })?;

    Ok(CompressResult {
        input_size,
        output_size,
        output_path: output_path.to_string_lossy().into_owned(),
    })
}

fn resolve_output_path(input: &Path, settings: &AppSettings) -> Result<PathBuf> {
    match settings.output_mode {
        OutputMode::Overwrite => Ok(input.to_path_buf()),

        OutputMode::Alongside => {
            let stem = input
                .file_stem()
                .ok_or_else(|| anyhow!("无法获取文件名"))?
                .to_string_lossy();
            let ext = input
                .extension()
                .map(|e| e.to_string_lossy().into_owned())
                .unwrap_or_default();
            let new_name = if ext.is_empty() {
                format!("{}-tiny", stem)
            } else {
                format!("{}-tiny.{}", stem, ext)
            };
            let dir = input.parent().ok_or_else(|| anyhow!("无法获取父目录"))?;
            Ok(dir.join(new_name))
        }

        OutputMode::Directory => {
            if settings.output_directory.is_empty() {
                bail!("请先在设置中指定输出目录");
            }
            let dir = Path::new(&settings.output_directory);
            let filename = input
                .file_name()
                .ok_or_else(|| anyhow!("无法获取文件名"))?;
            Ok(dir.join(filename))
        }
    }
}
