use anyhow::{Result, bail};
use fs_err as fs;
use std::path::PathBuf;
use strum::IntoEnumIterator;

#[derive(Debug, Clone)]
struct InitConfig {
    project_name: String,
    project_type: ProjectType,
    i18n: bool,
}

#[derive(Debug, Clone, Copy)]
enum ProjectType {
    PluginCpp { plugin_type: PluginType },
    PluginRust { plugin_type: PluginType },
    Script { script_type: ScriptType },
}

#[derive(Debug, Clone, Copy, strum::EnumString, strum::Display, strum::EnumIter)]
enum PluginType {
    #[strum(serialize = "入力プラグイン")]
    Input,
    #[strum(serialize = "出力プラグイン")]
    Output,
    #[strum(serialize = "フィルタプラグイン")]
    Filter,
    #[strum(serialize = "スクリプトモジュール")]
    ScriptModule,
    #[strum(serialize = "汎用プラグイン")]
    Common,
}
impl PluginType {
    pub fn suffix(&self) -> &'static str {
        match self {
            PluginType::Input => "aui2",
            PluginType::Output => "auo2",
            PluginType::Filter => "auf2",
            PluginType::ScriptModule => "mod2",
            PluginType::Common => "aux2",
        }
    }
}

#[derive(Debug, Clone, Copy, strum::EnumString, strum::Display, strum::EnumIter)]
enum ScriptType {
    #[strum(serialize = "アニメーション効果")]
    Anm,
    #[strum(serialize = "カスタムオブジェクト")]
    Obj,
    #[strum(serialize = "カメラ効果")]
    Cam,
    #[strum(serialize = "シーンチェンジ")]
    Scn,
    #[strum(serialize = "トラックバー移動方法")]
    Tra,
}
impl ScriptType {
    pub fn suffix(&self) -> &'static str {
        match self {
            ScriptType::Anm => "anm2",
            ScriptType::Obj => "obj2",
            ScriptType::Cam => "cam2",
            ScriptType::Scn => "scn2",
            ScriptType::Tra => "tra2",
        }
    }
}

pub fn run() -> Result<()> {
    let path = PathBuf::from("aviutl2.toml");
    if path.exists() {
        bail!("aviutl2.toml は既に存在します");
    }
    let current_dir = std::env::current_dir()?;

    let init_config = if dialoguer::console::user_attended() {
        ask_init_config(&current_dir)?
    } else {
        let project_name = slugify(
            current_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("my_aviutl2_project"),
        );
        InitConfig {
            project_name,
            project_type: ProjectType::PluginCpp {
                plugin_type: PluginType::Common,
            },
            i18n: false,
        }
    };
    let template = init_template(&init_config);
    fs::write(&path, template)?;
    log::info!("aviutl2.toml を作成しました");

    let gitignore_path = PathBuf::from(".gitignore");
    if gitignore_path.exists() {
        let mut content = fs::read_to_string(&gitignore_path)?;
        content.push_str("\n# AviUtl2 CLI\n/.aviutl2-cli\n/release\n");
        fs::write(&gitignore_path, content)?;
        log::info!(".gitignore を更新しました");
    } else {
        fs::write(&gitignore_path, "# AviUtl2 CLI\n/.aviutl2-cli\n/release\n")?;
        log::info!(".gitignore を作成しました");
    }
    Ok(())
}

fn ask_init_config(current_dir: &std::path::Path) -> Result<InitConfig> {
    let project_name = dialoguer::Input::new()
        .with_prompt("プロジェクト名を入力してください")
        .default(slugify(
            current_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("my_aviutl2_project"),
        ))
        .interact_text()?;

    let project_type = dialoguer::Select::new()
        .with_prompt("プロジェクトの種類を選択してください")
        .items(["プラグイン（C++）", "プラグイン（Rust）", "スクリプト"])
        .interact()?;

    let project_type = match project_type {
        0 | 1 => {
            let plugin_type = dialoguer::Select::new()
                .with_prompt("プラグインの種類を選択してください")
                .items(
                    PluginType::iter()
                        .map(|t| t.to_string())
                        .collect::<Vec<_>>(),
                )
                .interact()?;
            let plugin_type = PluginType::iter().nth(plugin_type).unwrap();
            if project_type == 0 {
                ProjectType::PluginCpp { plugin_type }
            } else {
                ProjectType::PluginRust { plugin_type }
            }
        }
        2 => {
            let script_type = dialoguer::Select::new()
                .with_prompt("スクリプトの種類を選択してください")
                .items(
                    ScriptType::iter()
                        .map(|t| t.to_string())
                        .collect::<Vec<_>>(),
                )
                .interact()?;
            let script_type = ScriptType::iter().nth(script_type).unwrap();
            ProjectType::Script { script_type }
        }
        _ => unreachable!(),
    };

    let i18n = dialoguer::Confirm::new()
        .with_prompt(format!("English.{project_name}.aul2 を使用しますか？"))
        .interact()?;

    Ok(InitConfig {
        project_name,
        project_type,
        i18n,
    })
}

