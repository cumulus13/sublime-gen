// File: src\main.rs
// Author: Hadi Cahyadi <cumulus13@gmail.com>
// Date: 2025-11-17
// Description: Simple SublimeText Project File generator
// License: MIT

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use clap::{Parser, ArgAction};
use clap_version_flag::colorful_version;

#[derive(Parser)]
#[command(
    name = "sublime-gen",
    about = "Simple SublimeText Project File generator",
    disable_version_flag = true
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
        short = 'V',
        long = "version",
        help = "Show version information",
        action = ArgAction::SetTrue
    )]
    version: bool,
}

fn get_project_name(path: &Path, user_name: &Option<String>) -> String {
    if let Some(name) = user_name {
        if !name.is_empty() {
            return name.clone();
        }
    }
    
    // Otherwise, take it from the directory name
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string()
}

fn generate_project_file(path: &Path, name: &str) -> std::io::Result<()> {
    // Make sure the path is a directory
    let path = if path.is_file() {
        path.parent().unwrap_or(path)
    } else {
        path
    };

    let project_file = path.join(format!("{}.sublime-project", name));
    let mut file = File::create(&project_file)?;
    
    let content = r#"{
    "folders":
    [
        {
            "path": "."
        }
    ],
    "settings":
    {
        "tab_size": 4
    }
}
"#;
    
    file.write_all(content.as_bytes())?;
    println!("✓ The project file is created: {}", project_file.display());
    Ok(())
}

fn generate_workspace_file(path: &Path, name: &str) -> std::io::Result<()> {
    // Make sure the path is a directory
    let path = if path.is_file() {
        path.parent().unwrap_or(path)
    } else {
        path
    };

    let workspace_file = path.join(format!("{}.sublime-workspace", name));
    let mut file = File::create(&workspace_file)?;
    
    let content = r#"{
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
    
    file.write_all(content.as_bytes())?;
    println!("✓ The workspace file is created: {}", workspace_file.display());
    Ok(())
}

fn main() {
    let cli = CLI::parse();

    // Handle version flag
    if cli.version {
        let version = colorful_version!();
        version.print_and_exit();
    }

    // Process project path
    let project_path = if cli.project_path == Path::new(".") {
        // If default or user input is ".", use current directory
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
        project_path.parent().unwrap_or(&project_path).to_path_buf()
    } else {
        project_path
    };

    println!("Creating Sublime project '{}' in: {}", project_name, target_dir.display());
    println!();
    
    // Generate files
    if let Err(e) = generate_project_file(&target_dir, &project_name) {
        eprintln!("Error creating project file: {}", e);
        process::exit(1);
    }
    
    if let Err(e) = generate_workspace_file(&target_dir, &project_name) {
        eprintln!("Error creating workspace file: {}", e);
        process::exit(1);
    }
    
    println!();
    println!("Finished! The Sublime project file has been successfully created.");
}