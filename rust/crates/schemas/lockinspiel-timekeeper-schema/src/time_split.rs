use std::borrow::Cow;

#[cfg(feature = "diesel")]
use crate::schema::timekeeper;
#[cfg(feature = "diesel")]
use diesel::{
    AsChangeset, Associations, ExpressionMethods, HasQuery, Identifiable, Insertable, QueryDsl,
};
use jiff::Span;
use lockinspiel_common_schema::{error::EyreErrorWrapper, sql_types::Interval};
use serde::{Deserialize, Serialize};
#[cfg(feature = "utoipa")]
use utoipa::{IntoResponses, ToSchema};
use utoipa_e2e::IntoPath;

#[cfg_attr(feature = "diesel", derive(Insertable))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[cfg_attr(
    feature = "utoipa",
    schema(examples(InsertableTimeSplitTimer::placeholder))
)]
#[derive(Deserialize, Serialize, Debug)]
#[cfg_attr(feature = "diesel", diesel(table_name = timekeeper::time_split_timer))]
#[cfg_attr(feature = "diesel", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct InsertableTimeSplitTimer<'a> {
    pub len: Interval,
    pub name: Cow<'a, str>,
    pub work: bool,
}

impl InsertableTimeSplitTimer<'_> {
    pub fn placeholder() -> Self {
        Self {
            len: Interval(Span::new().minutes(25)),
            name: Cow::Borrowed("Work"),
            work: true,
        }
    }
}

#[cfg_attr(feature = "diesel", derive(HasQuery, Associations, Identifiable))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[cfg_attr(feature = "utoipa", schema(examples(TimeSplitTimer::placeholder)))]
#[derive(Deserialize, Serialize, Debug)]
#[cfg_attr(feature = "diesel", diesel(table_name = timekeeper::time_split_timer))]
#[cfg_attr(feature = "diesel", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "diesel", diesel(belongs_to(TimeSplit<'_>)))]
#[cfg_attr(feature = "diesel", diesel(base_query = timekeeper::time_split_timer::table.filter(timekeeper::time_split_timer::deleted.eq(false))))]
pub struct TimeSplitTimer<'a> {
    pub id: i32,
    pub time_split_id: i32,
    pub order_idx: i32,
    pub len: Interval,
    pub name: Cow<'a, str>,
    pub work: bool,
}

impl TimeSplitTimer<'_> {
    pub fn placeholder() -> Self {
        Self {
            id: 35,
            time_split_id: 989,
            order_idx: 0,
            len: Interval(Span::new().minutes(25)),
            name: Cow::Borrowed("Work"),
            work: true,
        }
    }
}

#[cfg_attr(feature = "diesel", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[cfg_attr(feature = "utoipa", schema(examples(InsertableTimeSplit::placeholder)))]
#[derive(Deserialize, Serialize, Debug)]
#[cfg_attr(feature = "diesel", diesel(table_name = timekeeper::time_split))]
#[cfg_attr(feature = "diesel", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct InsertableTimeSplit<'a> {
    pub name: Cow<'a, str>,
    pub description: Option<Cow<'a, str>>,
}

impl InsertableTimeSplit<'_> {
    pub fn placeholder() -> Self {
        Self {
            name: Cow::Borrowed("Pomodoro"),
            description: Some(Cow::Borrowed("The certified hood classic")),
        }
    }
}

#[cfg_attr(feature = "diesel", derive(HasQuery, Identifiable))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[cfg_attr(feature = "utoipa", schema(examples(TimeSplit::placeholder)))]
#[derive(Deserialize, Serialize, Debug)]
#[cfg_attr(feature = "diesel", diesel(table_name = timekeeper::time_split))]
#[cfg_attr(feature = "diesel", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "diesel", diesel(base_query = timekeeper::time_split::table.filter(timekeeper::time_split::deleted.eq(false))))]
pub struct TimeSplit<'a> {
    pub id: i32,
    pub name: Cow<'a, str>,
    pub description: Option<Cow<'a, str>>,
}

impl TimeSplit<'_> {
    pub fn placeholder() -> Self {
        Self {
            id: 98,
            name: Cow::Borrowed("Pomodoro"),
            description: Some(Cow::Borrowed("The certified hood classic")),
        }
    }
}

#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[cfg_attr(
    feature = "utoipa",
    schema(examples(InsertablePackagedTimeSplit::placeholder))
)]
#[derive(Deserialize, Serialize, Debug)]
pub struct InsertablePackagedTimeSplit<'a> {
    pub time_split: InsertableTimeSplit<'a>,
    pub timers: Vec<InsertableTimeSplitTimer<'a>>,
}

