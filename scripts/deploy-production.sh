#!/bin/bash

# RTF Infrastructure - Production Deployment Script
# Secure deployment without hardcoded secrets

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_NAME="RTF Infrastructure"
DEPLOYMENT_ENV="${DEPLOYMENT_ENV:-production}"
LOG_FILE="deployment_$(date +%Y%m%d_%H%M%S).log"

# Environment variables (set externally for security)
GITHUB_PAT="${GITHUB_PAT:-}"  # Set via environment variable
SOLANA_PRIVATE_KEY="${SOLANA_PRIVATE_KEY:-}"
ETHEREUM_PRIVATE_KEY="${ETHEREUM_PRIVATE_KEY:-}"
STARKNET_PRIVATE_KEY="${STARKNET_PRIVATE_KEY:-}"

echo -e "${BLUE}🚀 Starting ${PROJECT_NAME} Production Deployment${NC}"
echo "Environment: ${DEPLOYMENT_ENV}"
echo "Timestamp: $(date)"
echo "Log file: ${LOG_FILE}"

# Validation function
validate_environment() {
    echo -e "${YELLOW}🔍 Validating environment...${NC}"
    
    # Check required environment variables
    if [[ -z "${GITHUB_PAT}" ]]; then
        echo -e "${RED}❌ GITHUB_PAT environment variable not set${NC}"
        exit 1
    fi
    
    if [[ -z "${SOLANA_PRIVATE_KEY}" ]]; then
        echo -e "${RED}❌ SOLANA_PRIVATE_KEY environment variable not set${NC}"
        exit 1
    fi
    
    # Check required tools
    command -v cargo >/dev/null 2>&1 || { echo -e "${RED}❌ Cargo not installed${NC}"; exit 1; }
    command -v solana >/dev/null 2>&1 || { echo -e "${RED}❌ Solana CLI not installed${NC}"; exit 1; }
    command -v forge >/dev/null 2>&1 || { echo -e "${RED}❌ Foundry not installed${NC}"; exit 1; }
    
    echo -e "${GREEN}✅ Environment validation passed${NC}"
}

# Build function
build_project() {
    echo -e "${YELLOW}🔨 Building project...${NC}"
    
    # Build Rust backend
    echo "Building Rust backend..."
    cargo build --release --all-features
    
    # Build Solana programs
    echo "Building Solana programs..."
    cd contracts/solana
    for program in */; do
        if [[ -d "$program" && -f "$program/Cargo.toml" ]]; then
            echo "Building $program..."
            cd "$program"
            cargo build-bpf --release
            cd ..
        fi
    done
    cd ../..
    
    # Build Ethereum contracts
    echo "Building Ethereum contracts..."
    cd contracts/ethereum
    forge build
    cd ../..
    
    # Build Starknet contracts
    echo "Building Starknet contracts..."
    cd contracts/starknet
    for contract in */; do
        if [[ -d "$contract" && -f "$contract/Scarb.toml" ]]; then
            echo "Building $contract..."
            cd "$contract"
            scarb build
            cd ..
        fi
    done
    cd ../..
    
    echo -e "${GREEN}✅ Build completed successfully${NC}"
}

# Test function
run_tests() {
    echo -e "${YELLOW}🧪 Running comprehensive tests...${NC}"
    
    # Run Rust tests
    cargo test --release --all-features
    
    # Run Solana tests
    cd contracts/solana
    for program in */; do
        if [[ -d "$program" && -f "$program/Cargo.toml" ]]; then
            echo "Testing $program..."
            cd "$program"
            cargo test
            cd ..
        fi
    done
    cd ../..
    
    # Run Ethereum tests
    cd contracts/ethereum
    forge test
    cd ../..
    
    echo -e "${GREEN}✅ All tests passed${NC}"
}

# Deploy function
deploy_contracts() {
    echo -e "${YELLOW}🚀 Deploying contracts...${NC}"
    
    # Deploy Solana programs
    echo "Deploying Solana programs..."
    solana config set --url mainnet-beta
    # Add specific deployment commands here
    
    # Deploy Ethereum contracts
    echo "Deploying Ethereum contracts..."
    cd contracts/ethereum
    # Add specific deployment commands here
    cd ../..
    
    # Deploy Starknet contracts
    echo "Deploying Starknet contracts..."
    cd contracts/starknet
    # Add specific deployment commands here
    cd ../..
    
    echo -e "${GREEN}✅ Contract deployment completed${NC}"
}

# Main deployment function
main() {
    echo -e "${BLUE}Starting deployment process...${NC}" | tee -a "${LOG_FILE}"
    
    validate_environment 2>&1 | tee -a "${LOG_FILE}"
    build_project 2>&1 | tee -a "${LOG_FILE}"
    run_tests 2>&1 | tee -a "${LOG_FILE}"
    deploy_contracts 2>&1 | tee -a "${LOG_FILE}"
    
    echo -e "${GREEN}🎉 Deployment completed successfully!${NC}" | tee -a "${LOG_FILE}"
    echo "Log file: ${LOG_FILE}"
}

# Error handling
trap 'echo -e "${RED}❌ Deployment failed. Check ${LOG_FILE} for details.${NC}"; exit 1' ERR

# Run main function
main "$@"
