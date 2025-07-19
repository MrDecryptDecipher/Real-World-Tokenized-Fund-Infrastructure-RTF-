#!/bin/bash

# RTF Infrastructure Deployment Script
# Advanced Multi-Chain Fund Management Platform
# Version: 1.0.0

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="/home/ubuntu/Sandeep/projects/RTF"
BACKUP_DIR="/home/ubuntu/backups/rtf"
LOG_FILE="/var/log/rtf-deploy.log"
ENVIRONMENT="${1:-production}"
SKIP_TESTS="${2:-false}"

# ASCII Art Banner
echo -e "${PURPLE}"
cat << "EOF"
╔══════════════════════════════════════════════════════════════════════════════╗
║                                                                              ║
║    ██████╗ ████████╗███████╗    ██████╗ ███████╗██████╗ ██╗      ██████╗ ██╗ ║
║    ██╔══██╗╚══██╔══╝██╔════╝    ██╔══██╗██╔════╝██╔══██╗██║     ██╔═══██╗╚██╗║
║    ██████╔╝   ██║   █████╗      ██║  ██║█████╗  ██████╔╝██║     ██║   ██║ ╚██║
║    ██╔══██╗   ██║   ██╔══╝      ██║  ██║██╔══╝  ██╔═══╝ ██║     ██║   ██║ ██╔╝║
║    ██║  ██║   ██║   ██║         ██████╔╝███████╗██║     ███████╗╚██████╔╝██╔╝ ║
║    ╚═╝  ╚═╝   ╚═╝   ╚═╝         ╚═════╝ ╚══════╝╚═╝     ╚══════╝ ╚═════╝ ╚═╝  ║
║                                                                              ║
║              Real-World Tokenized Fund Infrastructure                        ║
║                    Advanced Multi-Chain Deployment                          ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Logging function
log() {
    local level=$1
    shift
    local message="$*"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo -e "${timestamp} [${level}] ${message}" | tee -a "${LOG_FILE}"
}

# Error handling
error_exit() {
    log "ERROR" "$1"
    echo -e "${RED}❌ Deployment failed: $1${NC}"
    exit 1
}

# Success message
success() {
    log "INFO" "$1"
    echo -e "${GREEN}✅ $1${NC}"
}

