use core::registry::PluginRegistry;
use protocol::ConfigIR;
use std::env;
use std::fs;
use std::path::Path;

fn build_registry() -> PluginRegistry {
    let mut registry = PluginRegistry::new();
    registry.register(Box::new(lang_js::plugin::JsPlugin));
    registry.register(Box::new(lang_css::plugin::CssPlugin));
    registry.register(Box::new(lang_python::plugin::PythonPlugin));
    registry.register(Box::new(lang_rust::plugin::RustPlugin));
    registry.register(Box::new(lang_go::plugin::GoPlugin));
    registry
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 || args[1] != "format" {
        eprintln!("Usage: native_runner format <file> [--output <outfile>]");
        std::process::exit(1);
    }

    let input_file = &args[2];
    let mut output_file = input_file.clone();
    if args.len() >= 5 && args[3] == "--output" {
        output_file = args[4].clone();
    }

    let path = Path::new(input_file);

    // Strip .out / .out2 / .ref suffixes to recover the real extension
    let real_ext = {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if matches!(ext, "out" | "out2" | "ref") {
            let stem = Path::new(path.file_stem().unwrap_or_default());
            stem.extension()
                .and_then(|e| e.to_str())
                .unwrap_or(ext)
                .to_string()
        } else {
            ext.to_string()
        }
    };

    let source = fs::read(path).expect("Failed to read input file");
    let config = ConfigIR::default();

    let registry = build_registry();
    let result = registry
        .format_by_ext(&real_ext, &source, &config)
        .map_err(|e| e.to_string());

    match result {
        Ok(formatted) => {
            fs::write(&output_file, &formatted).expect("Failed to write output");
        }
        Err(e) => {
            eprintln!("Formatting error for .{}: {}", real_ext, e);
            std::process::exit(1);
        }
    }
}
