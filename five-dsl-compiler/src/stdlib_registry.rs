use std::path::PathBuf;

pub const STDLIB_PREFIX: &str = "std::";

pub fn is_stdlib_module(module_path: &str) -> bool {
    module_path == "std" || module_path.starts_with(STDLIB_PREFIX)
}

pub fn bundled_stdlib_source(module_path: &str) -> Option<&'static str> {
    match module_path {
        "std::prelude" => Some(include_str!("../../five-stdlib/std/prelude.v")),
        "std::builtins" => Some(include_str!("../../five-stdlib/std/builtins.v")),
        "std::interfaces::system_program" => Some(include_str!(
            "../../five-stdlib/std/interfaces/system_program.v"
        )),
        "std::interfaces::spl_token" => {
            Some(include_str!("../../five-stdlib/std/interfaces/spl_token.v"))
        }
        _ => None,
    }
}

pub fn bundled_stdlib_virtual_path(module_path: &str) -> PathBuf {
    let rel = module_path.replace("::", "/");
    PathBuf::from(format!("<bundled-stdlib>/{}.v", rel))
}