# Warning message
warning() {
    log "WARN" "$1"
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Info message
info() {
    log "INFO" "$1"
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Check if running as root
check_root() {
    if [[ $EUID -eq 0 ]]; then
        error_exit "This script should not be run as root for security reasons"
    fi
}

# Check system requirements
check_requirements() {
    info "Checking system requirements..."
    
    # Check if required commands exist
    local required_commands=("cargo" "node" "npm" "pm2" "nginx" "psql" "redis-cli" "git")
    for cmd in "${required_commands[@]}"; do
        if ! command -v "$cmd" &> /dev/null; then
            error_exit "Required command '$cmd' not found. Please install it first."
        fi
    done
    
    # Check Rust version
    local rust_version=$(cargo --version | cut -d' ' -f2)
    if [[ $(echo "$rust_version 1.75.0" | tr " " "\n" | sort -V | head -n1) != "1.75.0" ]]; then
        error_exit "Rust version 1.75.0 or higher required. Current: $rust_version"
    fi
    
    # Check Node.js version
    local node_version=$(node --version | cut -d'v' -f2)
    if [[ $(echo "$node_version 20.0.0" | tr " " "\n" | sort -V | head -n1) != "20.0.0" ]]; then
        error_exit "Node.js version 20.0.0 or higher required. Current: $node_version"
    fi
    
    # Check available disk space (minimum 10GB)
    local available_space=$(df "$PROJECT_ROOT" | awk 'NR==2 {print $4}')
    if [[ $available_space -lt 10485760 ]]; then # 10GB in KB
        error_exit "Insufficient disk space. At least 10GB required."
    fi
    
    # Check available memory (minimum 4GB)
    local available_memory=$(free -m | awk 'NR==2{print $7}')
    if [[ $available_memory -lt 4096 ]]; then
        warning "Low available memory detected. Deployment may be slow."
    fi
    
    success "System requirements check passed"
}

# Create backup
create_backup() {
    if [[ "$ENVIRONMENT" == "production" ]]; then
        info "Creating backup before deployment..."
        
        local backup_timestamp=$(date '+%Y%m%d_%H%M%S')
        local backup_path="${BACKUP_DIR}/rtf_backup_${backup_timestamp}"
        
        mkdir -p "$backup_path"
        
        # Backup database
        if pg_dump rtf_production > "${backup_path}/database.sql" 2>/dev/null; then
            success "Database backup created"
        else
            warning "Database backup failed or database doesn't exist"
        fi
        
        # Backup configuration
        if [[ -d "${PROJECT_ROOT}/config" ]]; then
            cp -r "${PROJECT_ROOT}/config" "${backup_path}/"
            success "Configuration backup created"
        fi
        
        # Backup PM2 processes
        pm2 save > /dev/null 2>&1 || true
        if [[ -f ~/.pm2/dump.pm2 ]]; then
            cp ~/.pm2/dump.pm2 "${backup_path}/"
            success "PM2 processes backup created"
        fi
        
        success "Backup created at $backup_path"
    fi
}

# Stop existing services safely
stop_services() {
    info "Stopping existing RTF services..."
    
    # Stop PM2 processes gracefully
    local rtf_processes=("rtf-api" "rtf-oracle" "rtf-compliance" "rtf-treasury" "rtf-zk-nav" "rtf-cross-chain" "rtf-metrics" "rtf-frontend")
    
    for process in "${rtf_processes[@]}"; do
        if pm2 describe "$process" > /dev/null 2>&1; then
            info "Stopping $process..."
            pm2 stop "$process" > /dev/null 2>&1 || warning "Failed to stop $process"
        fi
    done
    
    # Wait for graceful shutdown
    sleep 5
    
    success "Services stopped"
}

# Build Rust components
build_rust() {
    info "Building Rust components..."
    
    cd "$PROJECT_ROOT"
    
    # Clean previous builds
    cargo clean
    
    # Build with optimizations
    if [[ "$ENVIRONMENT" == "production" ]]; then
        RUSTFLAGS="-C target-cpu=native" cargo build --release --workspace
    else
        cargo build --workspace
    fi
    
    success "Rust components built successfully"
}

# Build frontend
build_frontend() {
    info "Building frontend application..."
    
    cd "${PROJECT_ROOT}/frontend"
    
    # Install dependencies
    npm ci --production=false
    
    # Build for production
    if [[ "$ENVIRONMENT" == "production" ]]; then
        npm run build
    else
        npm run build:dev
    fi
    
    success "Frontend built successfully"
}

# Run tests
run_tests() {
    if [[ "$SKIP_TESTS" == "true" ]]; then
        warning "Skipping tests as requested"
        return
    fi
    
    info "Running comprehensive test suite..."
    
    cd "$PROJECT_ROOT"
    
    # Run Rust tests
    info "Running Rust unit tests..."
    cargo test --workspace --release
    
    # Run integration tests
    info "Running integration tests..."
    cargo test --test integration --release
    
    # Run property-based tests
    info "Running property-based tests..."
    cargo test --features proptest --release
    
    # Run frontend tests
    info "Running frontend tests..."
    cd "${PROJECT_ROOT}/frontend"
    npm test -- --coverage --watchAll=false
    
    # Run end-to-end tests
    info "Running end-to-end tests..."
    cd "$PROJECT_ROOT"
    ./scripts/e2e-tests.sh
    
    success "All tests passed"
}

# Deploy smart contracts
deploy_contracts() {
    info "Deploying smart contracts..."
    
    cd "$PROJECT_ROOT"
    
    # Deploy Solana contracts
    info "Deploying Solana contracts..."
    ./scripts/deploy-solana.sh "$ENVIRONMENT"
    
    # Deploy Ethereum contracts
    info "Deploying Ethereum contracts..."
    ./scripts/deploy-ethereum.sh "$ENVIRONMENT"
    
    # Deploy Starknet contracts
    info "Deploying Starknet contracts..."
    ./scripts/deploy-starknet.sh "$ENVIRONMENT"
    
    success "Smart contracts deployed"
}

# Setup database
setup_database() {
    info "Setting up database..."
    
    local db_name="rtf_${ENVIRONMENT}"
    
    # Create database if it doesn't exist
    if ! psql -lqt | cut -d \| -f 1 | grep -qw "$db_name"; then
        createdb "$db_name"
        success "Database $db_name created"
    fi
    
    # Run migrations
    cd "$PROJECT_ROOT"
    sqlx migrate run --database-url "postgresql://localhost/$db_name"
    
    success "Database setup completed"
}

# Configure NGINX
configure_nginx() {
    info "Configuring NGINX..."
    
    # Copy NGINX configuration
    sudo cp "${PROJECT_ROOT}/infrastructure/nginx/rtf.conf" /etc/nginx/sites-available/
    
    # Enable site
    sudo ln -sf /etc/nginx/sites-available/rtf.conf /etc/nginx/sites-enabled/
    
    # Test NGINX configuration
    if sudo nginx -t; then
        success "NGINX configuration is valid"
    else
        error_exit "NGINX configuration is invalid"
    fi
    
    # Reload NGINX
    sudo systemctl reload nginx
    
    success "NGINX configured and reloaded"
}

# Start services
start_services() {
    info "Starting RTF services..."
    
    cd "$PROJECT_ROOT"
    
    # Start PM2 processes
    pm2 start infrastructure/pm2/ecosystem.config.js --env "$ENVIRONMENT"
    
    # Wait for services to start
    sleep 10
    
    # Check service health
    local services=("rtf-api" "rtf-oracle" "rtf-compliance" "rtf-treasury" "rtf-zk-nav" "rtf-cross-chain" "rtf-metrics" "rtf-frontend")
    
    for service in "${services[@]}"; do
        if pm2 describe "$service" | grep -q "online"; then
            success "$service is running"
        else
            error_exit "$service failed to start"
        fi
    done
    
    # Save PM2 configuration
    pm2 save
    
    success "All services started successfully"
}

# Health check
health_check() {
    info "Performing health checks..."
    
    local api_url="http://localhost:2102"
    local max_attempts=30
    local attempt=1
    
    while [[ $attempt -le $max_attempts ]]; do
        if curl -f -s "${api_url}/health" > /dev/null; then
            success "API health check passed"
            break
        fi
        
        if [[ $attempt -eq $max_attempts ]]; then
            error_exit "API health check failed after $max_attempts attempts"
        fi
        
        info "Waiting for API to be ready... (attempt $attempt/$max_attempts)"
        sleep 2
        ((attempt++))
    done
    
    # Test key endpoints
    local endpoints=("/api/v1/health" "/api/v1/status" "/metrics")
    
    for endpoint in "${endpoints[@]}"; do
        if curl -f -s "${api_url}${endpoint}" > /dev/null; then
            success "Endpoint $endpoint is responding"
        else
            warning "Endpoint $endpoint is not responding"
        fi
    done
    
    success "Health checks completed"
}

# Performance verification
verify_performance() {
    info "Verifying performance metrics..."
    
    # Test API response time (target: <400ms)
    local response_time=$(curl -o /dev/null -s -w '%{time_total}' http://localhost:2102/api/v1/health)
    local response_time_ms=$(echo "$response_time * 1000" | bc)
    
    if (( $(echo "$response_time_ms < 400" | bc -l) )); then
        success "API response time: ${response_time_ms}ms (target: <400ms)"
    else
        warning "API response time: ${response_time_ms}ms exceeds target of 400ms"
    fi
    
    # Check memory usage
    local memory_usage=$(ps aux | grep rtf | awk '{sum+=$6} END {print sum/1024}')
    info "Total memory usage: ${memory_usage}MB"
    
    # Check CPU usage
    local cpu_usage=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1)
    info "Current CPU usage: ${cpu_usage}%"
    
    success "Performance verification completed"
}

# Cleanup
cleanup() {
    info "Performing cleanup..."
    
    # Clean build artifacts
    cd "$PROJECT_ROOT"
    cargo clean
    
    # Clean npm cache
    cd "${PROJECT_ROOT}/frontend"
    npm cache clean --force
    
    # Clean old logs (keep last 7 days)
    find /var/log -name "rtf-*.log" -mtime +7 -delete 2>/dev/null || true
    
    success "Cleanup completed"
}

# Main deployment function
main() {
    local start_time=$(date +%s)
    
    echo -e "${CYAN}🚀 Starting RTF Infrastructure Deployment${NC}"
    echo -e "${CYAN}Environment: $ENVIRONMENT${NC}"
    echo -e "${CYAN}Skip Tests: $SKIP_TESTS${NC}"
    echo ""
    
    # Pre-deployment checks
    check_root
    check_requirements
    
    # Deployment steps
    create_backup
    stop_services
    build_rust
    build_frontend
    run_tests
    deploy_contracts
    setup_database
    configure_nginx
    start_services
    health_check
    verify_performance
    cleanup
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    echo ""
    echo -e "${GREEN}🎉 RTF Infrastructure Deployment Completed Successfully!${NC}"
    echo -e "${GREEN}⏱️  Total deployment time: ${duration} seconds${NC}"
    echo ""
    echo -e "${BLUE}📊 Service Status:${NC}"
    pm2 status
    echo ""
    echo -e "${BLUE}🌐 Access URLs:${NC}"
    echo -e "${BLUE}  Frontend: http://localhost:2101${NC}"
    echo -e "${BLUE}  API: http://localhost:2102${NC}"
    echo -e "${BLUE}  Metrics: http://localhost:2108/metrics${NC}"
    echo ""
    echo -e "${PURPLE}🔗 Next Steps:${NC}"
    echo -e "${PURPLE}  1. Configure SSL certificates for production${NC}"
    echo -e "${PURPLE}  2. Set up monitoring and alerting${NC}"
    echo -e "${PURPLE}  3. Configure backup automation${NC}"
    echo -e "${PURPLE}  4. Review security settings${NC}"
    echo ""
}

# Run main function
main "$@"
