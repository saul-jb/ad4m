use deno_core::anyhow;
use deno_core::error::generic_error;
use deno_core::ModuleLoader;
use deno_core::ModuleSource;
use deno_core::ModuleSourceFuture;
use deno_core::ModuleSpecifier;
use deno_core::ModuleType;
use deno_core::ResolutionKind;
use deno_runtime::deno_core::error::AnyError;
use tracing::info;
use url::Url;
use std::collections::HashMap;
use std::pin::Pin;

pub struct StringModuleLoader {
    modules: HashMap<String, String>,
}

impl StringModuleLoader {
    pub fn new() -> Self {
        StringModuleLoader {
            modules: HashMap::new(),
        }
    }

    pub fn add_module(&mut self, specifier: &str, code: &str) {
        self.modules.insert(specifier.to_string(), code.to_string());
    }
}

impl ModuleLoader for StringModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _kind: ResolutionKind,
    ) -> Result<ModuleSpecifier, AnyError> {
        let module_specifier = deno_core::resolve_import(specifier, referrer)?;
        Ok(module_specifier)
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: std::option::Option<&Url>,
        _is_dyn_import: bool,
    ) -> Pin<Box<ModuleSourceFuture>> {
        let path = module_specifier.to_file_path().map_err(|_| {
            generic_error(format!(
                "Provided module specifier \"{module_specifier}\" is not a file URL."
            ))
        });
        match path {
            Ok(path) => {
                let module_type = if let Some(extension) = path.extension() {
                    let ext = extension.to_string_lossy().to_lowercase();
                    if ext == "json" {
                        ModuleType::Json
                    } else {
                        ModuleType::JavaScript
                    }
                } else {
                    ModuleType::JavaScript
                };

                let code =
                    std::fs::read_to_string(path).expect("Could not read file path to string");
                let module_specifier = module_specifier.clone();
                let fut = async move {
                    Ok(ModuleSource::new(module_type, code.into(), &module_specifier))
                };
                Box::pin(fut)
            }
            Err(_err) => {
                info!("Module is not a file path, importing as raw module string");
                let module_code = self.modules.get(module_specifier.as_str()).cloned();
                let module_specifier = module_specifier.clone();
                let fut = async move {
                    match module_code {
                        Some(code) => Ok(ModuleSource::new(deno_core::ModuleType::JavaScript, code.into(), &module_specifier)),
                        None => Err(anyhow::anyhow!("Module not found: {}", module_specifier)),
                    }
                };
                Box::pin(fut)
            }
        }
    }
}
