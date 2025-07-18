pub mod nodejs;
pub mod python;
pub mod rust;
pub mod java;
pub mod cpp;
pub mod web;
pub mod general;
pub mod dotnet;
pub mod go;
pub mod ruby;
pub mod php;
pub mod swift;

use regex::Regex;
use nodejs::NodejsPatterns;
use python::PythonPatterns;
use rust::RustPatterns;
use java::JavaPatterns;
use cpp::CppPatterns;
use web::WebPatterns;
use general::GeneralPatterns;
use dotnet::DotnetPatterns;
use go::GoPatterns;
use ruby::RubyPatterns;
use php::PhpPatterns;
use swift::SwiftPatterns;

pub struct ExternalPatterns {
    patterns: Vec<Regex>,
}

impl ExternalPatterns {
    pub fn new() -> Self {
        let mut patterns = Vec::new();
        
        // Add patterns from each language/technology
        let nodejs = NodejsPatterns::new();
        patterns.extend(nodejs.get_external_patterns().iter().cloned());
        
        let python = PythonPatterns::new();
        patterns.extend(python.get_external_patterns().iter().cloned());
        
        let rust = RustPatterns::new();
        patterns.extend(rust.get_external_patterns().iter().cloned());
        
        let java = JavaPatterns::new();
        patterns.extend(java.get_external_patterns().iter().cloned());
        
        let cpp = CppPatterns::new();
        patterns.extend(cpp.get_external_patterns().iter().cloned());
        
        let web = WebPatterns::new();
        patterns.extend(web.get_external_patterns().iter().cloned());
        
        let general = GeneralPatterns::new();
        patterns.extend(general.get_external_patterns().iter().cloned());

        // Add new ecosystems
        let dotnet = DotnetPatterns::new();
        patterns.extend(dotnet.get_external_patterns().iter().cloned());
        
        let go = GoPatterns::new();
        patterns.extend(go.get_external_patterns().iter().cloned());
        
        let ruby = RubyPatterns::new();
        patterns.extend(ruby.get_external_patterns().iter().cloned());
        
        let php = PhpPatterns::new();
        patterns.extend(php.get_external_patterns().iter().cloned());
        
        let swift = SwiftPatterns::new();
        patterns.extend(swift.get_external_patterns().iter().cloned());

        Self { patterns }
    }

    pub fn matches(&self, path_str: &str) -> bool {
        self.patterns.iter().any(|pattern| pattern.is_match(path_str))
    }
}

pub struct CodeExtensions {
    extensions: Vec<String>,
}

impl CodeExtensions {
    pub fn new() -> Self {
        let mut extensions = Vec::new();
        
        // Add extensions from each language/technology
        let nodejs = NodejsPatterns::new();
        extensions.extend(nodejs.get_extensions().iter().cloned());
        
        let python = PythonPatterns::new();
        extensions.extend(python.get_extensions().iter().cloned());
        
        let rust = RustPatterns::new();
        extensions.extend(rust.get_extensions().iter().cloned());
        
        let java = JavaPatterns::new();
        extensions.extend(java.get_extensions().iter().cloned());
        
        let cpp = CppPatterns::new();
        extensions.extend(cpp.get_extensions().iter().cloned());
        
        let web = WebPatterns::new();
        extensions.extend(web.get_extensions().iter().cloned());
        
        let general = GeneralPatterns::new();
        extensions.extend(general.get_extensions().iter().cloned());

        // Add new ecosystems
        let dotnet = DotnetPatterns::new();
        extensions.extend(dotnet.get_extensions().iter().cloned());
        
        let go = GoPatterns::new();
        extensions.extend(go.get_extensions().iter().cloned());
        
        let ruby = RubyPatterns::new();
        extensions.extend(ruby.get_extensions().iter().cloned());
        
        let php = PhpPatterns::new();
        extensions.extend(php.get_extensions().iter().cloned());
        
        let swift = SwiftPatterns::new();
        extensions.extend(swift.get_extensions().iter().cloned());

        Self { extensions }
    }

    pub fn contains(&self, extension: &str) -> bool {
        self.extensions.contains(&extension.to_string())
    }

    pub fn get_script_names() -> Vec<&'static str> {
        let mut script_names = Vec::new();
        
        script_names.extend(NodejsPatterns::get_script_names());
        script_names.extend(PythonPatterns::get_script_names());
        script_names.extend(RustPatterns::get_script_names());
        script_names.extend(JavaPatterns::get_script_names());
        script_names.extend(CppPatterns::get_script_names());
        script_names.extend(WebPatterns::get_script_names());
        script_names.extend(GeneralPatterns::get_script_names());
        
        // Add new ecosystems
        script_names.extend(DotnetPatterns::get_script_names());
        script_names.extend(GoPatterns::get_script_names());
        script_names.extend(RubyPatterns::get_script_names());
        script_names.extend(PhpPatterns::get_script_names());
        script_names.extend(SwiftPatterns::get_script_names());
        
        script_names
    }
}

 