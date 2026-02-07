use anyhow::{anyhow, Context, Result};
use flate2::read::GzDecoder;
use reqwest::Client;
use serde::Deserialize;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use zip::ZipArchive;

#[derive(Debug, Clone)]
pub struct DownloadOptions {
    pub repo: String,
    pub version: Option<String>,
    pub asset: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DownloadedRelease {
    pub version: String,
    pub asset_name: String,
    pub binary: Vec<u8>,
}

#[derive(Deserialize)]
struct ReleaseResponse {
    tag_name: String,
    assets: Vec<ReleaseAsset>,
}

#[derive(Deserialize, Clone)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

pub async fn download_release_binary(options: &DownloadOptions) -> Result<DownloadedRelease> {
    let client = Client::new();
    let api_url = build_release_api_url(&options.repo, options.version.as_deref());

    let release: ReleaseResponse = client
        .get(api_url)
        .header("User-Agent", "cfai")
        .send()
        .await
        .context("请求 GitHub Release 失败")?
        .error_for_status()
        .context("GitHub Release 返回错误")?
        .json()
        .await
        .context("解析 GitHub Release 响应失败")?;

    let asset = match &options.asset {
        Some(name) => release
            .assets
            .iter()
            .find(|a| a.name == *name)
            .cloned()
            .ok_or_else(|| anyhow!("未找到指定的资源: {}", name))?,
        None => select_best_asset(&release.assets)?,
    };

    let bytes = client
        .get(&asset.browser_download_url)
        .header("User-Agent", "cfai")
        .send()
        .await
        .context("下载二进制失败")?
        .error_for_status()
        .context("下载二进制返回错误")?
        .bytes()
        .await
        .context("读取二进制内容失败")?
        .to_vec();

    let binary = extract_binary(&asset.name, &bytes)?;

    Ok(DownloadedRelease {
        version: release.tag_name,
        asset_name: asset.name,
        binary,
    })
}

pub fn install_binary(target_path: &Path, binary: &[u8], force: bool) -> Result<()> {
    if target_path.exists() && !force {
        return Err(anyhow!(
            "目标已存在: {} (使用 --force 覆盖)",
            target_path.display()
        ));
    }

    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).context("创建目标目录失败")?;
        let mut temp_file = NamedTempFile::new_in(parent).context("创建临时文件失败")?;
        temp_file
            .write_all(binary)
            .context("写入二进制失败")?;
        temp_file.flush().context("刷新写入失败")?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = temp_file.as_file().metadata()?.permissions();
            perms.set_mode(0o755);
            temp_file.as_file().set_permissions(perms)?;
        }

        if target_path.exists() {
            fs::remove_file(target_path).context("移除旧版本失败")?;
        }

        temp_file
            .persist(target_path)
            .map_err(|e| anyhow!("替换二进制失败: {}", e))?;
    }

    Ok(())
}

pub fn default_install_path() -> Result<PathBuf> {
    let binary_name = binary_name();
    let preferred = PathBuf::from("/usr/local/bin");
    if is_writable_dir(&preferred) {
        return Ok(preferred.join(binary_name));
    }

    let home = dirs::home_dir().ok_or_else(|| anyhow!("无法获取用户目录"))?;
    let local_bin = home.join(".local").join("bin");
    Ok(local_bin.join(binary_name))
}

pub fn resolve_install_path(path: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = path {
        if path.is_dir() {
            return Ok(path.join(binary_name()));
        }

        if let Some(file_name) = path.file_name().and_then(|v| v.to_str()) {
            if file_name == binary_name() || file_name.ends_with(".exe") {
                return Ok(path);
            }
        }

        return Ok(path.join(binary_name()));
    }

    default_install_path()
}

pub fn binary_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "cfai.exe"
    } else {
        "cfai"
    }
}

pub fn normalize_version(tag: &str) -> String {
    tag.trim_start_matches('v').to_string()
}

