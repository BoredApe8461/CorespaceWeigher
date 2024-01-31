#!/bin/bash

TRACKER_LOGS="logs/tracker-logs.out"
WATCHDOG_LOGS="logs/watchdog-logs.out"
TRACKER="./target/release/tracker"

reset_env() {
    PIDS=$(pgrep -f "$TRACKER")

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
nohup sh -c 'RUST_LOG=INFO ./target/release/tracker' > $TRACKER_LOGS 2>&1 &
# start the watchdog
nohup sh -c 'scripts/watchdog.sh' > $WATCHDOG_LOGS 2>&1 &
