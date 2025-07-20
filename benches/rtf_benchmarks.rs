//! # RTF Infrastructure Performance Benchmarks
//! 
//! Comprehensive benchmarks for all RTF components to ensure
//! sub-400ms performance targets are met.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use tokio::runtime::Runtime;
use std::time::Duration;

// Import RTF components for benchmarking
use rtf_bridge_defense::{BridgeDefenseSystem, DefenseConfig};
use rtf_governance::{GovernanceSystem, GovernanceConfig, DAOType, ProposalType, VoteType, VotingMechanism};
use rtf_esg_compliance::{ESGComplianceSystem, ESGConfig};

/// Benchmark configuration
struct BenchmarkConfig {
    pub message_sizes: Vec<usize>,
    pub concurrent_operations: Vec<usize>,
    pub test_iterations: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            message_sizes: vec![64, 256, 1024, 4096, 16384],
            concurrent_operations: vec![1, 10, 50, 100, 500],
            test_iterations: 1000,
        }
    }
}

/// Benchmark bridge defense system performance
fn benchmark_bridge_defense(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    
    // Initialize defense system
    let defense_config = DefenseConfig::default();
    let defense_system = rt.block_on(async {
        let system = BridgeDefenseSystem::new(defense_config).await.unwrap();
        system.start().await.unwrap();
        system
    });
    
    let mut group = c.benchmark_group("bridge_defense");
    group.measurement_time(Duration::from_secs(10));
    
    // Benchmark message processing with different sizes
    for size in &config.message_sizes {
        let test_message = vec![0u8; *size];
        
        group.bench_with_input(
            BenchmarkId::new("process_message", size),
            size,
            |b, _| {
                b.to_async(&rt).iter(|| async {
                    black_box(
                        defense_system
                            .process_message(&test_message, 1, 2)
                            .await
                            .unwrap()
                    )
                })
            },
        );
    }
    
    // Benchmark concurrent message processing
    for concurrency in &config.concurrent_operations {
        group.bench_with_input(
            BenchmarkId::new("concurrent_processing", concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let mut handles = Vec::new();
                    let test_message = vec![0u8; 1024];
                    
                    for _ in 0..concurrency {
                        let defense_system_ref = &defense_system;
                        let message_ref = &test_message;
                        let handle = tokio::spawn(async move {
                            defense_system_ref.process_message(message_ref, 1, 2).await
                        });
                        handles.push(handle);
                    }
                    
                    black_box(futures::future::try_join_all(handles).await.unwrap())
                })
            },
        );
    }
    
    group.finish();
    
    // Cleanup
    rt.block_on(async {
        defense_system.stop().await.unwrap();
    });
}

