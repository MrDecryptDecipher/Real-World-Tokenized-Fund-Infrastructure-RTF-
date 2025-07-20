//! # Cryptographic Utilities for RTF Infrastructure
//!
//! Advanced cryptographic primitives and utilities for secure operations
//! in the RTF protocol including hashing, encryption, and digital signatures.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use rand::{RngCore, CryptoRng};

/// Advanced hashing utilities
pub mod hashing {
    use super::*;
    use sha2::{Sha256, Sha512, Digest};
    use blake3::Hasher as Blake3Hasher;

    /// Hash algorithms supported
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum HashAlgorithm {
        Sha256,
        Sha512,
        Blake3,
        Keccak256,
    }

    /// Hash a message using the specified algorithm
    pub fn hash_message(data: &[u8], algorithm: HashAlgorithm) -> Result<Vec<u8>> {
        match algorithm {
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(data);
                Ok(hasher.finalize().to_vec())
            }
            HashAlgorithm::Sha512 => {
                let mut hasher = Sha512::new();
                hasher.update(data);
                Ok(hasher.finalize().to_vec())
            }
            HashAlgorithm::Blake3 => {
                let mut hasher = Blake3Hasher::new();
                hasher.update(data);
                Ok(hasher.finalize().as_bytes().to_vec())
            }
            HashAlgorithm::Keccak256 => {
                // Simulate Keccak256 (would use actual implementation in production)
                let mut hasher = Sha256::new();
                hasher.update(b"keccak256:");
                hasher.update(data);
                Ok(hasher.finalize().to_vec())
            }
        }
    }

    /// Merkle tree implementation
    #[derive(Debug, Clone)]
    pub struct MerkleTree {
        pub leaves: Vec<Vec<u8>>,
        pub root: Vec<u8>,
    }

    impl MerkleTree {
        /// Create a new Merkle tree from leaves
        pub fn new(leaves: Vec<Vec<u8>>) -> Result<Self> {
            if leaves.is_empty() {
                return Err(anyhow!("Cannot create Merkle tree with no leaves"));
            }

            let root = Self::compute_root(&leaves)?;
            Ok(Self { leaves, root })
        }

        /// Compute the Merkle root
        fn compute_root(leaves: &[Vec<u8>]) -> Result<Vec<u8>> {
            let mut current_level = leaves.to_vec();

            while current_level.len() > 1 {
                let mut next_level = Vec::new();

                for chunk in current_level.chunks(2) {
                    let combined = if chunk.len() == 2 {
                        [chunk[0].clone(), chunk[1].clone()].concat()
                    } else {
                        chunk[0].clone()
                    };

                    let hash = hash_message(&combined, HashAlgorithm::Sha256)?;
                    next_level.push(hash);
                }

                current_level = next_level;
            }

            Ok(current_level[0].clone())
        }

        /// Generate a Merkle proof for a leaf
        pub fn generate_proof(&self, leaf_index: usize) -> Result<Vec<Vec<u8>>> {
            if leaf_index >= self.leaves.len() {
                return Err(anyhow!("Leaf index out of bounds"));
            }

            let mut proof = Vec::new();
            let mut current_level = self.leaves.clone();
            let mut current_index = leaf_index;

            while current_level.len() > 1 {
                let sibling_index = if current_index % 2 == 0 {
                    current_index + 1
                } else {
                    current_index - 1
                };

                if sibling_index < current_level.len() {
                    proof.push(current_level[sibling_index].clone());
                }

                let mut next_level = Vec::new();
                for chunk in current_level.chunks(2) {
                    let combined = if chunk.len() == 2 {
                        [chunk[0].clone(), chunk[1].clone()].concat()
                    } else {
                        chunk[0].clone()
                    };

                    let hash = hash_message(&combined, HashAlgorithm::Sha256)?;
                    next_level.push(hash);
                }

                current_level = next_level;
                current_index /= 2;
            }

            Ok(proof)
        }

        /// Verify a Merkle proof
        pub fn verify_proof(
            leaf: &[u8],
            proof: &[Vec<u8>],
            root: &[u8],
            leaf_index: usize,
        ) -> Result<bool> {
            let mut current_hash = leaf.to_vec();
            let mut current_index = leaf_index;

            for sibling in proof {
                let combined = if current_index % 2 == 0 {
                    [current_hash, sibling.clone()].concat()
                } else {
                    [sibling.clone(), current_hash].concat()
                };

                current_hash = hash_message(&combined, HashAlgorithm::Sha256)?;
                current_index /= 2;
            }

            Ok(current_hash == root)
        }
    }
}

