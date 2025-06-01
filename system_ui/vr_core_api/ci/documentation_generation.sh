#!/bin/bash
# Documentation Generation for VR Headset Project
# This script automates the generation of documentation for all components

set -e  # Exit immediately if a command exits with a non-zero status

# Configuration
PROJECT_ROOT="/home/ubuntu/orb_slam3_project"
DOCS_DIR="${PROJECT_ROOT}/docs"
BUILD_DIR="${PROJECT_ROOT}/build"
DOCS_BUILD_DIR="${BUILD_DIR}/docs"
LOG_DIR="${BUILD_DIR}/logs"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
DOCS_LOG="${LOG_DIR}/docs_generation_${TIMESTAMP}.log"

# Create necessary directories
mkdir -p "${BUILD_DIR}"
mkdir -p "${DOCS_BUILD_DIR}"
mkdir -p "${LOG_DIR}"
mkdir -p "${DOCS_DIR}"

# Log function
log() {
    local message="$1"
    local level=${2:-"INFO"}
    local timestamp=$(date +"%Y-%m-%d %H:%M:%S")
    echo "[${timestamp}] [${level}] ${message}" | tee -a "${DOCS_LOG}"
}

# Error handler
error_handler() {
    log "Documentation generation failed at line $1" "ERROR"
    exit 1
}

trap 'error_handler $LINENO' ERR

# Print documentation information
log "Starting documentation generation for VR Headset Project"
log "Project root: ${PROJECT_ROOT}"
log "Documentation output directory: ${DOCS_BUILD_DIR}"

# Check for required tools
log "Checking for required tools..."
for tool in cargo rustdoc mdbook pandoc; do
    if ! command -v $tool &> /dev/null; then
        log "Required tool not found: $tool" "WARNING"
        if [ "$tool" = "mdbook" ]; then
            log "Installing mdbook..."
            cargo install mdbook
        elif [ "$tool" = "pandoc" ]; then
            log "Installing pandoc..."
            apt-get update && apt-get install -y pandoc
        fi
    fi
done
log "All required tools are available or have been installed"

# Generate Rust API documentation
generate_api_docs() {
    log "Generating Rust API documentation"
    
    local components=("vr_core_api" "vr_cli" "vr_web")
    
    for component in "${components[@]}"; do
        local component_dir="${PROJECT_ROOT}/system_ui/${component}"
        
        if [ -d "${component_dir}" ]; then
            log "Generating API docs for ${component}"
            
            # Create output directory
            mkdir -p "${DOCS_BUILD_DIR}/api/${component}"
            
            # Generate documentation
            cd "${component_dir}"
            RUSTDOCFLAGS="--html-in-header ${DOCS_DIR}/assets/docs-header.html" cargo doc --no-deps --document-private-items
            
            # Copy generated docs to build directory
            cp -r "${component_dir}/target/doc" "${DOCS_BUILD_DIR}/api/${component}/"
            
            log "API documentation for ${component} generated successfully"
        else
            log "Component directory not found: ${component}" "WARNING"
        fi
    done
    
    # Create index page for API documentation
    cat > "${DOCS_BUILD_DIR}/api/index.html" << EOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>VR Headset Project - API Documentation</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
        }
        h1 {
            border-bottom: 1px solid #eaecef;
            padding-bottom: 0.3em;
        }
        .component {
            margin-bottom: 20px;
            padding: 15px;
            border: 1px solid #e1e4e8;
            border-radius: 6px;
        }
        .component h2 {
            margin-top: 0;
        }
        a {
            color: #0366d6;
            text-decoration: none;
        }
        a:hover {
            text-decoration: underline;
        }
    </style>
