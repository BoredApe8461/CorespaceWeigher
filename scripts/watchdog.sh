#!/bin/bash

FILE_TO_WATCH="logs/tracker-logs.out"

WS_CONNECTION_ERROR="Failed to read message: Networking or low-level protocol error: WebSocket connection error: connection closed"
TRACKER="./target/release/tracker"

restart_tracker() {
    PIDS=$(pgrep -f "$TRACKER")

    if [ -z "$PIDS" ]; then
        echo "Process not found."
    else
        # Kill each process
        for PID in $PIDS; do
            kill -9 $PID
        done

        # start the tracker again
        nohup sh -c 'RUST_LOG=INFO ./target/release/tracker' > $FILE_TO_WATCH 2>&1 &
    fi
}

# Continuously watch the file
tail -f "$FILE_TO_WATCH" | while read LINE
do
    echo "$LINE" | grep "$WS_CONNECTION_ERROR" > /dev/null
    if [ $? = 0 ]
    then
       restart_tracker 
    fi
done
