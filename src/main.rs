use anyhow::Result;
use clap::Parser;
use colored::*;
use ts_unused_finder::{detect_unused_elements, Reporter};
use std::process;

#[derive(Parser)]
#[command(
    name = "ts-unused-finder",
    about = "A fast tool to find unused TypeScript/JavaScript code including React components, types, interfaces, functions, variables, and enums",
    version = "1.0.0"
)]
struct Cli {
    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Exit with error if unused components are found
    #[arg(short, long)]
    strict: bool,

    /// Path to configuration file
    #[arg(short, long)]
    config: Option<String>,


    /// Quiet output (errors only)
    #[arg(short, long)]
    quiet: bool,

    /// Number of parallel jobs
    #[arg(short, long)]
    jobs: Option<usize>,

    /// Detect TypeScript types
    #[arg(long)]
    types: bool,

    /// Detect TypeScript interfaces
    #[arg(long)]
    interfaces: bool,

    /// Detect functions
    #[arg(long)]
    functions: bool,

    /// Detect variables/constants
    #[arg(long)]
    variables: bool,

    /// Detect enums
    #[arg(long)]
    enums: bool,

    /// Detect all element types
    #[arg(long)]
    all: bool,
}


fn main() -> Result<()> {
    let cli = Cli::parse();

    // 並列処理数を設定
    if let Some(jobs) = cli.jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build_global()
            .unwrap();
    }

    // 検出実行
    let start_time = std::time::Instant::now();

    // CLIオプションから設定を作成
    let custom_config =
        if cli.all || cli.types || cli.interfaces || cli.functions || cli.variables || cli.enums {
            let mut config = ts_unused_finder::load_config(cli.config.as_deref()).unwrap_or_default();

            if cli.all {
                config.detection_types.components = true;
                config.detection_types.types = true;
                config.detection_types.interfaces = true;
                config.detection_types.functions = true;
                config.detection_types.variables = true;
                config.detection_types.enums = true;
            } else {
                config.detection_types.components = true; // Always detect components
                config.detection_types.types = cli.types;
                config.detection_types.interfaces = cli.interfaces;
                config.detection_types.functions = cli.functions;
                config.detection_types.variables = cli.variables;
                config.detection_types.enums = cli.enums;
            }

            Some(config)
        } else {
            None
        };

    let result = match detect_unused_elements(cli.config.as_deref(), custom_config) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("{} Error occurred: {}", "❌".red(), e);
            process::exit(1);
        }
    };

    let elapsed = start_time.elapsed();

    // 結果を出力
    if !cli.quiet {
        Reporter::print_results(&result, cli.verbose);

        // 実行時間を表示
        println!("\n⏱️  Execution time: {:.2}s", elapsed.as_secs_f64());

        // パフォーマンス情報
        if cli.verbose {
            println!("🚀 Accelerated by Rust implementation");
            println!("🔧 Threads used: {}", rayon::current_num_threads());
        }
    }

    // Strictモードでの終了処理
    if cli.strict && !result.unused.is_empty() {
        if !cli.quiet {
            eprintln!(
                "\n{} Found {} unused element{}",
                "❌".red(),
                result.unused.len().to_string().red().bold(),
                if result.unused.len() == 1 { "" } else { "s" }
            );
        }
        process::exit(1);
    }

    // 閾値チェック（設定ファイルから読み込み）
    if let Some(config) = load_config_for_threshold_check(cli.config.as_deref()) {
        if let Some(ci_config) = config.ci {
            if result.unused.len() > ci_config.max_unused_elements && ci_config.fail_on_exceed {
                if !cli.quiet {
                    eprintln!(
                        "\n{} Unused elements exceed threshold: {} > {}",
                        "⚠️".yellow(),
                        result.unused.len().to_string().yellow().bold(),
                        ci_config.max_unused_elements.to_string().green()
                    );
                }
                process::exit(1);
            }
        }
    }

    Ok(())
}

fn load_config_for_threshold_check(config_path: Option<&str>) -> Option<ts_unused_finder::Config> {
    ts_unused_finder::load_config(config_path).ok()
}

#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use predicates::prelude::*;

    #[test]
    fn test_help_command() {
        let mut cmd = Command::cargo_bin("ts-unused-finder").unwrap();
        cmd.arg("--help");
        cmd.assert().success().stdout(predicate::str::contains(
            "A fast tool to find unused TypeScript/JavaScript code",
        ));
    }

    #[test]
    fn test_version_command() {
        let mut cmd = Command::cargo_bin("ts-unused-finder").unwrap();
        cmd.arg("--version");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("1.0.0"));
    }

    #[test]
    fn test_quiet_output() {
        let mut cmd = Command::cargo_bin("ts-unused-finder").unwrap();
        cmd.args(&["--quiet"]);
        cmd.assert().success();
    }
}
