#!/bin/bash

# --- Styling Variables ---
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BOLD='\033[1m'
DIM='\033[2m'
BRIGHT='\033[1;37m'
NC='\033[0m'

# --- UI Functions ---
function show_header() {
    clear
    echo -e "${BOLD}${GREEN}⚡ CFS${NC} - ${BOLD}Crypt File Share Protocol${NC}"
    echo -e "${BOLD}Version:${NC} 1.0.0   ${CYAN}"
    echo -e "${CYAN}──────────────────────────────────────────${NC}"
    echo ""
}

show_header

# --- Mode Selection ---
echo -e "${BRIGHT}Select Operation Mode:${NC}"
echo -e " ${CYAN}󰙨${NC} ${DIM}[${NC}${BRIGHT}0${NC}${DIM}]${NC} Diagnostic Tests"
echo -e " ${CYAN}󰒍${NC} ${DIM}[${NC}${BRIGHT}1${NC}${DIM}]${NC} Server Side"
echo -e " ${CYAN}󰇚${NC} ${DIM}[${NC}${BRIGHT}2${NC}${DIM}]${NC} Client Side"
echo -e " ${CYAN}@${NC} ${DIM}[${NC}${BRIGHT}3${NC}${DIM}]${NC} Performance Benchmark"
echo ""
printf "${BOLD}${CYAN}Selection » ${NC}"
read MODE_CHOICE

# --- Port Setup ---
printf "${BOLD}Port ${DIM}[default 45928] » ${NC}"
read USER_PORT
CFS_PORT=${USER_PORT:-45928}
export CFS_PORT

# --- Logic ---
case $MODE_CHOICE in
0)
    echo -e "\n${YELLOW}󰙨 Running Test Suite...${NC}"
    RUST_LOG=info cargo test -- --nocapture
    read -n 1 -s -r -p "Press any key to return to menu..."
    exec "$0" # Re-runs the script
    ;;

1)
    echo -e "\n${GREEN}🛰️ Starting Server on port ${BOLD}$CFS_PORT${NC}..."
    RUST_LOG=info cargo run --bin server
    ;;

2)
    LAST_IP_FILE=".last_ip"
    DEFAULT_IP=$([ -f "$LAST_IP_FILE" ] && cat "$LAST_IP_FILE" || echo "")

    if [ -n "$DEFAULT_IP" ]; then
        printf "${BOLD}Server IP ${DIM}[default $DEFAULT_IP] » ${NC}"
    else
        printf "${BOLD}Server IP » ${NC}"
    fi

    read USER_IP
    CFS_IP=${USER_IP:-$DEFAULT_IP}

    if [ -z "$CFS_IP" ]; then
        echo -e "${BOLD}${YELLOW}Error: IP required.${NC}"
        exit 1
    fi

    echo "$CFS_IP" >"$LAST_IP_FILE"
    export CFS_IP
    echo -e "\n${GREEN}📥 Connecting to ${BOLD}$CFS_IP:$CFS_PORT${NC}..."
    RUST_LOG=info cargo run --bin client
    ;;
3)
    echo -e "\n${YELLOW}Running Performance Benchmarks...${NC}"

    # Auto-install Valgrind if missing
    if ! command -v valgrind &>/dev/null; then
        echo -e "${YELLOW}⚙️ 'valgrind' is missing. Auto-installing...${NC}"
        # Checks if you are on Fedora (dnf) or Ubuntu/Debian (apt)
        if command -v dnf &>/dev/null; then
            sudo dnf install -y valgrind
        elif command -v apt &>/dev/null; then
            sudo apt update && sudo apt install -y valgrind
        else
            echo -e "${BOLD}❌ Could not detect package manager. Please install valgrind manually.${NC}"
            exit 1
        fi
    fi

    # Auto-install the iai-callgrind-runner if missing
    if ! command -v iai-callgrind-runner &>/dev/null && [ ! -f "$HOME/.cargo/bin/iai-callgrind-runner" ]; then
        echo -e "${YELLOW}⚙️ Cargo runner is missing. Auto-installing...${NC}"
        cargo install --version 0.11.1 iai-callgrind-runner
    fi

    # Run the benchmark (with the explicit path just in case)
    echo -e "${GREEN} Starting Benchmark...${NC}"
    cargo bench -p utils

    # If it STILL fails (e.g., version mismatch), try forcing the install
    if [ $? -ne 0 ]; then
        echo -e "\n${YELLOW}⚙️ Benchmark failed (likely version mismatch). Forcing runner update...${NC}"
        cargo install --version 0.11.1 iai-callgrind-runner
        echo -e "${GREEN} Retrying Benchmark...${NC}"
        IAI_CALLGRIND_RUNNER="$HOME/.cargo/bin/iai-callgrind-runner" cargo bench -p utils
    fi

    echo ""
    read -n 1 -s -r -p "Press any key to return to menu..."
    exec "$0"
    ;;
*)
    echo -e "${BOLD}${YELLOW}Invalid selection.${NC}"
    sleep 1
    exec "$0"
    ;;
esac
