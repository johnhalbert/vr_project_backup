#!/bin/bash
# Orange Pi CM5 VR OS Enhancements Installation Script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print banner
echo -e "${BLUE}=======================================================${NC}"
echo -e "${BLUE}  Orange Pi CM5 VR OS Enhancements Installation Script  ${NC}"
echo -e "${BLUE}=======================================================${NC}"

# Check if running as root
if [ "$(id -u)" -ne 0 ]; then
    echo -e "${RED}Error: This script must be run as root${NC}"
    exit 1
fi

# Check if running on Orange Pi CM5
if ! grep -q "Orange Pi CM5" /proc/device-tree/model 2>/dev/null; then
    echo -e "${YELLOW}Warning: This script is designed for Orange Pi CM5${NC}"
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Create output directory
mkdir -p ~/orangepi_os_output
mkdir -p ~/orangepi_os_logs

# Run scripts
echo -e "${GREEN}Running system service optimization...${NC}"
./system_service_optimization.sh

echo -e "${GREEN}Running file system optimization...${NC}"
./filesystem_optimization_16gb.sh

echo -e "${GREEN}Running CPU scheduling improvements...${NC}"
./cpu_scheduling_improvements.sh

echo -e "${GREEN}Running memory management optimizations...${NC}"
./memory_management_optimizations_16gb.sh

echo -e "${GREEN}Running device tree modifications...${NC}"
./device_tree_modifications.sh

echo -e "${GREEN}Installation completed successfully.${NC}"
echo -e "${GREEN}Please reboot to apply all changes.${NC}"
