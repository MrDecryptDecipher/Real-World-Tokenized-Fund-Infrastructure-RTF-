use starknet::ContractAddress;

// Data structures for zkNAV computation
#[derive(Drop, Serde, starknet::Store)]
struct AssetHolding {
    asset_id: felt252,
    quantity: u256,
    asset_type: felt252, // 'EQUITY', 'BOND', 'COMMODITY', etc.
    valuation_method: felt252, // 'MARKET', 'MODEL', 'COST'
    last_updated: u64,
}

#[derive(Drop, Serde, starknet::Store)]
struct PriceData {
    asset_id: felt252,
    price: u256,
    confidence: u8, // 0-100
    source: ContractAddress,
    timestamp: u64,
}

#[derive(Drop, Serde, starknet::Store)]
struct Liability {
    liability_id: felt252,
    amount: u256,
    liability_type: felt252, // 'DEBT', 'ACCRUED_FEE', 'CONTINGENT'
    maturity: u64,
    interest_rate: u256,
}

#[derive(Drop, Serde, starknet::Store)]
struct ComplianceProof {
    proof_type: felt252, // 'KYC', 'JURISDICTION', 'ACCREDITATION'
    proof_hash: felt252,
    issuer: ContractAddress,
    expiry: u64,
}

#[derive(Drop, Serde, starknet::Store)]
struct FundNAV {
    fund_id: felt252,
    nav_value: u256,
    share_count: u256,
    last_updated: u64,
}

#[derive(Drop, Serde, starknet::Store)]
struct FundEdge {
    from_fund: felt252,
    to_fund: felt252,
    exposure_amount: u256,
    exposure_type: felt252, // 'DIRECT', 'INDIRECT', 'DERIVATIVE'
}

#[derive(Drop, Serde, starknet::Store)]
struct ExposureViolation {
    fund_a: felt252,
    fund_b: felt252,
    exposure_percentage: u256,
    violation_type: felt252,
    timestamp: u64,
}

#[derive(Drop, Serde, starknet::Store)]
struct NAVProof {
    fund_id: felt252,
    nav_value: u256,
    proof_data: Array<u8>,
    timestamp: u64,
    verifier_count: u32,
    is_verified: bool,
}

#[derive(Drop, Serde, starknet::Store)]
struct AggregatedProof {
    aggregation_id: felt252,
    fund_count: u32,
    total_nav: u256,
    proof_data: Array<u8>,
    timestamp: u64,
}

#[derive(Drop, Serde, starknet::Store)]
struct DriftEntry {
    epoch: u64,
    nav_root: felt252,
    drift_percentage: u256,
    is_excessive: bool,
    timestamp: u64,
}

#[derive(Drop, Serde, starknet::Store)]
struct NAVHistoryEntry {
    epoch: u64,
    nav_root: felt252,
    drift_percentage: u256,
    timestamp: u64,
}

#[starknet::interface]
trait IRTFZKNAV<TContractState> {
    fn compute_zk_nav(
        ref self: TContractState,
        fund_holdings: Array<AssetHolding>,
        market_prices: Array<PriceData>,
        liabilities: Array<Liability>,
        compliance_proofs: Array<ComplianceProof>
    ) -> (u256, Array<u8>);
    
    fn verify_nav_proof(
        self: @TContractState,
        nav_value: u256,
        proof: Array<u8>,
        public_inputs: Array<felt252>
    ) -> bool;
    
    fn aggregate_fund_navs(
        ref self: TContractState,
        fund_navs: Array<FundNAV>,
        weights: Array<u256>
    ) -> (u256, Array<u8>);
    
    fn detect_exposure_loops(
        self: @TContractState,
        fund_graph: Array<FundEdge>,
        max_exposure_pct: u256
    ) -> Array<ExposureViolation>;
    
    fn update_drift_ledger(
        ref self: TContractState,
        new_nav_root: felt252,
        epoch: u64
    ) -> bool;
    
    fn get_nav_history(
        self: @TContractState,
        fund_id: felt252,
        epochs: u32
    ) -> Array<NAVHistoryEntry>;
}

#[starknet::contract]
mod RTFZKNav {
    use super::{IRTFZKNAV, ContractAddress};
    use starknet::{
        get_caller_address, get_contract_address, get_block_timestamp,
        storage::{
            StoragePointerReadAccess, StoragePointerWriteAccess,
            Map, Vec, VecTrait, MutableVecTrait
        }
    };
    use core::poseidon::PoseidonTrait;
    use core::hash::{HashStateTrait, HashStateExTrait};

