fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("wasm32") {
        cc::Build::new()
            .file("src/wasm_c_stubs.c")
            .flag("-Wno-unused-parameter")
            .compile("wasm_c_stubs");
    }
}
