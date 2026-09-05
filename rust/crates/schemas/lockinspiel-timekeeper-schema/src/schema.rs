// @generated automatically by Diesel CLI.

pub mod timekeeper {
    diesel::table! {
        timekeeper.tag (id) {
            id -> Int4,
            name -> Varchar,
            user_id -> Uuid,
            deleted -> Bool,
        }
    }

    diesel::table! {
        timekeeper.time_split (id) {
            id -> Int4,
            user_id -> Uuid,
            name -> Varchar,
            description -> Nullable<Varchar>,
            deleted -> Bool,
        }
    }

    diesel::table! {
        timekeeper.time_split_timer (id) {
            id -> Int4,
            order_idx -> Int4,
            time_split_id -> Int4,
            len -> Interval,
            name -> Varchar,
            work -> Bool,
            deleted -> Bool,
        }
    }

    diesel::table! {
        timekeeper.timesheet (user_id, start_time) {
            start_time -> Timestamptz,
            end_time -> Timestamptz,
            user_id -> Uuid,
            tags -> Array<Nullable<Int4>>,
            time_split_timer -> Nullable<Int4>,
        }
    }

    diesel::joinable!(time_split_timer -> time_split (time_split_id));
    diesel::joinable!(timesheet -> time_split_timer (time_split_timer));

    diesel::allow_tables_to_appear_in_same_query!(tag, time_split, time_split_timer, timesheet,);
}
