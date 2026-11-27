// File: src\main.rs
// Author: Hadi Cahyadi <cumulus13@gmail.com>
// Date: 2025-11-17
// Description: Simple SublimeText Project File generator
// License: MIT

use std::collections::HashSet;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;

use clap::{ArgAction, Parser};
use clap_color_help::default_styles;
use clap_version_flag::colorful_version;

const GITIGNORE_HEADER: &str = "# Ignore Sublime Text project files";

/// Pola dasar yang selalu dipastikan ada di .gitignore
const BASE_PATTERNS: &[&str] = &[
    "*.sublime-project",
    "*.sublime-workspace",
    "*.bak",
    "BUG*",
    "PLAN*",
    "ANSWER*",
    "src/tmp*",
    "download*/",
    "gitignore_backup/",
    ".pt/",
    "*__pycache__/",
    "*egg-info*/",
    "*.exe",
    "*.tmp",
    "*.env",
    "config.toml",
    "config.yaml",
    "config.json",
    "config.ini",
];

#[derive(Parser)]
#[command(
    name = "sublime-gen",
    about = "Simple SublimeText Project File generator",
    disable_version_flag = true,
    styles = default_styles()
)]
struct CLI {
    #[arg(
        short,
        long,
        help = "Directory path for the project (default: current directory)",
        default_value = ".",
        action = ArgAction::Set
    )]
    project_path: PathBuf,

    #[arg(
        short = 'n',
        long = "name",
        help = "Name of the project (default: directory name)",
        action = ArgAction::Set
    )]
    project_name: Option<String>,

    #[arg(
        short = 'i',
        long = "ignore",
        help = "Extra pattern to add to .gitignore (repeatable)",
        action = ArgAction::Append
    )]
    extra_ignore: Vec<String>,

    #[arg(
        short = 'f',
        long = "force",
        help = "Overwrite existing .sublime-project / .sublime-workspace files",
        action = ArgAction::SetTrue
    )]
    force: bool,

    #[arg(
        short = 'd',
        long = "dry-run",
        help = "Show what would be done without writing anything",
        action = ArgAction::SetTrue
    )]
    dry_run: bool,

    #[arg(
        long = "no-gitignore",
        help = "Do not touch .gitignore",
        action = ArgAction::SetTrue
    )]
    no_gitignore: bool,

    #[arg(
        short = 'V',
        long = "version",
        help = "Show version information",
        action = ArgAction::SetTrue
    )]
    version: bool,
}

// ---------------------------------------------------------------------------
// Project type detection
// ---------------------------------------------------------------------------

#[derive(Default)]
struct ProjectKind {
    rust: bool,
    node: bool,
    python: bool,
}

fn detect_kind(dir: &Path) -> ProjectKind {
    ProjectKind {
        rust: dir.join("Cargo.toml").exists(),
        node: dir.join("package.json").exists(),
        python: dir.join("pyproject.toml").exists()
            || dir.join("requirements.txt").exists()
            || dir.join("setup.py").exists(),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn get_project_name(path: &Path, user_name: &Option<String>) -> String {
    if let Some(name) = user_name {
        let name = name.trim();
        if !name.is_empty() {
            return name.to_string();
        }
    }

    // Otherwise, take it from the directory name (fallback: canonicalize for "..", etc.)
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .or_else(|| {
            fs::canonicalize(path)
                .ok()
                .and_then(|p| p.file_name().and_then(|n| n.to_str()).map(|s| s.to_string()))
        })
        .unwrap_or_else(|| "project".to_string())
}

/// Tulis file hanya jika belum ada (kecuali --force). Mendukung dry-run.
fn write_file(path: &Path, content: &str, force: bool, dry_run: bool, label: &str) -> io::Result<()> {
    let exists = path.exists();

    if exists && !force {
        println!(
            "• {} already exists, skipped (use --force to overwrite): {}",
            label,
            path.display()
        );
        return Ok(());
    }

    if dry_run {
        println!(
            "[dry-run] would {} {}: {}",
            if exists { "overwrite" } else { "create" },
            label,
            path.display()
        );
        return Ok(());
    }

    fs::write(path, content)?;
    println!(
        "✓ The {} is {}: {}",
        label,
        if exists { "overwritten" } else { "created" },
        path.display()
    );
    Ok(())
}

fn json_string_list(items: &[String], indent: usize) -> String {
    let pad = " ".repeat(indent);
    items
        .iter()
        .map(|e| format!("{}\"{}\"", pad, e))
        .collect::<Vec<_>>()
        .join(",\n")
}

// ---------------------------------------------------------------------------
// File generators
// ---------------------------------------------------------------------------

fn project_content(dir: &Path) -> String {
    let kind = detect_kind(dir);
    let mut excludes: Vec<String> = vec![".git".into()];
    if kind.rust {
        excludes.push("target".into());
    }
    if kind.node {
        excludes.push("node_modules".into());
    }
    if kind.python {
        excludes.push("__pycache__".into());
        excludes.push(".venv".into());
        excludes.push("*.egg-info".into());
    }

    format!(
        r#"{{
    "folders":
    [
        {{
            "path": ".",
            "folder_exclude_patterns":
            [
{}
            ]
        }}
    ],
    "settings":
    {{
        "tab_size": 4
    }}
}}
"#,
        json_string_list(&excludes, 16)
    )
}

const WORKSPACE_CONTENT: &str = r#"{
    "auto_complete":
    {
        "selected_items":
        [
        ]
    },
    "expanded_folders":
    [
        "/"
    ],
    "file_history":
    [
    ],
    "selected_group": 0
}
"#;