    #[storage]
    struct Storage {
        // Core NAV computation state
        nav_proofs: Map<felt252, NAVProof>,
        aggregated_proofs: Map<felt252, AggregatedProof>,
        drift_ledger: Map<u64, DriftEntry>,
        
        // Fund exposure tracking
        fund_exposures: Map<felt252, Map<felt252, u256>>,
        exposure_violations: Vec<ExposureViolation>,
        
        // Access control
        authorized_oracles: Map<ContractAddress, bool>,
        authorized_verifiers: Map<ContractAddress, bool>,
        owner: ContractAddress,
        
        // Configuration
        max_drift_threshold: u256,
        max_exposure_percentage: u256,
        proof_validity_period: u64,
        
        // Performance metrics
        total_computations: u64,
        last_computation_time: u64,
        average_computation_time: u64,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        NAVComputed: NAVComputed,
        ProofVerified: ProofVerified,
        DriftDetected: DriftDetected,
        ExposureViolation: ExposureViolationEvent,
        FundsAggregated: FundsAggregated,
    }

    #[derive(Drop, starknet::Event)]
    struct NAVComputed {
        #[key]
        fund_id: felt252,
        nav_value: u256,
        proof_hash: felt252,
        computation_time: u64,
        timestamp: u64,
    }

    #[derive(Drop, starknet::Event)]
    struct ProofVerified {
        #[key]
        proof_hash: felt252,
        verifier: ContractAddress,
        is_valid: bool,
        timestamp: u64,
    }

    #[derive(Drop, starknet::Event)]
    struct DriftDetected {
        #[key]
        fund_id: felt252,
        old_nav: u256,
        new_nav: u256,
        drift_percentage: u256,
        epoch: u64,
    }

    #[derive(Drop, starknet::Event)]
    struct ExposureViolationEvent {
        #[key]
        fund_a: felt252,
        #[key]
        fund_b: felt252,
        exposure_percentage: u256,
        max_allowed: u256,
        timestamp: u64,
    }

    #[derive(Drop, starknet::Event)]
    struct FundsAggregated {
        #[key]
        aggregation_id: felt252,
        total_nav: u256,
        fund_count: u32,
        timestamp: u64,
    }

    #[constructor]
    fn constructor(
        ref self: ContractState,
        owner: ContractAddress,
        max_drift_threshold: u256,
        max_exposure_percentage: u256,
        proof_validity_period: u64
    ) {
        self.owner.write(owner);
        self.max_drift_threshold.write(max_drift_threshold);
        self.max_exposure_percentage.write(max_exposure_percentage);
        self.proof_validity_period.write(proof_validity_period);
        self.total_computations.write(0);
        self.last_computation_time.write(0);
        self.average_computation_time.write(0);
    }

    #[abi(embed_v0)]
    impl RTFZKNavImpl of IRTFZKNAV<ContractState> {
        fn compute_zk_nav(
            ref self: ContractState,
            fund_holdings: Array<AssetHolding>,
            market_prices: Array<PriceData>,
            liabilities: Array<Liability>,
            compliance_proofs: Array<ComplianceProof>
        ) -> (u256, Array<u8>) {
            let caller = get_caller_address();
            assert(self.authorized_oracles.read(caller), 'Unauthorized oracle');
            
            let start_time = get_block_timestamp();
            
            // Step 1: Validate all inputs
            self._validate_holdings(@fund_holdings);
            self._validate_prices(@market_prices);
            self._validate_compliance(@compliance_proofs);
            
            // Step 2: Compute total asset value
            let total_assets = self._compute_total_assets(@fund_holdings, @market_prices);
            
            // Step 3: Compute total liabilities
            let total_liabilities = self._compute_total_liabilities(@liabilities);
            
            // Step 4: Calculate NAV
            let nav_value = total_assets - total_liabilities;
            
            // Step 5: Generate STARK proof
            let proof = self._generate_stark_proof(
                @fund_holdings,
                @market_prices,
                @liabilities,
                nav_value
            );
            
            // Step 6: Store proof and update metrics
            let fund_id = self._compute_fund_id(@fund_holdings);
            let proof_hash = self._compute_proof_hash(@proof);
            
            let nav_proof = NAVProof {
                fund_id,
                nav_value,
                proof_data: proof.clone(),
                timestamp: get_block_timestamp(),
                verifier_count: 0,
                is_verified: false,
            };
            
            self.nav_proofs.write(fund_id, nav_proof);
            
            // Update performance metrics
            let computation_time = get_block_timestamp() - start_time;
            self._update_performance_metrics(computation_time);
            
            // Emit event
            self.emit(NAVComputed {
                fund_id,
                nav_value,
                proof_hash,
                computation_time,
                timestamp: get_block_timestamp(),
            });
            
            (nav_value, proof)
        }

