use trace_caller::trace;

#[trace]
fn add(x: u32, y: u32) -> u32 {
    x + y
}

fn main() {
    let result = add(3, 4);
    println!("Result: {}", result);
}
