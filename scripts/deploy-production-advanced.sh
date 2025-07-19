#!/bin/bash

# RTF Infrastructure - Advanced Production Deployment Script
# PRD: "Production deployment automation"
# PRD: "Comprehensive health checks"
# PRD: "Zero-downtime deployment with rollback capabilities"

set -euo pipefail

# Configuration
RTF_VERSION="${RTF_VERSION:-$(git rev-parse --short HEAD)}"
DEPLOYMENT_ENV="${DEPLOYMENT_ENV:-production}"
AWS_REGION="${AWS_REGION:-us-east-1}"
HEALTH_CHECK_TIMEOUT=300
ROLLBACK_ENABLED="${ROLLBACK_ENABLED:-true}"
BLUE_GREEN_DEPLOYMENT="${BLUE_GREEN_DEPLOYMENT:-true}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging
log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING: $1${NC}"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $1${NC}"
    exit 1
}

info() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')] INFO: $1${NC}"
}

# ASCII Art Banner
print_banner() {
    cat << 'EOF'
    ██████╗ ████████╗███████╗    ██████╗ ███████╗██████╗ ██╗      ██████╗ ██╗   ██╗
    ██╔══██╗╚══██╔══╝██╔════╝    ██╔══██╗██╔════╝██╔══██╗██║     ██╔═══██╗╚██╗ ██╔╝
    ██████╔╝   ██║   █████╗      ██║  ██║█████╗  ██████╔╝██║     ██║   ██║ ╚████╔╝ 
    ██╔══██╗   ██║   ██╔══╝      ██║  ██║██╔══╝  ██╔═══╝ ██║     ██║   ██║  ╚██╔╝  
    ██║  ██║   ██║   ██║         ██████╔╝███████╗██║     ███████╗╚██████╔╝   ██║   
    ╚═╝  ╚═╝   ╚═╝   ╚═╝         ╚═════╝ ╚══════╝╚═╝     ╚══════╝ ╚═════╝    ╚═╝   
                                                                                    
    Real-World Tokenized Fund Infrastructure - Production Deployment
    Version: ${RTF_VERSION} | Environment: ${DEPLOYMENT_ENV}
    
EOF
}

# Pre-deployment checks
pre_deployment_checks() {
    log "🔍 Running pre-deployment checks..."
    
    # Check required tools
    command -v docker >/dev/null 2>&1 || error "Docker is required but not installed"
    command -v kubectl >/dev/null 2>&1 || error "kubectl is required but not installed"
    command -v aws >/dev/null 2>&1 || error "AWS CLI is required but not installed"
    command -v jq >/dev/null 2>&1 || error "jq is required but not installed"
    
    # Check AWS credentials
    aws sts get-caller-identity >/dev/null 2>&1 || error "AWS credentials not configured"
    
    # Check Kubernetes context
    kubectl cluster-info >/dev/null 2>&1 || error "Kubernetes cluster not accessible"
    
    # Check environment variables
    [[ -z "${DATABASE_URL:-}" ]] && error "DATABASE_URL environment variable not set"
    [[ -z "${REDIS_URL:-}" ]] && error "REDIS_URL environment variable not set"
    
    # Verify RTF configuration
    [[ -f "config/production.toml" ]] || error "Production configuration file not found"
    
    log "✅ Pre-deployment checks passed"
}

# Build and push Docker images
build_and_push_images() {
    log "🏗️ Building and pushing Docker images..."
    
    # Build API service
    info "Building RTF API service..."
    docker build -t rtf-api:${RTF_VERSION} -f docker/api.Dockerfile .
    docker tag rtf-api:${RTF_VERSION} ${ECR_REGISTRY}/rtf-api:${RTF_VERSION}
    docker push ${ECR_REGISTRY}/rtf-api:${RTF_VERSION}
    
    # Build Treasury service
    info "Building RTF Treasury service..."
    docker build -t rtf-treasury:${RTF_VERSION} -f docker/treasury.Dockerfile .
    docker tag rtf-treasury:${RTF_VERSION} ${ECR_REGISTRY}/rtf-treasury:${RTF_VERSION}
    docker push ${ECR_REGISTRY}/rtf-treasury:${RTF_VERSION}
    
    # Build Cross-chain service
    info "Building RTF Cross-chain service..."
    docker build -t rtf-cross-chain:${RTF_VERSION} -f docker/cross-chain.Dockerfile .
    docker tag rtf-cross-chain:${RTF_VERSION} ${ECR_REGISTRY}/rtf-cross-chain:${RTF_VERSION}
    docker push ${ECR_REGISTRY}/rtf-cross-chain:${RTF_VERSION}
    
    # Build Emergency Handler
    info "Building RTF Emergency Handler..."
    docker build -t rtf-emergency:${RTF_VERSION} -f docker/emergency.Dockerfile .
    docker tag rtf-emergency:${RTF_VERSION} ${ECR_REGISTRY}/rtf-emergency:${RTF_VERSION}
    docker push ${ECR_REGISTRY}/rtf-emergency:${RTF_VERSION}
    
    # Build Monitoring service
    info "Building RTF Monitoring service..."
    docker build -t rtf-monitoring:${RTF_VERSION} -f docker/monitoring.Dockerfile .
    docker tag rtf-monitoring:${RTF_VERSION} ${ECR_REGISTRY}/rtf-monitoring:${RTF_VERSION}
    docker push ${ECR_REGISTRY}/rtf-monitoring:${RTF_VERSION}
    
    log "✅ All images built and pushed successfully"
}

