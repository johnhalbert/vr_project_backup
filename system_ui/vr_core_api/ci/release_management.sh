#!/bin/bash
# Release Management for VR Headset Project
# This script automates the release process for the VR headset system

set -e  # Exit immediately if a command exits with a non-zero status

# Configuration
PROJECT_ROOT="/home/ubuntu/orb_slam3_project"
BUILD_DIR="${PROJECT_ROOT}/build"
RELEASE_DIR="${BUILD_DIR}/releases"
ARTIFACTS_DIR="${BUILD_DIR}/artifacts"
LOG_DIR="${BUILD_DIR}/logs"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RELEASE_LOG="${LOG_DIR}/release_${TIMESTAMP}.log"

# Release information
RELEASE_TYPE=${1:-"patch"}  # major, minor, patch
RELEASE_NOTES=${2:-""}  # Path to release notes file
RELEASE_BRANCH=${3:-"main"}  # Branch to release from
RELEASE_PREFIX="v"  # Prefix for version tags

# Create necessary directories
mkdir -p "${BUILD_DIR}"
mkdir -p "${RELEASE_DIR}"
mkdir -p "${LOG_DIR}"

# Log function
log() {
    local message="$1"
    local level=${2:-"INFO"}
    local timestamp=$(date +"%Y-%m-%d %H:%M:%S")
    echo "[${timestamp}] [${level}] ${message}" | tee -a "${RELEASE_LOG}"
}

# Error handler
error_handler() {
    log "Release management failed at line $1" "ERROR"
    exit 1
}

trap 'error_handler $LINENO' ERR

# Print release information
log "Starting release management for VR Headset Project"
log "Release type: ${RELEASE_TYPE}"
log "Release branch: ${RELEASE_BRANCH}"
log "Project root: ${PROJECT_ROOT}"

# Check for required tools
log "Checking for required tools..."
for tool in git jq zip unzip; do
    if ! command -v $tool &> /dev/null; then
        log "Required tool not found: $tool" "ERROR"
        exit 1
    fi
done
log "All required tools are available"

# Get current version
get_current_version() {
    log "Getting current version"
    
    # Check if version file exists
    if [ -f "${PROJECT_ROOT}/VERSION" ]; then
        local version=$(cat "${PROJECT_ROOT}/VERSION")
        log "Current version from VERSION file: ${version}"
        echo "${version}"
        return
    fi
    
    # Try to get version from Cargo.toml
    if [ -f "${PROJECT_ROOT}/system_ui/vr_core_api/Cargo.toml" ]; then
        local version=$(grep -m 1 '^version = ' "${PROJECT_ROOT}/system_ui/vr_core_api/Cargo.toml" | cut -d '"' -f 2)
        if [ -n "${version}" ]; then
            log "Current version from Cargo.toml: ${version}"
            echo "${version}"
            return
        fi
    fi
    
    # Try to get version from git tags
    if git -C "${PROJECT_ROOT}" describe --tags --abbrev=0 2>/dev/null; then
        local version=$(git -C "${PROJECT_ROOT}" describe --tags --abbrev=0 | sed "s/^${RELEASE_PREFIX}//")
        log "Current version from git tags: ${version}"
        echo "${version}"
        return
    fi
    
    # Default to 0.1.0 if no version found
    log "No version found, defaulting to 0.1.0" "WARNING"
    echo "0.1.0"
}

# Calculate next version
calculate_next_version() {
    local current_version="$1"
    local release_type="$2"
    
    log "Calculating next version based on ${current_version} and type ${release_type}"
    
    # Split version into major, minor, patch
    local major=$(echo "${current_version}" | cut -d. -f1)
    local minor=$(echo "${current_version}" | cut -d. -f2)
    local patch=$(echo "${current_version}" | cut -d. -f3)
    
    # Calculate next version based on release type
    if [ "${release_type}" = "major" ]; then
        major=$((major + 1))
        minor=0
        patch=0
    elif [ "${release_type}" = "minor" ]; then
        minor=$((minor + 1))
        patch=0
    else  # patch
        patch=$((patch + 1))
    fi
    
    local next_version="${major}.${minor}.${patch}"
    log "Next version: ${next_version}"
    echo "${next_version}"
}

