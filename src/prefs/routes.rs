use crate::prefs::handlers::*;
use actix_web::{middleware::from_fn, web};
use actixutils::middleware::authority;
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        // Defaults
        .service(
            web::scope("/defaults")
                .wrap(from_fn(authority(1)))
                .route("/set", web::post().to(set_default))
                .route("/get", web::get().to(get_default)),
        )
        // Preferences
        .route("/preferences/set", web::post().to(set_preference))
        .route("/preferences/get", web::get().to(get_preference));
}
