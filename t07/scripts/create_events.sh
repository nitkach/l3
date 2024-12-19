#!/bin/bash

URL="http://127.0.0.1:3000/events"

curl -X POST "$URL" -H "Content-Type: application/json" -d '{"event_type": "type1", "data": "EVENT1 MESSAGE1"}'
curl -X POST "$URL" -H "Content-Type: application/json" -d '{"event_type": "type1", "data": "EVENT1 MESSAGE2"}'
curl -X POST "$URL" -H "Content-Type: application/json" -d '{"event_type": "type1", "data": "EVENT1 MESSAGE3"}'
curl -X POST "$URL" -H "Content-Type: application/json" -d '{"event_type": "type2", "data": "EVENT2 MESSAGE1"}'
curl -X POST "$URL" -H "Content-Type: application/json" -d '{"event_type": "type3", "data": "EVENT3 MESSAGE1"}'