# Update version in files
update_version_files() {
    local version="$1"
    log "Updating version in files to ${version}"
    
    # Update VERSION file
    echo "${version}" > "${PROJECT_ROOT}/VERSION"
    log "Updated VERSION file"
    
    # Update Cargo.toml files
    find "${PROJECT_ROOT}" -name "Cargo.toml" -type f -exec sed -i "s/^version = \".*\"/version = \"${version}\"/" {} \;
    log "Updated version in Cargo.toml files"
    
    # Update package.json files
    find "${PROJECT_ROOT}" -name "package.json" -type f -exec sed -i "s/\"version\": \".*\"/\"version\": \"${version}\"/" {} \;
    log "Updated version in package.json files"
}

# Generate changelog
generate_changelog() {
    local version="$1"
    local release_notes="$2"
    log "Generating changelog for version ${version}"
    
    local changelog_file="${PROJECT_ROOT}/CHANGELOG.md"
    local temp_changelog="${BUILD_DIR}/CHANGELOG.tmp"
    local date_str=$(date +"%Y-%m-%d")
    
    # Create changelog file if it doesn't exist
    if [ ! -f "${changelog_file}" ]; then
        echo "# Changelog" > "${changelog_file}"
        echo "" >> "${changelog_file}"
        echo "All notable changes to this project will be documented in this file." >> "${changelog_file}"
        echo "" >> "${changelog_file}"
    fi
    
    # Create new changelog entry
    echo "## [${version}] - ${date_str}" > "${temp_changelog}"
    echo "" >> "${temp_changelog}"
    
    # Add release notes if provided
    if [ -n "${release_notes}" ] && [ -f "${release_notes}" ]; then
        cat "${release_notes}" >> "${temp_changelog}"
    else
        # Generate changelog from git commits
        echo "### Added" >> "${temp_changelog}"
        echo "" >> "${temp_changelog}"
        git -C "${PROJECT_ROOT}" log --pretty=format:"- %s" --grep="^feat" $(git -C "${PROJECT_ROOT}" describe --tags --abbrev=0 2>/dev/null || echo "HEAD~100")..HEAD | grep -v Merge | sed 's/^feat: //' | sed 's/^feat(\([^)]*\)): /\1: /' >> "${temp_changelog}" || echo "- No new features" >> "${temp_changelog}"
        echo "" >> "${temp_changelog}"
        echo "" >> "${temp_changelog}"
        
        echo "### Fixed" >> "${temp_changelog}"
        echo "" >> "${temp_changelog}"
        git -C "${PROJECT_ROOT}" log --pretty=format:"- %s" --grep="^fix" $(git -C "${PROJECT_ROOT}" describe --tags --abbrev=0 2>/dev/null || echo "HEAD~100")..HEAD | grep -v Merge | sed 's/^fix: //' | sed 's/^fix(\([^)]*\)): /\1: /' >> "${temp_changelog}" || echo "- No bug fixes" >> "${temp_changelog}"
        echo "" >> "${temp_changelog}"
        echo "" >> "${temp_changelog}"
        
        echo "### Changed" >> "${temp_changelog}"
        echo "" >> "${temp_changelog}"
        git -C "${PROJECT_ROOT}" log --pretty=format:"- %s" --grep="^refactor\|^style\|^perf" $(git -C "${PROJECT_ROOT}" describe --tags --abbrev=0 2>/dev/null || echo "HEAD~100")..HEAD | grep -v Merge | sed 's/^refactor: //' | sed 's/^refactor(\([^)]*\)): /\1: /' | sed 's/^style: //' | sed 's/^style(\([^)]*\)): /\1: /' | sed 's/^perf: //' | sed 's/^perf(\([^)]*\)): /\1: /' >> "${temp_changelog}" || echo "- No changes" >> "${temp_changelog}"
        echo "" >> "${temp_changelog}"
        echo "" >> "${temp_changelog}"
        
        echo "### Documentation" >> "${temp_changelog}"
        echo "" >> "${temp_changelog}"
        git -C "${PROJECT_ROOT}" log --pretty=format:"- %s" --grep="^docs" $(git -C "${PROJECT_ROOT}" describe --tags --abbrev=0 2>/dev/null || echo "HEAD~100")..HEAD | grep -v Merge | sed 's/^docs: //' | sed 's/^docs(\([^)]*\)): /\1: /' >> "${temp_changelog}" || echo "- No documentation changes" >> "${temp_changelog}"
        echo "" >> "${temp_changelog}"
    fi
    
    # Add link to version comparison
    echo "" >> "${temp_changelog}"
    local prev_version=$(get_current_version)
    echo "[${version}]: https://github.com/vr-headset-project/orb_slam3_project/compare/${RELEASE_PREFIX}${prev_version}...${RELEASE_PREFIX}${version}" >> "${temp_changelog}"
    echo "" >> "${temp_changelog}"
    
    # Prepend new changelog to existing file
    if [ -f "${changelog_file}" ]; then
        # Get the first line (title)
        head -n 1 "${changelog_file}" > "${changelog_file}.new"
        echo "" >> "${changelog_file}.new"
        
        # Add the new content
        cat "${temp_changelog}" >> "${changelog_file}.new"
        echo "" >> "${changelog_file}.new"
        
        # Add the rest of the old file, skipping the title
        tail -n +2 "${changelog_file}" >> "${changelog_file}.new"
        
        # Replace the old file
        mv "${changelog_file}.new" "${changelog_file}"
    else
        # Create new file
        echo "# Changelog" > "${changelog_file}"
        echo "" >> "${changelog_file}"
        cat "${temp_changelog}" >> "${changelog_file}"
    fi
    
    # Clean up
    rm -f "${temp_changelog}"
    
    log "Changelog generated: ${changelog_file}"
}

