use anyhow::{anyhow, Context, Result};
use regex::Regex;
use serde_json::json;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::{DirEntry, WalkDir};

use crate::common_helperfuncs::{path, pathp, PathE};

#[derive(Debug)]
struct LintIssue {
    file: Option<String>,
    line: Option<u32>,
    column: Option<u32>,
    severity: String,
    code: Option<String>,
    message: String,
}

pub fn runit() -> Result<()> {
    println!("Nifty TypeScript lint");

    let roots = lint_roots();
    print_roots(&roots);

    let files = collect_typescript_files(&roots)?;
    if files.is_empty() {
        println!("\nNo TypeScript files found.");
        return Ok(());
    }

    println!(
        "\nScanning {} TypeScript files with tsc --noEmit...",
        files.len()
    );

    let config_path = write_tsc_config(&files)?;
    let cwd =
        common_ancestor(&roots).unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    let tsc_result = run_tsc(&config_path, &cwd);

    let _ = fs::remove_file(&config_path);

    let (output, tsc_success) = tsc_result?;
    let issues = parse_tsc_output(&output);

    print_issues(&issues, output.trim());

    if issues.is_empty() && tsc_success {
        Ok(())
    } else if issues.is_empty() {
        Err(anyhow!("tsc failed without parseable diagnostics"))
    } else {
        let errors = count_severity(&issues, "error");
        let warnings = count_severity(&issues, "warning");
        Err(anyhow!(
            "lint found {} error{} and {} warning{}",
            errors,
            plural(errors),
            warnings,
            plural(warnings)
        ))
    }
}

fn lint_roots() -> Vec<(String, PathBuf)> {
    vec![
        ("core client".to_string(), path(PathE::ClientSrc)),
        ("core server".to_string(), path(PathE::ServerSrc)),
        (
            "instance client".to_string(),
            path(PathE::InstanceClientSrc),
        ),
        (
            "instance server".to_string(),
            path(PathE::InstanceServerSrc),
        ),
    ]
}

fn print_roots(roots: &[(String, PathBuf)]) {
    println!("Roots:");
    for (label, root) in roots {
        println!("  • {:15} {}", label, root.display());
    }
}

fn collect_typescript_files(roots: &[(String, PathBuf)]) -> Result<Vec<PathBuf>> {
    let mut seen = HashSet::new();
    let mut files = Vec::new();

    for (_, root) in roots {
        if !root.exists() {
            println!("  ! skipping missing root: {}", root.display());
            continue;
        }

        for entry in WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| should_descend(entry))
        {
            let entry =
                entry.with_context(|| format!("failed to read under {}", root.display()))?;
            let path = entry.path();

            if !path.is_file() || !is_typescript_file(path) {
                continue;
            }

            let path = path.to_path_buf();
            let key = path.to_string_lossy().to_string();
            if seen.insert(key) {
                files.push(path);
            }
        }
    }

    files.sort();
    Ok(files)
}

fn should_descend(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    !matches!(
        name.as_ref(),
        ".git"
            | "node_modules"
            | "build"
            | "dist"
            | "coverage"
            | "static_dev"
            | "static_dist"
            | ".next"
            | ".turbo"
    )
}

fn is_typescript_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("ts" | "tsx" | "mts" | "cts")
    )
}

fn write_tsc_config(files: &[PathBuf]) -> Result<PathBuf> {
    let config_dir = path(PathE::TMPDirConfigs);
    fs::create_dir_all(&config_dir)
        .with_context(|| format!("failed to create lint config dir {}", config_dir.display()))?;

    let config_path = pathp(
        PathE::TMPDirConfigs,
        format!("nifty-lint-tsconfig-{}.json", std::process::id()),
    );

    let file_list: Vec<String> = files
        .iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect();

    let config = json!({
        "compilerOptions": {
            "noEmit": true,
            "target": "ESNext",
            "module": "NodeNext",
            "moduleResolution": "NodeNext",
            "lib": ["ESNext", "DOM", "DOM.Iterable", "WebWorker"],
            "skipLibCheck": true,
            "allowImportingTsExtensions": true,
            "allowSyntheticDefaultImports": true,
            "esModuleInterop": true,
            "resolveJsonModule": true,
            "strict": false,
            "strictNullChecks": false,
            "noImplicitAny": false,
            "strictPropertyInitialization": false
        },
        "files": file_list
    });

    fs::write(&config_path, serde_json::to_string_pretty(&config)?)
        .with_context(|| format!("failed to write lint tsconfig {}", config_path.display()))?;

    Ok(config_path)
}

