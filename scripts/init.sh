#!/bin/bash

TRACKER_LOGS_0="logs/tracker-logs-0.out"
TRACKER_LOGS_1="logs/tracker-logs-1.out"

WATCHDOG_LOGS_0="logs/watchdog-logs-0.out"
WATCHDOG_LOGS_1="logs/watchdog-logs-1.out"

TRACKER="./target/release/tracker"
WATCHDOG="scripts/watchdog.sh"

reset_env() {
    PIDS=$(pgrep -f "$TRACKER|$WATCHDOG")

    if [ -z "$PIDS" ]; then
        echo "Process not found."
    else
        # Kill each process
        for PID in $PIDS; do
            kill -9 $PID
        done
    fi
}

reset_env

# start the tracker again
nohup sh -c 'RUST_LOG=INFO ./target/release/tracker --rpc-index 0' > $TRACKER_LOGS_0 2>&1 &
nohup sh -c 'RUST_LOG=INFO ./target/release/tracker --rpc-index 1' > $TRACKER_LOGS_1 2>&1 &
# start the watchdog
nohup sh -c 'scripts/watchdog.sh "$1"' _ 0 > $WATCHDOG_LOGS_0 2>&1 &
nohup sh -c 'scripts/watchdog.sh "$1"' _ 1 > $WATCHDOG_LOGS_1 2>&1 &
