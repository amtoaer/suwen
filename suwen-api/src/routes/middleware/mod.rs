use anyhow::Context;
use axum::{Extension, extract::Request, middleware::Next, response::IntoResponse};
use axum_extra::extract::CookieJar;
use sea_orm::ColumnTrait;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter};
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
        let (user, identity) = suwen_entity::user::Entity::find_by_id(claims.id)
            .find_also_related(suwen_entity::identity::Entity)
            .one(&conn)
            .await?
            .context("identity not found")?;
        if user.id == 1 {
            req.extensions_mut()
                .insert(Identity::Admin { me: user, identity });
        } else {
            req.extensions_mut()
                .insert(Identity::Authenticated { me: user, identity });
        }
    } else if let Some(anonymous_id) = jar.get("anonymous") {
        let uuid = Uuid::parse_str(anonymous_id.value())
            .map_err(|_| ApiError::bad_request("Invalid anonymous ID format"))?;
        let identity = suwen_entity::identity::Entity::find()
            .filter(suwen_entity::identity::Column::Uuid.eq(uuid))
            .one(&conn)
            .await?;
        req.extensions_mut()
            .insert(Identity::Anonymous { uuid, identity });
    } else {
        req.extensions_mut().insert(Identity::None);
    }
    Ok(next.run(req).await)
}
