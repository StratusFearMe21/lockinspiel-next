REVOKE USAGE ON ALL SEQUENCES IN SCHEMA timekeeper FROM anon;
DROP TABLE timekeeper.tag;
DROP TABLE timekeeper.timesheet;
DROP TABLE timekeeper.time_split_timer;
DROP TABLE timekeeper.time_split;
REVOKE USAGE ON SCHEMA timekeeper FROM PUBLIC;
