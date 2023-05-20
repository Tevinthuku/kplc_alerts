use crate::authentication::AuthenticatedUserInfo;
use crate::errors::ApiError;
use crate::use_case_app_container::UseCaseAppContainer;
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use use_cases::authentication::SubscriberDetailsInput;

#[derive(Serialize, Deserialize, Debug)]
struct UserRequest {
    name: String,
    email: String,
}

#[tracing::instrument(err, skip(app), level = "info")]
async fn authentication(
    user_details: web::Json<UserRequest>,
    app: web::Data<UseCaseAppContainer>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let user: AuthenticatedUserInfo = (&req).try_into()?;

    let UserRequest { name, email } = user_details.into_inner();
    let auth_interactor = app.get_client().authentication();
    auth_interactor
        .authenticate(&user, SubscriberDetailsInput { name, email })
        .await
        .map_err(ApiError::InternalServerError)?;

    Ok(HttpResponse::Ok().finish())
}
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/authenticate")
            .service(web::resource("").route(web::post().to(authentication))),
    );
}
