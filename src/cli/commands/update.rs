use anyhow::{anyhow, Result};
use clap::Args;

use crate::cli::commands::self_update::{
    download_release_binary, normalize_version, DownloadOptions,
};
use crate::cli::output;

#[derive(Args, Debug)]
pub struct UpdateArgs {
    /// GitHub 仓库 (owner/repo)
    #[arg(long, default_value = "DoBestone/cfai")]
    pub repo: String,

    /// 指定版本 (如 v0.1.0)
    #[arg(long)]
    pub version: Option<String>,

    /// 指定 Release 资源名
    #[arg(long)]
    pub asset: Option<String>,

    /// 指定要更新的二进制路径 (默认当前可执行文件)
    #[arg(long)]
    pub path: Option<std::path::PathBuf>,

    /// 强制更新
    #[arg(long)]
    pub force: bool,
}

impl UpdateArgs {
    pub async fn execute(&self) -> Result<()> {
        output::title("更新 CFAI");

        let target = match &self.path {
            Some(path) => path.clone(),
            None => std::env::current_exe().map_err(|e| anyhow!("获取当前可执行文件失败: {}", e))?,
        };

        output::info(&format!("目标路径: {}", target.display()));

        let downloaded = download_release_binary(&DownloadOptions {
            repo: self.repo.clone(),
            version: self.version.clone(),
            asset: self.asset.clone(),
        })
        .await?;

        let current_version = normalize_version(env!("CARGO_PKG_VERSION"));
        let latest_version = normalize_version(&downloaded.version);
        if !self.force && current_version == latest_version {
            output::info("已是最新版本，无需更新");
            return Ok(());
        }

        crate::cli::commands::self_update::install_binary(&target, &downloaded.binary, true)?;
        output::success(&format!(
            "更新完成: {} ({} -> {})",
            target.display(),
            current_version,
            latest_version
        ));

        Ok(())
    }
}
