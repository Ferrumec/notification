use actix_web::web::ServiceConfig;
use async_trait::async_trait;
use emailgrid::EmailingContext;
use event_stream::{EventStream, Handler};
use ferrumec::crypto::Validate;
use push::Config;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
mod preferences;
pub struct Module {
    emailer: EmailingContext,
    push: Arc<Config>,
    pool: Pool<Sqlite>,
}
use chrono::{DateTime, Utc};
use uuid::Uuid;
struct OnNotification {
    es: Arc<dyn EventStream>,
    emailer: EmailingContext,
    push: Arc<Config>,
    pool: Pool<Sqlite>,
}
use crate::preferences::db::resolve_channels;
use crate::preferences::models::Channel;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_id: Uuid,
    pub event_version: String,
    pub occurred_at: DateTime<Utc>,
    pub producer: String,
    pub correlation_id: Option<Uuid>,
    pub trace_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
}

#[async_trait]
impl Handler for OnNotification {
    async fn handle(&self, subject: String, message: Vec<u8>) {
        let message = String::from_utf8(message).unwrap();
        let event: Event = from_str(&message).unwrap();
        let channel =
            &mut resolve_channels(&self.pool, &event.user_id.unwrap().to_string(), &subject)
                .await
                .unwrap()[0];
        match channel {
            Channel::Email => {
                self.emailer.send(subject, message);
            }
            Channel::Push => {
                self.push.push(event.producer, message);
            }
            Channel::Sms => (),
        }
    }
}

impl Module {
    pub fn new(
        pool: Pool<Sqlite>,
        emailer: EmailingContext,
        validator: Arc<dyn Validate>,
        es: Arc<dyn EventStream>,
    ) -> Self {
        let push = Arc::new(Config::new(validator));
        let module = Self {
            emailer,
            push,
            pool,
        };
        module.subscribe(es);
        module
    }

    pub fn configure(&self, cfg: &mut ServiceConfig, namespace: &str) {
        self.push.config(cfg, namespace);
    }
    pub fn subscribe(&self, es: Arc<dyn EventStream>) {
        es.clone().subscribe(
            "notification".to_string(),
            Arc::new(OnNotification {
                es,
                push: self.push.clone(),
                emailer: self.emailer.clone(),
                pool: self.pool.clone(),
            }),
        );
    }
}