# Commit version changes
commit_version_changes() {
    local version="$1"
    log "Committing version changes for ${version}"
    
    cd "${PROJECT_ROOT}"
    
    # Add changed files
    git add VERSION CHANGELOG.md $(find . -name "Cargo.toml" -o -name "package.json")
    
    # Commit changes
    git commit -m "chore: bump version to ${version}"
    
    log "Version changes committed"
}

# Create release tag
create_release_tag() {
    local version="$1"
    log "Creating release tag ${RELEASE_PREFIX}${version}"
    
    cd "${PROJECT_ROOT}"
    
    # Create annotated tag
    git tag -a "${RELEASE_PREFIX}${version}" -m "Release ${version}"
    
    log "Release tag created"
}

# Build release artifacts
build_release_artifacts() {
    local version="$1"
    log "Building release artifacts for version ${version}"
    
    # Run build pipeline in release mode
    "${PROJECT_ROOT}/system_ui/vr_core_api/ci/build_pipeline.sh" "release" "false"
    
    # Run documentation generation
    "${PROJECT_ROOT}/system_ui/vr_core_api/ci/documentation_generation.sh"
    
    log "Release artifacts built"
}

# Package release
package_release() {
    local version="$1"
    log "Packaging release ${version}"
    
    local release_package_dir="${RELEASE_DIR}/${version}"
    mkdir -p "${release_package_dir}"
    
    # Copy artifacts
    cp -r "${ARTIFACTS_DIR}"/* "${release_package_dir}/"
    
    # Copy documentation
    mkdir -p "${release_package_dir}/docs"
    cp -r "${BUILD_DIR}/docs"/* "${release_package_dir}/docs/"
    
    # Copy changelog
    cp "${PROJECT_ROOT}/CHANGELOG.md" "${release_package_dir}/"
    
    # Create version file
    echo "${version}" > "${release_package_dir}/VERSION"
    
    # Create release metadata
    cat > "${release_package_dir}/release.json" << EOF
{
  "version": "${version}",
  "timestamp": "$(date -Iseconds)",
  "commit": "$(git -C "${PROJECT_ROOT}" rev-parse HEAD)",
  "branch": "${RELEASE_BRANCH}",
  "tag": "${RELEASE_PREFIX}${version}",
  "type": "${RELEASE_TYPE}",
  "components": [
    {"name": "core_api", "version": "${version}"},
    {"name": "cli", "version": "${version}"},
    {"name": "web", "version": "${version}"}
  ]
}
EOF
    
    # Create release package
    cd "${RELEASE_DIR}"
    local release_package="vr_headset_${version}.zip"
    zip -r "${release_package}" "${version}"
    
    log "Release packaged: ${RELEASE_DIR}/${release_package}"
}

# Generate release notes
generate_release_notes() {
    local version="$1"
    log "Generating release notes for version ${version}"
    
    local release_notes_file="${RELEASE_DIR}/release_notes_${version}.md"
    
    # Extract relevant section from changelog
    sed -n "/## \[${version}\]/,/## \[/p" "${PROJECT_ROOT}/CHANGELOG.md" | sed '$d' > "${release_notes_file}"
    
    # Add header if file is empty
    if [ ! -s "${release_notes_file}" ]; then
        echo "# Release Notes for Version ${version}" > "${release_notes_file}"
        echo "" >> "${release_notes_file}"
        echo "This release includes various improvements and bug fixes." >> "${release_notes_file}"
    fi
    
    log "Release notes generated: ${release_notes_file}"
    echo "${release_notes_file}"
}

# Create GitHub release
create_github_release() {
    local version="$1"
    local release_notes_file="$2"
    log "Creating GitHub release for version ${version}"
    
    # This would typically use the GitHub API, but we'll just simulate it for now
    log "GitHub release would be created with tag ${RELEASE_PREFIX}${version} and release notes from ${release_notes_file}"
    
    # Create a simulated GitHub release file
    local github_release_file="${RELEASE_DIR}/github_release_${version}.json"
    cat > "${github_release_file}" << EOF
{
  "tag_name": "${RELEASE_PREFIX}${version}",
  "target_commitish": "${RELEASE_BRANCH}",
  "name": "Release ${version}",
  "body": "$(cat "${release_notes_file}" | sed 's/"/\\"/g' | tr '\n' ' ')",
  "draft": false,
  "prerelease": false,
  "assets": [
    {
      "name": "vr_headset_${version}.zip",
      "path": "${RELEASE_DIR}/vr_headset_${version}.zip",
      "content_type": "application/zip"
    }
  ]
}
EOF
    
    log "Simulated GitHub release created: ${github_release_file}"
}

# Main release process
main() {
    log "Starting main release process"
    
    # Get current version
    local current_version=$(get_current_version)
    
    # Calculate next version
    local next_version=$(calculate_next_version "${current_version}" "${RELEASE_TYPE}")
    
    # Update version in files
    update_version_files "${next_version}"
    
    # Generate changelog
    generate_changelog "${next_version}" "${RELEASE_NOTES}"
    
    # Commit version changes
    commit_version_changes "${next_version}"
    
    # Create release tag
    create_release_tag "${next_version}"
    
    # Build release artifacts
    build_release_artifacts "${next_version}"
    
    # Package release
    package_release "${next_version}"
    
    # Generate release notes
    local release_notes_file=$(generate_release_notes "${next_version}")
    
    # Create GitHub release
    create_github_release "${next_version}" "${release_notes_file}"
    
    log "Release ${next_version} completed successfully"
}

# Execute main function
main
