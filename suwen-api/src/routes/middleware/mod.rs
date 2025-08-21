use anyhow::Context;
use axum::{Extension, extract::Request, middleware::Next, response::IntoResponse};
use axum_extra::extract::CookieJar;
use sea_orm::{DatabaseConnection, EntityTrait};
use uuid::Uuid;

use crate::{
    auth::{Claims, Identity},
    wrapper::ApiError,
};

pub(crate) async fn auth(
    jar: CookieJar,
    Extension(conn): Extension<DatabaseConnection>,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, ApiError> {
    if let Some(jwt_token) = jar.get("jwt") {
        let claims = Claims::decode(jwt_token.value())?;
        let me = suwen_entity::user::Entity::find_by_id(claims.id)
            .one(&conn)
            .await?
            .context("user not found")?;
        // TODO: 修改为对比与数据库中是否一致
        if claims.passwd_version != 1 {
            return Err(ApiError::unauthorized("Password version mismatch"));
        }
        // TODO: 修改为严谨的判断
        if me.id == 1 {
            req.extensions_mut().insert(Identity::Admin { me });
        } else {
            req.extensions_mut().insert(Identity::Authenticated { me });
        }
    }
    if let Some(anonymous_id) = jar.get("anonymous") {
        let id = Uuid::parse_str(anonymous_id.value())
            .map_err(|_| ApiError::bad_request("Invalid anonymous ID format"))?;
        req.extensions_mut().insert(Identity::Anonymous { id });
    } else {
        req.extensions_mut().insert(Identity::None);
    }
    Ok(next.run(req).await)
}
