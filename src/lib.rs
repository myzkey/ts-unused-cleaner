pub mod config;
pub mod detector;
pub mod reporter;
pub mod types;

pub use config::{adjust_config_for_monorepo, load_config};
pub use detector::UnusedElementDetector;
pub use reporter::Reporter;
pub use types::*;

use anyhow::Result;

/// 未使用要素を検出するメイン関数
pub fn detect_unused_elements(
    config_path: Option<&str>,
    custom_config: Option<Config>,
) -> Result<DetectionResult> {
    // 設定を読み込み
    let mut config = load_config(config_path)?;

    // カスタム設定をマージ
    if let Some(custom) = custom_config {
        config = merge_configs(config, custom);
    }

    // モノレポ対応
    config = adjust_config_for_monorepo(config)?;

    // 検出実行
    let mut detector = UnusedElementDetector::new(config)?;
    let result = detector.detect()?;

    Ok(result)
}

/// 設定をマージする
fn merge_configs(base: Config, custom: Config) -> Config {
    Config {
        search_dirs: if custom.search_dirs.is_empty() {
            base.search_dirs
        } else {
            custom.search_dirs
        },
        exclude_patterns: if custom.exclude_patterns.is_empty() {
            base.exclude_patterns
        } else {
            // カスタムパターンがある場合はデフォルトと結合
            let mut patterns = crate::types::default_exclude_patterns();
            patterns.extend(custom.exclude_patterns);
            patterns.sort();
            patterns.dedup();
            patterns
        },
        detection_types: custom.detection_types,
        ci: custom.ci.or(base.ci),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_configs() {
        let base = Config::default();
        let custom = Config {
            search_dirs: vec!["custom/src".to_string()],
            exclude_patterns: vec![],
            detection_types: DetectionTypes::default(),
            ci: None,
        };

        let merged = merge_configs(base.clone(), custom);
        assert_eq!(merged.search_dirs, vec!["custom/src"]);
    }
}
