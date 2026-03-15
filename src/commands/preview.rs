use anyhow::{Context, Result};
use std::process::Command;

use crate::config::load_config;
use crate::util::{copy_dir_contents, find_aviutl2_data_dir, preview_dir};

pub fn run(
    profile: Option<String>,
    skip_start: bool,
    refresh: bool,
    args: Vec<String>,
) -> Result<()> {
    let config = load_config()?;
    let dev = config
        .development
        .as_ref()
        .context("preview.aviutl2_version を省略する場合は development 設定が必要です")?;
    let aviutl2_version = config
        .preview
        .aviutl2_version
        .as_deref()
        .unwrap_or(&dev.aviutl2_version);
    let install_dir = preview_dir(&config.preview)?;
    super::prepare::aviutl2_in(&install_dir, aviutl2_version)?;

    let profile = profile
        .or_else(|| config.preview.profile.clone())
        .or_else(|| config.release.profile.clone())
        .unwrap_or_else(|| "release".to_string());
    let include = config
        .preview
        .include
        .as_deref()
        .or(config.release.include.as_deref());
    super::develop::run_optional_commands(config.preview.prebuild.as_ref(), &config.build_group)?;
    let artifacts =
        super::develop::resolve_artifacts(&config, Some(&profile), include, refresh)?;
    let stage_dir = super::release::build_release_stage_from_artifacts(artifacts)?;
    let data_dir = find_aviutl2_data_dir(&install_dir)?;
    copy_dir_contents(&stage_dir, &data_dir, true)?;
    log::info!("プレビュー用に成果物を配置しました");
    super::develop::run_optional_commands(config.preview.postbuild.as_ref(), &config.build_group)?;

    if !skip_start {
        let aviutl_exe = data_dir.parent().unwrap_or(&data_dir).join("aviutl2.exe");
        if aviutl_exe.exists() {
            log::info!("AviUtl2 を起動します: {}", aviutl_exe.display());
            Command::new(aviutl_exe)
                .args(&args)
                .spawn()
                .with_context(|| "AviUtl2 の起動に失敗しました")?;
        } else {
            log::warn!("AviUtl2.exe が見つかりません: {}", aviutl_exe.display());
        }
    }
    Ok(())
}
