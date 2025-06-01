#!/bin/bash
# Version Control Integration for VR Headset Project
# This script sets up Git hooks and CI/CD integration for the project

set -e  # Exit immediately if a command exits with a non-zero status

# Configuration
PROJECT_ROOT="/home/ubuntu/orb_slam3_project"
CI_DIR="${PROJECT_ROOT}/system_ui/vr_core_api/ci"
HOOKS_DIR="${PROJECT_ROOT}/.git/hooks"
LOG_DIR="${PROJECT_ROOT}/build/logs"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
VC_LOG="${LOG_DIR}/version_control_${TIMESTAMP}.log"

# Create necessary directories
mkdir -p "${LOG_DIR}"

# Log function
log() {
    local message="$1"
    local level=${2:-"INFO"}
    local timestamp=$(date +"%Y-%m-%d %H:%M:%S")
    echo "[${timestamp}] [${level}] ${message}" | tee -a "${VC_LOG}"
}

# Error handler
error_handler() {
    log "Version control integration failed at line $1" "ERROR"
    exit 1
}

trap 'error_handler $LINENO' ERR

# Print version control information
log "Starting version control integration for VR Headset Project"
log "Project root: ${PROJECT_ROOT}"

# Check if git is initialized
check_git_initialized() {
    log "Checking if Git is initialized"
    
    if [ -d "${PROJECT_ROOT}/.git" ]; then
        log "Git repository is already initialized"
        return 0
    else
        log "Git repository is not initialized, initializing now"
        cd "${PROJECT_ROOT}"
        git init
        log "Git repository initialized"
        return 0
    fi
}

# Setup Git configuration
setup_git_config() {
    log "Setting up Git configuration"
    
    cd "${PROJECT_ROOT}"
    
    # Set user information if not already set
    if [ -z "$(git config --get user.name)" ]; then
        git config --local user.name "VR Headset CI"
        log "Git user.name set to 'VR Headset CI'"
    fi
    
    if [ -z "$(git config --get user.email)" ]; then
        git config --local user.email "ci@vr-headset-project.local"
        log "Git user.email set to 'ci@vr-headset-project.local'"
    fi
    
    # Set core configuration
    git config --local core.autocrlf input
    git config --local core.safecrlf warn
    git config --local core.fileMode true
    
    log "Git configuration completed"
}