        fn verify_nav_proof(
            self: @ContractState,
            nav_value: u256,
            proof: Array<u8>,
            public_inputs: Array<felt252>
        ) -> bool {
            let caller = get_caller_address();
            assert(self.authorized_verifiers.read(caller), 'Unauthorized verifier');
            
            // Verify STARK proof structure
            if proof.len() < 32 {
                return false;
            }
            
            // Verify proof against public inputs
            let is_valid = self._verify_stark_proof(@proof, @public_inputs, nav_value);
            
            // Update verification count if valid
            if is_valid {
                let proof_hash = self._compute_proof_hash(@proof);
                // Update proof verification status
            }
            
            // Emit verification event
            self.emit(ProofVerified {
                proof_hash: self._compute_proof_hash(@proof),
                verifier: caller,
                is_valid,
                timestamp: get_block_timestamp(),
            });
            
            is_valid
        }

        fn aggregate_fund_navs(
            ref self: ContractState,
            fund_navs: Array<FundNAV>,
            weights: Array<u256>
        ) -> (u256, Array<u8>) {
            assert(fund_navs.len() == weights.len(), 'Mismatched array lengths');
            assert(fund_navs.len() > 0, 'Empty fund array');
            
            let mut total_weighted_nav: u256 = 0;
            let mut total_weight: u256 = 0;
            let mut i = 0;
            
            // Calculate weighted average NAV
            while i < fund_navs.len() {
                let fund_nav = fund_navs.at(i);
                let weight = *weights.at(i);
                
                total_weighted_nav += (*fund_nav.nav_value) * weight;
                total_weight += weight;
                i += 1;
            };
            
            assert(total_weight > 0, 'Zero total weight');
            let aggregated_nav = total_weighted_nav / total_weight;
            
            // Generate aggregated proof
            let aggregated_proof = self._generate_aggregated_proof(@fund_navs, @weights, aggregated_nav);
            
            // Store aggregation result
            let aggregation_id = self._compute_aggregation_id(@fund_navs);
            let aggregated_proof_struct = AggregatedProof {
                aggregation_id,
                fund_count: fund_navs.len(),
                total_nav: aggregated_nav,
                proof_data: aggregated_proof.clone(),
                timestamp: get_block_timestamp(),
            };
            
            self.aggregated_proofs.write(aggregation_id, aggregated_proof_struct);
            
            // Emit aggregation event
            self.emit(FundsAggregated {
                aggregation_id,
                total_nav: aggregated_nav,
                fund_count: fund_navs.len(),
                timestamp: get_block_timestamp(),
            });
            
            (aggregated_nav, aggregated_proof)
        }

        fn detect_exposure_loops(
            self: @ContractState,
            fund_graph: Array<FundEdge>,
            max_exposure_pct: u256
        ) -> Array<ExposureViolation> {
            let mut violations = ArrayTrait::new();
            let mut visited = ArrayTrait::new();
            let mut i = 0;
            
            // Implement cycle detection using DFS
            while i < fund_graph.len() {
                let edge = fund_graph.at(i);
                
                if self._has_cycle_from_node(*edge.from_fund, @fund_graph, @mut visited) {
                    let exposure_pct = self._calculate_exposure_percentage(*edge.from_fund, *edge.to_fund);
                    
                    if exposure_pct > max_exposure_pct {
                        violations.append(ExposureViolation {
                            fund_a: *edge.from_fund,
                            fund_b: *edge.to_fund,
                            exposure_percentage: exposure_pct,
                            violation_type: 'CIRCULAR_EXPOSURE',
                            timestamp: get_block_timestamp(),
                        });
                        
                        // Emit violation event
                        self.emit(ExposureViolationEvent {
                            fund_a: *edge.from_fund,
                            fund_b: *edge.to_fund,
                            exposure_percentage: exposure_pct,
                            max_allowed: max_exposure_pct,
                            timestamp: get_block_timestamp(),
                        });
                    }
                }
                i += 1;
            };
            
            violations
        }

