use super::channel::Channel;
use super::db::{Defaults, Preferences};
use actix_web::web;
use actix_web::{HttpResponse, Responder};
use serde::Deserialize;
use tracing::error; // Added for logging
use std::sync::Arc;


use sqlx::{Pool,Sqlite};
#[derive(Clone)]
pub struct AppState {
    pub defaults: Arc<Defaults>,
    pub preferences: Preferences,
}

impl AppState {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        let defaults = Arc::new(Defaults::new(pool.clone()));
        let preferences = Preferences::new(pool, defaults.clone());
        Self {
            defaults,
            preferences,
        }
    }
}
// ---------------------------------------------------------------------------
// Defaults
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct DefaultSetRequest {
    pub subject: String,
    pub channel: Channel,
}

#[derive(Deserialize)]
pub struct DefaultGetQuery {
    pub subject: String,
}

pub async fn set_default(
    state: web::Data<AppState>,
    body: web::Json<DefaultSetRequest>,
) -> impl Responder {
    match state.defaults.set(&body.subject, body.channel).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            error!(error = %e, subject = %body.subject, "Failed to set default channel");
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}

pub async fn get_default(
    state: web::Data<AppState>,
    query: web::Query<DefaultGetQuery>,
) -> impl Responder {
    match state.defaults.get(&query.subject).await {
        Ok(Some(channel)) => HttpResponse::Ok().json(channel),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            error!(error = %e, subject = %query.subject, "Failed to get default channel");
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}

// ---------------------------------------------------------------------------
// Preferences
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct PreferenceSetRequest {
    pub user: String,
    pub subject: String,
    pub channel: Channel,
}

#[derive(Deserialize)]
pub struct PreferenceGetQuery {
    pub user: String,
    pub subject: String,
}

pub async fn set_preference(
    state: web::Data<AppState>,
    body: web::Json<PreferenceSetRequest>,
) -> impl Responder {
    match state
        .preferences
        .set(&body.user, &body.subject, body.channel)
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            error!(
                error = %e,
                user = %body.user,
                subject = %body.subject,
                "Failed to set user preference"
            );
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}

pub async fn get_preference(
    state: web::Data<AppState>,
    query: web::Query<PreferenceGetQuery>,
) -> impl Responder {
    match state.preferences.get(&query.user, &query.subject).await {
        Ok(Some(channel)) => HttpResponse::Ok().json(channel),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            error!(
                error = %e,
                user = %query.user,
                subject = %query.subject,
                "Failed to get user preference"
            );
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}
