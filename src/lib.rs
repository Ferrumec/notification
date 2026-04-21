use actix_web::web::ServiceConfig;
use async_trait::async_trait;
use e2schema::domain::notification::{NotificationChannel, NotificationSent};
use e2schema::events::envelop::Event;
use event_stream::{EventStream, Handler};
use ferrumec::EmailPayload;
use ferrumec::{Emailer, crypto::Validate};
use push::Config;
use serde_json::from_str;
use std::sync::Arc;

pub struct Module {
    emailer: Arc<dyn Emailer>,
    push: Arc<Config>,
}

struct OnNotification {
    es: Arc<dyn EventStream>,
    emailer: Arc<dyn Emailer>,
    push: Arc<Config>,
}

#[async_trait]
impl Handler for OnNotification {
    async fn handle(&self, message: Vec<u8>) {
        let payload: Event<NotificationSent> =
            from_str(&String::from_utf8(message).unwrap()).unwrap();
        match payload.data.channel {
            NotificationChannel::Email => {
                self.emailer
                    .send(&from_str::<EmailPayload>(&payload.data.message).unwrap());
            }
            NotificationChannel::Push => {
                self.push.push(payload.producer, payload.data.message);
            }
            NotificationChannel::Sms => (),
        }
    }
}

pub fn subscribe(es: Arc<dyn EventStream>, emailer: Arc<dyn Emailer>, push: Arc<Config>) {
    es.clone().subscribe(
        "notification".to_string(),
        Arc::new(OnNotification { es, push, emailer }),
    );
}

impl Module {
    pub fn new(emailer: Arc<dyn Emailer>, validator: Arc<dyn Validate>) -> Self {
        let push = Arc::new(Config::new(validator));
        Self { emailer, push }
    }

    pub fn configure(&self, cfg: &mut ServiceConfig, namespace: &str) {
        self.push.config(cfg, namespace);
    }
}