# Create .gitignore file
create_gitignore() {
    log "Creating .gitignore file"
    
    cat > "${PROJECT_ROOT}/.gitignore" << EOF
# Build artifacts
/build/
/target/
**/target/

# Dependencies
/node_modules/
**/node_modules/

# IDE files
.idea/
.vscode/
*.iml
*.swp
*.swo

# Logs
*.log
logs/

# Temporary files
*.tmp
*.temp
.DS_Store

# Rust specific
Cargo.lock
**/*.rs.bk

# Environment variables
.env
.env.local

# Compiled files
*.o
*.so
*.dylib
*.dll
*.exe
*.out

# Generated documentation
/docs/generated/

# Test coverage
/coverage/
.nyc_output/

# Package files
*.zip
*.tar.gz
*.tgz

# Specific to VR Headset Project
/system_ui/vr_core_api/target/
/system_ui/vr_cli/target/
/system_ui/vr_web/target/
/system_ui/vr_web/frontend/vr_frontend/dist/
/system_ui/vr_web/frontend/vr_frontend/node_modules/
EOF
    
    log ".gitignore file created"
}

# Setup pre-commit hook
setup_pre_commit_hook() {
    log "Setting up pre-commit hook"
    
    mkdir -p "${HOOKS_DIR}"
    
    cat > "${HOOKS_DIR}/pre-commit" << 'EOF'
#!/bin/bash
# Pre-commit hook for VR Headset Project

# Configuration
PROJECT_ROOT=$(git rev-parse --show-toplevel)
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

echo "Running pre-commit checks..."

# Check for Rust formatting
if command -v rustfmt &> /dev/null; then
    echo "Checking Rust formatting..."
    
    # Get all staged Rust files
    RUST_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep '\.rs$')
    
    if [ -n "$RUST_FILES" ]; then
        for file in $RUST_FILES; do
            if [ -f "$file" ]; then
                rustfmt --check "$file"
                if [ $? -ne 0 ]; then
                    echo "Error: $file is not properly formatted. Run 'rustfmt $file' to fix."
                    exit 1
                fi
            fi
        done
    fi
fi

# Check for Cargo.toml version consistency
if command -v cargo &> /dev/null; then
    echo "Checking Cargo.toml version consistency..."
    
    # Get all staged Cargo.toml files
    CARGO_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep 'Cargo.toml$')
    
    if [ -n "$CARGO_FILES" ]; then
        # Extract versions and check consistency
        VERSIONS=$(grep -h '^version = ' $CARGO_FILES | sort | uniq)
        VERSION_COUNT=$(echo "$VERSIONS" | wc -l)
        
        if [ $VERSION_COUNT -gt 1 ]; then
            echo "Error: Inconsistent versions found in Cargo.toml files:"
            echo "$VERSIONS"
            echo "Please ensure all components have the same version."
            exit 1
        fi
    fi
fi

# Run clippy for Rust code quality
if command -v cargo-clippy &> /dev/null; then
    echo "Running Clippy for Rust code quality..."
    
    # Get directories with staged Rust files
    RUST_DIRS=$(git diff --cached --name-only --diff-filter=ACM | grep '\.rs$' | xargs -I{} dirname {} | sort | uniq)
    
    if [ -n "$RUST_DIRS" ]; then
        for dir in $RUST_DIRS; do
            if [ -f "$dir/Cargo.toml" ]; then
                (cd "$dir" && cargo clippy -- -D warnings)
                if [ $? -ne 0 ]; then
                    echo "Error: Clippy found issues in $dir"
                    exit 1
                fi
            fi
        done
    fi
fi

# Check for large files
echo "Checking for large files..."
# Find files larger than 10MB
LARGE_FILES=$(git diff --cached --name-only --diff-filter=ACM | xargs -I{} du -m {} 2>/dev/null | awk '$1 > 10 {print $2}')

if [ -n "$LARGE_FILES" ]; then
    echo "Error: The following files are too large (>10MB):"
    echo "$LARGE_FILES"
    echo "Please remove these files from your commit."
    exit 1
fi

echo "Pre-commit checks passed!"
exit 0
EOF
    
    chmod +x "${HOOKS_DIR}/pre-commit"
    log "Pre-commit hook created and made executable"
}

# Setup pre-push hook
setup_pre_push_hook() {
    log "Setting up pre-push hook"
    
    cat > "${HOOKS_DIR}/pre-push" << 'EOF'
#!/bin/bash
# Pre-push hook for VR Headset Project

# Configuration
PROJECT_ROOT=$(git rev-parse --show-toplevel)
CI_DIR="${PROJECT_ROOT}/system_ui/vr_core_api/ci"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

echo "Running pre-push checks..."

# Run unit tests
if [ -f "${CI_DIR}/test_automation.sh" ]; then
    echo "Running unit tests..."
    "${CI_DIR}/test_automation.sh" "unit" "simulation" "false"
    if [ $? -ne 0 ]; then
        echo "Error: Unit tests failed. Push aborted."
        exit 1
    fi
fi

# Check for security vulnerabilities in dependencies
if command -v cargo-audit &> /dev/null; then
    echo "Checking for security vulnerabilities in dependencies..."
    
    # Get all directories with Cargo.toml
    CARGO_DIRS=$(find "$PROJECT_ROOT" -name "Cargo.toml" -exec dirname {} \; | sort | uniq)
    
    if [ -n "$CARGO_DIRS" ]; then
        for dir in $CARGO_DIRS; do
            (cd "$dir" && cargo audit)
            if [ $? -ne 0 ]; then
                echo "Warning: Security vulnerabilities found in $dir dependencies."
                echo "Consider updating the affected dependencies before pushing."
                # Not failing the push, just warning
            fi
        done
    fi
fi

echo "Pre-push checks passed!"
exit 0
EOF
    
    chmod +x "${HOOKS_DIR}/pre-push"
    log "Pre-push hook created and made executable"
}

# Create CI workflow configuration
create_ci_workflow() {
    log "Creating CI workflow configuration"
    
    mkdir -p "${PROJECT_ROOT}/.github/workflows"
    
    cat > "${PROJECT_ROOT}/.github/workflows/ci.yml" << EOF
name: VR Headset CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]

jobs:
  build:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Set up Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: aarch64-unknown-linux-gnu
        override: true
    
    - name: Install cross-compilation tools
      run: |
        sudo apt-get update
        sudo apt-get install -y gcc-aarch64-linux-gnu g++-aarch64-linux-gnu
    
    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: \${{ runner.os }}-cargo-\${{ hashFiles('**/Cargo.lock') }}
    
    - name: Run build pipeline
      run: ./system_ui/vr_core_api/ci/build_pipeline.sh debug true
    
    - name: Run tests
      run: ./system_ui/vr_core_api/ci/test_automation.sh all simulation true
    
    - name: Upload artifacts
      uses: actions/upload-artifact@v3
      with:
        name: vr-headset-artifacts
        path: build/artifacts/
    
    - name: Upload test results
      uses: actions/upload-artifact@v3
      with:
        name: vr-headset-test-results
        path: build/test_results/

  deploy-staging:
    needs: build
    if: github.ref == 'refs/heads/develop'
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Download artifacts
      uses: actions/download-artifact@v3
      with:
        name: vr-headset-artifacts
        path: build/artifacts/
    
    - name: Set up SSH
      uses: webfactory/ssh-agent@v0.7.0
      with:
        ssh-private-key: \${{ secrets.STAGING_SSH_KEY }}
    
    - name: Deploy to staging
      run: |
        export TARGET_HOST_OVERRIDE=\${{ secrets.STAGING_HOST }}
        ./system_ui/vr_core_api/ci/deployment_automation.sh staging full false

  deploy-production:
    needs: build
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    environment: production
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Download artifacts
      uses: actions/download-artifact@v3
      with:
        name: vr-headset-artifacts
        path: build/artifacts/
    
    - name: Set up SSH
      uses: webfactory/ssh-agent@v0.7.0
      with:
        ssh-private-key: \${{ secrets.PRODUCTION_SSH_KEY }}
    
    - name: Deploy to production
      run: |
        export TARGET_HOST_OVERRIDE=\${{ secrets.PRODUCTION_HOST }}
        ./system_ui/vr_core_api/ci/deployment_automation.sh production full false
EOF
    
    log "CI workflow configuration created"
}