fn build_release_api_url(repo: &str, version: Option<&str>) -> String {
    match version {
        Some(tag) => format!("https://api.github.com/repos/{}/releases/tags/{}", repo, tag),
        None => format!("https://api.github.com/repos/{}/releases/latest", repo),
    }
}

fn select_best_asset(assets: &[ReleaseAsset]) -> Result<ReleaseAsset> {
    let (os_patterns, arch_patterns) = detect_patterns();
    let mut candidates: Vec<ReleaseAsset> = assets
        .iter()
        .filter(|asset| {
            let name = asset.name.to_lowercase();
            name.contains("cfai")
                && os_patterns.iter().any(|p| name.contains(p))
                && arch_patterns.iter().any(|p| name.contains(p))
        })
        .cloned()
        .collect();

    if candidates.is_empty() {
        candidates = assets
            .iter()
            .filter(|asset| asset.name.to_lowercase().contains("cfai"))
            .cloned()
            .collect();
    }

    candidates
        .into_iter()
        .max_by_key(|asset| asset.size)
        .ok_or_else(|| anyhow!("未找到可用的二进制资源"))
}

fn detect_patterns() -> (Vec<&'static str>, Vec<&'static str>) {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let os_patterns = match os {
        "macos" => vec!["apple-darwin", "darwin", "macos", "osx"],
        "windows" => vec!["windows", "win"],
        _ => vec!["linux", "gnu", "musl"],
    };

    let arch_patterns = match arch {
        "aarch64" => vec!["aarch64", "arm64"],
        "x86_64" => vec!["x86_64", "amd64"],
        _ => vec![arch],
    };

    (os_patterns, arch_patterns)
}

fn extract_binary(asset_name: &str, bytes: &[u8]) -> Result<Vec<u8>> {
    let lower = asset_name.to_lowercase();
    if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
        let decoder = GzDecoder::new(bytes);
        let mut archive = tar::Archive::new(decoder);
        return extract_from_tar(&mut archive);
    }

    if lower.ends_with(".zip") {
        let cursor = std::io::Cursor::new(bytes);
        let mut archive = ZipArchive::new(cursor).context("读取 zip 失败")?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).context("读取 zip 条目失败")?;
            let name = file.name().to_lowercase();
            // 匹配 cfai 可执行文件
            let is_binary = name.ends_with(binary_name())
                || (name.contains("cfai")
                    && !name.ends_with(".md")
                    && !name.ends_with(".txt")
                    && !name.contains('.'));
            if is_binary {
                let mut out = Vec::new();
                file.read_to_end(&mut out)
                    .context("读取 zip 二进制失败")?;
                return Ok(out);
            }
        }

        return Err(anyhow!("zip 中未找到可执行文件"));
    }

    Ok(bytes.to_vec())
}

fn extract_from_tar<R: Read>(archive: &mut tar::Archive<R>) -> Result<Vec<u8>> {
    for entry in archive.entries().context("读取 tar 失败")? {
        let mut entry = entry.context("读取 tar 条目失败")?;
        let path = entry
            .path()
            .context("读取 tar 路径失败")?
            .to_string_lossy()
            .to_string();
        let path_lower = path.to_lowercase();

        // 匹配 cfai 可执行文件（包括 cfai, cfai-darwin-arm64 等格式）
        let is_binary = path_lower.ends_with(binary_name())
            || (path_lower.contains("cfai")
                && !path_lower.ends_with(".md")
                && !path_lower.ends_with(".txt")
                && !path_lower.contains('.'));

        if is_binary {
            let mut out = Vec::new();
            entry.read_to_end(&mut out).context("读取 tar 二进制失败")?;
            return Ok(out);
        }
    }

    Err(anyhow!("压缩包中未找到可执行文件"))
}

fn is_writable_dir(path: &Path) -> bool {
    if !path.exists() || !path.is_dir() {
        return false;
    }

    let test_path = path.join(".cfai_write_test");
    match fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(&test_path)
    {
        Ok(_) => {
            let _ = fs::remove_file(test_path);
            true
        }
        Err(_) => false,
    }
}