fn run_tsc(config_path: &Path, cwd: &Path) -> Result<(String, bool)> {
    let output = Command::new("tsc")
        .arg("-p")
        .arg(config_path)
        .arg("--pretty")
        .arg("false")
        .arg("--noErrorTruncation")
        .current_dir(cwd)
        .output()
        .with_context(|| "failed to execute tsc. Install TypeScript or put tsc on PATH")?;

    let mut combined = String::new();
    combined.push_str(&String::from_utf8_lossy(&output.stdout));
    combined.push_str(&String::from_utf8_lossy(&output.stderr));

    if !output.status.success() && combined.trim().is_empty() {
        return Err(anyhow!("tsc failed with status {}", output.status));
    }

    Ok((combined, output.status.success()))
}

fn parse_tsc_output(output: &str) -> Vec<LintIssue> {
    let file_re = Regex::new(r#"^(.*)\((\d+),(\d+)\):\s+(error|warning)\s+(TS\d+):\s+(.*)$"#)
        .expect("valid TypeScript file diagnostic regex");
    let global_re = Regex::new(r#"^(error|warning)\s+(TS\d+):\s+(.*)$"#)
        .expect("valid TypeScript global diagnostic regex");

    let mut issues = Vec::new();
    let mut current: Option<LintIssue> = None;

    for line in output.lines() {
        if let Some(captures) = file_re.captures(line) {
            push_current(&mut issues, &mut current);
            current = Some(LintIssue {
                file: Some(captures[1].to_string()),
                line: captures[2].parse().ok(),
                column: captures[3].parse().ok(),
                severity: captures[4].to_string(),
                code: Some(captures[5].to_string()),
                message: captures[6].to_string(),
            });
        } else if let Some(captures) = global_re.captures(line) {
            push_current(&mut issues, &mut current);
            current = Some(LintIssue {
                file: None,
                line: None,
                column: None,
                severity: captures[1].to_string(),
                code: Some(captures[2].to_string()),
                message: captures[3].to_string(),
            });
        } else if let Some(issue) = current.as_mut() {
            if !line.trim().is_empty() {
                issue.message.push('\n');
                issue.message.push_str(line.trim());
            }
        }
    }

    push_current(&mut issues, &mut current);
    issues
}

fn push_current(issues: &mut Vec<LintIssue>, current: &mut Option<LintIssue>) {
    if let Some(issue) = current.take() {
        issues.push(issue);
    }
}

fn print_issues(issues: &[LintIssue], raw_output: &str) {
    if issues.is_empty() {
        if raw_output.is_empty() {
            println!("\n✓ No TypeScript errors or warnings found.");
        } else {
            println!("\nTypeScript output:\n{}", raw_output);
        }
        return;
    }

    let errors = count_severity(issues, "error");
    let warnings = count_severity(issues, "warning");
    println!(
        "\n✗ Found {} error{} and {} warning{}:\n",
        errors,
        plural(errors),
        warnings,
        plural(warnings)
    );

    let mut last_file: Option<&str> = None;
    for issue in issues {
        let file = issue.file.as_deref().unwrap_or("<global>");
        if last_file != Some(file) {
            println!("{}", file);
            last_file = Some(file);
        }

        let icon = if issue.severity == "warning" {
            "⚠"
        } else {
            "✖"
        };
        let location = match (issue.line, issue.column) {
            (Some(line), Some(column)) => format!("{}:{}", line, column),
            _ => "-".to_string(),
        };
        let code = issue.code.as_deref().unwrap_or("TS");

        let mut lines = issue.message.lines();
        if let Some(first_line) = lines.next() {
            println!("  {} {:>8} {} {}", icon, location, code, first_line);
        }
        for line in lines {
            println!("             {}", line);
        }
    }
}

fn count_severity(issues: &[LintIssue], severity: &str) -> usize {
    issues
        .iter()
        .filter(|issue| issue.severity == severity)
        .count()
}

fn plural(count: usize) -> &'static str {
    if count == 1 {
        ""
    } else {
        "s"
    }
}

fn common_ancestor(roots: &[(String, PathBuf)]) -> Option<PathBuf> {
    let existing_roots: Vec<PathBuf> = roots
        .iter()
        .map(|(_, path)| path.clone())
        .filter(|path| path.exists())
        .collect();

    let mut ancestor = existing_roots.first()?.clone();
    while !existing_roots
        .iter()
        .all(|path| path.starts_with(&ancestor))
    {
        if !ancestor.pop() {
            return None;
        }
    }

    Some(ancestor)
}
