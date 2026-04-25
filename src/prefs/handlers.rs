use super::channel::Channel;
use super::db::{Defaults, Preferences};
use actix_web::web;
use actix_web::{HttpResponse, Responder};
use serde::Deserialize;
#[derive(Clone)]
pub struct AppState {
    pub defaults: Defaults,
    pub preferences: Preferences,
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
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn get_default(
    state: web::Data<AppState>,
    query: web::Query<DefaultGetQuery>,
) -> impl Responder {
    match state.defaults.get(&query.subject).await {
        Ok(Some(channel)) => HttpResponse::Ok().json(channel),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
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
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn get_preference(
    state: web::Data<AppState>,
    query: web::Query<PreferenceGetQuery>,
) -> impl Responder {
    match state.preferences.get(&query.user, &query.subject).await {
        Ok(Some(channel)) => HttpResponse::Ok().json(channel),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
