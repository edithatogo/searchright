#![forbid(unsafe_code)]
fn main() {
    let result = rustls_native_certs::load_native_certs();
    println!("certificates={} errors={}", result.certs.len(), result.errors.len());
    for error in result.errors { println!("error_context={}", error.context); }
}
