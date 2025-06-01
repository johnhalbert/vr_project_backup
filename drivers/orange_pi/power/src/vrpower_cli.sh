#!/bin/bash
# CLI tool for Orange Pi CM5 VR Headset Power Management

# Constants
PROGRAM_NAME="vrpower"
VERSION="1.0.0"
DEVICE_PATH="/dev/orangepi-vr-power"

# Profile names
PROFILE_NAMES=("high-performance" "balanced" "power-save" "custom")

# Function to print usage
print_usage() {
    echo "Usage: $PROGRAM_NAME [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  profile [get|set] [PROFILE]  Get or set power profile"
    echo "  battery                      Show battery status"
    echo "  thermal                      Show thermal status"
    echo "  monitor                      Monitor power status in real-time"
    echo "  help                         Show this help message"
    echo "  version                      Show version information"
    echo ""
    echo "Profiles:"
    echo "  high-performance             High performance profile for demanding VR applications"
    echo "  balanced                     Balanced profile for general VR use"
    echo "  power-save                   Power saving profile for extended battery life"
    echo ""
    echo "Examples:"
    echo "  $PROGRAM_NAME profile get"
    echo "  $PROGRAM_NAME profile set balanced"
    echo "  $PROGRAM_NAME battery"
    echo "  $PROGRAM_NAME monitor"
}

# Function to print version
print_version() {
    echo "$PROGRAM_NAME version $VERSION"
}

# Function to check if the device exists
check_device() {
    if [ ! -e "$DEVICE_PATH" ]; then
        echo "Error: Device $DEVICE_PATH not found"
        echo "Make sure the VR power management driver is loaded"
        exit 1
    fi
}

# Function to get profile
get_profile() {
    check_device
    
    # Read from sysfs
    if [ -e "/sys/class/orangepi-vr-power/power_profile" ]; then
        PROFILE_NUM=$(cat /sys/class/orangepi-vr-power/power_profile)
        echo "Current power profile: ${PROFILE_NAMES[$PROFILE_NUM]}"
    else
        # Fallback to using the service
        if command -v vr-power-mgr &> /dev/null; then
            vr-power-mgr -s | grep "Power profile" | sed 's/Power profile: //'
        else
            echo "Error: Cannot get power profile"
            exit 1
        fi
    fi
}

# Function to set profile
set_profile() {
    check_device
    
    # Convert profile name to number
    PROFILE_NUM=-1
    case "$1" in
        "high-performance")
            PROFILE_NUM=0
            ;;
        "balanced")
            PROFILE_NUM=1
            ;;
        "power-save")
            PROFILE_NUM=2
            ;;
        *)
            echo "Error: Invalid profile name: $1"
            echo "Valid profiles: high-performance, balanced, power-save"
            exit 1
            ;;
    esac
    
    # Write to sysfs
    if [ -e "/sys/class/orangepi-vr-power/power_profile" ]; then
        echo $PROFILE_NUM > /sys/class/orangepi-vr-power/power_profile
        echo "Power profile set to: $1"
    else
        # Fallback to using the service
        if command -v vr-power-mgr &> /dev/null; then
            vr-power-mgr -p $PROFILE_NUM
            echo "Power profile set to: $1"
        else
            echo "Error: Cannot set power profile"
            exit 1
        fi
    fi
}

