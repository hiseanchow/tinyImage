use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::settings::{AppSettings, OutputMode};

// 全局共享 Client：避免每次压缩都重建 TLS 上下文和连接池
// 在 Windows 上 TLS 初始化开销尤为明显，共享后可复用 keep-alive 连接
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

pub fn compress_image(file_path: &str, settings: &AppSettings) -> Result<CompressResult> {
    if settings.api_key.is_empty() {
        bail!("API Key 未配置，请在设置中填写 TinyPNG API Key");
    }

    let path = Path::new(file_path);
    if !path.exists() {
        bail!("文件不存在: {}", file_path);
    }

    let input_data = fs::read(path)?;
    let input_size = input_data.len() as u64;

    // 上传到 TinyPNG API
    let upload_resp = client()
        .post("https://api.tinify.com/shrink")
        .basic_auth("api", Some(&settings.api_key))
        .header("Content-Type", "application/octet-stream")
        .body(input_data)
        .send()?;

    let status = upload_resp.status();

    if !status.is_success() {
        let err: TinyPngError = upload_resp
            .json()
            .unwrap_or(TinyPngError { message: format!("HTTP 错误: {}", status) });
        bail!("TinyPNG 上传失败: {}", err.message);
    }

    let tinify_resp: TinyPngResponse = upload_resp.json()?;

    // 下载压缩后的文件，并检查 HTTP 状态
    let download_resp = client()
        .get(&tinify_resp.output.url)
        .basic_auth("api", Some(&settings.api_key))
        .send()?;

    if !download_resp.status().is_success() {
        bail!("下载压缩文件失败: HTTP {}", download_resp.status());
    }

    let compressed_data = download_resp.bytes()?;

    // 验证数据合理性：防止写入空文件或错误响应体（通常只有几十字节）
    if compressed_data.len() < 64 {
        bail!(
            "下载的压缩文件异常（{}字节），请重试",
            compressed_data.len()
        );
    }

    let output_size = compressed_data.len() as u64;

    // 确定输出路径
    let output_path = resolve_output_path(path, settings)?;

    // 确保父目录存在
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // 先写临时文件，成功后再原子替换，避免 overwrite 模式下中途失败损坏原图
    let tmp_path = output_path.with_extension("__tinytmp__");
    fs::write(&tmp_path, &compressed_data).map_err(|e| {
        anyhow!("写入临时文件失败: {}", e)
    })?;
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
