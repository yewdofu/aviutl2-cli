use assert_cmd::Command;
use fs_err as fs;
use predicates::str::contains;
use std::{io::Read, path::Path};
use tempfile::tempdir;

fn write_file(path: &Path, content: &str) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)
}

#[test]
fn e2e_init_creates_config_and_updates_gitignore() -> anyhow::Result<()> {
    let temp = tempdir()?;
    let project_dir = temp.path().join("my_aviutl2_project");
    fs::create_dir_all(&project_dir)?;

    let gitignore_path = project_dir.join(".gitignore");
    write_file(&gitignore_path, "target\n")?;

    Command::new(assert_cmd::cargo::cargo_bin!("au2"))
        .current_dir(&project_dir)
        .arg("init")
        .assert()
        .success();

    let config_path = project_dir.join("aviutl2.toml");
    let config = fs::read_to_string(&config_path)?;
    assert!(config.contains("name = \"my_aviutl2_project\""));

    let gitignore = fs::read_to_string(&gitignore_path)?;
    assert!(gitignore.contains("/.aviutl2-cli"));
    assert!(gitignore.contains("/release"));

    Ok(())
}

#[test]
fn e2e_init_fails_when_config_exists() -> anyhow::Result<()> {
    let temp = tempdir()?;
    let project_dir = temp.path().join("existing_project");
    fs::create_dir_all(&project_dir)?;

    let config_path = project_dir.join("aviutl2.toml");
    write_file(
        &config_path,
        r#"[project]
            id = "sevenc-nanashi.aviutl2-cli.existing-project"
            name = "existing_project"
            version = "0.1.0"
            "#,
    )?;

    Command::new(assert_cmd::cargo::cargo_bin!("au2"))
        .current_dir(&project_dir)
        .arg("init")
        .assert()
        .failure()
        .stderr(contains("aviutl2.toml は既に存在します"));

    Ok(())
}

#[test]
fn e2e_prepare_schema_writes_schema_file() -> anyhow::Result<()> {
    let temp = tempdir()?;
    let project_dir = temp.path().join("schema_project");
    fs::create_dir_all(&project_dir)?;

    let config_path = project_dir.join("aviutl2.toml");
    write_file(
        &config_path,
        dedent::dedent!(
            r#"[project]
               id = "sevenc-nanashi.aviutl2-cli.schema-project"
               name = "schema"
               version = "0.1.0"

               [artifacts]

               [development]
               aviutl2_version = "latest"
               install_dir = "devdir"
           "#
        ),
    )?;

    Command::new(assert_cmd::cargo::cargo_bin!("au2"))
        .current_dir(&project_dir)
        .arg("prepare:schema")
        .assert()
        .success();

    let schema_path = project_dir.join(".aviutl2-cli").join("aviutl2.schema.json");
    assert!(schema_path.exists());

    Ok(())
}

