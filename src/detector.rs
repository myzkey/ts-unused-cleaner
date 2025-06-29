use crate::types::{
    ElementInfo, ElementMap, ElementUsage, Config, DetectionResult, DetectionStats,
    DetectorError, ElementType, Usage,
};
use rayon::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub struct UnusedElementDetector {
    config: Config,
    definition_patterns: HashMap<ElementType, Vec<Regex>>,
    usage_patterns: HashMap<String, Vec<Regex>>,
}

impl UnusedElementDetector {
    pub fn new(config: Config) -> Result<Self, DetectorError> {
        let mut definition_patterns = HashMap::new();

        // コンポーネント定義パターン
        if config.detection_types.components {
            definition_patterns.insert(ElementType::Component, vec![
                Regex::new(r"export\s+default\s+function\s+([A-Z][a-zA-Z0-9]*)")?,
                Regex::new(r"export\s+const\s+([A-Z][a-zA-Z0-9]*)\s*=\s*(?:React\.)?(?:memo|forwardRef)")?,
                Regex::new(r"export\s+const\s+([A-Z][a-zA-Z0-9]*)\s*=\s*\([^)]*\)\s*=>")?,
                Regex::new(r"export\s+function\s+([A-Z][a-zA-Z0-9]*)")?,
                Regex::new(r"const\s+([A-Z][a-zA-Z0-9]*)\s*=\s*React\.forwardRef")?,
                Regex::new(r"const\s+([A-Z][a-zA-Z0-9]*)\s*=\s*forwardRef")?,
            ]);
        }

        // TypeScript型定義パターン
        if config.detection_types.types {
            definition_patterns.insert(
                ElementType::Type,
                vec![
                    Regex::new(r"export\s+type\s+([A-Z][a-zA-Z0-9]*)")?,
                    Regex::new(r"type\s+([A-Z][a-zA-Z0-9]*)\s*=")?,
                ],
            );
        }

        // インターフェース定義パターン
        if config.detection_types.interfaces {
            definition_patterns.insert(
                ElementType::Interface,
                vec![
                    Regex::new(r"export\s+interface\s+([A-Z][a-zA-Z0-9]*)")?,
                    Regex::new(r"interface\s+([A-Z][a-zA-Z0-9]*)")?,
                ],
            );
        }

        // 関数定義パターン
        if config.detection_types.functions {
            definition_patterns.insert(ElementType::Function, vec![
                Regex::new(r"export\s+function\s+([a-z][a-zA-Z0-9]*)")?,
                Regex::new(r"export\s+const\s+([a-z][a-zA-Z0-9]*)\s*=\s*(?:async\s+)?(?:\([^)]*\)\s*=>|\([^)]*\)\s*:\s*[^=]+\s*=>)")?,
                Regex::new(r"const\s+([a-z][a-zA-Z0-9]*)\s*=\s*(?:async\s+)?(?:\([^)]*\)\s*=>|\([^)]*\)\s*:\s*[^=]+\s*=>)")?,
            ]);
        }

        // 変数/定数定義パターン
        if config.detection_types.variables {
            definition_patterns.insert(
                ElementType::Variable,
                vec![
                    Regex::new(r"export\s+const\s+([A-Z_][A-Z0-9_]*)\s*=")?,
                    Regex::new(r"export\s+let\s+([a-z][a-zA-Z0-9]*)\s*=")?,
                    Regex::new(r"const\s+([A-Z_][A-Z0-9_]*)\s*=")?,
                ],
            );
        }

        // enum定義パターン
        if config.detection_types.enums {
            definition_patterns.insert(
                ElementType::Enum,
                vec![
                    Regex::new(r"export\s+enum\s+([A-Z][a-zA-Z0-9]*)")?,
                    Regex::new(r"enum\s+([A-Z][a-zA-Z0-9]*)")?,
                ],
            );
        }

        Ok(Self {
            config,
            definition_patterns,
            usage_patterns: HashMap::new(),
        })
    }