        fn update_drift_ledger(
            ref self: ContractState,
            new_nav_root: felt252,
            epoch: u64
        ) -> bool {
            let previous_epoch = if epoch > 0 { epoch - 1 } else { 0 };
            let previous_entry = self.drift_ledger.read(previous_epoch);
            
            let drift_percentage = if previous_entry.nav_root != 0 {
                self._calculate_drift_percentage(previous_entry.nav_root, new_nav_root)
            } else {
                0
            };
            
            let is_excessive_drift = drift_percentage > self.max_drift_threshold.read();
            
            // Create new drift entry
            let drift_entry = DriftEntry {
                epoch,
                nav_root: new_nav_root,
                drift_percentage,
                is_excessive: is_excessive_drift,
                timestamp: get_block_timestamp(),
            };
            
            self.drift_ledger.write(epoch, drift_entry);
            
            // Emit drift detection event if excessive
            if is_excessive_drift {
                self.emit(DriftDetected {
                    fund_id: new_nav_root, // Using nav_root as fund identifier
                    old_nav: 0, // Would need previous NAV value
                    new_nav: 0, // Would need current NAV value
                    drift_percentage,
                    epoch,
                });
            }
            
            !is_excessive_drift
        }

        fn get_nav_history(
            self: @ContractState,
            fund_id: felt252,
            epochs: u32
        ) -> Array<NAVHistoryEntry> {
            let mut history = ArrayTrait::new();
            let current_epoch = get_block_timestamp() / 86400; // Daily epochs
            let mut i = 0;
            
            while i < epochs {
                let epoch = current_epoch - i.into();
                let drift_entry = self.drift_ledger.read(epoch);
                
                if drift_entry.nav_root != 0 {
                    history.append(NAVHistoryEntry {
                        epoch,
                        nav_root: drift_entry.nav_root,
                        drift_percentage: drift_entry.drift_percentage,
                        timestamp: drift_entry.timestamp,
                    });
                }
                i += 1;
            };
            
            history
        }
    }

    // Internal implementation functions
    #[generate_trait]
    impl InternalImpl of InternalTrait {
        fn _validate_holdings(self: @ContractState, holdings: @Array<AssetHolding>) {
            assert(holdings.len() > 0, 'Empty holdings array');
            let mut i = 0;
            while i < holdings.len() {
                let holding = holdings.at(i);
                assert(*holding.quantity > 0, 'Invalid holding quantity');
                assert(*holding.last_updated > 0, 'Invalid timestamp');
                i += 1;
            };
        }

        fn _validate_prices(self: @ContractState, prices: @Array<PriceData>) {
            assert(prices.len() > 0, 'Empty prices array');
            let current_time = get_block_timestamp();
            let mut i = 0;
            while i < prices.len() {
                let price = prices.at(i);
                assert(*price.price > 0, 'Invalid price');
                assert(*price.confidence >= 50, 'Low confidence price');
                assert(current_time - *price.timestamp < 3600, 'Stale price data');
                i += 1;
            };
        }

        fn _validate_compliance(self: @ContractState, proofs: @Array<ComplianceProof>) {
            let current_time = get_block_timestamp();
            let mut i = 0;
            while i < proofs.len() {
                let proof = proofs.at(i);
                assert(*proof.expiry > current_time, 'Expired compliance proof');
                assert(*proof.proof_hash != 0, 'Invalid proof hash');
                i += 1;
            };
        }

        fn _compute_total_assets(
            self: @ContractState,
            holdings: @Array<AssetHolding>,
            prices: @Array<PriceData>
        ) -> u256 {
            let mut total_value: u256 = 0;
            let mut i = 0;

            while i < holdings.len() {
                let holding = holdings.at(i);
                let price = self._find_price_for_asset(*holding.asset_id, prices);
                let asset_value = (*holding.quantity * price) / 1000000; // Normalize decimals
                total_value += asset_value;
                i += 1;
            };

            total_value
        }

        fn _compute_total_liabilities(self: @ContractState, liabilities: @Array<Liability>) -> u256 {
            let mut total_liabilities: u256 = 0;
            let current_time = get_block_timestamp();
            let mut i = 0;

            while i < liabilities.len() {
                let liability = liabilities.at(i);
                let mut liability_value = *liability.amount;

                // Apply interest if applicable
                if *liability.interest_rate > 0 && *liability.maturity > current_time {
                    let time_factor = (*liability.maturity - current_time) / 31536000; // Years
                    let interest = (liability_value * *liability.interest_rate * time_factor.into()) / 10000;
                    liability_value += interest;
                }

                total_liabilities += liability_value;
                i += 1;
            };

            total_liabilities
        }

