use crate::types::{Config, DetectorError};
use std::fs;
use std::path::Path;

/// 設定ファイルを読み込む
pub fn load_config(config_path: Option<&str>) -> Result<Config, DetectorError> {
    let config_path = config_path
        .map(|p| p.to_string())
        .or_else(|| find_config_file())
        .unwrap_or_else(|| "".to_string());

    if config_path.is_empty() || !Path::new(&config_path).exists() {
        return Ok(Config::default());
    }

    let content = fs::read_to_string(&config_path)?;

    if config_path.ends_with(".json") {
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    } else {
        // JavaScript設定ファイルの場合は、基本的なJSONパーサーを使用
        // 実際の実装では、より高度なJSパーサーを使用することも可能
        Err(DetectorError::Config {
            message:
                "JavaScript config files are not supported in Rust version. Please use JSON format."
                    .to_string(),
        })
    }
}

/// 標準的な設定ファイルを探す
fn find_config_file() -> Option<String> {
    let config_file = "tuc.config.json";

    if Path::new(config_file).exists() {
        Some(config_file.to_string())
    } else {
        None
    }
}

/// モノレポ構造を検出してパスを調整
pub fn adjust_config_for_monorepo(mut config: Config) -> Result<Config, DetectorError> {
    let package_json_path = "package.json";

    if !Path::new(package_json_path).exists() {
        return Ok(config);
    }

    let package_json_content = fs::read_to_string(package_json_path)?;
    let package_json: serde_json::Value = serde_json::from_str(&package_json_content)?;

    // モノレポ構造を検出
    let is_monorepo =
        package_json.get("workspaces").is_some() || Path::new("pnpm-workspace.yaml").exists();

    if is_monorepo {
        let apps_dir = Path::new("apps");
        if apps_dir.exists() {
            let mut new_search_dirs = Vec::new();

            // apps/* ディレクトリを検索
            for entry in fs::read_dir(apps_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    let app_name = entry.file_name().to_string_lossy().to_string();

                    for dir in &config.search_dirs {
                        new_search_dirs.push(format!("apps/{}/{}", app_name, dir));
                    }
                }
            }

            if !new_search_dirs.is_empty() {
                config.search_dirs = new_search_dirs;
            }
        }
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_load_default_config() {
        let config = load_config(None).unwrap();
        assert!(!config.search_dirs.is_empty());
        assert!(!config.exclude_patterns.is_empty());
        assert_eq!(config.search_dirs, vec!["src"]);
    }

    #[test]
    fn test_load_json_config() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.json");

        let config_content = r#"
        {
            "search_dirs": ["custom/src"]
        }
        "#;

        let mut file = File::create(&config_path).unwrap();
        file.write_all(config_content.as_bytes()).unwrap();

        let config = load_config(Some(config_path.to_str().unwrap())).unwrap();
        assert_eq!(config.search_dirs, vec!["custom/src"]);
        // デフォルトのexclude_patternsが使用される
        assert!(config
            .exclude_patterns
            .contains(&"node_modules".to_string()));
    }

    #[test]
    fn test_monorepo_adjustment() {
        let mut config = Config::default();
        config.search_dirs = vec!["src".to_string()];

        // モノレポでない場合はそのまま
        let adjusted = adjust_config_for_monorepo(config.clone()).unwrap();
        assert_eq!(adjusted.search_dirs, vec!["src"]);
    }
}
