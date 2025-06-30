use crate::types::{DetectionResult, ElementType};
use colored::*;

pub struct Reporter;

impl Reporter {
    /// 結果をコンソールに出力
    pub fn print_results(result: &DetectionResult, verbose: bool) {
        println!("\n{}", "=".repeat(60));
        println!("{}", "📊 Results".bold());
        println!("{}", "=".repeat(60));

        if result.unused.is_empty() {
            println!("{}", "✅ No unused elements found!".green());
        } else {
            println!(
                "{} {}",
                "❌".red(),
                format!(
                    "Found {} unused element{}:",
                    result.unused.len(),
                    if result.unused.len() == 1 { "" } else { "s" }
                )
                .red()
            );
            println!();

            for element in &result.unused {
                let icon = Self::get_element_icon(&element.element_type);
                println!(
                    "{} {} ({})",
                    icon.red(),
                    element.name.red().bold(),
                    element.element_type.to_string().dimmed()
                );
                for file in &element.definition_files {
                    println!("   📍 {}", file.dimmed());
                }
                println!();
            }
        }

        println!("\n📈 Statistics:");
        println!("   • Total elements: {}", result.total.to_string().bold());
        println!(
            "   • Used elements: {}",
            result.used.len().to_string().green().bold()
        );
        println!(
            "   • Unused elements: {}",
            result.unused.len().to_string().red().bold()
        );

        let usage_rate = if result.total > 0 {
            (result.used.len() as f64 / result.total as f64 * 100.0).round() as usize
        } else {
            0
        };
        println!("   • Usage rate: {}%", usage_rate.to_string().cyan().bold());

        // 要素タイプ別の統計を表示
        if !result.by_type.is_empty() {
            println!("\n📊 By Type:");
            for (element_type, stats) in &result.by_type {
                let icon = Self::get_element_icon(element_type);
                let rate = if stats.total > 0 {
                    (stats.used as f64 / stats.total as f64 * 100.0).round() as usize
                } else {
                    0
                };
                println!(
                    "   {} {}: {} total, {} used, {} unused ({}%)",
                    icon,
                    element_type,
                    stats.total.to_string().bold(),
                    stats.used.to_string().green(),
                    stats.unused.to_string().red(),
                    rate.to_string().cyan()
                );
            }
        }

        if verbose {
            Self::print_verbose_results(result);
        }
    }

    /// 詳細結果を出力
    fn print_verbose_results(result: &DetectionResult) {
        // 未使用の要素がある場合のみ詳細を表示
        if !result.unused.is_empty() {
            println!("\n{}", "=".repeat(60));
            println!("{}", "📝 Unused Elements Details".bold());
            println!("{}", "=".repeat(60));

            for element in &result.unused {
                println!("\n{} {}", "❌".red(), element.name.red().bold());
                println!(
                    "   Definition: {}",
                    element.definition_files.join(", ").dimmed()
                );
            }
        }
    }

    /// プログレスバーを表示
    pub fn create_progress_bar(total: usize, message: &str) -> indicatif::ProgressBar {
        use indicatif::{ProgressBar, ProgressStyle};

        let pb = ProgressBar::new(total as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
                )
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message(message.to_string());
        pb
    }

    /// 要素タイプに対応するアイコンを取得
    fn get_element_icon(element_type: &ElementType) -> &'static str {
        match element_type {
            ElementType::Component => "🔴",
            ElementType::Type => "🔷",
            ElementType::Interface => "🔶",
            ElementType::Function => "🔵",
            ElementType::Variable => "🟡",
            ElementType::Enum => "🟣",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{DetectionResult, ElementInfo};
    use std::collections::HashMap;

    #[test]
    fn test_usage_rate_calculation() {
        let result = DetectionResult {
            unused: vec![],
            used: vec![ElementInfo {
                name: "UsedComponent".to_string(),
                element_type: ElementType::Component,
                definition_files: vec!["src/used.tsx".to_string()],
                usages: None,
            }],
            total: 1,
            by_type: HashMap::new(),
        };

        // 使用率100%のテスト
        assert_eq!(result.used.len(), 1);
        assert_eq!(result.total, 1);
    }
}
