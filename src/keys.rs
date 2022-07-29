use openssl::rsa::Rsa;

use openssl::pkey::Private;
use std::fs;

pub fn generate_rsa(bits: usize) -> Rsa<Private> {
    openssl::rsa::Rsa::generate(bits as u32).unwrap()
}

// pub fn print_rsa_pem(key_pair: Rsa<Private>) {
//     println!("{}", String::from_utf8(key_pair.public_key_to_pem().unwrap()).unwrap());
//     println!("{}", String::from_utf8(key_pair.private_key_to_pem().unwrap()).unwrap());
// }

pub fn load_rsa_pem(private_path: &str) -> Rsa<Private> {
    let private_key = fs::read(private_path).unwrap();
    Rsa::<Private>::private_key_from_pem(&private_key).unwrap()
}
