use std::str;

fn main() {
    let source = b"@base: #f938ab;\n@padding-small: 3px;\n";
    let mut parser = tree_sitter::Parser::new();
    let lang = tree_sitter_css::LANGUAGE.into();
    parser.set_language(&lang).unwrap();
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        eprintln!("kind={:?} named={} text={:?}", child.kind(), child.is_named(), std::str::from_utf8(&source[child.byte_range()]).unwrap());
        let mut c2 = child.walk();
        for gc in child.children(&mut c2) {
            eprintln!("  child kind={:?} named={} field={:?} text={:?}", gc.kind(), gc.is_named(), child.field_name_for_child(gc.id() as u32), std::str::from_utf8(&source[gc.byte_range()]).unwrap());
        }
    }
}
