use tree_sitter::Parser;

fn main() {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_css::LANGUAGE.into()).unwrap();
    let src = std::fs::read_to_string("tests/formatter-test-suite/originals/test.less").unwrap();
    let tree = parser.parse(&src, None).unwrap();
    println!("{}", tree.root_node().to_sexp());
}
