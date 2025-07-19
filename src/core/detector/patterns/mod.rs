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
pub mod kotlin;
pub mod haskell;
pub mod elixir;
pub mod julia;
pub mod lua;
pub mod zig;
pub mod clojure;
pub mod erlang;
pub mod dart;
pub mod perl;
pub mod r;
pub mod matlab;

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
use kotlin::KotlinPatterns;
use haskell::HaskellPatterns;
use elixir::ElixirPatterns;
use julia::JuliaPatterns;
use lua::LuaPatterns;
use zig::ZigPatterns;
use clojure::ClojurePatterns;
use erlang::ErlangPatterns;
use dart::DartPatterns;
use perl::PerlPatterns;
use r::RPatterns;
use matlab::MatlabPatterns;

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
        
        let kotlin = KotlinPatterns::new();
        patterns.extend(kotlin.get_external_patterns().iter().cloned());
        
        let haskell = HaskellPatterns::new();
        patterns.extend(haskell.get_external_patterns().iter().cloned());
        
        let elixir = ElixirPatterns::new();
        patterns.extend(elixir.get_external_patterns().iter().cloned());
        
        let julia = JuliaPatterns::new();
        patterns.extend(julia.get_external_patterns().iter().cloned());
        
        let lua = LuaPatterns::new();
        patterns.extend(lua.get_external_patterns().iter().cloned());
        
        let zig = ZigPatterns::new();
        patterns.extend(zig.get_external_patterns().iter().cloned());
        
        let clojure = ClojurePatterns::new();
        patterns.extend(clojure.get_external_patterns().iter().cloned());
        
        let erlang = ErlangPatterns::new();
        patterns.extend(erlang.get_external_patterns().iter().cloned());
        
        let dart = DartPatterns::new();
        patterns.extend(dart.get_external_patterns().iter().cloned());
        
        let perl = PerlPatterns::new();
        patterns.extend(perl.get_external_patterns().iter().cloned());
        
        let r = RPatterns::new();
        patterns.extend(r.get_external_patterns().iter().cloned());
        
        let matlab = MatlabPatterns::new();
        patterns.extend(matlab.get_external_patterns().iter().cloned());

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
        
        let kotlin = KotlinPatterns::new();
        extensions.extend(kotlin.get_extensions().iter().cloned());
        
        let haskell = HaskellPatterns::new();
        extensions.extend(haskell.get_extensions().iter().cloned());
        
        let elixir = ElixirPatterns::new();
        extensions.extend(elixir.get_extensions().iter().cloned());
        
        let julia = JuliaPatterns::new();
        extensions.extend(julia.get_extensions().iter().cloned());
        
        let lua = LuaPatterns::new();
        extensions.extend(lua.get_extensions().iter().cloned());
        
        let zig = ZigPatterns::new();
        extensions.extend(zig.get_extensions().iter().cloned());
        
        let clojure = ClojurePatterns::new();
        extensions.extend(clojure.get_extensions().iter().cloned());
        
        let erlang = ErlangPatterns::new();
        extensions.extend(erlang.get_extensions().iter().cloned());
        
        let dart = DartPatterns::new();
        extensions.extend(dart.get_extensions().iter().cloned());
        
        let perl = PerlPatterns::new();
        extensions.extend(perl.get_extensions().iter().cloned());
        
        let r = RPatterns::new();
        extensions.extend(r.get_extensions().iter().cloned());
        
        let matlab = MatlabPatterns::new();
        extensions.extend(matlab.get_extensions().iter().cloned());

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
        script_names.extend(DotnetPatterns::get_script_names());
        script_names.extend(GoPatterns::get_script_names());
        script_names.extend(RubyPatterns::get_script_names());
        script_names.extend(PhpPatterns::get_script_names());
        script_names.extend(SwiftPatterns::get_script_names());
        script_names.extend(KotlinPatterns::get_script_names());
        script_names.extend(HaskellPatterns::get_script_names());
        script_names.extend(ElixirPatterns::get_script_names());
        script_names.extend(JuliaPatterns::get_script_names());
        script_names.extend(LuaPatterns::get_script_names());
        script_names.extend(ZigPatterns::get_script_names());
        script_names.extend(ClojurePatterns::get_script_names());
        script_names.extend(ErlangPatterns::get_script_names());
        script_names.extend(DartPatterns::get_script_names());
        script_names.extend(PerlPatterns::get_script_names());
        script_names.extend(RPatterns::get_script_names());
        script_names.extend(MatlabPatterns::get_script_names());
        
        script_names
    }
}

 