# Function to show battery status
show_battery() {
    check_device
    
    # Read from sysfs
    if [ -e "/sys/class/orangepi-vr-power/battery_status" ]; then
        BATTERY_INFO=$(cat /sys/class/orangepi-vr-power/battery_status)
        
        # Parse and format the output
        STATUS=$(echo "$BATTERY_INFO" | grep -o "status=[0-9]*" | cut -d= -f2)
        CAPACITY=$(echo "$BATTERY_INFO" | grep -o "capacity=[0-9]*" | cut -d= -f2)
        VOLTAGE=$(echo "$BATTERY_INFO" | grep -o "voltage=[0-9]*" | cut -d= -f2)
        CURRENT=$(echo "$BATTERY_INFO" | grep -o "current=[-0-9]*" | cut -d= -f2)
        TEMP=$(echo "$BATTERY_INFO" | grep -o "temp=[0-9]*" | cut -d= -f2)
        
        # Convert status number to string
        case "$STATUS" in
            0) STATUS_STR="Charging" ;;
            1) STATUS_STR="Discharging" ;;
            2) STATUS_STR="Full" ;;
            *) STATUS_STR="Unknown" ;;
        esac
        
        # Format output
        echo "Battery Status:"
        echo "  Status: $STATUS_STR"
        echo "  Capacity: $CAPACITY%"
        echo "  Voltage: $VOLTAGE mV"
        echo "  Current: $CURRENT mA"
        echo "  Temperature: $(echo "scale=1; $TEMP/10" | bc)°C"
    else
        # Fallback to using the service
        if command -v vr-power-mgr &> /dev/null; then
            vr-power-mgr -s | grep -A 10 "Battery status"
        else
            echo "Error: Cannot get battery status"
            exit 1
        fi
    fi
}

# Function to show thermal status
show_thermal() {
    check_device
    
    # Read from sysfs
    if [ -e "/sys/class/orangepi-vr-power/thermal_status" ]; then
        THERMAL_INFO=$(cat /sys/class/orangepi-vr-power/thermal_status)
        
        # Parse and format the output
        echo "Thermal Status:"
        
        # Process each line
        echo "$THERMAL_INFO" | while read -r line; do
            ZONE=$(echo "$line" | grep -o "zone=[0-9]*" | cut -d= -f2)
            STATUS=$(echo "$line" | grep -o "status=[0-9]*" | cut -d= -f2)
            TEMP=$(echo "$line" | grep -o "temp=[0-9]*" | cut -d= -f2)
            
            # Convert zone number to string
            case "$ZONE" in
                0) ZONE_STR="CPU" ;;
                1) ZONE_STR="GPU" ;;
                2) ZONE_STR="NPU" ;;
                3) ZONE_STR="Battery" ;;
                4) ZONE_STR="Ambient" ;;
                *) ZONE_STR="Unknown" ;;
            esac
            
            # Convert status number to string
            case "$STATUS" in
                0) STATUS_STR="Normal" ;;
                1) STATUS_STR="Warning" ;;
                2) STATUS_STR="Critical" ;;
                3) STATUS_STR="Emergency" ;;
                *) STATUS_STR="Unknown" ;;
            esac
            
            # Format output
            echo "  $ZONE_STR: $STATUS_STR, $(echo "scale=1; $TEMP/10" | bc)°C"
        done
    else
        echo "Error: Cannot get thermal status"
        exit 1
    fi
}

# Function to monitor power status in real-time
monitor_power() {
    check_device
    
    echo "Monitoring power status (press Ctrl+C to exit)..."
    echo ""
    
    while true; do
        clear
        echo "=== VR Power Management Status ==="
        echo ""
        
        # Get profile
        echo "Power Profile:"
        get_profile
        echo ""
        
        # Get battery status
        show_battery
        echo ""
        
        # Get thermal status
        show_thermal
        echo ""
        
        # Update every 2 seconds
        sleep 2
    done
}

# Main command processing
case "$1" in
    "profile")
        case "$2" in
            "get")
                get_profile
                ;;
            "set")
                if [ -z "$3" ]; then
                    echo "Error: No profile specified"
                    echo "Usage: $PROGRAM_NAME profile set [PROFILE]"
                    exit 1
                fi
                set_profile "$3"
                ;;
            *)
                echo "Error: Invalid profile command: $2"
                echo "Usage: $PROGRAM_NAME profile [get|set] [PROFILE]"
                exit 1
                ;;
        esac
        ;;
    "battery")
        show_battery
        ;;
    "thermal")
        show_thermal
        ;;
    "monitor")
        monitor_power
        ;;
    "help")
        print_usage
        ;;
    "version")
        print_version
        ;;
    *)
        print_usage
        exit 1
        ;;
esac

exit 0
