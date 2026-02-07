#!/bin/bash

echo "----------------------------"
echo "             CFS            "
echo "  Crypt File Share Protocol "
echo "----------------------------"

echo "Choose mode:"
echo "1) Server"
echo "2) Client"
read -p "Selection [1-2]: " MODE_CHOICE

read -p "Enter Port [default 45928]: " USER_PORT
CFS_PORT=${USER_PORT:-45928}
export CFS_PORT

if [ "$MODE_CHOICE" == "1" ]; then
    echo "[*] Starting Server on port $CFS_PORT..."
    cargo run --bin server

elif [ "$MODE_CHOICE" == "2" ]; then
    # Load last used ip.
    LAST_IP_FILE=".last_ip"
    DEFAULT_IP=""

    if [ -f "$LAST_IP_FILE" ]; then
        DEFAULT_IP=$(cat "$LAST_IP_FILE")
        PROMPT="Enter Server IP Address [default $DEFAULT_IP]: "
    else
        PROMPT="Enter Server IP Address: "
    fi

    read -p "$PROMPT" USER_IP
    CFS_IP=${USER_IP:-$DEFAULT_IP}

    if [ -z "$CFS_IP" ]; then
        echo "Error: IP address is required for client mode."
        exit 1
    fi

    # Save this IP for next time
    echo "$CFS_IP" >"$LAST_IP_FILE"

    export CFS_IP
    echo "[*] Connecting to $CFS_IP:$CFS_PORT..."
    cargo run --bin client

else
    echo "Invalid selection."
    exit 1
fi
