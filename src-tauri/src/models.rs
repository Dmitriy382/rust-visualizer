use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStructure {
    pub root_path: String,
    pub modules: Vec<Module>,
    pub dependencies: Vec<Dependency>,
    pub relationships: Vec<Relationship>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub id: String,
    pub name: String,
    pub path: String,
    pub module_type: ModuleType,
    pub visibility: Visibility,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(PartialEq)]
pub enum ModuleType {
    Binary,
    Library,
    Module,
    Test,
    Example,
    Benchmark,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(PartialEq)]
pub enum Visibility {
    Public,
    Private,
    Crate,
    Super,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub item_type: ItemType,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemType {
    Function,
    Struct,
    Enum,
    Trait,
    Const,
    Static,
    Type,
    Macro,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub dep_type: DependencyType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DependencyType {
    Normal,
    Dev,
    Build,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub from: String,
    pub to: String,
    pub rel_type: RelationType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RelationType {
    Uses,
    Declares,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleMetrics {
    pub lines_of_code: usize,
    pub incoming_deps: usize,
    pub outgoing_deps: usize,
    pub complexity_score: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectProblems {
    pub cycles: Vec<Vec<String>>,
    pub unused_modules: Vec<String>,
    pub large_modules: Vec<String>,
    pub highly_coupled: Vec<String>,
}