fn generate_project_file(dir: &Path, name: &str, force: bool, dry_run: bool) -> io::Result<()> {
    let file = dir.join(format!("{}.sublime-project", name));
    write_file(&file, &project_content(dir), force, dry_run, "project file")
}

fn generate_workspace_file(dir: &Path, name: &str, force: bool, dry_run: bool) -> io::Result<()> {
    let file = dir.join(format!("{}.sublime-workspace", name));
    write_file(&file, WORKSPACE_CONTENT, force, dry_run, "workspace file")
}

// ---------------------------------------------------------------------------
// Smart .gitignore
// ---------------------------------------------------------------------------

/// Kumpulkan semua pola yang diinginkan (dasar + deteksi tipe proyek + tambahan user),
/// tanpa duplikasi dan dengan urutan terjaga.
fn desired_patterns(dir: &Path, extra: &[String]) -> Vec<String> {
    let kind = detect_kind(dir);
    let mut list: Vec<String> = BASE_PATTERNS.iter().map(|s| s.to_string()).collect();

    if kind.rust {
        list.push("target/".into());
    }
    if kind.node {
        list.push("node_modules/".into());
    }
    if kind.python {
        list.push(".venv/".into());
        list.push("*.pyc".into());
    }
    list.extend(extra.iter().map(|s| s.trim().to_string()));

    let mut seen = HashSet::new();
    list.into_iter()
        .filter(|p| !p.is_empty() && seen.insert(p.clone()))
        .collect()
}

/// Normalisasi untuk perbandingan: trim spasi saja (CRLF sudah dibuang oleh .lines()).
fn normalize(line: &str) -> &str {
    line.trim()
}

fn ensure_gitignore(dir: &Path, extra: &[String], dry_run: bool) -> io::Result<()> {
    let gitignore_path = dir.join(".gitignore");
    let existing = if gitignore_path.exists() {
        fs::read_to_string(&gitignore_path)?
    } else {
        String::new()
    };

    // Pola yang sudah ada (abaikan baris kosong dan komentar)
    let present: HashSet<&str> = existing
        .lines()
        .map(normalize)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();

    let wanted = desired_patterns(dir, extra);
    let total = wanted.len();
    let missing: Vec<&String> = wanted.iter().filter(|p| !present.contains(p.as_str())).collect();
    let skipped = total - missing.len();

    if missing.is_empty() {
        println!("✓ .gitignore is up to date ({} rules already present)", skipped);
        return Ok(());
    }

    // Pertahankan gaya line ending file yang sudah ada
    let eol = if existing.contains("\r\n") { "\r\n" } else { "\n" };

    let mut block = String::new();
    if !existing.is_empty() {
        if !existing.ends_with('\n') {
            block.push_str(eol);
        }
        // Beri satu baris kosong pemisah jika belum ada
        let ends_blank = existing.ends_with("\n\n") || existing.ends_with("\r\n\r\n");
        if !ends_blank {
            block.push_str(eol);
        }
    }
    if !existing.contains(GITIGNORE_HEADER) {
        block.push_str(GITIGNORE_HEADER);
        block.push_str(eol);
    }
    for p in &missing {
        block.push_str(p);
        block.push_str(eol);
    }

    if dry_run {
        println!(
            "[dry-run] would add {} new rule(s) to {} ({} already present):",
            missing.len(),
            gitignore_path.display(),
            skipped
        );
        for p in &missing {
            println!("    + {}", p);
        }
        return Ok(());
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&gitignore_path)?;
    file.write_all(block.as_bytes())?;

    println!(
        "✓ Updated .gitignore: {} (+{} added, {} already present)",
        gitignore_path.display(),
        missing.len(),
        skipped
    );
    for p in &missing {
        println!("    + {}", p);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    let cli = CLI::parse();

    // Handle version flag
    if cli.version {
        let version = colorful_version!();
        version.print_and_exit();
    }

    // Process project path
    let project_path = if cli.project_path == Path::new(".") {
        env::current_dir().unwrap_or_else(|_| {
            eprintln!("Error: Cannot get current directory");
            process::exit(1);
        })
    } else {
        cli.project_path.clone()
    };

    // Path validation
    if !project_path.exists() {
        eprintln!("Error: Path not found: {}", project_path.display());
        process::exit(1);
    }

    // Specify the project name
    let project_name = get_project_name(&project_path, &cli.project_name);

    // Specify the target directory (if path is a file, use its parent)
    let target_dir = if project_path.is_file() {
        project_path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."))
            .to_path_buf()
    } else {
        project_path
    };

    println!(
        "Creating Sublime project '{}' in: {}{}",
        project_name,
        target_dir.display(),
        if cli.dry_run { "  (dry-run)" } else { "" }
    );
    println!();

    // Generate files
    if let Err(e) = generate_project_file(&target_dir, &project_name, cli.force, cli.dry_run) {
        eprintln!("Error creating project file: {}", e);
        process::exit(1);
    }

    if let Err(e) = generate_workspace_file(&target_dir, &project_name, cli.force, cli.dry_run) {
        eprintln!("Error creating workspace file: {}", e);
        process::exit(1);
    }

    // Update .gitignore safely
    if !cli.no_gitignore {
        if let Err(e) = ensure_gitignore(&target_dir, &cli.extra_ignore, cli.dry_run) {
            eprintln!("Warning: Failed to update .gitignore: {}", e);
        }
    }

    println!();
    if cli.dry_run {
        println!("Dry-run finished. Nothing was written.");
    } else {
        println!("Finished! The Sublime project file has been successfully created.");
    }
}
