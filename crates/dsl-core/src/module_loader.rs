use crate::parser::{parse_program, Program};
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A loaded module with its path and parsed program
#[derive(Debug, Clone)]
pub struct Module {
    pub path: PathBuf,
    pub program: Program,
}

/// Module loader that handles file loading, caching, and dependency resolution
#[derive(Debug)]
pub struct ModuleLoader {
    /// Cache of loaded modules (keyed by absolute path)
    loaded_modules: HashMap<PathBuf, Module>,
    /// Stack of currently loading modules (for circular dependency detection)
    loading_stack: Vec<PathBuf>,
}

impl ModuleLoader {
    /// Create a new module loader
    pub fn new() -> Self {
        Self {
            loaded_modules: HashMap::new(),
            loading_stack: Vec::new(),
        }
    }

    /// Load a module from a path, relative to the current file
    ///
    /// # Arguments
    /// * `import_path` - The import path string (e.g., "./math.dsl", "../lib/utils.dsl")
    /// * `current_file` - The path of the file doing the importing (used to resolve relative paths)
    ///
    /// # Returns
    /// The loaded module, which may be from cache
    pub fn load_module(&mut self, import_path: &str, current_file: &Path) -> Result<Module> {
        // Resolve the import path to an absolute path
        let absolute_path = self.resolve_path(import_path, current_file)?;

        // Check if already loaded (cache hit)
        if let Some(cached_module) = self.loaded_modules.get(&absolute_path) {
            return Ok(cached_module.clone());
        }

        // Check for circular dependency
        if self.loading_stack.contains(&absolute_path) {
            let mut chain = String::from("Circular dependency detected:\n");
            for (i, path) in self.loading_stack.iter().enumerate() {
                chain.push_str(&format!("  {} -> {}\n", i + 1, path.display()));
            }
            chain.push_str(&format!(
                "  {} -> {} ⟲ (cycle!)\n",
                self.loading_stack.len() + 1,
                absolute_path.display()
            ));
            chain.push_str(
                "\nHint: Refactor shared code into a separate module that both files can import",
            );
            return Err(anyhow!(chain));
        }

        // Push to loading stack
        self.loading_stack.push(absolute_path.clone());

        // Read the file
        let source = std::fs::read_to_string(&absolute_path).map_err(|e| {
            self.loading_stack.pop(); // Clean up stack on error
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow!(
                    "Module not found: {}\n\nImported from: {}\n\nHints:\n  • Check the file path is correct\n  • Paths are relative to the importing file\n  • Use './' for same directory, '../' for parent\n  • File extension (.dsl) is required",
                    import_path,
                    current_file.display()
                )
            } else {
                anyhow!("Failed to read module {}: {}", absolute_path.display(), e)
            }
        })?;

        // Parse the module
        let program = parse_program(&source).map_err(|e| {
            self.loading_stack.pop(); // Clean up stack on error
            anyhow!("Parse error in module {}:\n{}", absolute_path.display(), e)
        })?;

        // Recursively load all imports in this module
        let imports = program.imports.clone();
        for import in imports {
            self.load_module(&import.path, &absolute_path)?;
        }

        // Pop from loading stack
        self.loading_stack.pop();

        // Create module and cache it
        let module = Module {
            path: absolute_path.clone(),
            program,
        };
        self.loaded_modules.insert(absolute_path, module.clone());

        Ok(module)
    }

    /// Resolve an import path to an absolute path
    fn resolve_path(&self, import_path: &str, current_file: &Path) -> Result<PathBuf> {
        // Validate the import path
        if !import_path.ends_with(".dsl") {
            return Err(anyhow!(
                "Invalid import path: {}\n\nImport paths must:\n  • End with '.dsl' extension",
                import_path
            ));
        }

        if !import_path.starts_with("./") && !import_path.starts_with("../") {
            return Err(anyhow!(
                "Invalid import path: {}\n\nImport paths must:\n  • Be relative paths starting with './' or '../'",
                import_path
            ));
        }

        // Get the directory of the current file
        let current_dir = if current_file.is_file() {
            current_file
                .parent()
                .ok_or_else(|| anyhow!("Invalid current file path: {}", current_file.display()))?
        } else {
            current_file
        };

        // Join with the import path
        let mut absolute_path = current_dir.join(import_path);

        // Canonicalize to resolve .. and . and get absolute path
        absolute_path = absolute_path.canonicalize().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow!(
                    "Module not found: {}\n\nImported from: {}\n\nHints:\n  • Check the file path is correct\n  • Paths are relative to the importing file",
                    import_path,
                    current_file.display()
                )
            } else {
                anyhow!("Failed to resolve import path {}: {}", import_path, e)
            }
        })?;

        Ok(absolute_path)
    }

    /// Get all loaded modules
    pub fn get_all_modules(&self) -> Vec<&Module> {
        self.loaded_modules.values().collect()
    }

    /// Get a module by its absolute path
    pub fn get_module(&self, path: &Path) -> Option<&Module> {
        self.loaded_modules.get(path)
    }

    /// Clear all loaded modules (useful for testing)
    pub fn clear(&mut self) {
        self.loaded_modules.clear();
        self.loading_stack.clear();
    }
}

impl Default for ModuleLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_simple_import() {
        let temp_dir = TempDir::new().unwrap();
        let math_path = temp_dir.path().join("math.dsl");
        let main_path = temp_dir.path().join("main.dsl");

        fs::write(&math_path, "def add(a, b) => a + b end").unwrap();
        fs::write(&main_path, "import \"./math.dsl\"\nadd(1, 2)").unwrap();

        let mut loader = ModuleLoader::new();
        let module = loader.load_module("./math.dsl", &main_path).unwrap();

        assert_eq!(module.program.functions.len(), 1);
        assert_eq!(module.program.functions[0].name, "add");
    }

    #[test]
    fn test_circular_dependency() {
        let temp_dir = TempDir::new().unwrap();
        let a_path = temp_dir.path().join("a.dsl");
        let b_path = temp_dir.path().join("b.dsl");

        fs::write(&a_path, "import \"./b.dsl\"").unwrap();
        fs::write(&b_path, "import \"./a.dsl\"").unwrap();

        let mut loader = ModuleLoader::new();
        let result = loader.load_module("./b.dsl", &a_path);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Circular dependency"));
    }

    #[test]
    fn test_caching() {
        let temp_dir = TempDir::new().unwrap();
        let math_path = temp_dir.path().join("math.dsl");
        let main_path = temp_dir.path().join("main.dsl");

        fs::write(&math_path, "def add(a, b) => a + b end").unwrap();
        fs::write(&main_path, "import \"./math.dsl\"").unwrap();

        let mut loader = ModuleLoader::new();

        // Load once
        let module1 = loader.load_module("./math.dsl", &main_path).unwrap();
        assert_eq!(loader.loaded_modules.len(), 1);

        // Load again - should be cached
        let module2 = loader.load_module("./math.dsl", &main_path).unwrap();
        assert_eq!(loader.loaded_modules.len(), 1);

        // Should be the same module
        assert_eq!(module1.path, module2.path);
    }

    #[test]
    fn test_invalid_path() {
        let temp_dir = TempDir::new().unwrap();
        let main_path = temp_dir.path().join("main.dsl");

        let mut loader = ModuleLoader::new();

        // Missing .dsl extension
        let result = loader.load_module("./math", &main_path);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Invalid import path"));

        // Not relative path
        let result = loader.load_module("math.dsl", &main_path);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Invalid import path"));
    }
}