/// Benchmark governance system performance
fn benchmark_governance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    
    // Initialize governance system
    let governance_config = GovernanceConfig::default();
    let governance_system = rt.block_on(async {
        GovernanceSystem::new(governance_config).await.unwrap()
    });
    
    let mut group = c.benchmark_group("governance");
    group.measurement_time(Duration::from_secs(10));
    
    // Benchmark proposal submission
    group.bench_function("submit_proposal", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(
                governance_system
                    .submit_proposal(
                        DAOType::Validator,
                        ProposalType::ParameterChange {
                            parameter: "test_param".to_string(),
                            old_value: "old".to_string(),
                            new_value: "new".to_string(),
                        },
                        "Benchmark Proposal".to_string(),
                        "Performance test proposal".to_string(),
                        "benchmark_proposer".to_string(),
                    )
                    .await
                    .unwrap()
            )
        })
    });
    
    // Benchmark voting with different mechanisms
    let proposal_id = rt.block_on(async {
        governance_system
            .submit_proposal(
                DAOType::Validator,
                ProposalType::ParameterChange {
                    parameter: "benchmark_param".to_string(),
                    old_value: "old".to_string(),
                    new_value: "new".to_string(),
                },
                "Voting Benchmark Proposal".to_string(),
                "Proposal for voting benchmarks".to_string(),
                "benchmark_proposer".to_string(),
            )
            .await
            .unwrap()
    });
    
    group.bench_function("cast_vote_simple", |b| {
        let mut counter = 0;
        b.to_async(&rt).iter(|| {
            counter += 1;
            let voter_id = format!("voter_{}", counter);
            let proposal_id_clone = proposal_id.clone();
            async move {
                black_box(
                    governance_system
                        .cast_vote(
                            proposal_id_clone,
                            voter_id,
                            VoteType::For,
                            1000,
                            VotingMechanism::Simple,
                        )
                        .await
                        .unwrap()
                )
            }
        })
    });
    
    group.bench_function("cast_vote_quadratic", |b| {
        let mut counter = 10000;
        b.to_async(&rt).iter(|| {
            counter += 1;
            let voter_id = format!("voter_{}", counter);
            let proposal_id_clone = proposal_id.clone();
            async move {
                black_box(
                    governance_system
                        .cast_vote(
                            proposal_id_clone,
                            voter_id,
                            VoteType::For,
                            1000,
                            VotingMechanism::Quadratic,
                        )
                        .await
                        .unwrap()
                )
            }
        })
    });
    
    // Benchmark concurrent voting
    for concurrency in &config.concurrent_operations {
        group.bench_with_input(
            BenchmarkId::new("concurrent_voting", concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let mut handles = Vec::new();
                    
                    for i in 0..concurrency {
                        let governance_ref = &governance_system;
                        let proposal_id_clone = proposal_id.clone();
                        let voter_id = format!("concurrent_voter_{}_{}", concurrency, i);
                        
                        let handle = tokio::spawn(async move {
                            governance_ref
                                .cast_vote(
                                    proposal_id_clone,
                                    voter_id,
                                    VoteType::For,
                                    1000,
                                    VotingMechanism::Simple,
                                )
                                .await
                        });
                        handles.push(handle);
                    }
                    
                    black_box(futures::future::try_join_all(handles).await.unwrap())
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark ESG compliance system performance
fn benchmark_esg_compliance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    
    // Initialize ESG system
    let esg_config = ESGConfig::default();
    let esg_system = rt.block_on(async {
        ESGComplianceSystem::new(esg_config).await.unwrap()
    });
    
    let mut group = c.benchmark_group("esg_compliance");
    group.measurement_time(Duration::from_secs(10));
    
    // Benchmark compliance checks
    group.bench_function("compliance_check", |b| {
        let mut counter = 0;
        b.to_async(&rt).iter(|| {
            counter += 1;
            let entity_id = format!("benchmark_entity_{}", counter);
            async move {
                black_box(
                    esg_system
                        .perform_compliance_check(&entity_id)
                        .await
                        .unwrap()
                )
            }
        })
    });
    
    // Benchmark concurrent compliance checks
    for concurrency in &config.concurrent_operations {
        group.bench_with_input(
            BenchmarkId::new("concurrent_compliance", concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let mut handles = Vec::new();
                    
                    for i in 0..concurrency {
                        let esg_ref = &esg_system;
                        let entity_id = format!("concurrent_entity_{}_{}", concurrency, i);
                        
                        let handle = tokio::spawn(async move {
                            esg_ref.perform_compliance_check(&entity_id).await
                        });
                        handles.push(handle);
                    }
                    
                    black_box(futures::future::try_join_all(handles).await.unwrap())
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark cryptographic operations
fn benchmark_crypto_operations(c: &mut Criterion) {
    use rtf_crypto::{hashing, symmetric, signatures};
    use rtf_post_quantum::{dilithium, PostQuantumKeyManager};
    use rtf_zk_proofs::{zksnark, ZKProofManager};
    use rand::rngs::OsRng;
    
    let mut group = c.benchmark_group("crypto_operations");
    group.measurement_time(Duration::from_secs(10));
    
    let config = BenchmarkConfig::default();
    
    // Benchmark hashing operations
    for size in &config.message_sizes {
        let test_data = vec![0u8; *size];
        
        group.bench_with_input(
            BenchmarkId::new("sha256_hash", size),
            size,
            |b, _| {
                b.iter(|| {
                    black_box(
                        hashing::hash_message(&test_data, hashing::HashAlgorithm::Sha256).unwrap()
                    )
                })
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("blake3_hash", size),
            size,
            |b, _| {
                b.iter(|| {
                    black_box(
                        hashing::hash_message(&test_data, hashing::HashAlgorithm::Blake3).unwrap()
                    )
                })
            },
        );
    }
    
    // Benchmark symmetric encryption
    let mut rng = OsRng;
    let symmetric_key = symmetric::SymmetricKey::generate(&mut rng);
    
    for size in &config.message_sizes {
        let test_data = vec![0u8; *size];
        
        group.bench_with_input(
            BenchmarkId::new("aes_encrypt", size),
            size,
            |b, _| {
                b.iter(|| {
                    black_box(symmetric_key.encrypt(&test_data, &mut OsRng).unwrap())
                })
            },
        );
    }
    
    // Benchmark digital signatures
    let ed25519_keypair = signatures::Ed25519KeyPair::generate(&mut rng);
    let test_message = b"benchmark message for signing";
    
    group.bench_function("ed25519_sign", |b| {
        b.iter(|| {
            black_box(ed25519_keypair.sign(test_message))
        })
    });
    
    let signature = ed25519_keypair.sign(test_message);
    group.bench_function("ed25519_verify", |b| {
        b.iter(|| {
            black_box(ed25519_keypair.verify(test_message, &signature).unwrap())
        })
    });
    
    // Benchmark post-quantum operations
    group.bench_function("dilithium_keygen", |b| {
        b.iter(|| {
            black_box(dilithium::KeyPair::generate(&mut OsRng).unwrap())
        })
    });
    
    let dilithium_keypair = dilithium::KeyPair::generate(&mut rng).unwrap();
    group.bench_function("dilithium_sign", |b| {
        b.iter(|| {
            black_box(dilithium_keypair.sign(test_message).unwrap())
        })
    });
    
    let dilithium_signature = dilithium_keypair.sign(test_message).unwrap();
    group.bench_function("dilithium_verify", |b| {
        b.iter(|| {
            black_box(dilithium_keypair.verify(test_message, &dilithium_signature).unwrap())
        })
    });
    
    group.finish();
}

/// Benchmark end-to-end system performance
fn benchmark_end_to_end(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("end_to_end");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(50); // Fewer samples for complex end-to-end tests
    
    // Benchmark complete workflow: ESG check -> Governance proposal -> Bridge message
    group.bench_function("complete_workflow", |b| {
        b.to_async(&rt).iter(|| async {
            // Initialize systems
            let defense_config = DefenseConfig::default();
            let governance_config = GovernanceConfig::default();
            let esg_config = ESGConfig::default();
            
            let defense_system = BridgeDefenseSystem::new(defense_config).await.unwrap();
            let governance_system = GovernanceSystem::new(governance_config).await.unwrap();
            let esg_system = ESGComplianceSystem::new(esg_config).await.unwrap();
            
            defense_system.start().await.unwrap();
            
            // Perform ESG compliance check
            let compliance_record = esg_system
                .perform_compliance_check("benchmark_entity")
                .await
                .unwrap();
            
            // Submit governance proposal based on compliance
            let proposal_id = governance_system
                .submit_proposal(
                    DAOType::ESG,
                    ProposalType::ESGCompliance {
                        standard: "Benchmark Standard".to_string(),
                        requirements: vec!["Requirement 1".to_string()],
                    },
                    "Benchmark Proposal".to_string(),
                    format!("Compliance score: {:.2}", compliance_record.overall_score),
                    "benchmark_system".to_string(),
                )
                .await
                .unwrap();
            
            // Cast vote
            governance_system
                .cast_vote(
                    proposal_id,
                    "benchmark_voter".to_string(),
                    VoteType::For,
                    1000,
                    VotingMechanism::Simple,
                )
                .await
                .unwrap();
            
            // Process cross-chain message
            let test_message = b"end-to-end benchmark message";
            let message_result = defense_system
                .process_message(test_message, 1, 2)
                .await
                .unwrap();
            
            defense_system.stop().await.unwrap();
            
            black_box((compliance_record, message_result))
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_bridge_defense,
    benchmark_governance,
    benchmark_esg_compliance,
    benchmark_crypto_operations,
    benchmark_end_to_end
);

criterion_main!(benches);
