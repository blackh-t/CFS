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
    echo -e "${CYAN}┌──────────────────────────────────────────┐${NC}"
    echo -e "${CYAN}│${NC}  ${BOLD}${GREEN}⚡ CFS${NC} - ${BOLD}Crypt File Share Protocol${NC}      ${CYAN}│${NC}"
    echo -e "${CYAN}│${NC}  ${BOLD}Version:${NC} 1.0.0   ${CYAN}                       │${NC}"
    echo -e "${CYAN}└──────────────────────────────────────────┘${NC}"
    echo ""
}

show_header

# --- Mode Selection ---
echo -e "${BRIGHT}Select Operation Mode:${NC}"
echo -e " ${CYAN}󰙨${NC} ${DIM}[${NC}${BRIGHT}0${NC}${DIM}]${NC} Diagnostic Tests"
echo -e " ${CYAN}󰒍${NC} ${DIM}[${NC}${BRIGHT}1${NC}${DIM}]${NC} Server Side"
echo -e " ${CYAN}󰇚${NC} ${DIM}[${NC}${BRIGHT}2${NC}${DIM}]${NC} Client Side"
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

*)
    echo -e "${BOLD}${YELLOW}Invalid selection.${NC}"
    sleep 1
    exec "$0"
    ;;
esac
