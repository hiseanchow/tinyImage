use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::settings::{AppSettings, OutputMode};

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
    let client = reqwest::blocking::Client::new();
    let upload_resp = client
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

    // 下载压缩后的文件
    let compressed_data = client
        .get(&tinify_resp.output.url)
        .basic_auth("api", Some(&settings.api_key))
        .send()?
        .bytes()?;

    let output_size = tinify_resp.output.size;

    // 确定输出路径
    let output_path = resolve_output_path(path, settings)?;

    // 确保父目录存在
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&output_path, &compressed_data)?;

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