    /// 未使用要素を検出
    pub fn detect(&mut self) -> Result<DetectionResult, DetectorError> {
        let enabled_types: Vec<&str> = {
            let mut types = Vec::new();
            if self.config.detection_types.components {
                types.push("components");
            }
            if self.config.detection_types.types {
                types.push("types");
            }
            if self.config.detection_types.interfaces {
                types.push("interfaces");
            }
            if self.config.detection_types.functions {
                types.push("functions");
            }
            if self.config.detection_types.variables {
                types.push("variables");
            }
            if self.config.detection_types.enums {
                types.push("enums");
            }
            types
        };

        println!("🔍 Scanning for unused {}...", enabled_types.join(", "));

        // 1. ソースファイルを並列で取得
        let source_files = self.get_component_files()?;
        println!("📁 Found {} source files", source_files.len());

        // 2. 要素定義を並列で抽出
        let element_map = self.extract_element_definitions(&source_files)?;
        println!("🔧 Discovered {} elements", element_map.definitions.len());

        // 3. 検索ファイルを取得
        let search_files = self.get_search_files()?;
        println!("📄 Searching for usage in {} files", search_files.len());

        // 4. 使用パターンを事前に生成
        self.prepare_usage_patterns(&element_map)?;

        // 5. 並列で使用箇所をチェック
        let (unused, used) = self.check_element_usage(&element_map, &search_files)?;

        // 6. 統計情報を生成
        let by_type = self.generate_statistics(&unused, &used);

        Ok(DetectionResult {
            total: element_map.definitions.len(),
            unused,
            used,
            by_type,
        })
    }

    /// コンポーネントファイルを取得
    fn get_component_files(&self) -> Result<Vec<String>, DetectorError> {
        let files_nested: Vec<Vec<String>> = self
            .config
            .search_dirs
            .par_iter()
            .map(|dir| self.get_files_in_dir(dir, &[".ts", ".tsx"]))
            .collect::<Result<Vec<_>, _>>()?;

        let files: Vec<String> = files_nested.into_iter().flatten().collect();

        Ok(files)
    }

    /// 検索ファイルを取得
    fn get_search_files(&self) -> Result<Vec<String>, DetectorError> {
        let files_nested: Vec<Vec<String>> = self
            .config
            .search_dirs
            .par_iter()
            .map(|dir| self.get_files_in_dir(dir, &[".ts", ".tsx"]))
            .collect::<Result<Vec<_>, _>>()?;

        let files: Vec<String> = files_nested.into_iter().flatten().collect();

        Ok(files)
    }

    /// ディレクトリ内のファイルを取得
    fn get_files_in_dir(
        &self,
        dir: &str,
        extensions: &[&str],
    ) -> Result<Vec<String>, DetectorError> {
        if !Path::new(dir).exists() {
            return Ok(Vec::new());
        }

        let files: Result<Vec<_>, _> = WalkDir::new(dir)
            .into_iter()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();

                // ファイルのみ処理
                if !entry.file_type().is_file() {
                    return None;
                }

                // 除外パターンをチェック
                if self.should_exclude(path) {
                    return None;
                }

                // 拡張子をチェック
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy();
                    if extensions
                        .iter()
                        .any(|&e| ext_str == e.trim_start_matches('.'))
                    {
                        return Some(Ok(path.to_string_lossy().to_string()));
                    }
                }