# Create branch protection configuration
create_branch_protection() {
    log "Creating branch protection configuration"
    
    mkdir -p "${PROJECT_ROOT}/.github"
    
    cat > "${PROJECT_ROOT}/.github/branch-protection.yml" << EOF
# Branch protection rules for VR Headset Project
# This file defines the branch protection rules to be applied to the repository
# It should be used with GitHub's branch protection API or UI

branches:
  - name: main
    protection:
      required_status_checks:
        strict: true
        contexts:
          - "build"
          - "deploy-production"
      required_pull_request_reviews:
        required_approving_review_count: 2
        dismiss_stale_reviews: true
        require_code_owner_reviews: true
      enforce_admins: true
      restrictions: null
      
  - name: develop
    protection:
      required_status_checks:
        strict: true
        contexts:
          - "build"
          - "deploy-staging"
      required_pull_request_reviews:
        required_approving_review_count: 1
        dismiss_stale_reviews: true
        require_code_owner_reviews: false
      enforce_admins: false
      restrictions: null
EOF
    
    log "Branch protection configuration created"
}

# Create CODEOWNERS file
create_codeowners() {
    log "Creating CODEOWNERS file"
    
    mkdir -p "${PROJECT_ROOT}/.github"
    
    cat > "${PROJECT_ROOT}/.github/CODEOWNERS" << EOF
# VR Headset Project CODEOWNERS
# This file defines code ownership for the repository
# Format: path/to/file/or/directory @username_or_team

# Default owners for everything in the repo
* @vr-headset-admin

# Core API
/system_ui/vr_core_api/ @vr-headset-core-team

# CLI Interface
/system_ui/vr_cli/ @vr-headset-cli-team

# Web Interface
/system_ui/vr_web/ @vr-headset-web-team

# CI/CD
/system_ui/vr_core_api/ci/ @vr-headset-devops
/.github/ @vr-headset-devops

# Documentation
/docs/ @vr-headset-docs
EOF
    
    log "CODEOWNERS file created"
}

