use actix_web::{web, HttpResponse, Responder};
use sqlx::SqlitePool;

use crate::preferences::{
    db,
    models::{ResolvedChannels, SetChannelDefaultsRequest, SetEventPrefsRequest},
};

// PUT /users/{user_id}/notification-preferences/channels
pub async fn set_channel_defaults(
    pool: web::Data<SqlitePool>,
    user_id: web::Path<String>,
    body: web::Json<SetChannelDefaultsRequest>,
) -> impl Responder {
    match db::upsert_channel_defaults(&pool, &user_id, &body.defaults).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => {
            tracing::error!("set_channel_defaults error: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

// GET /users/{user_id}/notification-preferences/channels
pub async fn get_channel_defaults(
    pool: web::Data<SqlitePool>,
    user_id: web::Path<String>,
) -> impl Responder {
    match db::get_channel_defaults(&pool, &user_id).await {
        Ok(defaults) => HttpResponse::Ok().json(defaults),
        Err(e) => {
            tracing::error!("get_channel_defaults error: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

// PUT /users/{user_id}/notification-preferences/events
pub async fn set_event_prefs(
    pool: web::Data<SqlitePool>,
    user_id: web::Path<String>,
    body: web::Json<SetEventPrefsRequest>,
) -> impl Responder {
    match db::upsert_event_prefs(&pool, &user_id, &body.prefs).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => {
            tracing::error!("set_event_prefs error: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

// GET /users/{user_id}/notification-preferences/events/{event_type}
pub async fn get_event_prefs(
    pool: web::Data<SqlitePool>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (user_id, event_type) = path.into_inner();
    match db::get_event_prefs(&pool, &user_id, &event_type).await {
        Ok(prefs) => HttpResponse::Ok().json(prefs),
        Err(e) => {
            tracing::error!("get_event_prefs error: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

// GET /users/{user_id}/notification-preferences/resolve/{event_type}
// Called internally by the orchestrator before dispatching.
pub async fn resolve_channels(
    pool: web::Data<SqlitePool>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (user_id, event_type) = path.into_inner();
    match db::resolve_channels(&pool, &user_id, &event_type).await {
        Ok(channels) => HttpResponse::Ok().json(ResolvedChannels {
            user_id,
            event_type,
            channels,
        }),
        Err(e) => {
            tracing::error!("resolve_channels error: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}