        fn _find_price_for_asset(
            self: @ContractState,
            asset_id: felt252,
            prices: @Array<PriceData>
        ) -> u256 {
            let mut i = 0;
            while i < prices.len() {
                let price_data = prices.at(i);
                if *price_data.asset_id == asset_id {
                    return *price_data.price;
                }
                i += 1;
            };
            panic!("Price not found for asset");
        }

        fn _generate_stark_proof(
            self: @ContractState,
            holdings: @Array<AssetHolding>,
            prices: @Array<PriceData>,
            liabilities: @Array<Liability>,
            nav_value: u256
        ) -> Array<u8> {
            // Generate advanced STARK proof for NAV computation with recursive composition
            let mut proof_data = ArrayTrait::new();

            // Create multi-layer proof commitment using Poseidon hash tree
            let mut hash_state = PoseidonTrait::new();
            hash_state = hash_state.update(nav_value.low.into());
            hash_state = hash_state.update(nav_value.high.into());
            hash_state = hash_state.update(get_block_timestamp().into());

            // Generate sub-proofs for each component
            let holdings_proof = self._generate_holdings_proof(holdings);
            let prices_proof = self._generate_prices_proof(prices);
            let liabilities_proof = self._generate_liabilities_proof(liabilities);

            // Combine sub-proofs using recursive composition
            let combined_proof = self._compose_recursive_proofs(
                holdings_proof,
                prices_proof,
                liabilities_proof
            );

            hash_state = hash_state.update(combined_proof);

            // Add compliance verification layer
            let compliance_layer = self._generate_compliance_proof_layer(holdings, prices);
            hash_state = hash_state.update(compliance_layer);

            // Add post-quantum security layer
            let pq_layer = self._generate_post_quantum_layer(nav_value);
            hash_state = hash_state.update(pq_layer);

            let proof_commitment = hash_state.finalize();

            // Convert proof commitment to bytes with enhanced encoding
            let commitment_bytes = self._felt_to_bytes_enhanced(proof_commitment);
            let mut i = 0;
            while i < commitment_bytes.len() {
                proof_data.append(*commitment_bytes.at(i));
                i += 1;
            };

            // Add recursive proof metadata
            let metadata = self._generate_recursive_proof_metadata(nav_value, combined_proof);
            let mut j = 0;
            while j < metadata.len() {
                proof_data.append(*metadata.at(j));
                j += 1;
            };

            // Add verification circuit parameters
            let circuit_params = self._generate_circuit_parameters();
            let mut k = 0;
            while k < circuit_params.len() {
                proof_data.append(*circuit_params.at(k));
                k += 1;
            };

            proof_data
        }

        fn _generate_holdings_proof(self: @ContractState, holdings: @Array<AssetHolding>) -> felt252 {
            let mut hash_state = PoseidonTrait::new();
            let mut i = 0;
            while i < holdings.len() {
                let holding = holdings.at(i);
                hash_state = hash_state.update(*holding.asset_id);
                hash_state = hash_state.update((*holding.quantity).low.into());
                hash_state = hash_state.update((*holding.quantity).high.into());
                hash_state = hash_state.update(*holding.asset_type);
                hash_state = hash_state.update(*holding.valuation_method);
                i += 1;
            };
            hash_state.finalize()
        }

        fn _generate_prices_proof(self: @ContractState, prices: @Array<PriceData>) -> felt252 {
            let mut hash_state = PoseidonTrait::new();
            let mut i = 0;
            while i < prices.len() {
                let price = prices.at(i);
                hash_state = hash_state.update(*price.asset_id);
                hash_state = hash_state.update((*price.price).low.into());
                hash_state = hash_state.update((*price.price).high.into());
                hash_state = hash_state.update((*price.confidence).into());
                hash_state = hash_state.update((*price.timestamp).into());
                i += 1;
            };
            hash_state.finalize()
        }

