use crate::models::*;
use crate::parser::RustParser;
use anyhow::{Context, Result};
use cargo_metadata::MetadataCommand;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct ProjectAnalyzer {
    root_path: PathBuf,
    modules: Vec<Module>,
    dependencies: Vec<Dependency>,
    relationships: Vec<Relationship>,
}

impl ProjectAnalyzer {
    pub fn new(root_path: PathBuf) -> Self {
        Self {
            root_path,
            modules: Vec::new(),
            dependencies: Vec::new(),
            relationships: Vec::new(),
        }
    }

    pub fn initialize_data(&mut self, structure: ProjectStructure) {
        self.modules = structure.modules;
        self.dependencies = structure.dependencies;
        self.relationships = structure.relationships;
    }
    
    pub fn detect_cycles(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut rec_stack = std::collections::HashSet::new();
        
        for module in &self.modules {
            if !visited.contains(&module.id) {
                self.dfs_cycle(&module.id, &mut visited, &mut rec_stack, &mut Vec::new(), &mut cycles);
            }
        }
        
        cycles
    }
    
    fn dfs_cycle(
        &self,
        node: &str,
        visited: &mut std::collections::HashSet<String>,
        rec_stack: &mut std::collections::HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());
        
        for rel in &self.relationships {
            if rel.from == node {
                if !visited.contains(&rel.to) {
                    self.dfs_cycle(&rel.to, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(&rel.to) {
                    if let Some(pos) = path.iter().position(|x| x == &rel.to) {
                        cycles.push(path[pos..].to_vec());
                    }
                }
            }
        }
        
        path.pop();
        rec_stack.remove(node);
    }
    
    pub fn calculate_metrics(&self) -> std::collections::HashMap<String, ModuleMetrics> {
        use std::collections::HashMap;
        let mut metrics = HashMap::new();
        
        for module in &self.modules {
            let incoming = self.relationships.iter()
                .filter(|r| r.to == module.id)
                .count();
            let outgoing = self.relationships.iter()
                .filter(|r| r.from == module.id)
                .count();
            
            let lines = std::fs::read_to_string(&module.path)
                .map(|c| c.lines().count())
                .unwrap_or(0);
            
            metrics.insert(module.id.clone(), ModuleMetrics {
                lines_of_code: lines,
                incoming_deps: incoming,
                outgoing_deps: outgoing,
                complexity_score: module.items.len(),
            });
        }
        
        metrics
    }
    
    pub fn find_unused_modules(&self) -> Vec<String> {
        let mut unused = Vec::new();
        
        for module in &self.modules {
            let is_used = self.relationships.iter()
                .any(|r| r.to == module.id && r.rel_type == RelationType::Uses);
            
            let is_entry = module.module_type == ModuleType::Binary 
                        || module.module_type == ModuleType::Library;
            
            if !is_used && !is_entry {
                unused.push(module.name.clone());
            }
        }
        
        unused
    }




    pub fn analyze(&mut self) -> Result<ProjectStructure> {
        println!("Analyzing project at: {:?}", self.root_path);

        // Parse Cargo.toml and dependencies
        self.parse_dependencies()
            .context("Failed to parse dependencies")?;

        // Walk through source files
        self.walk_source_files()
            .context("Failed to walk source files")?;

        // Build relationships
        self.build_relationships();

        Ok(ProjectStructure {
            root_path: self.root_path.display().to_string(),
            modules: self.modules.clone(),
            dependencies: self.dependencies.clone(),
            relationships: self.relationships.clone(),
        })
    }

    fn parse_dependencies(&mut self) -> Result<()> {
        let metadata = MetadataCommand::new()
            .manifest_path(self.root_path.join("Cargo.toml"))
            .exec()
            .context("Failed to execute cargo metadata")?;

        for package in &metadata.packages {
            for dep in &package.dependencies {
                let dep_type = match dep.kind {
                    cargo_metadata::DependencyKind::Normal => DependencyType::Normal,
                    cargo_metadata::DependencyKind::Development => DependencyType::Dev,
                    cargo_metadata::DependencyKind::Build => DependencyType::Build,
                    _ => DependencyType::Normal,
                };

                self.dependencies.push(Dependency {
                    name: dep.name.clone(),
                    version: dep.req.to_string(),
                    dep_type,
                });
            }
        }

        Ok(())
    }

    fn walk_source_files(&mut self) -> Result<()> {
        let src_dir = self.root_path.join("src");
        if !src_dir.exists() {
            return Ok(());
        }

        for entry in WalkDir::new(&src_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                let relative_path = path
                    .strip_prefix(&self.root_path)
                    .unwrap_or(path)
                    .to_path_buf();

                let module_path = self.path_to_module_name(&relative_path);

                let mut parser = RustParser::new();
                match parser.parse_file(path, &module_path) {
                    Ok(module) => {
                        let uses = parser.get_uses();
                        let from_id = module.id.clone();

                        self.modules.push(module);

                        // Create relationships from use statements
                        for use_path in uses {
                            self.relationships.push(Relationship {
                                from: from_id.clone(),
                                to: use_path.replace("::", "_"),
                                rel_type: RelationType::Uses,
                            });
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to parse {}: {}", path.display(), e);
                    }
                }
            }
        }

        // Check for tests, examples, benches
        self.walk_additional_dirs("tests")?;
        self.walk_additional_dirs("examples")?;
        self.walk_additional_dirs("benches")?;

        Ok(())
    }

    fn walk_additional_dirs(&mut self, dir_name: &str) -> Result<()> {
        let dir = self.root_path.join(dir_name);
        if !dir.exists() {
            return Ok(());
        }

        for entry in WalkDir::new(&dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                let relative_path = path
                    .strip_prefix(&self.root_path)
                    .unwrap_or(path)
                    .to_path_buf();

                let module_path = self.path_to_module_name(&relative_path);

                let mut parser = RustParser::new();
                match parser.parse_file(path, &module_path) {
                    Ok(module) => {
                        self.modules.push(module);
                    }
                    Err(e) => {
                        eprintln!("Failed to parse {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(())
    }

    fn path_to_module_name(&self, path: &Path) -> String {
        let path_str = path.to_string_lossy();
        let path_str = path_str
            .trim_start_matches("src/")
            .trim_start_matches("tests/")
            .trim_start_matches("examples/")
            .trim_start_matches("benches/")
            .trim_end_matches(".rs");

        path_str
            .replace("/mod", "")
            .replace("/", "::")
            .replace("\\", "::")
    }

    fn build_relationships(&mut self) {
        // Build parent-child relationships for modules
        let mut module_map: HashMap<String, String> = HashMap::new();

        for module in &self.modules {
            module_map.insert(module.id.clone(), module.name.clone());
        }

        for module in &self.modules {
            let parts: Vec<&str> = module.name.split("::").collect();
            if parts.len() > 1 {
                let parent_name = parts[..parts.len() - 1].join("::");
                let parent_id = parent_name.replace("::", "_");

                if module_map.contains_key(&parent_id) {
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
