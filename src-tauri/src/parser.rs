use crate::models::*;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use syn::{visit::Visit, Item as SynItem, UseTree, Visibility as SynVis};

pub struct RustParser {
    current_module: String,
    items: Vec<Item>,
    uses: Vec<String>,
}

impl RustParser {
    pub fn new() -> Self {
        Self {
            current_module: String::new(),
            items: Vec::new(),
            uses: Vec::new(),
        }
    }

    pub fn parse_file(&mut self, path: &Path, module_path: &str) -> Result<Module> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        self.current_module = module_path.to_string();
        self.items.clear();
        self.uses.clear();

        let syntax = syn::parse_file(&content)
            .with_context(|| format!("Failed to parse file: {}", path.display()))?;

        self.visit_file(&syntax);

        let module_type = Self::determine_module_type(path);
        let id = module_path.replace("::", "_");

        Ok(Module {
            id,
            name: module_path.to_string(),
            path: path.display().to_string(),
            module_type,
            visibility: Visibility::Public,
            items: self.items.clone(),
        })
    }

    fn determine_module_type(path: &Path) -> ModuleType {
        let path_str = path.to_string_lossy();
        if path_str.contains("tests/") || path_str.contains("test.rs") {
            ModuleType::Test
        } else if path_str.contains("examples/") {
            ModuleType::Example
        } else if path_str.contains("benches/") {
            ModuleType::Benchmark
        } else if path_str.contains("main.rs") {
            ModuleType::Binary
        } else if path_str.contains("lib.rs") {
            ModuleType::Library
        } else {
            ModuleType::Module
        }
    }

    fn convert_visibility(vis: &SynVis) -> Visibility {
        match vis {
            SynVis::Public(_) => Visibility::Public,
            SynVis::Restricted(r) => {
                let path = &r.path;
                if path.is_ident("crate") {
                    Visibility::Crate
                } else if path.is_ident("super") {
                    Visibility::Super
                } else {
                    Visibility::Private
                }
            }
            SynVis::Inherited => Visibility::Private,
        }
    }

    pub fn get_uses(&self) -> Vec<String> {
        self.uses.clone()
    }
}

impl<'ast> Visit<'ast> for RustParser {
    fn visit_item(&mut self, item: &'ast SynItem) {
        match item {
            SynItem::Fn(func) => {
                self.items.push(Item {
                    name: func.sig.ident.to_string(),
                    item_type: ItemType::Function,
                    visibility: Self::convert_visibility(&func.vis),
                });
            }
            SynItem::Struct(s) => {
                self.items.push(Item {
                    name: s.ident.to_string(),
                    item_type: ItemType::Struct,
                    visibility: Self::convert_visibility(&s.vis),
                });
            }
            SynItem::Enum(e) => {
                self.items.push(Item {
                    name: e.ident.to_string(),
                    item_type: ItemType::Enum,
                    visibility: Self::convert_visibility(&e.vis),
                });
            }
            SynItem::Trait(t) => {
                self.items.push(Item {
                    name: t.ident.to_string(),
                    item_type: ItemType::Trait,
                    visibility: Self::convert_visibility(&t.vis),
                });
            }
            SynItem::Const(c) => {
                self.items.push(Item {
                    name: c.ident.to_string(),
                    item_type: ItemType::Const,
                    visibility: Self::convert_visibility(&c.vis),
                });
            }
            SynItem::Static(s) => {
                self.items.push(Item {
                    name: s.ident.to_string(),
                    item_type: ItemType::Static,
                    visibility: Self::convert_visibility(&s.vis),
                });
            }
            SynItem::Type(t) => {
                self.items.push(Item {
                    name: t.ident.to_string(),
                    item_type: ItemType::Type,
                    visibility: Self::convert_visibility(&t.vis),
                });
            }
            SynItem::Macro(m) => {
                if let Some(ident) = &m.ident {
                    self.items.push(Item {
                        name: ident.to_string(),
                        item_type: ItemType::Macro,
                        visibility: Visibility::Public,
                    });
                }
            }
            _ => {}
        }
        syn::visit::visit_item(self, item);
    }

    fn visit_item_use(&mut self, use_item: &'ast syn::ItemUse) {
        self.extract_use_paths(&use_item.tree);
        syn::visit::visit_item_use(self, use_item);
    }
}

impl RustParser {
    fn extract_use_paths(&mut self, tree: &UseTree) {
        match tree {
            UseTree::Path(p) => {
                let path = p.ident.to_string();
                self.extract_use_paths(&p.tree);
                if !path.is_empty() {
                    self.uses.push(path);
                }
            }
            UseTree::Name(n) => {
                self.uses.push(n.ident.to_string());
            }
            UseTree::Rename(r) => {
                self.uses.push(r.ident.to_string());
            }
            UseTree::Glob(_) => {}
            UseTree::Group(g) => {
                for item in &g.items {
                    self.extract_use_paths(item);
                }
            }
        }
    }
}
