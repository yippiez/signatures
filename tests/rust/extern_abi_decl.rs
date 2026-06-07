extern "C" fn c_abi() -> i32;
pub unsafe extern "C" fn full_c_abi(x: *mut u8) -> bool;
extern "Rust" fn rust_abi() {};
extern "system" fn sys_abi() {};
