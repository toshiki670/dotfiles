#!/usr/bin/env bash
#
# Development and testing script for dotfiles in container
#

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Detect docker compose command
if docker compose version >/dev/null 2>&1; then
    DOCKER_COMPOSE="docker compose"
elif command -v docker-compose >/dev/null 2>&1; then
    DOCKER_COMPOSE="docker-compose"
else
    echo -e "${RED}[ERROR]${NC} Docker Compose is not installed."
    echo "Please install Docker Compose: https://docs.docker.com/compose/install/"
    exit 1
fi

# Functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if Docker is running
check_docker() {
    if ! docker info > /dev/null 2>&1; then
        log_error "Docker is not running. Please start Docker."
        exit 1
    fi
    log_success "Docker is running"
}

# Build container
build() {
    log_info "Building Docker image..."
    $DOCKER_COMPOSE build
    log_success "Docker image built successfully"
}

# Start container
start() {
    log_info "Starting container..."
    $DOCKER_COMPOSE up -d
    log_success "Container started"
}

# Stop container
stop() {
    log_info "Stopping container..."
    $DOCKER_COMPOSE down
    log_success "Container stopped"
}

# Enter container shell
shell() {
    log_info "Entering container shell..."
    $DOCKER_COMPOSE exec dotfiles-dev /bin/zsh
}

# Run container interactively
run() {
    log_info "Running container interactively..."
    $DOCKER_COMPOSE run --rm dotfiles-dev
}

# Test chezmoi apply
test_apply() {
    log_info "Testing dotfiles setup in container..."
    $DOCKER_COMPOSE exec dotfiles-dev /bin/zsh -l -c "
        echo '=== Checking installed tools ==='
        which zsh nvim mise sheldon eza bat fd rg zoxide fzf delta
        echo ''
        echo '=== Checking dotfiles ==='
        ls -la ~/
        echo ''
        echo '=== Checking zsh configuration ==='
        cat ~/.zshrc | head -20
        echo ''
        echo '=== Shell info ==='
        echo \"SHELL: \$SHELL\"
        echo \"ZSH_VERSION: \$ZSH_VERSION\"
    "
    log_success "Dotfiles setup test completed"
}

# Clean up
clean() {
    log_info "Cleaning up Docker resources..."
    $DOCKER_COMPOSE down -v
    docker image rm dotfiles-dev 2>/dev/null || true
    log_success "Cleanup completed"
}

# Show usage
usage() {
    cat << EOF
Usage: $0 [COMMAND]

Commands:
    build       Build Docker image
    start       Start container in background
    stop        Stop running container
    shell       Open shell in running container
    run         Run container interactively (one-off)
    test        Test chezmoi apply in container
    clean       Clean up Docker resources
    help        Show this help message

Examples:
    $0 build        # Build the Docker image
    $0 run          # Start container and enter shell
    $0 test         # Test dotfiles installation
    $0 clean        # Remove all Docker resources

EOF
}

# Main
main() {
    check_docker

    case "${1:-help}" in
        build)
            build
            ;;
        start)
            start
            ;;
        stop)
            stop
            ;;
        shell)
            shell
            ;;
        run)
            run
            ;;
        test)
            start
            test_apply
            ;;
        clean)
            clean
            ;;
        help|*)
            usage
            ;;
    esac
}

main "$@"
