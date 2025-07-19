use crate::utils::errors::Result;
use super::types::{FunctionInfo, StructureInfo};

// Language-specific modules
pub mod rust;
pub mod python;
pub mod javascript;
pub mod java;
pub mod cpp;
pub mod go;
pub mod csharp;
pub mod php;
pub mod ruby;
pub mod swift;
pub mod kotlin;
pub mod dart;
pub mod erlang;
pub mod perl;
pub mod r;
pub mod matlab;
pub mod elixir;
pub mod julia;
pub mod lua;
pub mod zig;
pub mod clojure;
pub mod haskell;

/// Common trait for all language-specific complexity analyzers
pub trait LanguageAnalyzer {
    /// Analyze functions in code lines for complexity metrics
    fn analyze_functions(&self, lines: &[String]) -> Result<Vec<FunctionInfo>>;
    
    /// Analyze structures in code lines (classes, interfaces, etc.)
    fn analyze_structures(&self, lines: &[String]) -> Result<Vec<StructureInfo>>;
    
    /// Get the language name for this analyzer
    fn language_name(&self) -> &'static str;
    
    /// Get the file extensions this analyzer supports
    fn supported_extensions(&self) -> Vec<&'static str>;
}

/// Factory function to get the appropriate language analyzer
pub fn get_language_analyzer(extension: &str) -> Option<Box<dyn LanguageAnalyzer>> {
    match extension {
        "rs" => Some(Box::new(rust::RustAnalyzer::new())),
        "py" => Some(Box::new(python::PythonAnalyzer::new())),
        "js" | "jsx" | "ts" | "tsx" => Some(Box::new(javascript::JavaScriptAnalyzer::new())),
        "java" => Some(Box::new(java::JavaAnalyzer::new())),
        "cpp" | "cc" | "cxx" | "c" | "h" | "hpp" => Some(Box::new(cpp::CppAnalyzer::new())),
        "go" => Some(Box::new(go::GoAnalyzer::new())),
        "cs" => Some(Box::new(csharp::CSharpAnalyzer::new())),
        "php" => Some(Box::new(php::PhpAnalyzer::new())),
        "rb" => Some(Box::new(ruby::RubyAnalyzer::new())),
        "swift" => Some(Box::new(swift::SwiftAnalyzer::new())),
        "kt" => Some(Box::new(kotlin::KotlinAnalyzer::new())),
        "dart" => Some(Box::new(dart::DartAnalyzer::new())),
        "erl" | "hrl" => Some(Box::new(erlang::ErlangAnalyzer::new())),
        "pl" | "pm" => Some(Box::new(perl::PerlAnalyzer::new())),
        "r" | "R" => Some(Box::new(r::RAnalyzer::new())),
        "m" | "mlx" => Some(Box::new(matlab::MatlabAnalyzer::new())),
        "ex" | "exs" => Some(Box::new(elixir::ElixirAnalyzer::new())),
        "jl" => Some(Box::new(julia::JuliaAnalyzer::new())),
        "lua" => Some(Box::new(lua::LuaAnalyzer::new())),
        "zig" => Some(Box::new(zig::ZigAnalyzer::new())),
        "clj" | "cljs" | "cljc" | "edn" => Some(Box::new(clojure::ClojureAnalyzer::new())),
        "hs" | "lhs" => Some(Box::new(haskell::HaskellAnalyzer::new())),
        _ => None,
    }
} 