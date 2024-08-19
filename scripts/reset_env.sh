#!/bin/bash

TRACKER="./target/release/tracker"
WATCHDOG="scripts/watchdog.sh"

PIDS=$(pgrep -f "$TRACKER|$WATCHDOG")

if [ -z "$PIDS" ]; then
    echo "Process not found."
else
    # Kill each process
    for PID in $PIDS; do
        kill -9 $PID
    done
fi
