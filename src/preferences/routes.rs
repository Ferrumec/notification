use actix_web::web;
use super::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users/{user_id}/notification-preferences")
            .route("/channels",                         web::get().to(handlers::get_channel_defaults))
            .route("/channels",                         web::put().to(handlers::set_channel_defaults))
            .route("/events",                           web::put().to(handlers::set_event_prefs))
            .route("/events/{event_type}",              web::get().to(handlers::get_event_prefs))
            .route("/resolve/{event_type}",             web::get().to(handlers::resolve_channels)),
    );
}
