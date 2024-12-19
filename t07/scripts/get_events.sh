#!/bin/bash

BASE_URL="http://127.0.0.1:3000/events"

USER_IDS=(1 2 3 4)

for USER_ID in "${USER_IDS[@]}"; do
    echo "Fetching events for user ID: $USER_ID"
    curl -X GET "$BASE_URL/$USER_ID"
    echo -e "\n"
done