/// Symmetric encryption utilities
pub mod symmetric {
    use super::*;
    use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, NewAead}};

    /// Symmetric encryption key
    #[derive(Debug, Clone)]
    pub struct SymmetricKey {
        pub key_data: [u8; 32],
    }

    /// Encrypted data with nonce
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EncryptedData {
        pub ciphertext: Vec<u8>,
        pub nonce: [u8; 12],
    }

    impl SymmetricKey {
        /// Generate a new random symmetric key
        pub fn generate<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
            let mut key_data = [0u8; 32];
            rng.fill_bytes(&mut key_data);
            Self { key_data }
        }

        /// Encrypt data
        pub fn encrypt<R: RngCore + CryptoRng>(
            &self,
            data: &[u8],
            rng: &mut R,
        ) -> Result<EncryptedData> {
            let key = Key::from_slice(&self.key_data);
            let cipher = Aes256Gcm::new(key);

            let mut nonce_bytes = [0u8; 12];
            rng.fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);

            let ciphertext = cipher.encrypt(nonce, data)
                .map_err(|e| anyhow!("Encryption failed: {}", e))?;

            Ok(EncryptedData {
                ciphertext,
                nonce: nonce_bytes,
            })
        }

        /// Decrypt data
        pub fn decrypt(&self, encrypted_data: &EncryptedData) -> Result<Vec<u8>> {
            let key = Key::from_slice(&self.key_data);
            let cipher = Aes256Gcm::new(key);
            let nonce = Nonce::from_slice(&encrypted_data.nonce);

            let plaintext = cipher.decrypt(nonce, encrypted_data.ciphertext.as_ref())
                .map_err(|e| anyhow!("Decryption failed: {}", e))?;

            Ok(plaintext)
        }
    }
}

/// Digital signature utilities
pub mod signatures {
    use super::*;
    use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer, Verifier};

    /// Ed25519 key pair wrapper
    #[derive(Debug)]
    pub struct Ed25519KeyPair {
        keypair: Keypair,
    }

    impl Ed25519KeyPair {
        /// Generate a new Ed25519 key pair
        pub fn generate<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
            let keypair = Keypair::generate(rng);
            Self { keypair }
        }

        /// Get the public key
        pub fn public_key(&self) -> PublicKey {
            self.keypair.public
        }

        /// Sign a message
        pub fn sign(&self, message: &[u8]) -> Signature {
            self.keypair.sign(message)
        }

        /// Verify a signature
        pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<()> {
            self.keypair.public.verify(message, signature)
                .map_err(|e| anyhow!("Signature verification failed: {}", e))
        }
    }
}

/// Cryptographic utilities manager
#[derive(Debug)]
pub struct CryptoManager {
    symmetric_keys: HashMap<String, symmetric::SymmetricKey>,
    signature_keys: HashMap<String, signatures::Ed25519KeyPair>,
}

impl CryptoManager {
    /// Create a new crypto manager
    pub fn new() -> Self {
        Self {
            symmetric_keys: HashMap::new(),
            signature_keys: HashMap::new(),
        }
    }

    /// Generate and store a symmetric key
    pub fn generate_symmetric_key<R: RngCore + CryptoRng>(
        &mut self,
        key_id: String,
        rng: &mut R,
    ) {
        let key = symmetric::SymmetricKey::generate(rng);
        self.symmetric_keys.insert(key_id, key);
    }

    /// Generate and store a signature key pair
    pub fn generate_signature_keypair<R: RngCore + CryptoRng>(
        &mut self,
        key_id: String,
        rng: &mut R,
    ) {
        let keypair = signatures::Ed25519KeyPair::generate(rng);
        self.signature_keys.insert(key_id, keypair);
    }

    /// Encrypt data with a symmetric key
    pub fn encrypt_data<R: RngCore + CryptoRng>(
        &self,
        key_id: &str,
        data: &[u8],
        rng: &mut R,
    ) -> Result<symmetric::EncryptedData> {
        let key = self.symmetric_keys.get(key_id)
            .ok_or_else(|| anyhow!("Symmetric key not found: {}", key_id))?;
        key.encrypt(data, rng)
    }

    /// Decrypt data with a symmetric key
    pub fn decrypt_data(
        &self,
        key_id: &str,
        encrypted_data: &symmetric::EncryptedData,
    ) -> Result<Vec<u8>> {
        let key = self.symmetric_keys.get(key_id)
            .ok_or_else(|| anyhow!("Symmetric key not found: {}", key_id))?;
        key.decrypt(encrypted_data)
    }
}

impl Default for CryptoManager {
    fn default() -> Self {
        Self::new()
    }
}