                None
            })
            .collect();

        files
    }

    /// ファイルを除外すべきかチェック
    fn should_exclude(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in &self.config.exclude_patterns {
            if pattern.contains('*') {
                // ワイルドカードパターンの簡単な実装
                let regex_pattern = pattern.replace("*", ".*");
                if let Ok(regex) = Regex::new(&regex_pattern) {
                    if regex.is_match(&path_str) {
                        return true;
                    }
                }
            } else if path_str.contains(pattern) {
                return true;
            }
        }

        false
    }

    /// 要素定義を抽出
    fn extract_element_definitions(&self, files: &[String]) -> Result<ElementMap, DetectorError> {
        let definitions: HashMap<String, (ElementType, Vec<String>)> = files
            .par_iter()
            .map(|file| {
                let content = fs::read_to_string(file)?;
                let elements = self.extract_elements_from_content(&content);
                Ok((file.clone(), elements))
            })
            .collect::<Result<Vec<_>, DetectorError>>()?
            .into_iter()
            .fold(HashMap::new(), |mut acc, (file, elements)| {
                for (element_name, element_type) in elements {
                    acc.entry(element_name)
                        .or_insert_with(|| (element_type.clone(), Vec::new()))
                        .1
                        .push(file.clone());
                }
                acc
            });

        Ok(ElementMap { definitions })
    }

    /// ファイル内容から要素を抽出
    fn extract_elements_from_content(&self, content: &str) -> Vec<(String, ElementType)> {
        let mut elements = Vec::new();

        for (element_type, patterns) in &self.definition_patterns {
            for pattern in patterns {
                for cap in pattern.captures_iter(content) {
                    if let Some(element_name) = cap.get(1) {
                        elements.push((element_name.as_str().to_string(), element_type.clone()));
                    }
                }
            }
        }

        elements
    }

    /// 使用パターンを事前に準備
    fn prepare_usage_patterns(&mut self, element_map: &ElementMap) -> Result<(), DetectorError> {
        for (element_name, (element_type, _)) in &element_map.definitions {
            let patterns = match element_type {
                ElementType::Component => vec![
                    Regex::new(&format!(r"<{}(?:\s|>|/)", element_name))?,
                    Regex::new(&format!(r"import\s*\{{[^}}]*\b{}\b[^}}]*\}}", element_name))?,
                    Regex::new(&format!(r"import\s+{}\b", element_name))?,
                    Regex::new(&format!(r"\{{\s*{}\s*\}}", element_name))?,
                ],
                ElementType::Type | ElementType::Interface => vec![
                    Regex::new(&format!(r":\s*{}\b", element_name))?,
                    Regex::new(&format!(r"<{}>", element_name))?,
                    Regex::new(&format!(r"extends\s+{}\b", element_name))?,
                    Regex::new(&format!(r"implements\s+{}\b", element_name))?,
                    Regex::new(&format!(r"import\s*\{{[^}}]*\b{}\b[^}}]*\}}", element_name))?,
                ],
                ElementType::Function => vec![
                    Regex::new(&format!(r"\b{}(?:\s*\()", element_name))?,
                    Regex::new(&format!(r"import\s*\{{[^}}]*\b{}\b[^}}]*\}}", element_name))?,
                    Regex::new(&format!(r"import\s+{}\b", element_name))?,
                ],
                ElementType::Variable | ElementType::Enum => vec![
                    Regex::new(&format!(r"\b{}\b", element_name))?,
                    Regex::new(&format!(r"import\s*\{{[^}}]*\b{}\b[^}}]*\}}", element_name))?,
                ],
            };
            self.usage_patterns.insert(element_name.clone(), patterns);
        }
        Ok(())
    }

    /// 要素使用箇所をチェック
    fn check_element_usage(
        &self,
        element_map: &ElementMap,
        search_files: &[String],
    ) -> Result<(Vec<ElementInfo>, Vec<ElementInfo>), DetectorError> {
        let results: Vec<_> = element_map
            .definitions
            .par_iter()
            .map(|(element_name, (element_type, definition_files))| {
                let mut is_used = false;
                let mut usages = Vec::new();

                for search_file in search_files {
                    // 定義ファイル自体は除外
                    if definition_files.contains(search_file) {
                        continue;
                    }

                    if let Ok(content) = fs::read_to_string(search_file) {
                        let file_usages =
                            self.find_element_usage_in_content(&content, element_name);
                        if !file_usages.is_empty() {
                            is_used = true;
                            usages.push(ElementUsage {
                                file: search_file.clone(),
                                usages: file_usages,
                            });
                        }
                    }
                }

                (
                    element_name.clone(),
                    element_type.clone(),
                    definition_files.clone(),
                    is_used,
                    usages,
                )
            })
            .collect();

        let mut unused = Vec::new();
        let mut used = Vec::new();

        for (name, element_type, definition_files, is_used, usages) in results {
            if is_used {
                used.push(ElementInfo {
                    name,
                    element_type,
                    definition_files,
                    usages: Some(usages),
                });
            } else {
                unused.push(ElementInfo {
                    name,
                    element_type,
                    definition_files,
                    usages: None,
                });
            }
        }

        Ok((unused, used))
    }

    /// ファイル内容での要素使用箇所を検索
    fn find_element_usage_in_content(&self, content: &str, element_name: &str) -> Vec<Usage> {
        let mut usages = Vec::new();

        if let Some(patterns) = self.usage_patterns.get(element_name) {
            for pattern in patterns {
                for mat in pattern.find_iter(content) {
                    let line_number = content[..mat.start()].lines().count();
                    usages.push(Usage {
                        line: line_number,
                        context: mat.as_str().to_string(),
                    });
                }
            }
        }

        usages
    }

    /// 統計情報を生成
    fn generate_statistics(
        &self,
        unused: &[ElementInfo],
        used: &[ElementInfo],
    ) -> HashMap<ElementType, DetectionStats> {
        let mut stats = HashMap::new();

        // 使用されていない要素の統計
        for item in unused {
            let entry = stats
                .entry(item.element_type.clone())
                .or_insert(DetectionStats {
                    total: 0,
                    used: 0,
                    unused: 0,
                });
            entry.total += 1;
            entry.unused += 1;
        }

        // 使用されている要素の統計
        for item in used {
            let entry = stats
                .entry(item.element_type.clone())
                .or_insert(DetectionStats {
                    total: 0,
                    used: 0,
                    unused: 0,
                });
            entry.total += 1;
            entry.used += 1;
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_elements() {
        let config = Config::default();
        let detector = UnusedElementDetector::new(config).unwrap();

        let content = r#"
        export default function MyComponent() {}
        export const AnotherComponent = () => {}
        export function ThirdComponent() {}
        export type MyType = string;
        export interface MyInterface {}
        export const CONSTANT = "value";
        export enum MyEnum { A, B }
        "#;

        let elements = detector.extract_elements_from_content(content);
        let names: Vec<String> = elements.iter().map(|(name, _)| name.clone()).collect();
        
        assert!(names.contains(&"MyComponent".to_string()));
        assert!(names.contains(&"AnotherComponent".to_string()));
        assert!(names.contains(&"ThirdComponent".to_string()));
        assert!(names.contains(&"MyType".to_string()));
        assert!(names.contains(&"MyInterface".to_string()));
        assert!(names.contains(&"CONSTANT".to_string()));
        assert!(names.contains(&"MyEnum".to_string()));
    }

    #[test]
    fn test_find_usage() {
        let config = Config::default();
        let mut detector = UnusedElementDetector::new(config).unwrap();

        // 使用パターンを準備
        let mut component_map = ElementMap {
            definitions: HashMap::new(),
        };
        component_map.definitions.insert(
            "MyComponent".to_string(), 
            (ElementType::Component, vec!["test.tsx".to_string()])
        );
        detector.prepare_usage_patterns(&component_map).unwrap();

        let content = r#"
        import { MyComponent } from './components';
        <MyComponent prop="value" />
        "#;

        let usages = detector.find_element_usage_in_content(content, "MyComponent");
        assert!(!usages.is_empty());
    }

    #[test]
    fn test_should_exclude() {
        let config = Config::default();
        let detector = UnusedElementDetector::new(config).unwrap();

        assert!(detector.should_exclude(Path::new("node_modules/package/index.js")));
        assert!(detector.should_exclude(Path::new("src/components/Button.test.tsx")));
        assert!(detector.should_exclude(Path::new("src/components/Button.stories.tsx")));
        assert!(!detector.should_exclude(Path::new("src/components/Button.tsx")));
    }
}