# Create version control documentation
create_vc_documentation() {
    log "Creating version control documentation"
    
    mkdir -p "${PROJECT_ROOT}/docs/version_control"
    
    cat > "${PROJECT_ROOT}/docs/version_control/README.md" << EOF
# Version Control Guidelines for VR Headset Project

This document outlines the version control practices and workflows for the VR Headset Project.

## Branching Strategy

We follow a modified GitFlow workflow:

- \`main\`: Production-ready code. All code in this branch has been thoroughly tested and is ready for release.
- \`develop\`: Integration branch for features. This branch contains the latest development changes.
- \`feature/*\`: Feature branches for new functionality. Branch from \`develop\` and merge back to \`develop\`.
- \`bugfix/*\`: Bug fix branches. Branch from \`develop\` for non-critical bugs.
- \`hotfix/*\`: Hotfix branches for critical issues. Branch from \`main\` and merge to both \`main\` and \`develop\`.
- \`release/*\`: Release preparation branches. Branch from \`develop\` when preparing a new release.

## Commit Guidelines

- Use descriptive commit messages that explain what changes were made and why.
- Follow the conventional commits format: \`type(scope): description\`
  - Types: feat, fix, docs, style, refactor, test, chore
  - Example: \`feat(display): add support for higher refresh rates\`
- Keep commits focused on a single logical change.
- Reference issue numbers in commit messages when applicable.

## Pull Request Process

1. Create a pull request from your feature/bugfix branch to the appropriate target branch.
2. Ensure all CI checks pass.
3. Request reviews from appropriate team members.
4. Address all review comments.
5. Once approved, the PR can be merged.

## Code Review Guidelines

- Review for functionality, code quality, and adherence to project standards.
- Provide constructive feedback.
- Approve only when all issues have been addressed.
- For \`main\` branch, at least two approvals are required.
- For \`develop\` branch, at least one approval is required.

## CI/CD Integration

Our CI/CD pipeline automatically:

1. Builds the project for the target platform (Orange Pi CM5).
2. Runs all tests (unit, integration, system, performance, security).
3. Deploys to staging environment for changes to \`develop\`.
4. Deploys to production environment for changes to \`main\`.

## Git Hooks

The repository includes several Git hooks:

- \`pre-commit\`: Checks code formatting, runs linters, and prevents large files from being committed.
- \`pre-push\`: Runs unit tests and checks for security vulnerabilities in dependencies.

## Version Tagging

- We use semantic versioning (MAJOR.MINOR.PATCH).
- Tags are created for each release and should be signed.
- Example: \`v1.2.3\`

## Branch Protection

Both \`main\` and \`develop\` branches are protected:

- Direct pushes are not allowed.
- Pull requests require CI checks to pass.
- Pull requests require code review approvals.
- \`main\` branch requires stricter reviews than \`develop\`.
EOF
    
    log "Version control documentation created"
}

# Main function
main() {
    log "Starting main version control integration process"
    
    # Check if git is initialized
    check_git_initialized
    
    # Setup Git configuration
    setup_git_config
    
    # Create .gitignore file
    create_gitignore
    
    # Setup Git hooks
    setup_pre_commit_hook
    setup_pre_push_hook
    
    # Create CI workflow configuration
    create_ci_workflow
    
    # Create branch protection configuration
    create_branch_protection
    
    # Create CODEOWNERS file
    create_codeowners
    
    # Create version control documentation
    create_vc_documentation
    
    log "Version control integration completed successfully"
}

# Execute main function
main
