use std::{borrow::Cow, sync::Arc};

use axum::{Json, extract::State, http::StatusCode};
use color_eyre::eyre::{Context, OptionExt, eyre};
use diesel::{ExpressionMethods, HasQuery, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use lockinspiel_backend_common::{
    auth::DatabaseConnection,
    error::{self, EyreError, WithStatusCode},
};
use lockinspiel_common_schema::sql_types;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use lockinspiel_user_schema::{
    CreateProfileRoute, DeleteAvatarRoute, GetProfileRoute, InsertableUserProfile, PutAvatarQuery,
    PutAvatarRoute, UpdateProfileRoute, UserAvatarPutUrl, UserProfile, UserProfileChangeset,
    schema::user::profiles,
};
use uuid::Uuid;

use crate::url_resolver::{UrlLocation, UrlOrigin, UrlResolver};

#[derive(HasQuery, Deserialize, Serialize)]
#[diesel(table_name = profiles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbUserProfile<'a> {
    user_id: Uuid,
    display_name: Cow<'a, str>,
    bio: Cow<'a, str>,
    avatar_location: Option<sql_types::Json<UrlLocation<'static>>>,
}

impl DbUserProfile<'static> {
    async fn into_user_profile(self, resolver: &UrlResolver) -> UserProfile<'static> {
        let mut avatar_location = None;
        if let Some(location) = self.avatar_location {
            avatar_location = Some(Cow::Owned(resolver.resolve_get_url(location.0).await));
        }
        UserProfile {
            user_id: self.user_id,
            display_name: self.display_name,
            bio: self.bio,
            avatar_location,
        }
    }
}

#[instrument(skip_all)]
async fn get_user_profile(
    db: &mut DatabaseConnection,
    user_id: Uuid,
) -> Result<DbUserProfile<'static>, EyreError> {
    let profile = DbUserProfile::query()
        .filter(profiles::user_id.eq(user_id))
        .get_result(&mut db.connection)
        .await
        .optional()
        .wrap_err("Failed to insert user profile into database")
        .with_status_code(StatusCode::UNPROCESSABLE_ENTITY)?
        .ok_or_eyre("Failed to find user profile in database")
        .with_status_code(StatusCode::NOT_FOUND)?;

    Ok(profile)
}

#[utoipa_e2e::implementor_of(CreateProfileRoute)]
#[instrument(skip(db))]
pub async fn create_profile(
    mut db: DatabaseConnection,
    Json(new_profile): Json<InsertableUserProfile<'_>>,
) -> Result<(), error::EyreError> {
    let Some(user_id) = db.user.map(|u| u.sub) else {
        return Err(eyre!("You need to be logged in to create a user profile"))
            .with_status_code(StatusCode::UNAUTHORIZED);
    };

    diesel::insert_into(profiles::table)
        .values((new_profile, profiles::user_id.eq(user_id)))
        .execute(&mut db.connection)
        .await
        .wrap_err("Failed to insert user profile into database")
        .with_status_code(StatusCode::UNPROCESSABLE_ENTITY)?;

    Ok(())
}

#[utoipa_e2e::implementor_of(GetProfileRoute)]
#[instrument(skip(db))]
pub async fn get_profile(
    State(url_resolver): State<Arc<UrlResolver>>,
    mut db: DatabaseConnection,
) -> Result<Json<UserProfile<'static>>, error::EyreError> {
    let Some(user_id) = db.user.as_ref().map(|u| u.sub) else {
        return Err(eyre!("You need to be logged in to get your user profile"))
            .with_status_code(StatusCode::UNAUTHORIZED);
    };

    Ok(Json(
        get_user_profile(&mut db, user_id)
            .await?
            .into_user_profile(&url_resolver)
            .await,
    ))
}

#[utoipa_e2e::implementor_of(PutAvatarRoute)]
#[instrument(skip(db))]
pub async fn put_avatar(
    State(url_resolver): State<Arc<UrlResolver>>,
    mut db: DatabaseConnection,
    Json(put_avatar_query): Json<PutAvatarQuery<'_>>,
) -> Result<Json<UserAvatarPutUrl<'static>>, error::EyreError> {
    let Some(user_id) = db.user.as_ref().map(|u| u.sub) else {
        return Err(eyre!(
            "You need to be logged in to updated your user profile"
        ))
        .with_status_code(StatusCode::UNAUTHORIZED);
    };

    let current_profile = get_user_profile(&mut db, user_id).await?;

    if let Some(location) = current_profile.avatar_location {
        url_resolver.delete_url(location.0).await?;
    }

    let avatar_path = format!("avatars/{}.{}", user_id, put_avatar_query.file_extension);
    // TODO: Don't depend on S3 support
    let url_location = UrlLocation {
        location: UrlOrigin::S3,
        path: avatar_path.into(),
    };

    diesel::update(profiles::table)
        .filter(profiles::user_id.eq(user_id))
        .set(profiles::avatar_location.eq(sql_types::Json(&url_location)))
        .execute(&mut db.connection)
        .await
        .wrap_err("Failed to insert user profile into database")
        .with_status_code(StatusCode::UNPROCESSABLE_ENTITY)?;

    Ok(Json(UserAvatarPutUrl {
        url: Cow::Owned(
            url_resolver
                .resolve_put_url(
                    url_location,
                    format!("image/{}", put_avatar_query.file_extension),
                )
                .await,
        ),
    }))
}

#[utoipa_e2e::implementor_of(DeleteAvatarRoute)]
#[instrument(skip(db))]
pub async fn delete_avatar(
    State(url_resolver): State<Arc<UrlResolver>>,
    mut db: DatabaseConnection,
) -> Result<(), error::EyreError> {
    let Some(user_id) = db.user.as_ref().map(|u| u.sub) else {
        return Err(eyre!(
            "You need to be logged in to update your user profile"
        ))
        .with_status_code(StatusCode::UNAUTHORIZED);
    };

    let current_profile = get_user_profile(&mut db, user_id).await?;

    if let Some(location) = current_profile.avatar_location {
        url_resolver.delete_url(location.0).await?;
    }

    diesel::update(profiles::table)
        .filter(profiles::user_id.eq(user_id))
        .set(profiles::avatar_location.eq(None::<sql_types::Json<UrlLocation>>))
        .execute(&mut db.connection)
        .await
        .wrap_err("Failed to insert user profile into database")
        .with_status_code(StatusCode::UNPROCESSABLE_ENTITY)?;

    Ok(())
}

#[utoipa_e2e::implementor_of(UpdateProfileRoute)]
#[instrument(skip(db))]
pub async fn update_profile(
    mut db: DatabaseConnection,
    Json(updated_profile): Json<UserProfileChangeset<'_>>,
) -> Result<(), error::EyreError> {
    let Some(user_id) = db.user.map(|u| u.sub) else {
        return Err(eyre!(
            "You need to be logged in to update your user profile"
        ))
        .with_status_code(StatusCode::UNAUTHORIZED);
    };

    diesel::update(profiles::table)
        .filter(profiles::user_id.eq(user_id))
        .set(updated_profile)
        .execute(&mut db.connection)
        .await
        .wrap_err("Failed to insert user profile into database")
        .with_status_code(StatusCode::UNPROCESSABLE_ENTITY)?;

    Ok(())
}
