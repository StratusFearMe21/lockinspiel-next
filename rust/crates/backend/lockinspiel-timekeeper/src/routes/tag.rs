use axum::{Json, extract::Path, http::StatusCode};
use color_eyre::eyre::{Context, eyre};
use diesel::{ExpressionMethods, HasQuery, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lockinspiel_backend_common::{
    auth::DatabaseConnection,
    error::{self, WithStatusCode},
};
use lockinspiel_timekeeper_schema::{
    schema::timekeeper,
    tag::{
        CreateTagRoute, DeleteTagRoute, GetTagsRoute, InsertableTag, ModifyTagRoute, Tag, TagID,
    },
};
use uuid::Uuid;

#[utoipa_e2e::implementor_of(CreateTagRoute)]
pub async fn create_tag(
    mut db: DatabaseConnection,
    Json(tag): Json<InsertableTag<'_>>,
) -> Result<Json<TagID>, error::EyreError> {
    let Some(user_id) = db.user.map(|u| u.sub) else {
        return Err(eyre!("You need to be logged in to create a tag"))
            .with_status_code(StatusCode::UNAUTHORIZED);
    };

    let tag_id = diesel::insert_into(timekeeper::tag::table)
        .values((tag, timekeeper::tag::user_id.eq(user_id)))
        .returning(TagID::as_returning())
        .get_result(&mut db.connection)
        .await
        .wrap_err("Failed to insert tag into database")
        .with_status_code(StatusCode::UNPROCESSABLE_ENTITY)?;

    Ok(Json(tag_id))
}

#[utoipa_e2e::implementor_of(GetTagsRoute)]
pub async fn get_tags(
    mut db: DatabaseConnection,
) -> Result<Json<Vec<Tag<'static>>>, error::EyreError> {
    let user_id = db.user.map(|u| u.sub).unwrap_or(Uuid::nil());

    let tags = Tag::query()
        .filter(timekeeper::tag::user_id.eq_any([user_id, Uuid::nil()]))
        .get_results(&mut db.connection)
        .await
        .wrap_err("Failed to grab tags from database")
        .with_status_code(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(tags))
}

#[utoipa_e2e::implementor_of(ModifyTagRoute)]
pub async fn modify_tag(
    mut db: DatabaseConnection,
    Path(id): Path<i32>,
    Json(tag): Json<InsertableTag<'_>>,
) -> Result<(), error::EyreError> {
    diesel::update(timekeeper::tag::table)
        .filter(timekeeper::tag::id.eq(id))
        .set(tag)
        .execute(&mut db.connection)
        .await
        .wrap_err("Failed to update tag in database")
        .with_status_code(StatusCode::UNPROCESSABLE_ENTITY)?;

    Ok(())
}

#[utoipa_e2e::implementor_of(DeleteTagRoute)]
pub async fn delete_tag(
    mut db: DatabaseConnection,
    Path(id): Path<i32>,
) -> Result<(), error::EyreError> {
    diesel::update(timekeeper::tag::table)
        .filter(timekeeper::tag::id.eq(id))
        .set(timekeeper::tag::deleted.eq(true))
        .execute(&mut db.connection)
        .await
        .wrap_err("Failed to update tag in database")
        .with_status_code(StatusCode::UNPROCESSABLE_ENTITY)?;

    Ok(())
}
