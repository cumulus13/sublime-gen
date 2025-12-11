// File: src\main.rs
// Author: Hadi Cahyadi <cumulus13@gmail.com>
// Date: 2025-11-17
// Description: Simple SublimeText Project File generator
// License: MIT

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process;

fn print_help() {

    println!("Sublime Project Generator v{} by {}", env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_AUTHORS"));
    println!();
    println!("Usage:");
    println!("  sublime-gen [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("  -p, --path <PATH>    Directory path for the project (default: current directory)");
    println!("  -h, --help           Display this help message");
    println!();
    println!("EXAMPLE:");
    println!("  sublime-gen                    # Generate in the current directory");
    println!("  sublime-gen -p /path/to/dir    # Generate on a specific path");
}

fn generate_project_file(path: &PathBuf) -> std::io::Result<()> {
    let project_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");
    
    let project_file = path.join(format!("{}.sublime-project", project_name));
    let mut file = File::create(&project_file)?;
    
    let content = format!(
        r#"{{
    "folders":
    [
        {{
            "path": "."
        }}
    ],
    "settings":
    {{
        "tab_size": 4
    }}
}}
"#
    );
    
    file.write_all(content.as_bytes())?;
    println!("✓ The project file is created: {}", project_file.display());
    Ok(())
}

fn generate_workspace_file(path: &PathBuf) -> std::io::Result<()> {
    let project_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");
    
    let workspace_file = path.join(format!("{}.sublime-workspace", project_name));
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
    let args: Vec<String> = env::args().collect();
    
    // Parse arguments
    let mut path = env::current_dir().unwrap_or_else(|_| {
        eprintln!("Error: Cannot get current directory");
        process::exit(1);
    });
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                return;
            }
            "-p" | "--path" => {
                if i + 1 < args.len() {
                    path = PathBuf::from(&args[i + 1]);
                    i += 1;
                } else {
                    eprintln!("Error: The -p/--path option requires a PATH argument");
                    eprintln!("Use -h for help");
                    process::exit(1);
                }
            }
            _ => {
                eprintln!("Error: Unknown option: {}", args[i]);
                eprintln!("Use -h for help");
                process::exit(1);
            }
        }
        i += 1;
    }
    
    // Path validation
    if !path.exists() {
        eprintln!("Error: Path not found: {}", path.display());
        process::exit(1);
    }
    
    if !path.is_dir() {
        eprintln!("Error: Path is not a directory: {}", path.display());
        process::exit(1);
    }
    
    println!("Create a Sublime project in: {}", path.display());
    println!();
    
    // Generate files
    if let Err(e) = generate_project_file(&path) {
        eprintln!("Error creating project file: {}", e);
        process::exit(1);
    }
    
    if let Err(e) = generate_workspace_file(&path) {
        eprintln!("Error membuat file workspace: {}", e);
        process::exit(1);
    }
    
    println!();
    println!("Selesai! File Sublime project berhasil dibuat.");
}