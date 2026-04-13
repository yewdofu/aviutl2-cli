/**
 * aviutl2.toml の設定スキーマ定義。
 * JSON Schema は `nr typespec` で生成され、`src/schema.json` に反映されます。
 */
import "@typespec/json-schema";

using TypeSpec.JsonSchema;

@jsonSchema
model Config {
  /** プロジェクト設定 */
  project: Project;

  /** 成果物の定義 */
  artifacts?: Record<Artifact>;

  /** ビルドコマンドのグループ定義 */
  build_group?: Record<BuildCommand>;

  /** 開発用の設定 */
  development?: Development;

  /** プレビュー用の設定 */
  preview?: Preview;

  /** リリース用の設定 */
  release?: Release;

  /** AviUtl2 カタログの定義 */
  catalog?: Catalog;
}

model Project {
  /** プロジェクトID */
  id: string;

  /** プロジェクト名 */
  name: string;

  /** バージョン */
  version: string;
}

model Artifact {
  /** 成果物の有効/無効（デフォルトは true） */
  enabled?: boolean;

  /** 成果物のパス */
  source?: string;

  /** AviUtl2 の data 配下での配置先 */
  destination: string;

  /** ビルドコマンド */
  build?: BuildCommand;

  /** 配置方法（symlink / copy） */
  placement_method?: PlacementMethod;

  /** プロファイルごとの設定 */
  profiles?: Record<ArtifactProfile>;
}

model ArtifactProfile {
  /** 成果物の有効/無効（デフォルトは true） */
  enabled?: boolean;

  /** 成果物のパス */
  source?: string;

  /** ビルドコマンド */
  build?: BuildCommand;
}

/** 単一または複数のビルドコマンド */
alias BuildCommand = string | string[] | BuildGroupRef;

model BuildGroupRef {
  /** 利用する build_group のキー */
  group: string;
}

enum PlacementMethod {
  /** シンボリックリンク */
  symlink,

  /** コピー */
  copy,
}

model Development {
  /** AviUtl2 のバージョン */
  aviutl2_version: string;

  /** AviUtl2 のインストール先 */
  install_dir?: string;

  /** 使うプロファイル名（デフォルトは debug） */
  profile?: string;

  /** 事前ビルドコマンド */
  prebuild?: BuildCommand;

  /** 事後ビルドコマンド */
  postbuild?: BuildCommand;

  /** カタログからインストールするパッケージのリスト */
  catalog_dependencies?: CatalogDependency[];
}

model Preview {
  /** AviUtl2 のバージョン（省略時は development.aviutl2_version） */
  aviutl2_version?: string;

  /** AviUtl2 のインストール先 */
  install_dir?: string;

  /** 使うプロファイル名（デフォルトは release） */
  profile?: string;

  /** 含める成果物のリスト（省略時は release.include を使用） */
  include?: string[];

  /** 事前ビルドコマンド */
  prebuild?: BuildCommand;

  /** 事後ビルドコマンド */
  postbuild?: BuildCommand;

  /** カタログからインストールするパッケージのリスト */
  catalog_dependencies?: CatalogDependency[];
}

alias CatalogDependency = string | VerboseCatalogDependency;

model VerboseCatalogDependency {
  /** カタログからインストールするパッケージのID */
  id: string;

  // 将来のために予約...
}

model Release {
  /** 出力ディレクトリ */
  output_dir?: string;

  /** package.txt のテンプレート */
  package_template?: string;

  /** au2pkgでのID（package.ini の `id` に相当、デフォルトは `{id}`） */
  package_id?: string;

  /** au2pkgでのパッケージ名（package.ini の `name` に相当、デフォルトは `{name}`） */
  package_name?: string;

  /** au2pkgでの情報欄（package.ini の `information` に相当、デフォルトは `{name} v{version}`） */
  package_information?: string;

  /** アンインストール時に、作成したフォルダ内の別ファイルを削除するかどうか（package.ini の `uninstallSubFolderFile` に相当、デフォルトは false） */
  uninstall_subfolder_file?: boolean;

  /** zip の名前（`.au2pkg.zip` は自動付与、デフォルトは `{id}-v{version}.au2pkg.zip`） */
  zip_name?: string;

  /** 使うプロファイル名（デフォルトは release） */
  profile?: string;

  /** 含める成果物のリスト（省略時はすべて含める） */
  include?: string[];

  /** 事前ビルドコマンド */
  prebuild?: BuildCommand;

  /** 事後ビルドコマンド */
  postbuild?: BuildCommand;
}

model Catalog {
  /** ID */
  id: string;

  /** 説明文へのパス */
  description_path?: string;

  /** ライセンスへのパス */
  license_path?: string | CatalogLicensePath;

  /** ダウンロード元のGitHubリポジトリ */
  download_repo?: {
    owner: string;
    repo: string;
  };
}

alias LicenseType =
  | "MIT"
  | "Apache-2.0"
  | "BSD-2-Clause"
  | "BSD-3-Clause"
  | "CC0-1.0"
  | "GPL-2.0"
  | "GPL-3.0"
  | "Unlicense"
  | "custom";

model CatalogLicensePath {
  /** ライセンス種別 */
  type: LicenseType;

  /** ライセンス本文へのパス */
  path: string;
}
