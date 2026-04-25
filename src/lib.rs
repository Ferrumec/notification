use actix_web::web::ServiceConfig;
use async_trait::async_trait;
use event_stream::{EventStream, Handler};
use ferrumec::deps::email::EmailingContext;
use ferrumec::deps::signers::Validate;
use push::Config;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
mod prefs;

#[derive(Clone)]
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
    resolver: Preferences,
}
use crate::prefs::Channel;
use crate::prefs::db::{Defaults, Preferences};

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
        let channel = match self
            .resolver
            .get(&event.user_id.unwrap().to_string(), &subject)
            .await
        {
            Ok(r) => r.unwrap(),
            Err(e) => {
                eprintln!("Error in reading preferences");
                return ();
            }
        };
        match channel {
            Channel::Email => {
                self.emailer.send(subject, message);
            }
            Channel::Push => {
                self.push.push(event.producer, message);
            }
            Channel::Sms => (),
            Channel::InApp => (),
        }
    }
}

impl Module {
    pub async fn new(
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
        module.subscribe(es).await;
        module
    }

    pub fn config(&self, cfg: &mut ServiceConfig, namespace: &str) {
        self.push.config(cfg, namespace);
    }
    pub async fn subscribe(&self, es: Arc<dyn EventStream>) {
        let defaults = Defaults::new(self.pool.clone());
        es.clone()
            .subscribe(
                "notification".to_string(),
                Arc::new(OnNotification {
                    es,
                    push: self.push.clone(),
                    emailer: self.emailer.clone(),
                    resolver: Preferences::new(self.pool.clone(), defaults),
                }),
            )
            .await;
    }
}