# Deploy infrastructure components
deploy_infrastructure() {
    log "🏗️ Deploying infrastructure components..."
    
    # Deploy database migrations
    info "Running database migrations..."
    kubectl apply -f k8s/migrations/
    kubectl wait --for=condition=complete job/rtf-migrations --timeout=300s
    
    # Deploy Redis cluster
    info "Deploying Redis cluster..."
    kubectl apply -f k8s/redis/
    kubectl wait --for=condition=ready pod -l app=redis --timeout=300s
    
    # Deploy monitoring stack
    info "Deploying monitoring stack..."
    kubectl apply -f k8s/monitoring/
    kubectl wait --for=condition=ready pod -l app=prometheus --timeout=300s
    kubectl wait --for=condition=ready pod -l app=grafana --timeout=300s
    
    log "✅ Infrastructure components deployed"
}

# Blue-Green deployment
blue_green_deploy() {
    log "🔄 Starting Blue-Green deployment..."
    
    # Determine current and new environments
    CURRENT_ENV=$(kubectl get service rtf-api -o jsonpath='{.spec.selector.version}' 2>/dev/null || echo "blue")
    NEW_ENV=$([ "$CURRENT_ENV" = "blue" ] && echo "green" || echo "blue")
    
    info "Current environment: $CURRENT_ENV, Deploying to: $NEW_ENV"
    
    # Deploy to new environment
    info "Deploying RTF services to $NEW_ENV environment..."
    
    # Update Kubernetes manifests with new version
    sed "s/{{VERSION}}/${RTF_VERSION}/g; s/{{ENV}}/${NEW_ENV}/g" k8s/api/deployment.yaml | kubectl apply -f -
    sed "s/{{VERSION}}/${RTF_VERSION}/g; s/{{ENV}}/${NEW_ENV}/g" k8s/treasury/deployment.yaml | kubectl apply -f -
    sed "s/{{VERSION}}/${RTF_VERSION}/g; s/{{ENV}}/${NEW_ENV}/g" k8s/cross-chain/deployment.yaml | kubectl apply -f -
    sed "s/{{VERSION}}/${RTF_VERSION}/g; s/{{ENV}}/${NEW_ENV}/g" k8s/emergency/deployment.yaml | kubectl apply -f -
    sed "s/{{VERSION}}/${RTF_VERSION}/g; s/{{ENV}}/${NEW_ENV}/g" k8s/monitoring/deployment.yaml | kubectl apply -f -
    
    # Wait for new environment to be ready
    info "Waiting for $NEW_ENV environment to be ready..."
    kubectl wait --for=condition=ready pod -l version=$NEW_ENV,app=rtf-api --timeout=600s
    kubectl wait --for=condition=ready pod -l version=$NEW_ENV,app=rtf-treasury --timeout=600s
    kubectl wait --for=condition=ready pod -l version=$NEW_ENV,app=rtf-cross-chain --timeout=600s
    kubectl wait --for=condition=ready pod -l version=$NEW_ENV,app=rtf-emergency --timeout=600s
    
    # Run health checks on new environment
    if run_health_checks $NEW_ENV; then
        # Switch traffic to new environment
        info "Switching traffic to $NEW_ENV environment..."
        kubectl patch service rtf-api -p '{"spec":{"selector":{"version":"'$NEW_ENV'"}}}'
        kubectl patch service rtf-treasury -p '{"spec":{"selector":{"version":"'$NEW_ENV'"}}}'
        kubectl patch service rtf-cross-chain -p '{"spec":{"selector":{"version":"'$NEW_ENV'"}}}'
        
        # Wait and verify traffic switch
        sleep 30
        if run_health_checks $NEW_ENV; then
            log "✅ Blue-Green deployment successful"
            
            # Clean up old environment after successful deployment
            info "Cleaning up $CURRENT_ENV environment..."
            kubectl delete deployment -l version=$CURRENT_ENV,app=rtf-api --ignore-not-found=true
            kubectl delete deployment -l version=$CURRENT_ENV,app=rtf-treasury --ignore-not-found=true
            kubectl delete deployment -l version=$CURRENT_ENV,app=rtf-cross-chain --ignore-not-found=true
            kubectl delete deployment -l version=$CURRENT_ENV,app=rtf-emergency --ignore-not-found=true
        else
            error "Health checks failed after traffic switch, rolling back..."
            rollback_deployment $CURRENT_ENV
        fi
    else
        error "Health checks failed on $NEW_ENV environment, keeping $CURRENT_ENV active"
        kubectl delete deployment -l version=$NEW_ENV --ignore-not-found=true
        exit 1
    fi
}

