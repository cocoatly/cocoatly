use cocoatly_core::error::{CocoatlyError, Result};
use ring::signature::{Ed25519KeyPair, UnparsedPublicKey, ED25519};
use ring::rand::SystemRandom;

pub struct SignatureVerifier;

impl SignatureVerifier {
    pub fn verify_ed25519(public_key: &[u8], message: &[u8], signature: &[u8]) -> Result<()> {
        let public_key = UnparsedPublicKey::new(&ED25519, public_key);

        public_key
            .verify(message, signature)
            .map_err(|_| CocoatlyError::InvalidSignature(
                "Ed25519 signature verification failed".to_string()
            ))
    }
}

pub fn sign_data(private_key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    let key_pair = Ed25519KeyPair::from_pkcs8(private_key)
        .map_err(|_| CocoatlyError::InvalidSignature(
            "Invalid private key format".to_string()
        ))?;

    let signature = key_pair.sign(data);
    Ok(signature.as_ref().to_vec())
}

pub fn verify_signature(public_key: &[u8], data: &[u8], signature: &[u8]) -> Result<()> {
    SignatureVerifier::verify_ed25519(public_key, data, signature)
}

pub fn generate_keypair() -> Result<(Vec<u8>, Vec<u8>)> {
    let rng = SystemRandom::new();
    let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng)
        .map_err(|_| CocoatlyError::InvalidSignature(
            "Failed to generate keypair".to_string()
        ))?;

    let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
        .map_err(|_| CocoatlyError::InvalidSignature(
            "Failed to parse generated keypair".to_string()
        ))?;

    let public_key = key_pair.public_key().as_ref().to_vec();
    let private_key = pkcs8_bytes.as_ref().to_vec();

    Ok((private_key, public_key))
}
