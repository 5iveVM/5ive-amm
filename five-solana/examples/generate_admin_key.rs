/// Simple example to generate an admin keypair for FIVE VM
///
/// Usage: cargo run --example generate_admin_key
///
/// This will generate a random keypair and save it to admin_key.json
/// The public key will be printed to stdout for use in the program.
use std::fs;
use std::path::Path;

fn main() {
    // Generate a random keypair (using a simple approach)
    // In production, you'd use proper crypto libraries
    let mut seed = [0u8; 32];

    // Simple deterministic seed for testing (you should use proper randomness in production)
    for i in 0..32 {
        seed[i] = (i + 42) as u8;
    }

    let public_key = derive_public_key(&seed);

    // Print the public key in array format for use in the program
    println!("Admin Public Key (for five-solana/src/common.rs):");
    println!("pub const ADMIN_KEY: Pubkey = [");
    for (i, byte) in public_key.iter().enumerate() {
        if i > 0 && i % 8 == 0 {
            println!();
        }
        print!("    {}", byte);
        if i < public_key.len() - 1 {
            print!(", ");
        }
    }
    println!("\n];");

    // Save to file for reference
    let public_key_hex = public_key
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join("");

    let content = format!(
        r#"{{
  "public_key": "{}",
  "note": "This is a test key for development only. In production, use proper key management."
}}"#,
        public_key_hex
    );

    let path = Path::new("admin_key.json");
    fs::write(path, content).expect("Failed to write admin_key.json");

    println!("\nAdmin key saved to admin_key.json");
}

fn derive_public_key(secret: &[u8; 32]) -> [u8; 32] {
    // Simple mock for demonstration - in real usage, use proper crypto
    // This would normally use Ed25519 or similar
    let mut public = [0u8; 32];
    for i in 0..32 {
        public[i] = secret[i].wrapping_add(1);
    }
    public
}