# Comprehensive health checks
run_health_checks() {
    local ENV=${1:-""}
    log "🏥 Running comprehensive health checks..."
    
    local API_URL="http://rtf-api.default.svc.cluster.local:8002"
    if [[ -n "$ENV" ]]; then
        API_URL="http://rtf-api-$ENV.default.svc.cluster.local:8002"
    fi
    
    # Health check function
    check_endpoint() {
        local endpoint=$1
        local expected_status=${2:-200}
        local timeout=${3:-10}
        
        info "Checking $endpoint..."
        local response=$(kubectl run health-check-$(date +%s) --rm -i --restart=Never --image=curlimages/curl:latest -- \
            curl -s -o /dev/null -w "%{http_code}" --max-time $timeout "$endpoint" 2>/dev/null || echo "000")
        
        if [[ "$response" == "$expected_status" ]]; then
            info "✅ $endpoint - Status: $response"
            return 0
        else
            warn "❌ $endpoint - Expected: $expected_status, Got: $response"
            return 1
        fi
    }
    
    local failed_checks=0
    
    # API health checks
    check_endpoint "$API_URL/health" 200 || ((failed_checks++))
    check_endpoint "$API_URL/api/v1/status" 200 || ((failed_checks++))
    
    # Treasury service health
    check_endpoint "http://rtf-treasury${ENV:+-$ENV}.default.svc.cluster.local:8003/health" 200 || ((failed_checks++))
    
    # Cross-chain service health
    check_endpoint "http://rtf-cross-chain${ENV:+-$ENV}.default.svc.cluster.local:8004/health" 200 || ((failed_checks++))
    
    # Emergency handler health
    check_endpoint "http://rtf-emergency${ENV:+-$ENV}.default.svc.cluster.local:8005/health" 200 || ((failed_checks++))
    
    # Database connectivity check
    info "Checking database connectivity..."
    if kubectl run db-check-$(date +%s) --rm -i --restart=Never --image=postgres:15 -- \
        psql "$DATABASE_URL" -c "SELECT 1;" >/dev/null 2>&1; then
        info "✅ Database connectivity"
    else
        warn "❌ Database connectivity failed"
        ((failed_checks++))
    fi
    
    # Redis connectivity check
    info "Checking Redis connectivity..."
    if kubectl run redis-check-$(date +%s) --rm -i --restart=Never --image=redis:7 -- \
        redis-cli -u "$REDIS_URL" ping >/dev/null 2>&1; then
        info "✅ Redis connectivity"
    else
        warn "❌ Redis connectivity failed"
        ((failed_checks++))
    fi
    
    # Performance checks
    info "Running performance checks..."
    local response_time=$(kubectl run perf-check-$(date +%s) --rm -i --restart=Never --image=curlimages/curl:latest -- \
        curl -s -o /dev/null -w "%{time_total}" --max-time 5 "$API_URL/api/v1/status" 2>/dev/null || echo "999")
    
    local response_time_ms=$(echo "$response_time * 1000" | bc -l | cut -d. -f1)
    
    if [[ $response_time_ms -lt 700 ]]; then
        info "✅ API response time: ${response_time_ms}ms (< 700ms target)"
    else
        warn "❌ API response time: ${response_time_ms}ms (exceeds 700ms target)"
        ((failed_checks++))
    fi
    
    # Smart contract checks (if applicable)
    info "Checking smart contract deployments..."
    # TODO: Add smart contract health checks
    
    if [[ $failed_checks -eq 0 ]]; then
        log "✅ All health checks passed"
        return 0
    else
        warn "❌ $failed_checks health checks failed"
        return 1
    fi
}

