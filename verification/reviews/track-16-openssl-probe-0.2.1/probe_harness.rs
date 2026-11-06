// Review-only probe: uses safe APIs; never mutates this process environment.
fn main() {
    println!("ENV_RECOGNISED:{}", openssl_probe::has_ssl_cert_env_vars());
    let result = openssl_probe::probe();
    if let Some(file) = result.cert_file {
        println!("FILE:{}", file.display());
    }
    for dir in result.cert_dir {
        println!("DIR:{}", dir.display());
    }
    for dir in openssl_probe::candidate_cert_dirs() {
        println!("CANDIDATE:{}", dir.display());
    }
}
