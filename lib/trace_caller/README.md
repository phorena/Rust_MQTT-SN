# trace_caller
<p align="left">
  <a href="https://travis-ci.com/github/ayushmishra2005/trace_caller">
    <img alt="Build Status" src="https://travis-ci.com/ayushmishra2005/trace_caller.svg?branch=master">
  </a>
  <a href="https://crates.io/crates/trace_caller">
      <img alt="Latest version" src="https://img.shields.io/badge/crates.io-v0.2.0-orange.svg?longCache=true">
    </a>

  <img alt="MIT licensed" src="https://img.shields.io/badge/license-MIT-blue.svg">
  <img alt="Stability stable" src="https://img.shields.io/badge/stability-stable-green.svg">
</p>


This procedural attribute macro will allow retrieving the caller's source location in the stable Rust. Rust has also this feature, but it is currently unstable and available in nightly-only build. But with this attribute, you can use this feature on stable Rust.

It will help to trace the location of the caller of a function.

## Example

```rust
use trace_caller::trace;

#[trace]
fn add(x: u32, y: u32) -> u32 {
    x + y
}

fn main() {
    let result = add(3, 4);
    println!("Result: {}", result);
}
```

## Result
```text
Called from "src/main.rs" at line 11
Result: 7
```