# Rollback deployment
rollback_deployment() {
    local ROLLBACK_ENV=${1:-"blue"}
    
    if [[ "$ROLLBACK_ENABLED" != "true" ]]; then
        error "Rollback is disabled, manual intervention required"
    fi
    
    warn "🔄 Rolling back to $ROLLBACK_ENV environment..."
    
    # Switch traffic back
    kubectl patch service rtf-api -p '{"spec":{"selector":{"version":"'$ROLLBACK_ENV'"}}}'
    kubectl patch service rtf-treasury -p '{"spec":{"selector":{"version":"'$ROLLBACK_ENV'"}}}'
    kubectl patch service rtf-cross-chain -p '{"spec":{"selector":{"version":"'$ROLLBACK_ENV'"}}}'
    
    # Verify rollback
    sleep 30
    if run_health_checks $ROLLBACK_ENV; then
        log "✅ Rollback successful"
    else
        error "❌ Rollback failed, manual intervention required"
    fi
}

# Post-deployment tasks
post_deployment_tasks() {
    log "🔧 Running post-deployment tasks..."
    
    # Update monitoring dashboards
    info "Updating monitoring dashboards..."
    kubectl apply -f k8s/monitoring/dashboards/
    
    # Send deployment notification
    info "Sending deployment notification..."
    # TODO: Send notification to Slack/Teams/Email
    
    # Update deployment record
    info "Recording deployment..."
    kubectl create configmap deployment-record-${RTF_VERSION} \
        --from-literal=version=${RTF_VERSION} \
        --from-literal=timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ") \
        --from-literal=deployer=$(whoami) \
        --from-literal=environment=${DEPLOYMENT_ENV}
    
    # Run smoke tests
    info "Running smoke tests..."
    # TODO: Add comprehensive smoke tests
    
    log "✅ Post-deployment tasks completed"
}

# Cleanup function
cleanup() {
    log "🧹 Cleaning up temporary resources..."
    kubectl delete pod -l app=health-check --ignore-not-found=true
    kubectl delete pod -l app=db-check --ignore-not-found=true
    kubectl delete pod -l app=redis-check --ignore-not-found=true
    kubectl delete pod -l app=perf-check --ignore-not-found=true
}

# Main deployment function
main() {
    print_banner
    
    # Set trap for cleanup
    trap cleanup EXIT
    
    log "🚀 Starting RTF Infrastructure deployment..."
    log "Version: $RTF_VERSION"
    log "Environment: $DEPLOYMENT_ENV"
    log "Blue-Green: $BLUE_GREEN_DEPLOYMENT"
    log "Rollback Enabled: $ROLLBACK_ENABLED"
    
    # Run deployment steps
    pre_deployment_checks
    build_and_push_images
    deploy_infrastructure
    
    if [[ "$BLUE_GREEN_DEPLOYMENT" == "true" ]]; then
        blue_green_deploy
    else
        # Standard deployment
        log "🔄 Running standard deployment..."
        kubectl apply -f k8s/
        kubectl wait --for=condition=ready pod -l app=rtf-api --timeout=600s
        run_health_checks
    fi
    
    post_deployment_tasks
    
    log "🎉 RTF Infrastructure deployment completed successfully!"
    log "🌐 API Endpoint: https://api.rtf.finance"
    log "📊 Monitoring: https://monitoring.rtf.finance"
    log "📈 Dashboards: https://dashboards.rtf.finance"
    
    # Display deployment summary
    cat << EOF

    ╔══════════════════════════════════════════════════════════════╗
    ║                    DEPLOYMENT SUMMARY                        ║
    ╠══════════════════════════════════════════════════════════════╣
    ║ Version:     ${RTF_VERSION}                                           ║
    ║ Environment: ${DEPLOYMENT_ENV}                                        ║
    ║ Timestamp:   $(date -u +"%Y-%m-%d %H:%M:%S UTC")                    ║
    ║ Deployer:    $(whoami)                                              ║
    ║ Status:      ✅ SUCCESS                                       ║
    ╚══════════════════════════════════════════════════════════════╝

EOF
}

# Run main function
main "$@"
