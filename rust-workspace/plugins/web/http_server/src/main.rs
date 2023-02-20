use crate::use_case_app_container::UseCaseAppContainer;
use actix_web::{web, App, HttpServer};
use mock_store::store::Repository;
use use_cases::AppImpl;

mod errors;
mod routes;
mod use_case_app_container;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let repository = Repository;

    HttpServer::new(move || {
        let app = AppImpl::new(repository.clone());
        let app_container = UseCaseAppContainer::new(app);
        App::new()
            .configure(routes::config)
            .app_data(web::Data::new(app_container))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
