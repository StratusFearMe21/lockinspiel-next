use axum::{Json, http::StatusCode};
use color_eyre::eyre::{Context, OptionExt, eyre};
use diesel::{ExpressionMethods, HasQuery, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use lockinspiel_backend_common::{
    auth::DatabaseConnection,
    error::{self, WithStatusCode},
};
use lockinspiel_timekeeper_schema::{
    schema::timekeeper,
    timer::{
        GetTimerRoute, InsertableTimer, ModifyTimerRoute, PostTimerRoute, TimeSplitData, Timer,
    },
};

use super::conditional_query;

#[utoipa_e2e::implementor_of(PostTimerRoute)]
pub async fn post_timer(
    mut db: DatabaseConnection,
    Json(timesheet_data): Json<InsertableTimer>,
) -> Result<Json<Timer>, error::EyreError> {
    let Some(user_id) = db.user.map(|u| u.sub) else {
        return Err(eyre!("You need to be logged in to create a timer"))
            .with_status_code(StatusCode::UNAUTHORIZED);
    };

    let time_split_data = TimeSplitData::query()
        .filter(timekeeper::time_split_timer::id.eq(timesheet_data.time_split_timer))
        .get_result(&mut db.connection)
        .await
        .optional()
        .wrap_err("Failed to grab time_split for inserted timesheet")
        .with_status_code(StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or_eyre("Couldn't find a time_split for that time_split_timer")
        .with_status_code(StatusCode::NOT_FOUND)?;

    diesel::insert_into(timekeeper::timesheet::table)
        .values((
            timesheet_data.clone(),
            timekeeper::timesheet::user_id.eq(user_id),
        ))
        .execute(&mut db.connection)
        .await
        .wrap_err("Failed to insert timer into timesheet")
        .with_status_code(StatusCode::UNPROCESSABLE_ENTITY)?;

    Ok(Json(Timer {
        time_split_timer: Some(timesheet_data.time_split_timer),
        start_time: timesheet_data.start_time,
        end_time: timesheet_data.end_time,
        tags: timesheet_data.tags.into_iter().map(|t| Some(t)).collect(),
        time_split_data,
    }))
}

#[utoipa_e2e::implementor_of(GetTimerRoute)]
pub async fn get_timers(mut db: DatabaseConnection) -> Result<Json<Vec<Timer>>, error::EyreError> {
    let timers = conditional_query!(
        let Some(user) = db.user,
        timer,
        Timer::query().filter(timekeeper::timesheet::user_id.eq(user.sub)),
        Timer::query(),
        timer
            .order_by(timekeeper::timesheet::start_time.desc())
            // TODO: Add query parameters to allow multiple timers
            // to be returned
            .limit(1)
            .get_results(&mut db.connection)
            .await
    )
    .wrap_err("Failed to get requested timers")
    .with_status_code(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(timers))
}

#[utoipa_e2e::implementor_of(ModifyTimerRoute)]
pub async fn modify_timer(
    mut db: DatabaseConnection,
    Json(timesheet_data): Json<InsertableTimer>,
) -> Result<Json<Timer>, error::EyreError> {
    let Some(user_id) = db.user.map(|u| u.sub) else {
        return Err(eyre!("You need to be logged in to modify a timer"))
            .with_status_code(StatusCode::UNAUTHORIZED);
    };

    let time_split_data = TimeSplitData::query()
        .filter(timekeeper::time_split_timer::id.eq(timesheet_data.time_split_timer))
        .get_result(&mut db.connection)
        .await
        .optional()
        .wrap_err("Failed to grab time_split for updated timesheet")
        .with_status_code(StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or_eyre("Couldn't find a time_split for that time_split_timer")
        .with_status_code(StatusCode::NOT_FOUND)?;

    diesel::update(timekeeper::timesheet::table)
        .filter(timekeeper::timesheet::user_id.eq(user_id))
        .filter(timekeeper::timesheet::start_time.eq(timesheet_data.start_time))
        .set(timesheet_data.clone())
        .execute(&mut db.connection)
        .await
        .wrap_err("Failed to update timer in timesheet")
        .with_status_code(StatusCode::UNPROCESSABLE_ENTITY)?;

    Ok(Json(Timer {
        time_split_timer: Some(timesheet_data.time_split_timer),
        start_time: timesheet_data.start_time,
        end_time: timesheet_data.end_time,
        tags: timesheet_data.tags.into_iter().map(|t| Some(t)).collect(),
        time_split_data,
    }))
}