impl InsertablePackagedTimeSplit<'_> {
    pub fn placeholder() -> Self {
        Self {
            time_split: InsertableTimeSplit::placeholder(),
            timers: vec![InsertableTimeSplitTimer::placeholder()],
        }
    }
}

#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[cfg_attr(feature = "utoipa", schema(examples(PackagedTimeSplit::placeholder)))]
#[derive(Deserialize, Serialize, Debug)]
pub struct PackagedTimeSplit<'a> {
    pub time_split: TimeSplit<'a>,
    pub timers: Vec<TimeSplitTimer<'a>>,
}

impl PackagedTimeSplit<'_> {
    pub fn placeholder() -> Self {
        Self {
            time_split: TimeSplit::placeholder(),
            timers: vec![TimeSplitTimer::placeholder()],
        }
    }
}

#[cfg_attr(feature = "utoipa", derive(IntoResponses, ToSchema))]
pub enum TimeSplitVoidResponses<'a> {
    #[cfg_attr(feature = "utoipa", response(status = 200))]
    Success(()),
    #[cfg_attr(
        feature = "utoipa",
        response(status = "4XX", description = "It's your fault")
    )]
    YourFault(EyreErrorWrapper<'a>),
    #[cfg_attr(
        feature = "utoipa",
        response(status = "5XX", description = "We're having a skill issue")
    )]
    OurFault(EyreErrorWrapper<'a>),
}

#[cfg_attr(feature = "utoipa", derive(IntoResponses, ToSchema))]
pub enum TimeSplitArrayResponses<'a> {
    #[cfg_attr(feature = "utoipa", response(status = 200))]
    Success(Vec<PackagedTimeSplit<'a>>),
    #[cfg_attr(
        feature = "utoipa",
        response(status = "4XX", description = "It's your fault")
    )]
    YourFault(EyreErrorWrapper<'a>),
    #[cfg_attr(
        feature = "utoipa",
        response(status = "5XX", description = "We're having a skill issue")
    )]
    OurFault(EyreErrorWrapper<'a>),
}

#[cfg_attr(feature = "utoipa", derive(IntoResponses, ToSchema))]
pub enum TimeSplitResponses<'a> {
    #[cfg_attr(feature = "utoipa", response(status = 200))]
    Success(PackagedTimeSplit<'a>),
    #[cfg_attr(
        feature = "utoipa",
        response(status = "4XX", description = "It's your fault")
    )]
    YourFault(EyreErrorWrapper<'a>),
    #[cfg_attr(
        feature = "utoipa",
        response(status = "5XX", description = "We're having a skill issue")
    )]
    OurFault(EyreErrorWrapper<'a>),
}

#[derive(IntoPath)]
#[api_path(
    post,
    path = "/timekeeper/time-split",
    summary = "Create a time split",
    description = "Creates a new time split in the database. The returned time split ID can be used in other endpoints in this service. The timer lengths should use the `jiff::Span` format",
    tag = "Time split",
    responses(TimeSplitResponses),
    security(
        ("bearer_jwt" = []),
    )
)]
pub struct CreateTimeSplitRoute<'a> {
    #[body]
    pub body: InsertablePackagedTimeSplit<'a>,
}

#[derive(IntoPath)]
#[api_path(
    get,
    path = "/timekeeper/time-split",
    summary = "Retreive time splits",
    description = "Retrieves all the time splits associated with the user, as well as the default ones",
    tag = "Time split",
    responses(TimeSplitArrayResponses),
    security(
        ("bearer_jwt" = []),
        ()
    )
)]
pub struct GetTimeSplitsRoute {}

#[derive(IntoPath)]
#[api_path(
    put,
    path = "/timekeeper/time-split/{id}",
    summary = "Modify a time split",
    description = "Modifies the fields of the time split at the ID.",
    tag = "Time split",
    responses(TimeSplitVoidResponses),
    security(
        ("bearer_jwt" = []),
    )
)]
pub struct ModifyTimeSplitRoute<'a> {
    #[body]
    pub time_split: InsertableTimeSplit<'a>,
    #[param(Path)]
    pub id: i32,
}

#[derive(IntoPath)]
#[api_path(
    delete,
    path = "/timekeeper/time-split/{id}",
    summary = "Delete a time split",
    description = "Deletes the time split at the given ID. This just marks the time split as deleted, and doesn't actually delete the time split in the database. Timers posted with a deleted time split will still have that time split, the time split just won't appear when querying some endpoints.",
    tag = "Time split",
    responses(TimeSplitVoidResponses),
    security(
        ("bearer_jwt" = []),
    )
)]
pub struct DeleteTimeSplitRoute {
    #[param(Path)]
    pub id: i32,
}
