use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn foo(x: i32, y: i32) -> i32 {
    x * y
}