#[test]
fn e2e_release_writes_catalog_json() -> anyhow::Result<()> {
    let temp = tempdir()?;
    let project_dir = temp.path().join("catalog_project");
    fs::create_dir_all(&project_dir)?;
    fs::create_dir_all(project_dir.join("dist"))?;
    write_file(
        &project_dir.join("dist").join("plugin.auf"),
        "dummy-binary-content",
    )?;

    let config_path = project_dir.join("aviutl2.toml");
    write_file(
        &config_path,
        dedent::dedent!(
            r#"[project]
               id = "sevenc-nanashi.aviutl2-cli.catalog-project"
               name = "catalog-project"
               version = "1.2.3"

               [artifacts.plugin]
               source = "dist/plugin.auf"
               destination = "Plugin/plugin.auf"
               placement_method = "copy"

               [catalog]
               id = "my-plugin"
               name = "My Plugin"
               type = "filter"
               author = "nanashi"
               summary = "summary"
               homepage = "https://example.com"
               description = "desc"

               [catalog.license]
               type = "CC0-1.0"

               [catalog.download_source]
               type = "github"
               owner = "sevenc-nanashi"
               repo = "aviutl2-cli"

               [release]
               "#
        ),
    )?;

    Command::new(assert_cmd::cargo::cargo_bin!("au2"))
        .current_dir(&project_dir)
        .arg("release")
        .assert()
        .success();

    let catalog_json_path = project_dir.join("release").join("catalog.json");
    assert!(catalog_json_path.exists());
    let content = fs::read_to_string(&catalog_json_path)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;
    assert_eq!(json[0]["id"], "my-plugin");
    assert_eq!(
        json[0]["installer"]["source"]["github"]["repo"],
        "aviutl2-cli"
    );
    assert_eq!(
        json[0]["installer"]["source"]["github"]["pattern"],
        r"^sevenc-nanashi\.aviutl2-cli\.catalog-project-v[^/]+\.au2pkg\.zip$"
    );
    assert_eq!(json[0]["installer"]["install"][0]["action"], "download");
    assert_eq!(json[0]["installer"]["install"][1]["action"], "extract");
    assert_eq!(json[0]["installer"]["install"][2]["action"], "copy");
    assert_eq!(json[0]["version"][0]["version"], "1.2.3");
    assert_eq!(
        json[0]["version"][0]["file"][0]["path"],
        "{pluginsDir}/plugin.auf"
    );
    assert!(
        json[0]["version"][0]["file"][0]["XXH3_128"]
            .as_str()
            .unwrap_or_default()
            .len()
            == 32
    );

    Ok(())
}

#[test]
fn e2e_release_creates_package_information() -> anyhow::Result<()> {
    let temp = tempdir()?;
    let project_dir = temp.path().join("package_info_project");
    fs::create_dir_all(&project_dir)?;
    fs::create_dir_all(project_dir.join("dist"))?;
    write_file(
        &project_dir.join("dist").join("plugin.auf"),
        "dummy-binary-content",
    )?;

    let config_path = project_dir.join("aviutl2.toml");
    write_file(
        &config_path,
        dedent::dedent!(
            r#"[project]
               id = "sevenc-nanashi.aviutl2-cli.package-info-project"
               name = "package-info-project"
               version = "0.1.0"

               [artifacts.plugin]
               source = "dist/plugin.auf"
               destination = "Plugin/plugin.auf"
               placement_method = "copy"

               [release]
               package_template = "./package_template.txt"
           "#
        ),
    )?;

    write_file(
        &project_dir.join("package_template.txt"),
        "This is a package for {name} version {version}.",
    )?;

    Command::new(assert_cmd::cargo::cargo_bin!("au2"))
        .current_dir(&project_dir)
        .arg("release")
        .assert()
        .success();

    let zip_path = project_dir
        .join("release")
        .join("sevenc-nanashi.aviutl2-cli.package-info-project-v0.1.0.au2pkg.zip");
    assert!(zip_path.exists());
    let mut zip = zip::ZipArchive::new(std::fs::File::open(zip_path)?)?;

    let mut file = zip.by_name("package.txt")?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    assert_eq!(
        content,
        "This is a package for package-info-project version 0.1.0."
    );
    drop(file);

    let mut ini_file = zip.by_name("package.ini")?;
    let mut ini_content = String::new();
    ini_file.read_to_string(&mut ini_content)?;
    let mut ini = configparser::ini::Ini::new();
    ini.read(ini_content).unwrap();
    assert_eq!(
        ini.get_map_ref()
            .get("package")
            .and_then(|s| s.get("id").cloned())
            .flatten(),
        Some("sevenc-nanashi.aviutl2-cli.package-info-project".to_string())
    );
    assert_eq!(
        ini.get_map_ref()
            .get("package")
            .and_then(|s| s.get("information").cloned())
            .flatten(),
        Some("package-info-project v0.1.0".to_string())
    );

    Ok(())
}
