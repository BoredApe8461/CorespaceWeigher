#!/bin/bash

FILE_TO_WATCH="logs/tracker-logs-$1.out"

WS_CONNECTION_ERROR="Failed to read message: Networking or low-level protocol error: WebSocket connection error: connection closed"

# Continuously watch the file
tail -f "$FILE_TO_WATCH" | while read LINE
do
    echo "$LINE" | grep "$WS_CONNECTION_ERROR" > /dev/null
    if [ $? = 0 ]
    then
        ./scripts/init.sh
    fi
done
