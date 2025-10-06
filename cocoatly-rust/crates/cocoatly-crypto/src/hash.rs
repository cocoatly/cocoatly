use cocoatly_core::types::HashAlgorithm;
use cocoatly_core::error::{CocoatlyError, Result};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use blake3::Hasher as Blake3Hasher;
use sha2::{Sha256, Sha512, Digest};

pub struct HashComputer;

impl HashComputer {
    pub fn compute(data: &[u8], algorithm: &HashAlgorithm) -> String {
        match algorithm {
            HashAlgorithm::Blake3 => {
                let mut hasher = Blake3Hasher::new();
                hasher.update(data);
                hasher.finalize().to_hex().to_string()
            }
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(data);
                format!("{:x}", hasher.finalize())
            }
            HashAlgorithm::Sha512 => {
                let mut hasher = Sha512::new();
                hasher.update(data);
                format!("{:x}", hasher.finalize())
            }
        }
    }

    pub fn compute_stream<R: Read>(reader: &mut R, algorithm: &HashAlgorithm) -> Result<String> {
        let mut buffer = vec![0; 8192];

        match algorithm {
            HashAlgorithm::Blake3 => {
                let mut hasher = Blake3Hasher::new();
                loop {
                    let bytes_read = reader.read(&mut buffer)?;
                    if bytes_read == 0 {
                        break;
                    }
                    hasher.update(&buffer[..bytes_read]);
                }
                Ok(hasher.finalize().to_hex().to_string())
            }
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                loop {
                    let bytes_read = reader.read(&mut buffer)?;
                    if bytes_read == 0 {
                        break;
                    }
                    hasher.update(&buffer[..bytes_read]);
                }
                Ok(format!("{:x}", hasher.finalize()))
            }
            HashAlgorithm::Sha512 => {
                let mut hasher = Sha512::new();
                loop {
                    let bytes_read = reader.read(&mut buffer)?;
                    if bytes_read == 0 {
                        break;
                    }
                    hasher.update(&buffer[..bytes_read]);
                }
                Ok(format!("{:x}", hasher.finalize()))
            }
        }
    }
}

pub fn compute_file_hash<P: AsRef<Path>>(path: P, algorithm: &HashAlgorithm) -> Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    HashComputer::compute_stream(&mut reader, algorithm)
}

pub fn verify_hash(data: &[u8], expected: &str, algorithm: &HashAlgorithm) -> Result<()> {
    let computed = HashComputer::compute(data, algorithm);
    if computed == expected {
        Ok(())
    } else {
        Err(CocoatlyError::HashMismatch {
            expected: expected.to_string(),
            actual: computed,
        })
    }
}

pub fn verify_file_hash<P: AsRef<Path>>(
    path: P,
    expected: &str,
    algorithm: &HashAlgorithm
) -> Result<()> {
    let computed = compute_file_hash(path, algorithm)?;
    if computed == expected {
        Ok(())
    } else {
        Err(CocoatlyError::HashMismatch {
            expected: expected.to_string(),
            actual: computed,
        })
    }
}
