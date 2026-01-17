// Example: Generate a test password hash
// Run with: cargo run --example generate_test_password

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, Algorithm, Params, Version,
};

const MEMORY_SIZE: u32 = 65536;
const ITERATIONS: u32 = 3;
const PARALLELISM: u32 = 4;
const OUTPUT_LENGTH: usize = 32;

fn main() {
    let password = "TestPass123!";
    
    let salt = SaltString::generate(&mut OsRng);
    
    let params = Params::new(
        MEMORY_SIZE,
        ITERATIONS,
        PARALLELISM,
        Some(OUTPUT_LENGTH)
    ).expect("Failed to create Argon2 params");
    
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password");
    
    println!("Password: {}", password);
    println!("Hash: {}", password_hash.to_string());
}
