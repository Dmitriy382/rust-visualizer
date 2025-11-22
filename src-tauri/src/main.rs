#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod analyzer;
mod models;
mod parser;
mod python_analyzer; 

use analyzer::ProjectAnalyzer;
use std::path::{Path, PathBuf};
use std::fs;
use crate::models::{ProjectStructure, ModuleType, Visibility, DependencyType, ProjectProblems, ModuleMetrics, RelationType};
use python_analyzer::PythonAnalyzer;

#[tauri::command]
async fn analyze_project(path: String) -> Result<ProjectStructure, String> {
    let project_path = PathBuf::from(path);
    
    if !project_path.exists() {
        return Err("Project path does not exist".to_string());
    }
    
    // Check Rust project
    let cargo_toml = project_path.join("Cargo.toml");
    if cargo_toml.exists() {
        let mut analyzer = ProjectAnalyzer::new(project_path);
        return analyzer.analyze()
            .map_err(|e| format!("Rust analysis failed: {}", e));
    }
    
    // Check Python project
    let python_markers = ["setup.py", "requirements.txt", "pyproject.toml", "__init__.py"];
    for marker in python_markers {
        if project_path.join(marker).exists() {
            let mut analyzer = PythonAnalyzer::new(project_path);
            return analyzer.analyze()
                .map_err(|e| format!("Python analysis failed: {}", e));
        }
    }
    
    Err("Not a valid Rust or Python project".to_string())
}


#[tauri::command]
async fn read_file_content(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))
}

#[tauri::command]
async fn save_file_content(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to save file: {}", e))
}

#[tauri::command]
async fn generate_documentation(structure: ProjectStructure) -> Result<String, String> {
    let output_path = Path::new(&structure.root_path).join("PROJECT_STRUCTURE.md");
    
    let mut doc = String::new();
    
    // Header
    doc.push_str(&format!("# 📦 Project Structure\n\n"));
    doc.push_str(&format!("**Project:** `{}`\n\n", structure.root_path));
    
    // Statistics
    doc.push_str("## 📊 Statistics\n\n");
    doc.push_str(&format!("- **Total Modules:** {}\n", structure.modules.len()));
    doc.push_str(&format!("- **Dependencies:** {}\n", structure.dependencies.len()));
    doc.push_str(&format!("- **Relationships:** {}\n\n", structure.relationships.len()));
    
    let pub_count = structure.modules.iter().filter(|m| m.visibility == Visibility::Public).count();
    let test_count = structure.modules.iter().filter(|m| m.module_type == ModuleType::Test).count();
    
    doc.push_str(&format!("- **Public Modules:** {}\n", pub_count));
    doc.push_str(&format!("- **Tests:** {}\n\n", test_count));
    
    // Module Tree
    doc.push_str("## Module Tree\n\n");
    doc.push_str("```\n");
    for module in &structure.modules {
        let indent = module.name.matches("::").count();
        let prefix = "  ".repeat(indent);
        let icon = match module.module_type {
            ModuleType::Binary => "🔷",
            ModuleType::Library => "📚",
            ModuleType::Test => "🧪",
            ModuleType::Example => "📘",
            ModuleType::Benchmark => "⚡",
            _ => "📦",
        };
        doc.push_str(&format!("{}{}  {}\n", prefix, icon, module.name));
    }
    doc.push_str("```\n\n");
    
    // Dependencies
    if !structure.dependencies.is_empty() {
        doc.push_str("## Dependencies\n\n");
        doc.push_str("| Crate | Version | Type |\n");
        doc.push_str("|-------|---------|------|\n");
        for dep in &structure.dependencies {
            let dep_type = match dep.dep_type {
                DependencyType::Normal => "Production",
                DependencyType::Dev => "Development",
                DependencyType::Build => "Build",
            };
            doc.push_str(&format!("| `{}` | {} | {} |\n", dep.name, dep.version, dep_type));
        }
        doc.push_str("\n");
    }
    
    // Modules Detail
    doc.push_str("## Modules Detail\n\n");
    for module in &structure.modules {
        doc.push_str(&format!("### {} `{}`\n\n", 
            match module.module_type {
                ModuleType::Binary => "",
                ModuleType::Library => "",
                ModuleType::Test => "",
                ModuleType::Example => "",
                ModuleType::Benchmark => "⚡",
                _ => "",
            },
            module.name
        ));
        
        doc.push_str(&format!("- **Path:** `{}`\n", module.path));
        doc.push_str(&format!("- **Visibility:** {:?}\n", module.visibility));
        doc.push_str(&format!("- **Items:** {}\n\n", module.items.len()));
        
        if !module.items.is_empty() {
            doc.push_str("**Exported Items:**\n\n");
            for item in &module.items {
                let visibility = match item.visibility {
                    Visibility::Public => "pub",
                    _ => "priv",
                };
                doc.push_str(&format!("- `{}` **{:?}** `{}`\n", 
                    visibility, item.item_type, item.name));
            }
            doc.push_str("\n");
        }
    }
    
    // Module Graph
    doc.push_str("##Module Dependencies\n\n");
    doc.push_str("```mermaid\ngraph TD\n");
    for rel in &structure.relationships {
        doc.push_str(&format!("    {}[{}] --> {}[{}]\n", 
            rel.from.replace("_", ""), rel.from,
            rel.to.replace("_", ""), rel.to
        ));
    }
    doc.push_str("```\n\n");
    
    doc.push_str("---\n");
    doc.push_str(&format!("*Generated by Rust Project Visualizer*\n"));
    
    fs::write(&output_path, doc)
        .map_err(|e| format!("Failed to write documentation: {}", e))?;
    
    Ok(output_path.display().to_string())
}

#[tauri::command]
async fn analyze_problems(structure: ProjectStructure) -> Result<ProjectProblems, String> {
    let mut analyzer = ProjectAnalyzer::new(PathBuf::from(&structure.root_path));
    analyzer.initialize_data(structure.clone());

    let cycles = analyzer.detect_cycles();
    let unused = analyzer.find_unused_modules();
    let metrics = analyzer.calculate_metrics();
    
    let mut large_modules = Vec::new();
    let mut highly_coupled = Vec::new();
    
    for (id, metric) in &metrics {
        if metric.lines_of_code > 500 {
            if let Some(module) = structure.modules.iter().find(|m| &m.id == id) {
                large_modules.push(format!("{} ({} lines)", module.name, metric.lines_of_code));
            }
        }
        if metric.incoming_deps > 10 {
            if let Some(module) = structure.modules.iter().find(|m| &m.id == id) {
                highly_coupled.push(format!("{} ({} deps)", module.name, metric.incoming_deps));
            }
        }
    }
    
    Ok(ProjectProblems {
        cycles,
        unused_modules: unused,
        large_modules,
        highly_coupled,
    })
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            analyze_project,
            read_file_content,
            save_file_content,
            generate_documentation,
            analyze_problems
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
