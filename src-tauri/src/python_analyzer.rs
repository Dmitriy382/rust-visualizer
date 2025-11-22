use crate::models::*;
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct PythonAnalyzer {
    root_path: PathBuf,
    modules: Vec<Module>,
    dependencies: Vec<Dependency>,
    relationships: Vec<Relationship>,
}

impl PythonAnalyzer {
    pub fn new(root_path: PathBuf) -> Self {
        Self {
            root_path,
            modules: Vec::new(),
            dependencies: Vec::new(),
            relationships: Vec::new(),
        }
    }

    pub fn analyze(&mut self) -> Result<ProjectStructure> {
        self.parse_requirements()?;
        self.walk_python_files()?;
        self.build_relationships();

        Ok(ProjectStructure {
            root_path: self.root_path.display().to_string(),
            modules: self.modules.clone(),
            dependencies: self.dependencies.clone(),
            relationships: self.relationships.clone(),
        })
    }

    fn parse_requirements(&mut self) -> Result<()> {
        let req_files = ["requirements.txt", "setup.py", "pyproject.toml"];
        
        for req_file in req_files {
            let path = self.root_path.join(req_file);
            if path.exists() {
                if req_file == "requirements.txt" {
                    let content = fs::read_to_string(&path)?;
                    for line in content.lines() {
                        let line = line.trim();
                        if line.is_empty() || line.starts_with('#') {
                            continue;
                        }
                        
                        let parts: Vec<&str> = line.split("==").collect();
                        let name = parts[0].trim().to_string();
                        let version = parts.get(1).unwrap_or(&"*").trim().to_string();
                        
                        self.dependencies.push(Dependency {
                            name,
                            version,
                            dep_type: DependencyType::Normal,
                        });
                    }
                }
                break;
            }
        }
        Ok(())
    }

    fn walk_python_files(&mut self) -> Result<()> {
        for entry in WalkDir::new(&self.root_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("py") {
                let relative_path = path.strip_prefix(&self.root_path).unwrap_or(path);
                let module_path = self.path_to_module_name(relative_path);
                
                if let Ok((module, uses)) = self.parse_python_file(path, &module_path) {
                    let from_id = module.id.clone();
                    self.modules.push(module);

                    for use_path in uses {
                        self.relationships.push(Relationship {
                            from: from_id.clone(),
                            to: use_path.replace(".", "_"),
                            rel_type: RelationType::Uses,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    fn parse_python_file(&mut self, path: &Path, module_path: &str) -> Result<(Module, Vec<String>)> {
        let content = fs::read_to_string(path)?;
        let mut items = Vec::new();
        let mut uses = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            
            // Parse imports
            if line.starts_with("import ") || line.starts_with("from ") {
                if let Some(module) = self.extract_import(line) {
                    uses.push(module);
                }
            }
            
            // Parse functions
            if line.starts_with("def ") {
                if let Some(func_name) = self.extract_function_name(line) {
                    let visibility = if func_name.starts_with('_') {
                        Visibility::Private
                    } else {
                        Visibility::Public
                    };
                    
                    items.push(Item {
                        name: func_name,
                        item_type: ItemType::Function,
                        visibility,
                    });
                }
            }
            
            // Parse classes
            if line.starts_with("class ") {
                if let Some(class_name) = self.extract_class_name(line) {
                    let visibility = if class_name.starts_with('_') {
                        Visibility::Private
                    } else {
                        Visibility::Public
                    };
                    
                    items.push(Item {
                        name: class_name,
                        item_type: ItemType::Struct,
                        visibility,
                    });
                }
            }
        }

        let module_type = self.determine_module_type(path);
        let id = module_path.replace(".", "_").replace("/", "_");

        Ok((
            Module {
                id,
                name: module_path.to_string(),
                path: path.display().to_string(),
                module_type,
                visibility: Visibility::Public,
                items,
            },
            uses
        ))
    }

    fn extract_import(&self, line: &str) -> Option<String> {
        if line.starts_with("import ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            parts.get(1).map(|s| s.to_string())
        } else if line.starts_with("from ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            parts.get(1).map(|s| s.to_string())
        } else {
            None
        }
    }

    fn extract_function_name(&self, line: &str) -> Option<String> {
        let start = line.find("def ")? + 4;
        let end = line[start..].find('(')?;
        Some(line[start..start + end].trim().to_string())
    }

    fn extract_class_name(&self, line: &str) -> Option<String> {
        let start = line.find("class ")? + 6;
        let end = line[start..].find(|c| c == '(' || c == ':').unwrap_or(line.len() - start);
        Some(line[start..start + end].trim().to_string())
    }

    fn determine_module_type(&self, path: &Path) -> ModuleType {
        let path_str = path.to_string_lossy();
        if path_str.contains("test_") || path_str.contains("/tests/") {
            ModuleType::Test
        } else if path_str.contains("/examples/") {
            ModuleType::Example
        } else if path_str.ends_with("__main__.py") {
            ModuleType::Binary
        } else {
            ModuleType::Module
        }
    }

    fn path_to_module_name(&self, path: &Path) -> String {
        let path_str = path.to_string_lossy();
        path_str
            .trim_end_matches(".py")
            .replace("/__init__", "")
            .replace('/', ".")
            .replace('\\', ".")
    }

    fn build_relationships(&mut self) {
        // Build parent-child relationships
        for module in &self.modules {
            let parts: Vec<&str> = module.name.split('.').collect();
            if parts.len() > 1 {
                let parent_name = parts[..parts.len() - 1].join(".");
                let parent_id = parent_name.replace(".", "_");
                
                if self.modules.iter().any(|m| m.id == parent_id) {
                    self.relationships.push(Relationship {
                        from: parent_id,
                        to: module.id.clone(),
                        rel_type: RelationType::Declares,
                    });
                }
            }
        }
    }
}
