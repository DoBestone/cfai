use anyhow::Result;
use clap::Args;

use crate::cli::commands::self_update::{
    download_release_binary, resolve_install_path, DownloadOptions,
};
use crate::cli::output;

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// GitHub 仓库 (owner/repo)
    #[arg(long, default_value = "DoBestone/cfai")]
    pub repo: String,

    /// 指定版本 (如 v0.1.0)
    #[arg(long)]
    pub version: Option<String>,

    /// 指定 Release 资源名
    #[arg(long)]
    pub asset: Option<String>,

    /// 安装路径 (目录或完整文件路径)
    #[arg(long)]
    pub path: Option<std::path::PathBuf>,

    /// 覆盖已存在的二进制
    #[arg(long)]
    pub force: bool,
}

impl InstallArgs {
    pub async fn execute(&self) -> Result<()> {
        output::title("安装 CFAI");

        let target = resolve_install_path(self.path.clone())?;
        output::info(&format!("目标路径: {}", target.display()));

        let downloaded = download_release_binary(&DownloadOptions {
            repo: self.repo.clone(),
            version: self.version.clone(),
            asset: self.asset.clone(),
        })
        .await?;

        crate::cli::commands::self_update::install_binary(&target, &downloaded.binary, self.force)?;

        output::success(&format!(
            "安装完成: {} (版本 {})",
            target.display(),
            downloaded.version
        ));

        Ok(())
    }
}
