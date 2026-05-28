use actix_web::web;
use actix_web::web::ServiceConfig;
use actixutils::{Identity, OrphanWrapper, Validate};
use emailgrid::EmailingContext;
use event_stream::{EventStream, Handler};
use push::Config;
use serde::{Deserialize, Serialize};

use sqlx::{Pool, Sqlite};
use std::sync::Arc;
mod prefs;
use mgk::{Module as Mgk, Sender};

struct Push(Config);

struct Email(EmailingContext);

#[async_trait::async_trait]
impl Sender for Push {
    async fn send(&self, address: String, subject: String, message: String) {
        self.0.push("push".to_string(), message);
    }
}

#[async_trait::async_trait]
impl Sender for Email {
    async fn send(&self, address: String, subject: String, message: String) {
        self.0.send(subject, message).await;
    }
}

#[derive(Clone)]
pub struct Module {
    emailer: Mgk,
    push_mgk: Mgk,
    state: Arc<AppState>,
    push_: Config,
}
use chrono::{DateTime, Utc};
use uuid::Uuid;
struct OnNotification {
    state: Arc<AppState>,
    emailer: EmailingContext,
    push: Arc<Config>,
}

use crate::prefs::{AppState, Channel};

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
        let state = Arc::new(AppState::new(pool.clone()));
        let email = console
            .clone()
            .with_sender(Arc::new(Email(emailer)) as Arc<dyn Sender>);
        let module = Self {
            emailer: email,
            state: state.clone(),
            push_mgk,
            push_,
        };
        module
    }

    pub fn config(&self, cfg: &mut ServiceConfig, namespace: &str) {
        cfg.service(
            web::scope(namespace)
                .app_data(web::Data::from(self.state.clone()))
                .configure(|cfg| self.push_.config(cfg, "/push"))
                .configure(|cfg| self.emailer.config(cfg, "/email"))
                .configure(|cfg| self.push_mgk.config(cfg, "/push")),
        );
    }
}
