fn parse_input(input: &str) {
    let _ = input.len();
}

// rustfmt::skip
fn magic_table() {
    let _table = [
        1, 2,
        3, 4,
    ];
}

fn chained_methods() {
    let v = vec![1, 2, 3];
    v.into_iter().map(|x| x + 1).filter(|x| *x > 2).collect::<Vec<_>>();
}
