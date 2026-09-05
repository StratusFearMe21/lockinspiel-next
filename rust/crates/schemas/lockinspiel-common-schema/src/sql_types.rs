#[cfg(feature = "diesel")]
use diesel::{
    data_types::{PgInterval, PgTimestamp},
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    pg::{self, Pg, PgValue},
    serialize::{self, Output, ToSql},
    sql_types,
};
#[cfg(feature = "diesel")]
use jiff::SignedDuration;
use jiff::civil;
#[cfg(feature = "diesel")]
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

// A lot of this is pulled directly out of the diesel source code
//
// https://docs.rs/diesel/latest/src/diesel/pg/types/date_and_time/std_time.rs.html
// https://docs.rs/diesel/latest/src/diesel/pg/types/date_and_time/chrono.rs.html

#[cfg(feature = "diesel")]
fn pg_epoch_datetime() -> civil::DateTime {
    civil::DateTime::new(2000, 1, 1, 0, 0, 0, 0).expect("This is in supported range of jiff dates")
}

#[repr(transparent)]
#[cfg_attr(feature = "diesel", derive(AsExpression, FromSqlRow))]
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "diesel", diesel(sql_type = sql_types::Timestamp))]
pub struct DateTime(pub civil::DateTime);

#[cfg(feature = "diesel")]
impl FromSql<sql_types::Timestamp, Pg> for DateTime {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        let PgTimestamp(offset) = FromSql::<sql_types::Timestamp, Pg>::from_sql(bytes)?;
        match pg_epoch_datetime().checked_add(SignedDuration::from_micros(offset)) {
            Ok(v) => Ok(DateTime(v)),
            Err(e) => {
                let message = format!(
                    "Tried to deserialize a timestamp that is too large for Jiff: `{}`",
                    e
                );
                Err(message.into())
            }
        }
    }
}

#[cfg(feature = "diesel")]
impl ToSql<sql_types::Timestamp, Pg> for DateTime {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let time: i64 = match (self.0.duration_since(pg_epoch_datetime()))
            .as_micros()
            .try_into()
        {
            Ok(time) => time,
            Err(_) => {
                let error_message =
                    format!("{self:?} as microseconds is too large to fit in an i64");
                return Err(error_message.into());
            }
        };
        ToSql::<sql_types::Timestamp, Pg>::to_sql(&PgTimestamp(time), &mut out.reborrow())
    }
}

#[cfg(feature = "diesel")]
impl FromSql<sql_types::Timestamptz, Pg> for DateTime {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        FromSql::<sql_types::Timestamp, Pg>::from_sql(bytes)
    }
}

#[cfg(feature = "diesel")]
impl ToSql<pg::sql_types::Timestamptz, Pg> for DateTime {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        ToSql::<sql_types::Timestamp, Pg>::to_sql(self, out)
    }
}

#[repr(transparent)]
#[cfg_attr(feature = "diesel", derive(AsExpression, FromSqlRow))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "diesel", diesel(sql_type = sql_types::Interval))]
#[cfg_attr(feature = "utoipa", schema(value_type = String))]
pub struct Interval(pub jiff::Span);

#[cfg(feature = "diesel")]
impl FromSql<sql_types::Interval, Pg> for Interval {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        let PgInterval {
            microseconds,
            days,
            months,
        } = FromSql::<sql_types::Interval, Pg>::from_sql(bytes)?;

        Ok(Self(
            jiff::Span::new()
                .microseconds(microseconds)
                .days(days)
                .months(months),
        ))
    }
}

#[cfg(feature = "diesel")]
impl ToSql<sql_types::Interval, Pg> for Interval {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let months = (self.0.get_years() as i32 * 12) + self.0.get_months();
        let days = (self.0.get_weeks() * 7) + self.0.get_days();
        let microseconds = (self.0.get_hours() as i64 * 3_600_000_000)
            + (self.0.get_minutes() * 60_000_000)
            + (self.0.get_seconds() * 1_000_000)
            + (self.0.get_milliseconds() * 1000)
            + self.0.get_microseconds();

        ToSql::<sql_types::Interval, Pg>::to_sql(
            &PgInterval {
                microseconds,
                days,
                months,
            },
            &mut out.reborrow(),
        )
    }
}

mod serde_jiff_timestamp {
    use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<jiff::Timestamp, D::Error>
    where
        D: Deserializer<'de>,
    {
        let timestamp_micros = i64::deserialize(deserializer)?;
        Ok(jiff::Timestamp::from_microsecond(timestamp_micros).map_err(D::Error::custom)?)
    }

    pub fn serialize<S>(timestamp: &jiff::Timestamp, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        timestamp.as_microsecond().serialize(serializer)
    }
}

