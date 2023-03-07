use actix_web::web;

pub mod search_locations;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/locations").configure(search_locations::init_routes));
}
