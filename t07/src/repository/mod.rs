use crate::error::AppError;
use crate::model::{Event, Subscription};
use anyhow::Result;
use log::{debug, info};
use redis::{aio::MultiplexedConnection, AsyncCommands};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use ulid::{Generator, Ulid};

#[derive(Clone)]
pub(crate) struct Repository {
    redis_conn: MultiplexedConnection,
    ulid: Arc<Mutex<Generator>>,
}

impl Repository {
    pub(crate) async fn init(redis_hostname: &str) -> Result<Self> {
        let client = redis::Client::open(redis_hostname)?;
        let redis_conn = client.get_multiplexed_tokio_connection().await?;

        info!("Connected to redis database via: {redis_hostname}");

        let ulid = ulid::Generator::new();

        Ok(Self {
            redis_conn,
            ulid: Arc::new(Mutex::new(ulid)),
        })
    }

    pub(crate) async fn add_event(&mut self, event: &Event) -> Result<Ulid, AppError> {
        let ulid = {
            let mut generator = self.ulid.lock().unwrap();
            loop {
                if let Ok(ulid) = generator.generate() {
                    break ulid;
                }
                std::thread::yield_now();
            }
        };

        let events_added: u8 = self
            .redis_conn
            .sadd("events", ulid.to_string())
            .await
            .map_err(AppError::Redis)?;
        if events_added != 1 {
            unreachable!("ulid must be unique");
        }
        debug!("successfully added '{ulid}' to events");

        let key = get_key_event_ulid(&ulid.to_string());
        let _: String = self
            .redis_conn
            .hset_multiple(
                key,
                &[
                    ("event_type", event.get_type()),
                    ("event_data", event.get_data()),
                ],
            )
            .await
            .map_err(AppError::Redis)?;
        debug!(
            "successfully added '{}'",
            get_key_event_ulid(&ulid.to_string())
        );

        let key = get_key_events_type_event_type(event.get_type());
        let _: u8 = self
            .redis_conn
            .lpush(key, ulid.to_string())
            .await
            .map_err(AppError::Redis)?;
        debug!(
            "successfully added to '{}'",
            get_key_events_type_event_type(event.get_type())
        );

        Ok(ulid)
    }

    pub(crate) async fn add_subscription(
        &mut self,
        subscription: &Subscription,
    ) -> Result<(), AppError> {
        let _user_added: u8 = self
            .redis_conn
            .sadd("users", subscription.get_user_id())
            .await
            .map_err(AppError::Redis)?;
        debug!(
            "successfully added user with id = {} to 'users'",
            subscription.get_user_id()
        );

        let key = get_key_user_user_id_subscriptions(subscription.get_user_id());
        let _subscription_added: u8 = self
            .redis_conn
            .sadd(key, subscription.get_event_type())
            .await
            .map_err(AppError::Redis)?;
        debug!(
            "successfully added '{}' to user's subscriptions '{}'",
            subscription.get_event_type(),
            get_key_user_user_id_subscriptions(subscription.get_user_id())
        );

        let key = get_key_subscriptions_event_type(subscription.get_event_type());
        let _webhook_added: u8 = self
            .redis_conn
            .hset(
                key,
                subscription.get_user_id(),
                subscription.get_webhook_url(),
            )
            .await
            .map_err(AppError::Redis)?;
        debug!(
            "successfully added user id and webhook to '{}'",
            get_key_subscriptions_event_type(subscription.get_event_type())
        );

        Ok(())
    }

    pub(crate) async fn get_user_events(&mut self, user_id: u64) -> Result<Vec<Event>, AppError> {
        let user_matches: u8 = self
            .redis_conn
            .sismember("users", user_id)
            .await
            .map_err(AppError::Redis)?;

        if user_matches == 0 {
            return Err(AppError::NotFound(format!("user {user_id}")));
        }

        let key = get_key_user_user_id_subscriptions(user_id);
        let user_subscriptions: std::collections::HashSet<String> = self
            .redis_conn
            .smembers(key)
            .await
            .map_err(AppError::Redis)?;

        let mut events = Vec::new();

        for event_type in user_subscriptions {
            let key = get_key_events_type_event_type(&event_type);
            let ulids: Vec<String> = self
                .redis_conn
                .lrange(key, 0, -1)
                .await
                .map_err(AppError::Redis)?;

            for ulid in ulids {
                let key = get_key_event_ulid(&ulid);
                let mut event: HashMap<String, String> = self
                    .redis_conn
                    .hgetall(key)
                    .await
                    .map_err(AppError::Redis)?;
                let event = {
                    let event_type = event.remove("event_type").expect("always exist");
                    let data = event.remove("event_data").expect("always exist");
                    Event { event_type, data }
                };

                events.push(event);
            }
        }

        Ok(events)
    }

    pub(crate) async fn notify_subscribers(&mut self, event: &Event) -> Result<(), AppError> {
        let key = get_key_subscriptions_event_type(event.get_type());
        let users_webhooks: HashMap<u64, String> = self
            .redis_conn
            .hgetall(key)
            .await
            .map_err(AppError::Redis)?;

        for (user_id, webhook) in users_webhooks {
            send_notification(user_id, &webhook);
        }

        Ok(())
    }
}

fn send_notification(user_id: u64, webhook: &str) {
    info!("Send notification to user with id = {user_id} by webhook: {webhook}");
}

/// `subscriptions:{ event_type }`
fn get_key_subscriptions_event_type(event_type: &str) -> String {
    format!("subscriptions:{event_type}")
}

/// `user:{ user_id }:subscriptions`
fn get_key_user_user_id_subscriptions(user_id: u64) -> String {
    format!("user:{user_id}:subscriptions")
}

/// `events:type:{ event_type }`
fn get_key_events_type_event_type(event_type: &str) -> String {
    format!("events:type:{event_type}")
}

/// `event:{ ulid }`
fn get_key_event_ulid(ulid: &str) -> String {
    format!("event:{ulid}")
}
