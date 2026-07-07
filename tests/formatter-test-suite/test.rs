// ── CASE 1: Function spacing — should normalize ────────────────────────────
fn   main ( ) {
    let x=1;
    let y = 2 ;
}

// ── CASE 2: rustfmt::skip — must preserve the next item verbatim ───────────
#[rustfmt::skip]
let preserved_matrix = [
    1,0,0,
    0,1,0,
    0,0,1,
];

// ── CASE 3: Match expressions ─────────────────────────────────────────────
fn classify(n: i32) -> &'static str {
    match n {
        0          => "zero",
        1..=9   => "single digit",
        10..=99     => "double digit",
        _ => "large",
    }
}

// ── CASE 4: Long lines — method chains ────────────────────────────────────
fn chain_example() {
    let result = vec![1, 2, 3].into_iter().map(|x| x * 2).filter(|x| x % 3 == 0).collect::<Vec<_>>();
    let long_variable_name_that_forces_a_line_wrap_because_it_exceeds_one_hundred_characters_in_total_length = 42;
}

// ── CASE 5: Struct definitions — mixed spacing ────────────────────────────
struct Point {
    x:f64,
    y :f64,
    z : f64,
}

impl Point {
    fn new( x: f64 , y: f64 , z: f64 ) -> Self {
        Self { x ,y ,z }
    }

    fn distance(&self,other:&Point)->f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx*dx + dy*dy + dz*dz).sqrt()
    }
}

// ── CASE 6: Trait implementation ──────────────────────────────────────────
trait Animal {
    fn name(&self) -> &str;
    fn sound(&self) -> &str;
    fn describe(&self) -> String {
        format!("{} says {}", self.name(), self.sound())
    }
}

struct Dog { name: String }

impl Animal for Dog {
    fn name(&self)->&str { &self.name }
    fn sound(&self) -> &str { "woof" }
}

// ── CASE 7: Enums with variants ───────────────────────────────────────────
enum Shape {
    Circle { radius: f64 },
    Rectangle{width:f64,height:f64},
    Triangle( f64 , f64 , f64 ),
}

// ── CASE 8: Generics and lifetime annotations ─────────────────────────────
fn longest<'a>(x: &'a str,y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

struct Important<'a> {
    content:&'a str,
}

// ── CASE 9: Closures — trailing whitespace ────────────────────────────────
fn apply<F: Fn(i32) -> i32>(f: F, value: i32) -> i32 {   
    f(value)   
}

// ── CASE 10: use statements — grouped imports ─────────────────────────────
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::{self, Read, Write};

// ── CASE 11: Async functions ──────────────────────────────────────────────
async fn fetch_data(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::get(url).await?;
    let text = response.text().await?;
    Ok(text)
}

// ── CASE 12: Macro invocations ────────────────────────────────────────────
fn macro_examples() {
    println!( "hello {}" , "world" );
    vec![ 1,2,3 ];
    assert_eq!( 1+1 , 2 );
    eprintln!("error: {:?}", some_value);
}