fn init_template(config: &InitConfig) -> String {
    let mut lines = Vec::new();
    lines.push("#:schema ./.aviutl2-cli/aviutl2.schema.json".to_string());
    lines.push("# 設定ファイルについては https://github.com/sevenc-nanashi/aviutl2-cli を参照してください。".to_string());
    lines.push("[project]".to_string());
    lines.push(format!("name = \"{}\"", slugify(&config.project_name)));
    lines.push("version = \"0.1.0\"".to_string());
    lines.push("".to_string());
    lines.push("[development]".to_string());
    lines.push("aviutl2_version = \"latest\"".to_string());

    let project_slug = slugify(&config.project_name);

    if config.i18n {
        lines.push("".to_string());
        lines.push(format!("[artifacts.English-{project_slug}-aul2]"));
        lines.push(format!("destination = \"Language/English.{project_slug}.aul2\"",));
        lines.push(format!("source = \"./i18n/English.{project_slug}.aul2\"",));
    }
    match config.project_type {
        ProjectType::PluginCpp { plugin_type } => {
            lines.push("".to_string());
            lines.push(format!(
                "[artifacts.{}-{}]",
                project_slug,
                plugin_type.suffix()
            ));
            lines.push(format!(
                "destination = \"Plugin/{}.{}\"",
                project_slug,
                plugin_type.suffix()
            ));
            lines.push("".to_string());
            lines.push(format!(
                "[artifacts.{}-{}.profiles.debug]",
                project_slug,
                plugin_type.suffix()
            ));
            lines.push("build = [\"cmake -S . -B build -DCMAKE_BUILD_TYPE=Debug\", \"cmake --build build --config Debug\"]".to_string());
            lines.push(format!("source = \"build/Debug/{}.dll\"", project_slug));
            lines.push("".to_string());
            lines.push(format!(
                "[artifacts.{}-{}.profiles.release]",
                project_slug,
                plugin_type.suffix()
            ));
            lines.push("build = [\"cmake -S . -B build -DCMAKE_BUILD_TYPE=Release\", \"cmake --build build --config Release\"]".to_string());
            lines.push(format!("source = \"build/Release/{}.dll\"", project_slug));
        }
        ProjectType::PluginRust { plugin_type } => {
            lines.push("".to_string());
            lines.push(format!(
                "[artifacts.{}-{}]",
                project_slug,
                plugin_type.suffix()
            ));
            lines.push(format!(
                "destination = \"Plugin/{}.{}\"",
                project_slug,
                plugin_type.suffix()
            ));
            lines.push("".to_string());
            lines.push(format!(
                "[artifacts.{}-{}.profiles.debug]",
                project_slug,
                plugin_type.suffix()
            ));
            lines.push("build = \"cargo build\"".to_string());
            lines.push(format!("source = \"target/debug/{}.dll\"", project_slug));
            lines.push("".to_string());
            lines.push(format!(
                "[artifacts.{}-{}.profiles.release]",
                project_slug,
                plugin_type.suffix()
            ));
            lines.push("build = \"cargo build --release\"".to_string());
            lines.push(format!("source = \"target/release/{}.dll\"", project_slug));
        }
        ProjectType::Script { script_type } => {
            lines.push("".to_string());
            lines.push(format!(
                "[artifacts.{}-{}]",
                project_slug,
                script_type.suffix()
            ));
            lines.push(format!(
                "destination = \"Script/{}.{}\"",
                project_slug,
                script_type.suffix()
            ));
            lines.push(format!("source = \"src/{}.lua\"", project_slug));
        }
    }

    lines.join("\n")
}

fn slugify(s: &str) -> String {
    let mut slug = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
        } else if !slug.ends_with('_') {
            slug.push('_');
        }
    }
    slug.trim_matches('_').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("My Project"), "my_project");
        assert_eq!(slugify("Hello, World!"), "hello_world");
        assert_eq!(slugify("AviUtl2 Plugin"), "aviutl2_plugin");
        assert_eq!(slugify("  Leading and trailing  "), "leading_and_trailing");
        assert_eq!(slugify("Multiple   Spaces"), "multiple_spaces");
        assert_eq!(
            slugify("Special-Characters!@#$%^&*()"),
            "special_characters"
        );
        assert_eq!(slugify("Already_Slugified"), "already_slugified");
        assert_eq!(slugify(""), "");
    }

    #[test]
    fn test_init_template() {
        let config = InitConfig {
            project_name: "My Plugin".to_string(),
            project_type: ProjectType::PluginCpp {
                plugin_type: PluginType::Filter,
            },
            i18n: true,
        };
        let template = init_template(&config);
        insta::assert_snapshot!(template);
    }
}
