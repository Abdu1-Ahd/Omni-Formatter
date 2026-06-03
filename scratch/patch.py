import os

path = "crates/core/src/lib.rs"
with open(path, "r") as f:
    content = f.read()

start_marker = "#[wasm_bindgen]\npub fn format(request_json: &str) -> String {"

new_format = """#[wasm_bindgen]
pub fn format(request_json: &str) -> String {
    js_log("1. Entered format()");
    js_log(&format!("Request starts with: {}", &request_json.chars().take(20).collect::<String>()));
    
    // Test a basic string return
    js_log("2. Returning mock response");
    r#"{"edits": [], "formatter_chain": "mock", "is_noop": true}"#.to_string()
}
"""

start_idx = content.find(start_marker)
if start_idx == -1:
    print("Could not find start marker")
    exit(1)

# Find the end of the format function. It ends right before `fn language_id_to_ext`
end_marker = "fn language_id_to_ext"
end_idx = content.find(end_marker)
if end_idx == -1:
    print("Could not find end marker")
    exit(1)

# The end index is the `/// Map VS Code...` comment before language_id_to_ext
end_idx = content.rfind("/// Map VS Code", start_idx, end_idx)

new_content = content[:start_idx] + new_format + "\n" + content[end_idx:]

with open(path, "w") as f:
    f.write(new_content)

print("Patched lib.rs")
