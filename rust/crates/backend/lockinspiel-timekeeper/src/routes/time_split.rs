use std::collections::HashMap;

use axum::{Json, extract::Path, http::StatusCode};
use color_eyre::eyre::{Context, eyre};
use diesel::{BoolExpressionMethods, ExpressionMethods, HasQuery, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lockinspiel_backend_common::{
    auth::DatabaseConnection,
    error::{self, WithStatusCode},
};
use lockinspiel_timekeeper_schema::{
    schema::timekeeper,
    time_split::{
        CreateTimeSplitRoute, DeleteTimeSplitRoute, GetTimeSplitsRoute,
        InsertablePackagedTimeSplit, InsertableTimeSplit, ModifyTimeSplitRoute, PackagedTimeSplit,
        TimeSplit, TimeSplitTimer,
    },
};
use uuid::Uuid;

#[utoipa_e2e::implementor_of(CreateTimeSplitRoute)]
pub async fn create_time_split<'a>(
    mut db: DatabaseConnection,
    Json(InsertablePackagedTimeSplit { time_split, timers }): Json<InsertablePackagedTimeSplit<'a>>,
) -> Result<Json<PackagedTimeSplit<'a>>, error::EyreError> {
    let Some(user_id) = db.user.map(|u| u.sub) else {
        return Err(eyre!("You need to be logged in to create a time split"))
            .with_status_code(StatusCode::UNAUTHORIZED);
    };

    let time_split_id = diesel::insert_into(timekeeper::time_split::table)
        .values((&time_split, timekeeper::time_split::user_id.eq(user_id)))
        .returning(timekeeper::time_split::id)
        .get_result(&mut db.connection)
        .await
        .wrap_err("Failed to insert time split into database")
        .with_status_code(StatusCode::UNPROCESSABLE_ENTITY)?;

    let timers = diesel::insert_into(timekeeper::time_split_timer::table)
        .values(
            timers
                .into_iter()
                .enumerate()
                .map(|(order, t)| {
                    (
                        t,
                        timekeeper::time_split_timer::order_idx.eq(order as i32),
                        timekeeper::time_split_timer::time_split_id.eq(time_split_id),
                    )
                })
                .collect::<Vec<_>>(),
        )
        .returning(TimeSplitTimer::as_returning())
        .get_results(&mut db.connection)
        .await
        .wrap_err("Failed to insert time split timers into database")
        .with_status_code(StatusCode::UNPROCESSABLE_ENTITY)?;

    Ok(Json(PackagedTimeSplit {
        time_split: TimeSplit {
            id: time_split_id,
            name: time_split.name,
            description: time_split.description,
        },
        timers,
    }))
}

#[utoipa_e2e::implementor_of(GetTimeSplitsRoute)]
pub async fn get_time_splits<'a>(
    mut db: DatabaseConnection,
) -> Result<Json<Vec<PackagedTimeSplit<'a>>>, error::EyreError> {
    let time_splits = TimeSplit::query()
        .filter(
            timekeeper::time_split::user_id
                .eq(db.user.map_or_else(Uuid::nil, |u| u.sub))
                .or(timekeeper::time_split::user_id.is_null()),
        )
        .get_results(&mut db.connection)
        .await
        .wrap_err("Failed to retreive time splits from database")
        .with_status_code(StatusCode::INTERNAL_SERVER_ERROR)?;

    let time_split_timers = TimeSplitTimer::query()
        .filter(
            timekeeper::time_split_timer::time_split_id.eq_any(time_splits.iter().map(|t| t.id)),
        )
        .order_by(timekeeper::time_split_timer::order_idx)
        .get_results(&mut db.connection)
        .await
        .wrap_err("Failed to retreive time split timers from database")
        .with_status_code(StatusCode::INTERNAL_SERVER_ERROR)?;

    // From https://docs.rs/diesel/latest/src/diesel/associations/belongs_to.rs.html#291-352
    let id_indices: HashMap<_, _> = time_splits
        .iter()
        .enumerate()
        .map(|(i, u)| (u.id, i))
        .collect();

    let mut grouped: Vec<_> = time_splits
        .into_iter()
        .map(|time_split| PackagedTimeSplit {
            time_split,
            timers: Vec::new(),
        })
        .collect();

    time_split_timers
        .into_iter()
        .filter_map(|child| {
            let fk = &child.time_split_id;
            let i = id_indices.get(fk)?;

            Some((i, child))
        })
        .for_each(|(i, child)| grouped[*i].timers.push(child));

    Ok(Json(grouped))
}

#[utoipa_e2e::implementor_of(ModifyTimeSplitRoute)]
pub async fn modify_time_split<'a>(
    mut db: DatabaseConnection,
    Path(id): Path<i32>,
    Json(time_split): Json<InsertableTimeSplit<'a>>,
) -> Result<(), error::EyreError> {
    diesel::update(timekeeper::time_split::table)
        .filter(timekeeper::time_split::id.eq(id))
        .set(time_split)
        .execute(&mut db.connection)
        .await
        .wrap_err("Failed to update time split in database")
        .with_status_code(StatusCode::UNPROCESSABLE_ENTITY)?;

    Ok(())
}

#[utoipa_e2e::implementor_of(DeleteTimeSplitRoute)]
pub async fn delete_time_split<'a>(
    mut db: DatabaseConnection,
    Path(id): Path<i32>,
) -> Result<(), error::EyreError> {
    diesel::update(timekeeper::time_split_timer::table)
        .filter(timekeeper::time_split_timer::time_split_id.eq(id))
        .set(timekeeper::time_split_timer::deleted.eq(true))
        .execute(&mut db.connection)
        .await
        .wrap_err("Failed to delete time split timers in database")
        .with_status_code(StatusCode::INTERNAL_SERVER_ERROR)?;

    diesel::update(timekeeper::time_split::table)
        .filter(timekeeper::time_split::id.eq(id))
        .set(timekeeper::time_split::deleted.eq(true))
        .execute(&mut db.connection)
        .await
        .wrap_err("Failed to delete time split in database")
        .with_status_code(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}
