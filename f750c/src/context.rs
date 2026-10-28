use std::collections::HashMap;
use crate::CompilerInput;
use crate::value::SymbolName;

#[derive(Debug)]
pub struct CompilerContext {
    pub input: CompilerInput,
    source_file_ctx: HashMap<String, SourceFileContext>,
}

impl CompilerContext {
    pub fn new(input: CompilerInput) -> Self {
        Self {
            input,
            source_file_ctx: HashMap::new(),
        }
    }

    pub fn has_namespace(&self, namespace: &str) -> bool {
        self.input.sources.contains_key(namespace)
    }

    pub fn get_source_ctx(&self, namespace: &str) -> Option<&SourceFileContext> {
        self.source_file_ctx.get(namespace)
    }

    pub fn get_source_ctx_mut(&mut self, namespace: &str) -> &mut SourceFileContext {
        self.source_file_ctx.entry(namespace.to_string())
            .or_insert_with(|| SourceFileContext::new(namespace))
    }
}

#[derive(Debug, Clone)]
pub struct SourceFileContext {
    pub namespace: String,
    namespace_alias_map: HashMap<String, String>,
}

impl SourceFileContext {
    pub fn new(namespace: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
            namespace_alias_map: HashMap::new(),
        }
    }

    pub fn add_alias(&mut self, alias: &str, namespace: &str) {
        self.namespace_alias_map.insert(alias.to_string(), namespace.to_string());
    }

    pub fn has_alias(&self, alias: &str) -> bool {
        self.namespace_alias_map.contains_key(alias)
    }
    
    pub fn resolve_namespace(&self, namespace: &str) -> Option<String> {
        if self.namespace == namespace {
            return Some(self.namespace.clone());
        }

        if let Some(ret) = self.namespace_alias_map.get(namespace) {
            return Some(ret.clone());
        }

        None
    }

    pub fn resolve_namespace_symbol(&self, symbol: &mut SymbolName) {
        let Some(ns) = &symbol.namespace else {
            return;
        };

        if let Some(resolved) = self.resolve_namespace(ns) {
            symbol.namespace = Some(resolved);
        }
    }
}