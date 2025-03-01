## Task Description

Create a system for tracking events and notifying users using Redis.
The system should receive events via HTTP API, store them in Redis and send notifications to users subscribed to these events.
Use the `axum` framework to create the API and the asynchronous capabilities of Redis to store and retrieve data.

## Technical requirements

- Use the Axum framework to create an HTTP API
- Use Redis to store event and subscription data
- Implement mechanisms for user subscription to events
- Send notifications to users when an event occurs that they are subscribed to
- Provide error handling and logging

## Project structure

#### Main components:

- Event manager (manages receiving and storing events)
- Subscription manager (manages user subscriptions to events)
- Notification module (sends notifications to users)
- Redis to store event and subscription data

## API structure:

- `POST /events`: receiving new events
- `POST /subscribe`: user subscription to an event
- `GET /events/{user_id}`: getting events that the user is subscribed to

