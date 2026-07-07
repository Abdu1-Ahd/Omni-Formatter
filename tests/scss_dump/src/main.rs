
fn main() {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_css::language())
        .unwrap();
    let src = ".card {\n  .mixin();\n  .box-shadow(0 2px 4px, rgba(0,0,0,0.2));\n}";
    let tree = parser.parse(src, None).unwrap();
    let root = tree.root_node();

    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        println!(
            "{:?} - {}",
            child.kind(),
            &src[child.start_byte()..child.end_byte()]
        );
        for n in child.children(&mut root.walk()) {
            println!("  {:?} - {}", n.kind(), &src[n.start_byte()..n.end_byte()]);
            for nn in n.children(&mut root.walk()) {
                println!(
                    "    {:?} - {}",
                    nn.kind(),
                    &src[nn.start_byte()..nn.end_byte()]
                );
                for nnn in nn.children(&mut root.walk()) {
                    println!(
                        "      {:?} - {}",
                        nnn.kind(),
                        &src[nnn.start_byte()..nnn.end_byte()]
                    );
                }
            }
        }
    }
}
