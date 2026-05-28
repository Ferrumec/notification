use actix_web::web;
use actix_web::web::ServiceConfig;
use actixutils::{Identity, Validate};
use emailgrid::EmailingContext;
use event_stream::EventStream;
use push::Config;
use serde::{Deserialize, Serialize};

use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use mgk::{Module as Mgk, Sender};

struct Push(Config);

struct Email(EmailingContext);

#[async_trait::async_trait]
impl Sender for Push {
    async fn send(&self, _address: String, _subject: String, message: String) {
        self.0.push("push".to_string(), message);
    }
}

#[async_trait::async_trait]
impl Sender for Email {
    async fn send(&self, _address: String, subject: String, message: String) {
        self.0.send(subject, message).await;
    }
}

#[derive(Clone)]
pub struct Module {
    emailer: Mgk,
    push_mgk: Mgk,
    push_: Config,
}
use chrono::{DateTime, Utc};
use uuid::Uuid;
struct OnNotification {
    emailer: EmailingContext,
    push: Arc<Config>,
}


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

impl Module {
    pub async fn new(
        pool: Pool<Sqlite>,
        emailer: EmailingContext,
        validator: Arc<dyn Validate<Identity>>,
        es: Arc<dyn EventStream>,
    ) -> Self {
        let console = Mgk::new(pool.clone(), es.clone()).await;
        let push_ = Config::new(es, validator).await;
        let push_mgk = console.clone().with_sender(Arc::new(Push(push_.clone())));
        let email = console
            .clone()
            .with_sender(Arc::new(Email(emailer)) as Arc<dyn Sender>);
        
        Self {
            emailer: email,
            push_mgk,
            push_,
        }
    }

    pub fn config(&self, cfg: &mut ServiceConfig, namespace: &str) {
        cfg.service(
            web::scope(namespace)
                .configure(|cfg| self.push_.config(cfg, "/push"))
                .configure(|cfg| self.emailer.config(cfg, "/email"))
                .configure(|cfg| self.push_mgk.config(cfg, "/push")),
        );
    }
}
