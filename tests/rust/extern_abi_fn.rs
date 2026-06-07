extern "C" fn c_style() -> i32 { 1 }
pub extern "C" fn pub_c_style() -> i32 { 2 }
extern "Rust" fn rust_abi() -> i32 { 3 }
extern fn default_abi() -> i32 { 4 }
fn normal() -> i32 { 5 }