</head>
<body>
    <h1>VR Headset Project - API Documentation</h1>
    <p>This page provides links to the API documentation for all components of the VR Headset Project.</p>
    
    <div class="component">
        <h2>Core API</h2>
        <p>The Core API provides the fundamental functionality for the VR headset system.</p>
        <a href="./vr_core_api/doc/vr_core_api/index.html">View Core API Documentation</a>
    </div>
    
    <div class="component">
        <h2>CLI Interface</h2>
        <p>The CLI Interface provides command-line tools for interacting with the VR headset system.</p>
        <a href="./vr_cli/doc/vr_cli/index.html">View CLI Interface Documentation</a>
    </div>
    
    <div class="component">
        <h2>Web Interface</h2>
        <p>The Web Interface provides a web-based UI for configuring and monitoring the VR headset system.</p>
        <a href="./vr_web/doc/vr_web/index.html">View Web Interface Documentation</a>
    </div>
    
    <p>Generated on: $(date)</p>
</body>
</html>
EOF
    
    log "API documentation index page created"
}

# Generate developer guides
generate_developer_guides() {
    log "Generating developer guides"
    
    # Create mdbook structure
    local guides_dir="${DOCS_DIR}/developer_guides"
    mkdir -p "${guides_dir}/src"
    
    # Create book.toml
    cat > "${guides_dir}/book.toml" << EOF
[book]
authors = ["VR Headset Project Team"]
language = "en"
multilingual = false
src = "src"
title = "VR Headset Project - Developer Guides"

[output.html]
git-repository-url = "https://github.com/vr-headset-project/orb_slam3_project"
edit-url-template = "https://github.com/vr-headset-project/orb_slam3_project/edit/main/docs/developer_guides/{path}"
additional-css = ["theme/custom.css"]
additional-js = ["theme/custom.js"]

[output.html.playground]
editable = true
line-numbers = true

[output.html.search]
limit-results = 20
use-boolean-and = true
boost-title = 2
boost-hierarchy = 2
boost-paragraph = 1
expand = true
heading-split-level = 2
EOF
    
    # Create custom CSS
    mkdir -p "${guides_dir}/theme"
    cat > "${guides_dir}/theme/custom.css" << EOF
:root {
    --sidebar-width: 300px;
    --page-padding: 15px;
    --content-max-width: 750px;
    --menu-bar-height: 50px;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
}

.chapter li.chapter-item {
    line-height: 1.5em;
}

.content {
    padding: 0 var(--page-padding);
}

.content main {
    margin-top: 20px;
}

pre {
    position: relative;
}

pre > .buttons {
    position: absolute;
    right: 5px;
    top: 5px;
}
EOF
    
    # Create custom JS
    cat > "${guides_dir}/theme/custom.js" << EOF
// Custom JavaScript for the developer guides
document.addEventListener('DOMContentLoaded', function() {
    // Add version info to the bottom of the sidebar
    const sidebar = document.querySelector('.sidebar');
    if (sidebar) {
        const versionInfo = document.createElement('div');
        versionInfo.className = 'version-info';
        versionInfo.style.padding = '10px';
        versionInfo.style.fontSize = '0.8em';
        versionInfo.style.borderTop = '1px solid rgba(0, 0, 0, 0.07)';
        versionInfo.style.marginTop = '10px';
        versionInfo.innerHTML = 'Generated: ${TIMESTAMP}';
        sidebar.appendChild(versionInfo);
    }
});
EOF
    
    # Create SUMMARY.md
    cat > "${guides_dir}/src/SUMMARY.md" << EOF
# Summary

- [Introduction](./introduction.md)
- [Getting Started](./getting_started.md)
- [Architecture](./architecture.md)
- [Core API](./core_api/README.md)
  - [Hardware Access](./core_api/hardware_access.md)
  - [Configuration Management](./core_api/configuration_management.md)
  - [IPC Mechanisms](./core_api/ipc_mechanisms.md)
  - [Security](./core_api/security.md)
  - [Update System](./core_api/update_system.md)
  - [Telemetry and Logging](./core_api/telemetry_logging.md)
  - [Performance Optimization](./core_api/performance_optimization.md)
- [CLI Interface](./cli_interface/README.md)
  - [Command Structure](./cli_interface/command_structure.md)
  - [Configuration Commands](./cli_interface/configuration_commands.md)
  - [Monitoring Commands](./cli_interface/monitoring_commands.md)
  - [Scripting](./cli_interface/scripting.md)
- [Web Interface](./web_interface/README.md)
  - [API Layer](./web_interface/api_layer.md)
  - [Frontend Framework](./web_interface/frontend_framework.md)
  - [Configuration Interface](./web_interface/configuration_interface.md)
  - [Monitoring Dashboard](./web_interface/monitoring_dashboard.md)
- [Testing](./testing/README.md)
  - [Unit Testing](./testing/unit_testing.md)
  - [Integration Testing](./testing/integration_testing.md)
  - [System Testing](./testing/system_testing.md)
  - [Performance Testing](./testing/performance_testing.md)
  - [Security Testing](./testing/security_testing.md)
- [Continuous Integration](./ci/README.md)
  - [Build Pipeline](./ci/build_pipeline.md)
  - [Test Automation](./ci/test_automation.md)
  - [Deployment Automation](./ci/deployment_automation.md)
  - [Version Control](./ci/version_control.md)
  - [Documentation Generation](./ci/documentation_generation.md)
  - [Release Management](./ci/release_management.md)
- [Contributing](./contributing.md)
- [Troubleshooting](./troubleshooting.md)
- [FAQ](./faq.md)
EOF
    
    # Create introduction.md
    cat > "${guides_dir}/src/introduction.md" << EOF
# Introduction

Welcome to the VR Headset Project Developer Guides. This documentation is designed to help developers understand and contribute to the VR headset system.

## About the Project

The VR Headset Project is a comprehensive system for a virtual reality headset based on the Orange Pi CM5 platform. It includes:

- Core API for hardware access, configuration management, and system services
- CLI Interface for command-line interaction
- Web Interface for configuration and monitoring
- Comprehensive testing framework
- Continuous integration and deployment pipeline

## Purpose of This Guide

This guide provides detailed information for developers who want to:

- Understand the system architecture
- Develop new features or fix bugs
- Extend the system with new capabilities
- Test and validate changes
- Deploy the system to target devices

## How to Use This Guide

The guide is organized into sections that cover different aspects of the system. You can read it sequentially or jump to specific sections that interest you.

- **Getting Started**: Setup your development environment
- **Architecture**: Understand the system design
- **Component Guides**: Detailed information about each component
- **Testing**: Learn how to test your changes
- **Continuous Integration**: Understand the CI/CD pipeline
- **Contributing**: Guidelines for contributing to the project

## Target Audience

This guide is intended for software developers who are familiar with:

- Rust programming language
- Linux systems
- Embedded development
- Web technologies (for the Web Interface)

## Feedback and Contributions

We welcome feedback and contributions to both the project and this documentation. Please see the [Contributing](./contributing.md) section for more information.
EOF
    
    # Create CI documentation
    mkdir -p "${guides_dir}/src/ci"
    
    # Create CI README
    cat > "${guides_dir}/src/ci/README.md" << EOF
# Continuous Integration

The VR Headset Project uses a comprehensive Continuous Integration (CI) system to automate building, testing, and deploying the software. This section describes the CI system and how to use it.

## Overview

Our CI system automates the following processes:

- Building the software for the target platform (Orange Pi CM5)
- Running tests to ensure code quality and functionality
- Generating documentation
- Deploying the software to target environments
- Managing releases

## CI Components

The CI system consists of several components:

- [Build Pipeline](./build_pipeline.md): Compiles the code for the target platform
- [Test Automation](./test_automation.md): Runs various types of tests
- [Deployment Automation](./deployment_automation.md): Deploys the software to target devices
- [Version Control](./version_control.md): Integrates with Git for branch protection and code reviews
- [Documentation Generation](./documentation_generation.md): Generates API and developer documentation
- [Release Management](./release_management.md): Manages the release process

## CI Workflow

The typical CI workflow is as follows:

1. Developer pushes code to a feature branch
2. CI system builds the code and runs tests
3. Developer creates a pull request to merge to develop branch
4. CI system builds, tests, and generates documentation
5. Code reviewers approve the pull request
6. Code is merged to develop branch
7. CI system deploys to staging environment
8. When ready for release, a pull request is created to merge to main branch
9. After approval and merge, CI system deploys to production environment

## CI Configuration

The CI system is configured using the following files:

- \`.github/workflows/ci.yml\`: GitHub Actions workflow configuration
- \`system_ui/vr_core_api/ci/build_pipeline.sh\`: Build script
- \`system_ui/vr_core_api/ci/test_automation.sh\`: Test script
- \`system_ui/vr_core_api/ci/deployment_automation.sh\`: Deployment script
- \`system_ui/vr_core_api/ci/version_control_integration.sh\`: Version control setup
- \`system_ui/vr_core_api/ci/documentation_generation.sh\`: Documentation generation script
- \`system_ui/vr_core_api/ci/release_management.sh\`: Release management script

## Using the CI System

Developers typically don't need to interact directly with the CI system. It runs automatically when code is pushed or pull requests are created.

However, you can run the CI scripts locally for testing:

\`\`\`bash
# Run the build pipeline
./system_ui/vr_core_api/ci/build_pipeline.sh

# Run tests
./system_ui/vr_core_api/ci/test_automation.sh

# Generate documentation
./system_ui/vr_core_api/ci/documentation_generation.sh
\`\`\`

## CI Best Practices

- Always run tests locally before pushing code
- Keep the CI pipeline fast by optimizing build and test processes
- Fix CI failures immediately
- Use feature flags for long-running feature development
- Keep documentation up-to-date with code changes
EOF
    
    # Create build pipeline documentation
    cat > "${guides_dir}/src/ci/build_pipeline.md" << EOF
# Build Pipeline

The build pipeline is responsible for compiling the VR headset software for the target platform (Orange Pi CM5).

## Overview

The build pipeline performs the following tasks:

- Sets up the build environment
- Configures cross-compilation if needed
- Compiles the Core API, CLI Interface, and Web Interface components
- Packages the build artifacts

## Build Script

The build pipeline is implemented in \`system_ui/vr_core_api/ci/build_pipeline.sh\`. This script can be run locally or as part of the CI workflow.

## Usage

\`\`\`bash
./system_ui/vr_core_api/ci/build_pipeline.sh [build_type] [verbose]
\`\`\`

Parameters:
- \`build_type\`: \`debug\` (default) or \`release\`
- \`verbose\`: \`true\` or \`false\` (default)

Example:
\`\`\`bash
# Build in release mode with verbose output
./system_ui/vr_core_api/ci/build_pipeline.sh release true
\`\`\`

## Build Process

The build process consists of the following steps:

1. **Environment Setup**: Creates necessary directories and checks for required tools
2. **Cross-Compilation Setup**: Configures cross-compilation for the target platform if needed
3. **Core API Build**: Compiles the Core API library
4. **CLI Interface Build**: Compiles the CLI Interface executable
5. **Web Interface Build**: Compiles the Web Interface backend and frontend
6. **Artifact Packaging**: Packages the build artifacts for deployment

## Build Artifacts

The build pipeline produces the following artifacts:

- Core API library (\`libvr_core_api.rlib\`)
- CLI Interface executable (\`vr_cli\`)
- Web Interface backend executable (\`vr_web\`)
- Web Interface frontend assets

These artifacts are stored in the \`build/artifacts\` directory and are used by the deployment automation script.

## Build Configuration

The build pipeline can be configured using the following environment variables:

- \`PROJECT_ROOT\`: Root directory of the project
- \`BUILD_DIR\`: Directory for build outputs
- \`TARGET_PLATFORM\`: Target platform for cross-compilation

## Troubleshooting

Common build issues and solutions:

- **Missing dependencies**: Ensure all required tools are installed
- **Cross-compilation errors**: Check that the cross-compilation toolchain is properly installed
- **Build failures**: Check the build logs for specific error messages
EOF
    
    # Create documentation generation documentation
    cat > "${guides_dir}/src/ci/documentation_generation.md" << EOF
# Documentation Generation

The documentation generation system automatically creates API documentation and developer guides for the VR headset project.

## Overview

The documentation generation system performs the following tasks:

- Generates API documentation from Rust code using rustdoc
- Creates developer guides using mdBook
- Packages the documentation for deployment

## Documentation Script

The documentation generation is implemented in \`system_ui/vr_core_api/ci/documentation_generation.sh\`. This script can be run locally or as part of the CI workflow.

## Usage

\`\`\`bash
./system_ui/vr_core_api/ci/documentation_generation.sh
\`\`\`

## Documentation Process

The documentation generation process consists of the following steps:

1. **Environment Setup**: Creates necessary directories and checks for required tools
2. **API Documentation**: Generates API documentation for all Rust components
3. **Developer Guides**: Builds the developer guides using mdBook
4. **User Documentation**: Generates user documentation from Markdown files
5. **Documentation Packaging**: Packages the documentation for deployment

## Documentation Outputs

The documentation generation produces the following outputs:

- API documentation in HTML format
- Developer guides in HTML format
- User documentation in HTML and PDF formats

These outputs are stored in the \`build/docs\` directory and can be deployed to a documentation server.

## Writing Documentation

### API Documentation

API documentation is generated from doc comments in the Rust code. Follow these guidelines:

- Use \`///\` for documenting items
- Use Markdown formatting in doc comments
- Include examples where appropriate
- Document all public items

Example:
\`\`\`rust
/// Configures the display settings for the VR headset.
///
/// # Arguments
///
/// * \`resolution\` - The display resolution in pixels (width x height)
/// * \`refresh_rate\` - The display refresh rate in Hz
///
/// # Examples
///
/// \`\`\`
/// let display = Display::new();
/// display.configure(Resolution::new(1920, 1080), 90);
/// \`\`\`
pub fn configure(&mut self, resolution: Resolution, refresh_rate: u32) -> Result<(), DisplayError> {
    // Implementation
}
\`\`\`

### Developer Guides

Developer guides are written in Markdown and organized using mdBook. Follow these guidelines:

- Use clear, concise language
- Include code examples where appropriate
- Organize content logically
- Use headings to structure the content
- Include diagrams where helpful

## Viewing Documentation

After generating the documentation, you can view it locally:

\`\`\`bash
# View API documentation
firefox build/docs/api/index.html

# View developer guides
firefox build/docs/guides/index.html
\`\`\`

## Troubleshooting

Common documentation issues and solutions:

- **Missing tools**: Ensure rustdoc and mdBook are installed
- **Broken links**: Check for incorrect links in the documentation
- **Build failures**: Check the documentation generation logs for specific error messages
EOF
    
    # Build the developer guides
    if command -v mdbook &> /dev/null; then
        log "Building developer guides with mdbook"
        cd "${guides_dir}"
        mdbook build
        
        # Copy built guides to docs build directory
        mkdir -p "${DOCS_BUILD_DIR}/guides"
        cp -r "${guides_dir}/book" "${DOCS_BUILD_DIR}/guides"
        
        log "Developer guides built successfully"
    else
        log "mdbook not available, skipping developer guides build" "WARNING"
    fi
}

# Generate user documentation
generate_user_docs() {
    log "Generating user documentation"
    
    # Create user docs directory
    local user_docs_dir="${DOCS_DIR}/user_docs"
    mkdir -p "${user_docs_dir}"
    
    # Create user manual
    cat > "${user_docs_dir}/user_manual.md" << EOF
# VR Headset User Manual

## Introduction

Welcome to your new VR headset! This manual will guide you through setting up and using your device.

## Getting Started

### What's in the Box

- VR Headset
- Power Adapter
- USB-C Cable
- Quick Start Guide

### Initial Setup

1. Charge your headset using the included USB-C cable and power adapter
2. Power on the headset by pressing and holding the power button for 3 seconds
3. Follow the on-screen instructions to complete the initial setup

## Basic Operation

### Power On/Off

- **Power On**: Press and hold the power button for 3 seconds
- **Power Off**: Press and hold the power button for 5 seconds
- **Sleep Mode**: Press the power button once

### Navigation

- Use the touchpad on the right side of the headset to navigate menus
- Swipe left/right to move between options
- Tap to select
- Swipe down to go back

### Adjusting the Headset

- Adjust the head strap for a comfortable fit
- Use the IPD adjustment to match your interpupillary distance
- Adjust the focus using the focus wheel

## Configuration

### Display Settings

- **Resolution**: Adjust the display resolution
- **Refresh Rate**: Change the display refresh rate
- **Brightness**: Adjust the display brightness

### Audio Settings

- **Volume**: Adjust the audio volume
- **Microphone**: Configure microphone settings
- **Audio Output**: Select audio output device

### Tracking Settings

- **Room Setup**: Configure your play area
- **Controller Tracking**: Calibrate controller tracking
- **Head Tracking**: Adjust head tracking sensitivity

## Troubleshooting

### Common Issues

- **Headset Won't Power On**: Ensure the battery is charged
- **Display Issues**: Check display settings and connections
- **Tracking Problems**: Recalibrate tracking in a well-lit environment

### Factory Reset

If you need to reset your headset to factory settings:

1. Power off the headset
2. Press and hold the power button and volume up button for 10 seconds
3. Follow the on-screen instructions to complete the reset

## Maintenance

- Clean the lenses with the included microfiber cloth
- Keep the headset in a cool, dry place when not in use
- Regularly check for software updates

## Technical Support

For technical support, please contact:

- Email: support@vr-headset-project.com
- Phone: 1-800-VR-SUPPORT
- Website: www.vr-headset-project.com/support
EOF
    
    # Create quick start guide
    cat > "${user_docs_dir}/quick_start_guide.md" << EOF
# VR Headset Quick Start Guide

## 1. Unbox Your Headset

Remove all components from the box:
- VR Headset
- Power Adapter
- USB-C Cable

## 2. Charge Your Headset

Connect the USB-C cable to the headset and power adapter, then plug into a power outlet.
The LED indicator will show:
- Red: Charging
- Green: Fully Charged

## 3. Power On

Press and hold the power button for 3 seconds until the display turns on.

## 4. Initial Setup

Follow the on-screen instructions to:
- Select your language
- Connect to Wi-Fi
- Create or sign in to your account
- Set up your play area

## 5. Adjust for Comfort

- Adjust the head strap for a secure fit
- Set the IPD (interpupillary distance) to match your eyes
- Adjust the focus using the focus wheel

## 6. Start Using Your Headset

- Navigate menus using the touchpad
- Download apps from the store
- Explore virtual environments

## 7. For More Information

Refer to the full user manual or visit:
www.vr-headset-project.com/support
EOF
    
    # Create FAQ
    cat > "${user_docs_dir}/faq.md" << EOF
# Frequently Asked Questions

## General Questions

### How long does the battery last?
The battery lasts approximately 3-4 hours of continuous use, depending on the applications being used.

### Can I use the headset while it's charging?
Yes, you can use the headset while it's charging, but it may become warm during extended use.

### Is the headset waterproof?
No, the headset is not waterproof. Keep it away from water and moisture.

## Setup and Configuration

### How do I connect to Wi-Fi?
Go to Settings > Network > Wi-Fi, select your network, and enter the password.

### How do I adjust the display settings?
Go to Settings > Display to adjust resolution, refresh rate, and brightness.

### How do I pair controllers?
Go to Settings > Controllers > Pair New Controller and follow the on-screen instructions.

## Troubleshooting

### My headset won't turn on
Ensure the battery is charged. If the problem persists, try a factory reset.

### The display is blurry
Adjust the focus using the focus wheel. If the problem persists, clean the lenses with the included microfiber cloth.

### Tracking is not working properly
Ensure you're in a well-lit environment without reflective surfaces. Recalibrate tracking in Settings > Tracking > Calibrate.

## Software and Updates

### How do I update the software?
The headset will automatically check for updates when connected to Wi-Fi. You can manually check for updates in Settings > System > Software Update.

### Can I install custom applications?
Yes, you can enable developer mode in Settings > System > Developer Options and install custom applications.

### How do I manage storage?
Go to Settings > Storage to view storage usage and manage installed applications.

## Accessories

### Can I use external headphones?
Yes, you can connect headphones via the 3.5mm audio jack or Bluetooth.

### Are prescription lens adapters available?
Yes, prescription lens adapters are available from our online store.

### Can I use external controllers?
Yes, the headset supports standard Bluetooth controllers. Go to Settings > Controllers > Pair New Controller to connect.
EOF
    
    # Convert markdown to HTML and PDF if pandoc is available
    if command -v pandoc &> /dev/null; then
        log "Converting user documentation to HTML and PDF with pandoc"
        
        # Create output directories
        mkdir -p "${DOCS_BUILD_DIR}/user/html"
        mkdir -p "${DOCS_BUILD_DIR}/user/pdf"
        
        # Convert to HTML
        for md_file in "${user_docs_dir}"/*.md; do
            filename=$(basename "${md_file}" .md)
            pandoc "${md_file}" -o "${DOCS_BUILD_DIR}/user/html/${filename}.html" --standalone --metadata title="${filename}" -c style.css
        done
        
        # Create CSS for HTML
        cat > "${DOCS_BUILD_DIR}/user/html/style.css" << EOF
body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    line-height: 1.6;
    color: #333;
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
}
h1, h2, h3, h4, h5, h6 {
    color: #2c3e50;
    margin-top: 1.5em;
    margin-bottom: 0.5em;
}
h1 {
    border-bottom: 1px solid #eaecef;
    padding-bottom: 0.3em;
}
a {
    color: #0366d6;
    text-decoration: none;
}
a:hover {
    text-decoration: underline;
}
code {
    background-color: #f6f8fa;
    padding: 0.2em 0.4em;
    border-radius: 3px;
    font-family: SFMono-Regular, Consolas, "Liberation Mono", Menlo, monospace;
}
pre {
    background-color: #f6f8fa;
    padding: 16px;
    border-radius: 3px;
    overflow: auto;
}
pre code {
    background-color: transparent;
    padding: 0;
}
blockquote {
    border-left: 4px solid #dfe2e5;
    padding-left: 16px;
    margin-left: 0;
    color: #6a737d;
}
table {
    border-collapse: collapse;
    width: 100%;
    margin-bottom: 16px;
}
table, th, td {
    border: 1px solid #dfe2e5;
}
th, td {
    padding: 8px 16px;
    text-align: left;
}
th {
    background-color: #f6f8fa;
}
EOF
        
        # Convert to PDF
        for md_file in "${user_docs_dir}"/*.md; do
            filename=$(basename "${md_file}" .md)
            pandoc "${md_file}" -o "${DOCS_BUILD_DIR}/user/pdf/${filename}.pdf" --pdf-engine=wkhtmltopdf --standalone --metadata title="${filename}"
        done
        
        # Create index page for user documentation
        cat > "${DOCS_BUILD_DIR}/user/index.html" << EOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>VR Headset Project - User Documentation</title>
    <link rel="stylesheet" href="html/style.css">
</head>
<body>
    <h1>VR Headset Project - User Documentation</h1>
    <p>This page provides links to the user documentation for the VR Headset Project.</p>
    
    <h2>HTML Documentation</h2>
    <ul>
        <li><a href="html/user_manual.html">User Manual</a></li>
        <li><a href="html/quick_start_guide.html">Quick Start Guide</a></li>
        <li><a href="html/faq.html">Frequently Asked Questions</a></li>
    </ul>
    
    <h2>PDF Documentation</h2>
    <ul>
        <li><a href="pdf/user_manual.pdf">User Manual (PDF)</a></li>
        <li><a href="pdf/quick_start_guide.pdf">Quick Start Guide (PDF)</a></li>
        <li><a href="pdf/faq.pdf">Frequently Asked Questions (PDF)</a></li>
    </ul>
    
    <p>Generated on: $(date)</p>
</body>
</html>
EOF
        
        log "User documentation converted successfully"
    else
        log "pandoc not available, skipping HTML and PDF conversion" "WARNING"
        
        # Copy markdown files directly
        mkdir -p "${DOCS_BUILD_DIR}/user/markdown"
        cp "${user_docs_dir}"/*.md "${DOCS_BUILD_DIR}/user/markdown/"
    fi
}

# Create main documentation index
create_main_index() {
    log "Creating main documentation index"
    
    cat > "${DOCS_BUILD_DIR}/index.html" << EOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>VR Headset Project - Documentation</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
        }
        h1 {
            border-bottom: 1px solid #eaecef;
            padding-bottom: 0.3em;
        }
        .doc-section {
            margin-bottom: 20px;
            padding: 15px;
            border: 1px solid #e1e4e8;
            border-radius: 6px;
        }
        .doc-section h2 {
            margin-top: 0;
        }
        a {
            color: #0366d6;
            text-decoration: none;
        }
        a:hover {
            text-decoration: underline;
        }
    </style>
</head>
<body>
    <h1>VR Headset Project - Documentation</h1>
    <p>Welcome to the VR Headset Project documentation. This page provides links to all available documentation.</p>
    
    <div class="doc-section">
        <h2>API Documentation</h2>
        <p>Detailed documentation of the code API for developers.</p>
        <a href="./api/index.html">View API Documentation</a>
    </div>
    
    <div class="doc-section">
        <h2>Developer Guides</h2>
        <p>Comprehensive guides for developers working on the VR headset project.</p>
        <a href="./guides/index.html">View Developer Guides</a>
    </div>
    
    <div class="doc-section">
        <h2>User Documentation</h2>
        <p>Documentation for end users of the VR headset.</p>
        <a href="./user/index.html">View User Documentation</a>
    </div>
    
    <p>Generated on: $(date)</p>
</body>
</html>
EOF
    
    log "Main documentation index created"
}

# Package documentation
package_documentation() {
    log "Packaging documentation"
    
    cd "${BUILD_DIR}"
    
    # Create archive
    local archive_name="vr_headset_docs_${TIMESTAMP}.zip"
    zip -r "${archive_name}" "docs"
    
    log "Documentation packaged: ${archive_name}"
}

# Main function
main() {
    log "Starting main documentation generation process"
    
    # Create assets directory
    mkdir -p "${DOCS_DIR}/assets"
    
    # Create docs header for rustdoc
    cat > "${DOCS_DIR}/assets/docs-header.html" << EOF
<style>
    .sidebar {
        background-color: #f5f5f5;
    }
    .sidebar h2 {
        color: #2c3e50;
    }
    .content {
        max-width: 960px;
    }
    .docblock h1, .docblock h2, .docblock h3, .docblock h4, .docblock h5 {
        color: #2c3e50;
    }
    .docblock code {
        background-color: #f6f8fa;
    }
    .docblock pre {
        background-color: #f6f8fa;
    }
</style>
EOF
    
    # Generate API documentation
    generate_api_docs
    
    # Generate developer guides
    generate_developer_guides
    
    # Generate user documentation
    generate_user_docs
    
    # Create main index
    create_main_index
    
    # Package documentation
    package_documentation
    
    log "Documentation generation completed successfully"
}

# Execute main function
main