// SAFETY: Must remain a transparent struct
// containing the same type as `Timestamptz`
#[repr(transparent)]
#[cfg_attr(feature = "diesel", derive(AsExpression, FromSqlRow))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "diesel", diesel(sql_type = sql_types::Timestamp))]
#[cfg_attr(feature = "utoipa", schema(value_type = i64))]
/// The returned timestamp is the number of **microseconds** since
/// the UNIX epoch
pub struct Timestamp(#[serde(with = "serde_jiff_timestamp")] pub jiff::Timestamp);

// SAFETY: Must remain a transparent struct
// containing the same type as `Timestamp`
#[repr(transparent)]
#[cfg_attr(feature = "diesel", derive(AsExpression, FromSqlRow))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "diesel", diesel(sql_type = pg::sql_types::Timestamptz))]
#[cfg_attr(feature = "utoipa", schema(value_type = i64))]
/// The returned timestamp is the number of **microseconds** since
/// the UNIX epoch
pub struct Timestamptz(#[serde(with = "serde_jiff_timestamp")] pub jiff::Timestamp);

#[cfg(feature = "diesel")]
pub fn pg_epoch_timestamp() -> jiff::Timestamp {
    let thirty_years = SignedDuration::from_secs(946_684_800);
    jiff::Timestamp::UNIX_EPOCH + thirty_years
}

#[cfg(feature = "diesel")]
const USEC_PER_SEC: i64 = 1_000_000;
#[cfg(feature = "diesel")]
const NANO_PER_USEC: i32 = 1_000;

#[cfg(feature = "diesel")]
fn usecs_to_duration(usecs_passed: i64) -> SignedDuration {
    let seconds = usecs_passed / USEC_PER_SEC;
    let subsecond_usecs = usecs_passed % USEC_PER_SEC;
    let subseconds = subsecond_usecs as i32 * NANO_PER_USEC;
    SignedDuration::new(seconds, subseconds)
}

#[cfg(feature = "diesel")]
fn duration_to_usecs(duration: SignedDuration) -> i64 {
    let seconds = duration.as_secs() * USEC_PER_SEC;
    let subseconds = duration.subsec_micros();
    seconds + i64::from(subseconds)
}

#[cfg(feature = "diesel")]
impl ToSql<sql_types::Timestamp, Pg> for Timestamp {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let duration = self.0.duration_since(pg_epoch_timestamp());
        let time_since_epoch = duration_to_usecs(duration);
        ToSql::<sql_types::BigInt, Pg>::to_sql(&time_since_epoch, &mut out.reborrow())
    }
}

#[cfg(feature = "diesel")]
impl FromSql<sql_types::Timestamp, Pg> for Timestamp {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        let usecs_passed = <i64 as FromSql<sql_types::BigInt, Pg>>::from_sql(bytes)?;
        let time_passed = usecs_to_duration(usecs_passed);

        Ok(Timestamp(pg_epoch_timestamp() + time_passed))
    }
}

#[cfg(feature = "diesel")]
impl ToSql<sql_types::Timestamptz, Pg> for Timestamptz {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let timestamp = unsafe { std::mem::transmute::<&'b Timestamptz, &'b Timestamp>(self) };
        ToSql::<sql_types::Timestamp, Pg>::to_sql(timestamp, out)
    }
}

#[cfg(feature = "diesel")]
impl FromSql<sql_types::Timestamptz, Pg> for Timestamptz {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        let timestamp: Timestamp = FromSql::<sql_types::Timestamp, Pg>::from_sql(bytes)?;
        Ok(Self(timestamp.0))
    }
}

// From https://github.com/PPakalns/diesel_json/blob/9443c0168952bf2cb4ef156c9771438f9632ede0/src/lib.rs
#[cfg_attr(feature = "diesel", derive(AsExpression, FromSqlRow))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(transparent)]
#[cfg_attr(feature = "diesel", diesel(sql_type = sql_types::Jsonb))]
pub struct Json<T: Sized>(pub T);

impl<T> Json<T> {
    pub fn new(value: T) -> Json<T> {
        Json(value)
    }
}

#[cfg(feature = "diesel")]
impl<T> FromSql<sql_types::Jsonb, Pg> for Json<T>
where
    T: std::fmt::Debug + DeserializeOwned,
{
    fn from_sql(bytes: PgValue) -> diesel::deserialize::Result<Self> {
        let value = <serde_json::Value as FromSql<sql_types::Jsonb, Pg>>::from_sql(bytes)?;
        Ok(Json(serde_json::from_value::<T>(value)?))
    }
}

#[cfg(feature = "diesel")]
impl<T> ToSql<sql_types::Jsonb, Pg> for Json<T>
where
    T: std::fmt::Debug + Serialize,
{
    fn to_sql(&self, out: &mut diesel::serialize::Output<Pg>) -> diesel::serialize::Result {
        let value = serde_json::to_value(self)?;
        <serde_json::Value as ToSql<sql_types::Jsonb, Pg>>::to_sql(&value, &mut out.reborrow())
    }
}