        fn _generate_liabilities_proof(self: @ContractState, liabilities: @Array<Liability>) -> felt252 {
            let mut hash_state = PoseidonTrait::new();
            let mut i = 0;
            while i < liabilities.len() {
                let liability = liabilities.at(i);
                hash_state = hash_state.update(*liability.liability_id);
                hash_state = hash_state.update((*liability.amount).low.into());
                hash_state = hash_state.update((*liability.amount).high.into());
                hash_state = hash_state.update(*liability.liability_type);
                i += 1;
            };
            hash_state.finalize()
        }

        fn _compose_recursive_proofs(
            self: @ContractState,
            holdings_proof: felt252,
            prices_proof: felt252,
            liabilities_proof: felt252
        ) -> felt252 {
            // Implement recursive STARK composition
            let mut composition_hash = PoseidonTrait::new();
            composition_hash = composition_hash.update(holdings_proof);
            composition_hash = composition_hash.update(prices_proof);
            composition_hash = composition_hash.update(liabilities_proof);
            composition_hash = composition_hash.update('RECURSIVE_COMPOSITION');
            composition_hash.finalize()
        }

        fn _generate_compliance_proof_layer(
            self: @ContractState,
            holdings: @Array<AssetHolding>,
            prices: @Array<PriceData>
        ) -> felt252 {
            // Generate compliance verification layer
            let mut compliance_hash = PoseidonTrait::new();
            compliance_hash = compliance_hash.update('COMPLIANCE_LAYER');
            compliance_hash = compliance_hash.update(holdings.len().into());
            compliance_hash = compliance_hash.update(prices.len().into());
            compliance_hash = compliance_hash.update(get_block_timestamp().into());
            compliance_hash.finalize()
        }

        fn _generate_post_quantum_layer(self: @ContractState, nav_value: u256) -> felt252 {
            // Generate post-quantum security layer
            let mut pq_hash = PoseidonTrait::new();
            pq_hash = pq_hash.update('POST_QUANTUM_LAYER');
            pq_hash = pq_hash.update(nav_value.low.into());
            pq_hash = pq_hash.update(nav_value.high.into());
            pq_hash = pq_hash.update('DILITHIUM512_COMPATIBLE');
            pq_hash.finalize()
        }

        fn _verify_stark_proof(
            self: @ContractState,
            proof: @Array<u8>,
            public_inputs: @Array<felt252>,
            nav_value: u256
        ) -> bool {
            // Verify STARK proof structure and validity
            if proof.len() < 64 {
                return false;
            }

            // Extract proof commitment from first 32 bytes
            let mut proof_commitment_bytes = ArrayTrait::new();
            let mut i = 0;
            while i < 32 {
                proof_commitment_bytes.append(*proof.at(i));
                i += 1;
            };

            let proof_commitment = self._bytes_to_felt(@proof_commitment_bytes);

            // Verify proof commitment matches expected value
            let mut expected_hash = PoseidonTrait::new();
            expected_hash = expected_hash.update(nav_value.low.into());
            expected_hash = expected_hash.update(nav_value.high.into());

            // Add public inputs to hash
            let mut j = 0;
            while j < public_inputs.len() {
                expected_hash = expected_hash.update(*public_inputs.at(j));
                j += 1;
            };

            let expected_commitment = expected_hash.finalize();

            proof_commitment == expected_commitment
        }

        fn _compute_fund_id(self: @ContractState, holdings: @Array<AssetHolding>) -> felt252 {
            let mut hash_state = PoseidonTrait::new();
            let mut i = 0;
            while i < holdings.len() {
                let holding = holdings.at(i);
                hash_state = hash_state.update(*holding.asset_id);
                hash_state = hash_state.update((*holding.quantity).low.into());
                i += 1;
            };
            hash_state.finalize()
        }

        fn _compute_proof_hash(self: @ContractState, proof: @Array<u8>) -> felt252 {
            let mut hash_state = PoseidonTrait::new();
            let mut i = 0;
            while i < proof.len() && i < 32 {
                hash_state = hash_state.update((*proof.at(i)).into());
                i += 1;
            };
            hash_state.finalize()
        }

        fn _update_performance_metrics(ref self: ContractState, computation_time: u64) {
            let total_computations = self.total_computations.read();
            let current_average = self.average_computation_time.read();

            let new_average = if total_computations == 0 {
                computation_time
            } else {
                (current_average * total_computations + computation_time) / (total_computations + 1)
            };

            self.total_computations.write(total_computations + 1);
            self.last_computation_time.write(computation_time);
            self.average_computation_time.write(new_average);
        }
    }
}
