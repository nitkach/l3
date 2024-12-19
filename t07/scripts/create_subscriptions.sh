#!/bin/bash

URL="http://127.0.0.1:3000/subscribe"

curl -X POST "$URL" -H "Content-Type: application/json" -d '{"event_type": "type1", "user_id": 1, "webhook_url": "USER 1 WEBHOOK"}'
curl -X POST "$URL" -H "Content-Type: application/json" -d '{"event_type": "type1", "user_id": 2, "webhook_url": "USER 2 WEBHOOK"}'
curl -X POST "$URL" -H "Content-Type: application/json" -d '{"event_type": "type1", "user_id": 3, "webhook_url": "USER 3 WEBHOOK"}'
curl -X POST "$URL" -H "Content-Type: application/json" -d '{"event_type": "type2", "user_id": 4, "webhook_url": "USER 4 WEBHOOK"}'
curl -X POST "$URL" -H "Content-Type: application/json" -d '{"event_type": "type3", "user_id": 3, "webhook_url": "USER 3 WEBHOOK"}'
