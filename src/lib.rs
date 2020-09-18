#[no_mangle]
/// # Safety
/// This is not very safe.
pub unsafe extern "C" fn run() {
    println!("Hello from libfoobar.so");
}
