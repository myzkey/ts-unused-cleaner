use crate::types::{
    Config, DetectionResult, DetectionStats, DetectorError, ElementInfo, ElementType,
    ElementUsage, Usage,
};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use swc_common::{BytePos, Span, Spanned};
use swc_ecma_ast::*;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig};
use walkdir::WalkDir;

pub struct UnusedElementDetector {
    config: Config,
}

#[derive(Debug, Clone)]
struct ElementDefinition {
    name: String,
    element_type: ElementType,
    file: String,
    should_ignore: bool,
}

#[derive(Debug, Clone)]
struct ElementReference {
    name: String,
    file: String,
    line: usize,
    context: String,
}

impl UnusedElementDetector {
    pub fn new(config: Config) -> Result<Self, DetectorError> {
        Ok(Self {
            config,
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

        // 1. 定義検出用ファイル（除外適用）と使用検出用ファイル（除外なし）を分離
        let definition_files = self.get_source_files_for_definitions()?;
        let all_files = self.get_all_source_files()?;
        println!("📁 Found {} source files ({} for definitions, {} for usage scanning)",
                 all_files.len(), definition_files.len(), all_files.len());

        // 2. AST解析で要素定義を抽出（除外パターン適用）
        let definitions = self.extract_definitions(&definition_files)?;
        println!("🔧 Discovered {} elements", definitions.len());

        // 3. AST解析で使用箇所を検索（全ファイルから）
        let references = self.extract_references(&all_files)?;
        println!("📄 Found {} references", references.len());

        // 4. 使用状況を分析
        let (unused, used) = self.analyze_usage(&definitions, &references)?;

        // 5. 統計情報を生成
        let by_type = self.generate_statistics(&unused, &used);

        Ok(DetectionResult {
            total: definitions.len(),
            unused,
            used,
            by_type,
        })
    }

    /// 定義検出用ソースファイルを取得（除外パターン適用）
    fn get_source_files_for_definitions(&self) -> Result<Vec<String>, DetectorError> {
        let files_nested: Vec<Vec<String>> = self
            .config
            .search_dirs
            .par_iter()
            .map(|dir| self.get_files_in_dir(dir, &[".ts", ".tsx"]))
            .collect::<Result<Vec<_>, _>>()?;

        let files: Vec<String> = files_nested.into_iter().flatten().collect();
        Ok(files)
    }

    /// 全ソースファイルを取得（使用検出用、除外パターンなし）
    fn get_all_source_files(&self) -> Result<Vec<String>, DetectorError> {
        let files_nested: Vec<Vec<String>> = self
            .config
            .search_dirs
            .par_iter()
            .map(|dir| self.get_files_in_dir_no_exclude(dir, &[".ts", ".tsx"]))
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

                if !entry.file_type().is_file() {
                    return None;
                }

                if self.should_exclude(path) {
                    return None;
                }

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

    /// ディレクトリ内のファイルを取得（除外パターンなし）
    fn get_files_in_dir_no_exclude(
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

                if !entry.file_type().is_file() {
                    return None;
                }

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
                // 簡単なワイルドカードマッチング
                let parts: Vec<&str> = pattern.split('*').collect();
                if parts.len() == 2 {
                    if path_str.starts_with(parts[0]) && path_str.ends_with(parts[1]) {
                        return true;
                    }
                } else if pattern.ends_with("/**") {
                    let prefix = &pattern[..pattern.len() - 3];
                    if path_str.starts_with(prefix) {
                        return true;
                    }
                }
            } else if path_str.contains(pattern) {
                return true;
            }
        }

        false
    }

    /// AST解析で要素定義を抽出
    fn extract_definitions(
        &self,
        files: &[String],
    ) -> Result<Vec<ElementDefinition>, DetectorError> {
        let config = self.config.clone();
        let definitions: Vec<Vec<ElementDefinition>> = files
            .par_iter()
            .map(|file| {
                let content = fs::read_to_string(file)?;
                let defs = parse_file_for_definitions_static(file, &content, &config)?;
                Ok(defs)
            })
            .collect::<Result<Vec<_>, DetectorError>>()?;

        Ok(definitions.into_iter().flatten().collect())
    }


    /// AST解析で参照を抽出
    fn extract_references(&self, files: &[String]) -> Result<Vec<ElementReference>, DetectorError> {
        let references: Vec<Vec<ElementReference>> = files
            .par_iter()
            .map(|file| {
                let content = fs::read_to_string(file)?;
                let refs = parse_file_for_references_static(file, &content)?;
                Ok(refs)
            })
            .collect::<Result<Vec<_>, DetectorError>>()?;

        Ok(references.into_iter().flatten().collect())
    }


    /// 使用状況を分析
    fn analyze_usage(
        &self,
        definitions: &[ElementDefinition],
        references: &[ElementReference],
    ) -> Result<(Vec<ElementInfo>, Vec<ElementInfo>), DetectorError> {
        let mut unused = Vec::new();
        let mut used = Vec::new();

        for def in definitions {
            // Skip if element has ignore comment
            if def.should_ignore {
                continue;
            }

            let mut element_usages = Vec::new();
            let mut is_used = false;

            for ref_item in references {
                // 同じファイル内の定義は除外
                if ref_item.file == def.file {
                    continue;
                }

                if ref_item.name == def.name {
                    is_used = true;
                    element_usages.push(ElementUsage {
                        file: ref_item.file.clone(),
                        usages: vec![Usage {
                            line: ref_item.line,
                            context: ref_item.context.clone(),
                        }],
                    });
                }
            }

            let element_info = ElementInfo {
                name: def.name.clone(),
                element_type: def.element_type.clone(),
                definition_files: vec![def.file.clone()],
                usages: if is_used { Some(element_usages) } else { None },
            };

            if is_used {
                used.push(element_info);
            } else {
                unused.push(element_info);
            }
        }

        Ok((unused, used))
    }

    /// 統計情報を生成
    fn generate_statistics(
        &self,
        unused: &[ElementInfo],
        used: &[ElementInfo],
    ) -> HashMap<ElementType, DetectionStats> {
        let mut stats = HashMap::new();

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

/// 定義を収集するVisitor
struct DefinitionVisitor {
    file: String,
    config: Config,
    definitions: Vec<ElementDefinition>,
    content: String,
}

impl DefinitionVisitor {
    fn new(file: String, config: &Config, content: String) -> Self {
        Self {
            file,
            config: config.clone(),
            definitions: Vec::new(),
            content,
        }
    }

    fn visit_module(&mut self, module: &Module) {
        for item in &module.body {
            self.visit_module_item(item);
        }
    }

    fn visit_module_item(&mut self, item: &ModuleItem) {
        match item {
            ModuleItem::ModuleDecl(decl) => self.visit_module_decl(decl),
            ModuleItem::Stmt(stmt) => self.visit_stmt(stmt),
        }
    }

    fn visit_module_decl(&mut self, decl: &ModuleDecl) {
        match decl {
            ModuleDecl::ExportDecl(export_decl) => {
                self.visit_export_decl(&export_decl.decl);
            }
            ModuleDecl::ExportDefaultDecl(export_default) => {
                self.visit_export_default_decl(export_default);
            }
            _ => {}
        }
    }

    fn visit_export_decl(&mut self, decl: &Decl) {
        match decl {
            Decl::Fn(func_decl) if self.config.detection_types.functions => {
                if let Some(name) = self.extract_function_name(&func_decl.ident) {
                    if self.is_camel_case(&name) {
                        let should_ignore = self.has_ignore_comment(func_decl.span());
                        self.definitions.push(ElementDefinition {
                            name,
                            element_type: ElementType::Function,
                            file: self.file.clone(),
                            should_ignore,
                        });
                    }
                }
            }
            Decl::Var(var_decl) => {
                for decl in &var_decl.decls {
                    if let Pat::Ident(ident) = &decl.name {
                        let name = ident.id.sym.to_string();

                        if let Some(init) = &decl.init {
                            let should_ignore = self.has_ignore_comment(decl.span);
                            // コンポーネント検出
                            if self.config.detection_types.components && self.is_component_pattern(&name, init) {
                                self.definitions.push(ElementDefinition {
                                    name: name.clone(),
                                    element_type: ElementType::Component,
                                    file: self.file.clone(),
                                    should_ignore,
                                });
                            }
                            // 関数検出
                            else if self.config.detection_types.functions && self.is_function_pattern(init) && self.is_camel_case(&name) {
                                self.definitions.push(ElementDefinition {
                                    name: name.clone(),
                                    element_type: ElementType::Function,
                                    file: self.file.clone(),
                                    should_ignore,
                                });
                            }
                            // 変数検出
                            else if self.config.detection_types.variables && self.is_constant_case(&name) {
                                self.definitions.push(ElementDefinition {
                                    name: name.clone(),
                                    element_type: ElementType::Variable,
                                    file: self.file.clone(),
                                    should_ignore,
                                });
                            }
                        }
                    }
                }
            }
            Decl::TsTypeAlias(type_alias) if self.config.detection_types.types => {
                let name = type_alias.id.sym.to_string();
                if self.is_pascal_case(&name) {
                    let should_ignore = self.has_ignore_comment(type_alias.span());
                    self.definitions.push(ElementDefinition {
                        name,
                        element_type: ElementType::Type,
                        file: self.file.clone(),
                        should_ignore,
                    });
                }
            }
            Decl::TsInterface(interface) if self.config.detection_types.interfaces => {
                let name = interface.id.sym.to_string();
                if self.is_pascal_case(&name) {
                    let should_ignore = self.has_ignore_comment(interface.span());
                    self.definitions.push(ElementDefinition {
                        name,
                        element_type: ElementType::Interface,
                        file: self.file.clone(),
                        should_ignore,
                    });
                }
            }
            Decl::TsEnum(enum_decl) if self.config.detection_types.enums => {
                let name = enum_decl.id.sym.to_string();
                if self.is_pascal_case(&name) {
                    let should_ignore = self.has_ignore_comment(enum_decl.span());
                    self.definitions.push(ElementDefinition {
                        name,
                        element_type: ElementType::Enum,
                        file: self.file.clone(),
                        should_ignore,
                    });
                }
            }
            _ => {}
        }
    }

    fn visit_export_default_decl(&mut self, export_default: &ExportDefaultDecl) {
        match &export_default.decl {
            DefaultDecl::Fn(func_expr) if self.config.detection_types.components => {
                if let Some(ident) = &func_expr.ident {
                    let name = ident.sym.to_string();
                    if self.is_pascal_case(&name) {
                        let should_ignore = self.has_ignore_comment(export_default.span());
                        self.definitions.push(ElementDefinition {
                            name,
                            element_type: ElementType::Component,
                            file: self.file.clone(),
                            should_ignore,
                        });
                    }
                }
            }
            _ => {}
        }
    }

    fn visit_stmt(&mut self, _stmt: &Stmt) {
        // Stmtの処理は必要に応じて実装
    }

    // ヘルパーメソッド
    fn extract_function_name(&self, ident: &Ident) -> Option<String> {
        Some(ident.sym.to_string())
    }

    fn is_component_pattern(&self, name: &str, expr: &Expr) -> bool {
        self.is_pascal_case(name) && (
            self.is_arrow_function(expr) ||
            self.is_react_component_call(expr)
        )
    }

    fn is_function_pattern(&self, expr: &Expr) -> bool {
        self.is_arrow_function(expr)
    }

    fn is_arrow_function(&self, expr: &Expr) -> bool {
        matches!(expr, Expr::Arrow(_))
    }

    fn is_react_component_call(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Call(call_expr) => {
                if let Callee::Expr(callee) = &call_expr.callee {
                    match &**callee {
                        Expr::Ident(ident) => {
                            matches!(ident.sym.as_ref(), "memo" | "forwardRef")
                        }
                        Expr::Member(member) => {
                            if let Expr::Ident(obj) = &*member.obj {
                                obj.sym.as_ref() == "React" &&
                                if let MemberProp::Ident(prop) = &member.prop {
                                    matches!(prop.sym.as_ref(), "memo" | "forwardRef")
                                } else { false }
                            } else { false }
                        }
                        _ => false
                    }
                } else { false }
            }
            _ => false
        }
    }

    fn is_pascal_case(&self, name: &str) -> bool {
        !name.is_empty() && name.chars().next().unwrap().is_uppercase()
    }

    fn is_camel_case(&self, name: &str) -> bool {
        !name.is_empty() && name.chars().next().unwrap().is_lowercase()
    }

    fn is_constant_case(&self, name: &str) -> bool {
        name.chars().all(|c| c.is_uppercase() || c == '_' || c.is_numeric())
    }

    fn has_ignore_comment(&self, span: Span) -> bool {
        // Get the line number of the element
        let start_pos = span.lo.0 as usize;
        let lines: Vec<&str> = self.content.lines().collect();
        let mut char_count = 0;
        let mut target_line = 0;

        for (i, line) in lines.iter().enumerate() {
            let line_length = line.len() + 1; // +1 for newline
            if char_count + line_length > start_pos {
                target_line = i;
                break;
            }
            char_count += line_length;
        }

        // Check the previous line for ignore comment
        if target_line > 0 {
            let prev_line = lines[target_line - 1].trim();
            if prev_line == "// @ts-unused-ignore" {
                return true;
            }
        }

        // Also check if there's an inline comment on the same line
        if target_line < lines.len() {
            let current_line = lines[target_line];
            if current_line.contains("// @ts-unused-ignore") {
                return true;
            }
        }

        false
    }
}

/// 参照を収集するVisitor
struct ReferenceVisitor {
    file: String,
    references: Vec<ElementReference>,
}

impl ReferenceVisitor {
    fn new(file: String, _content: &str) -> Self {
        Self {
            file,
            references: Vec::new(),
        }
    }

    fn visit_module(&mut self, module: &Module) {
        for item in &module.body {
            self.visit_module_item(item);
        }
    }

    fn visit_module_item(&mut self, item: &ModuleItem) {
        match item {
            ModuleItem::ModuleDecl(decl) => self.visit_module_decl(decl),
            ModuleItem::Stmt(stmt) => self.visit_stmt(stmt),
        }
    }

    fn visit_module_decl(&mut self, decl: &ModuleDecl) {
        match decl {
            ModuleDecl::Import(import_decl) => {
                for specifier in &import_decl.specifiers {
                    match specifier {
                        ImportSpecifier::Named(named) => {
                            let name = if let Some(imported) = &named.imported {
                                match imported {
                                    ModuleExportName::Ident(ident) => ident.sym.to_string(),
                                    ModuleExportName::Str(str_lit) => str_lit.value.to_string(),
                                }
                            } else {
                                named.local.sym.to_string()
                            };

                            self.references.push(ElementReference {
                                name,
                                file: self.file.clone(),
                                line: 1,
                                context: "import".to_string(),
                            });
                        }
                        ImportSpecifier::Default(default) => {
                            self.references.push(ElementReference {
                                name: default.local.sym.to_string(),
                                file: self.file.clone(),
                                line: 1,
                                context: "import".to_string(),
                            });
                        }
                        ImportSpecifier::Namespace(namespace) => {
                            self.references.push(ElementReference {
                                name: namespace.local.sym.to_string(),
                                file: self.file.clone(),
                                line: 1,
                                context: "import".to_string(),
                            });
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        // 各種文の処理
        match stmt {
            Stmt::Expr(expr_stmt) => {
                self.visit_expr(&expr_stmt.expr);
            }
            Stmt::Decl(decl) => {
                self.visit_decl(decl);
            }
            _ => {}
        }
    }

    fn visit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Ident(ident) => {
                self.references.push(ElementReference {
                    name: ident.sym.to_string(),
                    file: self.file.clone(),
                    line: 1,
                    context: "usage".to_string(),
                });
            }
            Expr::Call(call_expr) => {
                self.visit_callee(&call_expr.callee);
                for arg in &call_expr.args {
                    self.visit_expr_or_spread(arg);
                }
            }
            Expr::JSXElement(jsx_elem) => {
                self.visit_jsx_element(jsx_elem);
            }
            _ => {
                // 他の式の処理は必要に応じて実装
            }
        }
    }

    fn visit_expr_or_spread(&mut self, expr: &ExprOrSpread) {
        self.visit_expr(&expr.expr);
    }

    fn visit_callee(&mut self, callee: &Callee) {
        match callee {
            Callee::Expr(expr) => {
                self.visit_expr(expr);
            }
            _ => {}
        }
    }

    fn visit_jsx_element(&mut self, jsx_elem: &JSXElement) {
        if let JSXElementName::Ident(ident) = &jsx_elem.opening.name {
            self.references.push(ElementReference {
                name: ident.sym.to_string(),
                file: self.file.clone(),
                line: 1,
                context: "jsx".to_string(),
            });
        }

        for child in &jsx_elem.children {
            if let JSXElementChild::JSXElement(child_elem) = child {
                self.visit_jsx_element(child_elem);
            }
        }
    }

    fn visit_decl(&mut self, _decl: &Decl) {
        // Declの処理は必要に応じて実装
    }

}

/// 静的関数：ファイルをASTで解析して定義を抽出
fn parse_file_for_definitions_static(
    file: &str,
    content: &str,
    config: &Config,
) -> Result<Vec<ElementDefinition>, DetectorError> {
    let input = StringInput::new(content, BytePos(0), BytePos(content.len() as u32));

    let lexer = Lexer::new(
        Syntax::Typescript(TsConfig {
            tsx: file.ends_with(".tsx"),
            decorators: true,
            dts: file.ends_with(".d.ts"),
            no_early_errors: true,
            disallow_ambiguous_jsx_like: false,
        }),
        Default::default(),
        input,
        None,
    );

    let mut parser = Parser::new_from(lexer);
    let module = parser.parse_module()
        .map_err(|e| DetectorError::ParseError(format!("Failed to parse {}: {:?}", file, e)))?;

    let mut visitor = DefinitionVisitor::new(file.to_string(), config, content.to_string());
    visitor.visit_module(&module);

    Ok(visitor.definitions)
}

/// 静的関数：ファイルをASTで解析して参照を抽出
fn parse_file_for_references_static(
    file: &str,
    content: &str,
) -> Result<Vec<ElementReference>, DetectorError> {
    let input = StringInput::new(content, BytePos(0), BytePos(content.len() as u32));

    let lexer = Lexer::new(
        Syntax::Typescript(TsConfig {
            tsx: file.ends_with(".tsx"),
            decorators: true,
            dts: file.ends_with(".d.ts"),
            no_early_errors: true,
            disallow_ambiguous_jsx_like: false,
        }),
        Default::default(),
        input,
        None,
    );

    let mut parser = Parser::new_from(lexer);
    let module = parser.parse_module()
        .map_err(|e| DetectorError::ParseError(format!("Failed to parse {}: {:?}", file, e)))?;

    let mut visitor = ReferenceVisitor::new(file.to_string(), content);
    visitor.visit_module(&module);

    Ok(visitor.references)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DetectionTypes;

    fn create_test_config() -> Config {
        Config {
            detection_types: DetectionTypes {
                components: true,
                types: true,
                interfaces: true,
                functions: true,
                variables: true,
                enums: true,
            },
            search_dirs: vec![".".to_string()],
            exclude_patterns: vec![],
            ci: None,
        }
    }

    #[test]
    fn test_ignore_comment_detection() {
        let config = create_test_config();

        // Test TypeScript type with ignore comment
        let content = r#"// @ts-unused-ignore
export type UnusedType = string;

export type UsedType = number;
"#;
        let result = parse_file_for_definitions_static("test.ts", content, &config).unwrap();

        // Find UnusedType and UsedType
        let unused_type = result.iter().find(|def| def.name == "UnusedType").unwrap();
        let used_type = result.iter().find(|def| def.name == "UsedType").unwrap();

        assert!(unused_type.should_ignore, "UnusedType should be marked for ignore");
        assert!(!used_type.should_ignore, "UsedType should not be marked for ignore");
    }

    #[test]
    fn test_ignore_comment_interface() {
        let config = create_test_config();

        let content = r#"// @ts-unused-ignore
export interface UnusedInterface {
    prop: string;
}

export interface UsedInterface {
    prop: number;
}
"#;
        let result = parse_file_for_definitions_static("test.ts", content, &config).unwrap();

        let unused_interface = result.iter().find(|def| def.name == "UnusedInterface").unwrap();
        let used_interface = result.iter().find(|def| def.name == "UsedInterface").unwrap();

        assert!(unused_interface.should_ignore);
        assert!(!used_interface.should_ignore);
    }

    #[test]
    fn test_ignore_comment_function() {
        let config = create_test_config();

        let content = r#"// @ts-unused-ignore
export function unusedFunction() {
    return "hello";
}

export function usedFunction() {
    return "world";
}
"#;
        let result = parse_file_for_definitions_static("test.ts", content, &config).unwrap();

        let unused_function = result.iter().find(|def| def.name == "unusedFunction").unwrap();
        let used_function = result.iter().find(|def| def.name == "usedFunction").unwrap();

        assert!(unused_function.should_ignore);
        assert!(!used_function.should_ignore);
    }

    #[test]
    fn test_ignore_comment_component() {
        let config = create_test_config();

        let content = r#"// @ts-unused-ignore
export const UnusedComponent = () => {
    return <div>Unused</div>;
};

export const UsedComponent = () => {
    return <div>Used</div>;
};
"#;
        let result = parse_file_for_definitions_static("test.tsx", content, &config).unwrap();

        let unused_component = result.iter().find(|def| def.name == "UnusedComponent").unwrap();
        let used_component = result.iter().find(|def| def.name == "UsedComponent").unwrap();

        assert!(unused_component.should_ignore);
        assert!(!used_component.should_ignore);
    }

    #[test]
    fn test_ignore_comment_enum() {
        let config = create_test_config();

        let content = r#"// @ts-unused-ignore
export enum UnusedEnum {
    VALUE1 = "value1",
    VALUE2 = "value2"
}

export enum UsedEnum {
    VALUE3 = "value3",
    VALUE4 = "value4"
}
"#;
        let result = parse_file_for_definitions_static("test.ts", content, &config).unwrap();

        let unused_enum = result.iter().find(|def| def.name == "UnusedEnum").unwrap();
        let used_enum = result.iter().find(|def| def.name == "UsedEnum").unwrap();

        assert!(unused_enum.should_ignore);
        assert!(!used_enum.should_ignore);
    }

    #[test]
    fn test_inline_ignore_comment() {
        let config = create_test_config();

        let content = r#"export type UnusedType = string; // @ts-unused-ignore
export type UsedType = number;
"#;
        let result = parse_file_for_definitions_static("test.ts", content, &config).unwrap();

        let unused_type = result.iter().find(|def| def.name == "UnusedType").unwrap();
        let used_type = result.iter().find(|def| def.name == "UsedType").unwrap();

        assert!(unused_type.should_ignore, "UnusedType should be marked for ignore with inline comment");
        assert!(!used_type.should_ignore, "UsedType should not be marked for ignore");
    }
}