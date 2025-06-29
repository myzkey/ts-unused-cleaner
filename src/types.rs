use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 検索するディレクトリ
    pub search_dirs: Vec<String>,
    /// 除外するファイル/ディレクトリのパターン
    #[serde(default = "default_exclude_patterns")]
    pub exclude_patterns: Vec<String>,
    /// 検出する要素の種類
    #[serde(default)]
    pub detection_types: DetectionTypes,
    /// CI設定
    pub ci: Option<CiConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionTypes {
    /// Reactコンポーネントを検出するか
    pub components: bool,
    /// TypeScript型を検出するか
    pub types: bool,
    /// TypeScriptインターフェースを検出するか
    pub interfaces: bool,
    /// 関数を検出するか
    pub functions: bool,
    /// 変数/定数を検出するか
    pub variables: bool,
    /// enumを検出するか
    pub enums: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiConfig {
    /// 未使用要素の許容数
    pub max_unused_elements: usize,
    /// 許容数を超えた場合にCIを失敗させるか
    pub fail_on_exceed: bool,
    /// 警告レベル
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementUsage {
    pub file: String,
    pub usages: Vec<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub line: usize,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementInfo {
    pub name: String,
    pub element_type: ElementType,
    pub definition_files: Vec<String>,
    pub usages: Option<Vec<ElementUsage>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ElementType {
    Component,
    Type,
    Interface,
    Function,
    Variable,
    Enum,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    pub unused: Vec<ElementInfo>,
    pub used: Vec<ElementInfo>,
    pub total: usize,
    pub by_type: HashMap<ElementType, DetectionStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionStats {
    pub total: usize,
    pub used: usize,
    pub unused: usize,
}

#[derive(Debug, Clone)]
pub struct ElementMap {
    pub definitions: HashMap<String, (ElementType, Vec<String>)>,
}

/// デフォルトの除外パターンを返す
pub fn default_exclude_patterns() -> Vec<String> {
    vec![
        "node_modules".to_string(),
        ".next".to_string(),
        "dist".to_string(),
        ".turbo".to_string(),
        "build".to_string(),
        "out".to_string(),
        "__tests__".to_string(),
        "*.test.ts".to_string(),
        "*.test.tsx".to_string(),
        "*.test.js".to_string(),
        "*.test.jsx".to_string(),
        "*.spec.ts".to_string(),
        "*.spec.tsx".to_string(),
        "*.spec.js".to_string(),
        "*.spec.jsx".to_string(),
        "*.stories.ts".to_string(),
        "*.stories.tsx".to_string(),
        "*.stories.js".to_string(),
        "*.stories.jsx".to_string(),
        "*.d.ts".to_string(),
        ".git".to_string(),
        ".vscode".to_string(),
        ".idea".to_string(),
        "coverage".to_string(),
        ".nyc_output".to_string(),
        "*.min.js".to_string(),
        "*.min.css".to_string(),
    ]
}

impl Default for Config {
    fn default() -> Self {
        Self {
            search_dirs: vec!["src".to_string()],
            exclude_patterns: default_exclude_patterns(),
            detection_types: DetectionTypes::default(),
            ci: Some(CiConfig {
                max_unused_elements: 5,
                fail_on_exceed: true,
                log_level: "warn".to_string(),
            }),
        }
    }
}

impl Default for DetectionTypes {
    fn default() -> Self {
        Self {
            components: true,
            types: true,
            interfaces: true,
            functions: true,
            variables: true,
            enums: true,
        }
    }
}

impl std::fmt::Display for ElementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElementType::Component => write!(f, "Component"),
            ElementType::Type => write!(f, "Type"),
            ElementType::Interface => write!(f, "Interface"),
            ElementType::Function => write!(f, "Function"),
            ElementType::Variable => write!(f, "Variable"),
            ElementType::Enum => write!(f, "Enum"),
        }
    }
}

impl std::hash::Hash for ElementType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl PartialEq for ElementType {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

impl Eq for ElementType {}

#[derive(Debug, thiserror::Error)]
pub enum DetectorError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Config error: {message}")]
    Config { message: String },

    #[error("File not found: {path}")]
    FileNotFound { path: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.search_dirs, vec!["src"]);
        assert!(!config.exclude_patterns.is_empty());
        assert!(config.exclude_patterns.contains(&"node_modules".to_string()));
        assert!(config.exclude_patterns.contains(&"*.test.ts".to_string()));
        assert!(config.detection_types.components);
        assert!(config.detection_types.types);
        assert!(config.detection_types.interfaces);
        assert!(config.detection_types.functions);
        assert!(config.detection_types.variables);
        assert!(config.detection_types.enums);
    }

    #[test]
    fn test_default_exclude_patterns() {
        let patterns = default_exclude_patterns();
        assert!(patterns.contains(&"node_modules".to_string()));
        assert!(patterns.contains(&"*.test.ts".to_string()));
        assert!(patterns.contains(&"*.test.tsx".to_string()));
        assert!(patterns.contains(&"*.spec.ts".to_string()));
        assert!(patterns.contains(&"*.stories.ts".to_string()));
        assert!(patterns.contains(&"dist".to_string()));
        assert!(patterns.contains(&".next".to_string()));
    }

    #[test]
    fn test_element_type_display() {
        assert_eq!(ElementType::Component.to_string(), "Component");
        assert_eq!(ElementType::Type.to_string(), "Type");
        assert_eq!(ElementType::Interface.to_string(), "Interface");
        assert_eq!(ElementType::Function.to_string(), "Function");
        assert_eq!(ElementType::Variable.to_string(), "Variable");
        assert_eq!(ElementType::Enum.to_string(), "Enum");
    }

    #[test]
    fn test_detection_types_default() {
        let detection_types = DetectionTypes::default();
        assert!(detection_types.components);
        assert!(detection_types.types);
        assert!(detection_types.interfaces);
        assert!(detection_types.functions);
        assert!(detection_types.variables);
        assert!(detection_types.enums);
    }
}
