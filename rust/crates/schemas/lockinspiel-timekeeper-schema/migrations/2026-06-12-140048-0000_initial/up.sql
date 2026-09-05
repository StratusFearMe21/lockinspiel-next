GRANT USAGE ON SCHEMA timekeeper TO anon;

CREATE OR REPLACE FUNCTION timekeeper.uid()
RETURNS uuid
LANGUAGE sql
STABLE
AS $$
  SELECT current_setting('app.current_user_id', true)::uuid;
$$;

CREATE OR REPLACE FUNCTION timekeeper.set_uid(uid uuid)
RETURNS text
LANGUAGE sql
STABLE
AS $$
  SELECT set_config('app.current_user_id', uid::text, false);
$$;

CREATE TABLE timekeeper.time_split(
    id SERIAL PRIMARY KEY,
    user_id uuid NOT NULL,
    name VARCHAR NOT NULL,
    description VARCHAR,
    deleted BOOLEAN NOT NULL DEFAULT false
);

GRANT INSERT, SELECT, UPDATE, DELETE ON timekeeper.time_split TO authenticated;
GRANT SELECT ON timekeeper.time_split TO anon;

ALTER TABLE timekeeper.time_split ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can create a time_split."
ON timekeeper.time_split FOR INSERT
TO authenticated
WITH CHECK ( timekeeper.uid() = user_id );

CREATE POLICY "time_splits are viewable by anyone"
ON timekeeper.time_split FOR SELECT
TO anon
USING ( true );

CREATE POLICY "Users can update their own time_splits."
ON timekeeper.time_split FOR UPDATE
TO authenticated
USING ( timekeeper.uid() = user_id )
WITH CHECK ( timekeeper.uid() = user_id );

CREATE POLICY "Users can delete their own time_splits."
ON timekeeper.time_split FOR DELETE
TO authenticated
USING ( timekeeper.uid() = user_id );

CREATE TABLE timekeeper.time_split_timer(
    id SERIAL PRIMARY KEY,
    order_idx INTEGER NOT NULL,
    time_split_id INTEGER NOT NULL REFERENCES timekeeper.time_split(id),
    len INTERVAL NOT NULL,
    name VARCHAR NOT NULL,
    work BOOLEAN NOT NULL,
    deleted BOOLEAN NOT NULL DEFAULT false
);

GRANT INSERT, SELECT, UPDATE, DELETE ON timekeeper.time_split_timer TO authenticated;
GRANT SELECT ON timekeeper.time_split_timer TO anon;

ALTER TABLE timekeeper.time_split_timer ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can create a time_split_timer."
ON timekeeper.time_split_timer FOR INSERT
TO authenticated
WITH CHECK (
    EXISTS (
        SELECT 1
        FROM timekeeper.time_split
        WHERE timekeeper.time_split.id = time_split_id
        AND timekeeper.time_split.user_id = timekeeper.uid()
    )
);

CREATE POLICY "time_split_timers are viewable by anyone"
ON timekeeper.time_split_timer FOR SELECT
TO anon
USING ( true );

CREATE POLICY "Users can update their own time_split_timers."
ON timekeeper.time_split_timer FOR UPDATE
TO authenticated
USING (
    EXISTS (
        SELECT 1
        FROM timekeeper.time_split
        WHERE timekeeper.time_split.id = time_split_id
        AND timekeeper.time_split.user_id = timekeeper.uid()
    )
)
WITH CHECK (
    EXISTS (
        SELECT 1
        FROM timekeeper.time_split
        WHERE timekeeper.time_split.id = time_split_id
        AND timekeeper.time_split.user_id = timekeeper.uid()
    )
);

CREATE POLICY "Users can delete their own time_split_timers."
ON timekeeper.time_split_timer FOR DELETE
TO authenticated
USING (
    EXISTS (
        SELECT 1
        FROM timekeeper.time_split
        WHERE timekeeper.time_split.id = time_split_id
        AND timekeeper.time_split.user_id = timekeeper.uid()
    )
);

CREATE TABLE timekeeper.timesheet(
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    user_id uuid NOT NULL,
    tags INTEGER[] NOT NULL DEFAULT '{}',
    time_split_timer INTEGER REFERENCES timekeeper.time_split_timer(id),
    PRIMARY KEY(user_id, start_time)
) WITH (
    tsdb.hypertable,
    tsdb.segmentby = 'user_id',
    tsdb.partition_column = 'start_time',
    tsdb.orderby = 'start_time DESC',
    tsdb.create_default_indexes = false,
    tsdb.chunk_interval='7 days'
);

CREATE INDEX ON timekeeper.timesheet (user_id, start_time DESC);

GRANT INSERT, SELECT, UPDATE, DELETE ON timekeeper.timesheet TO authenticated;
GRANT SELECT ON timekeeper.timesheet TO anon;

-- CREATE VIEW timesheet AS
-- SELECT
--   *
-- FROM
--   raw_timesheet_data
-- WHERE
--     user_id = timekeeper.uid()
-- WITH CHECK OPTION;

-- ALTER TABLE timesheet ENABLE ROW LEVEL SECURITY;

-- CREATE POLICY "Users can create a timesheet."
-- ON timesheet FOR INSERT
-- TO authenticated
-- WITH CHECK ( (SELECT timekeeper.uid()) = user_id );

-- CREATE POLICY "Public timesheets are viewable only by authenticated users"
-- ON timesheet FOR SELECT
-- TO authenticated
-- USING ( true );

-- CREATE POLICY "Users can update their own timesheets."
-- ON timesheet FOR UPDATE
-- TO authenticated
-- USING ( (SELECT timekeeper.uid()) = user_id )
-- WITH CHECK ( (SELECT timekeeper.uid()) = user_id );

-- CREATE POLICY "Users can delete their own timesheets."
-- ON timesheet FOR DELETE
-- TO authenticated
-- USING ( (SELECT timekeeper.uid()) = user_id );

CREATE TABLE timekeeper.tag(
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE,
    user_id uuid NOT NULL,
    deleted BOOLEAN NOT NULL DEFAULT false
);

GRANT INSERT, SELECT, UPDATE, DELETE ON timekeeper.tag TO authenticated;

ALTER TABLE timekeeper.tag ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can create a tag."
ON timekeeper.tag FOR INSERT
TO authenticated
WITH CHECK ( timekeeper.uid() = user_id );

CREATE POLICY "Tags are viewable by anyone"
ON timekeeper.tag FOR SELECT
TO anon
USING ( true );

CREATE POLICY "Users can update their own tags."
ON timekeeper.tag FOR UPDATE
TO authenticated
USING ( timekeeper.uid() = user_id )
WITH CHECK ( timekeeper.uid() = user_id );

CREATE POLICY "Users can delete their own tags."
ON timekeeper.tag FOR DELETE
TO authenticated
USING ( timekeeper.uid() = user_id );

INSERT INTO timekeeper.time_split (id, name, user_id) VALUES (0, '_paused_', '00000000-0000-0000-0000-000000000000');
INSERT INTO timekeeper.time_split (name, description, user_id) VALUES
    ('Pomodoro', 'Classic, tried, and true', '00000000-0000-0000-0000-000000000000'),
    ('Time Magazine', 'Based on studies', '00000000-0000-0000-0000-000000000000'),
    ('Tyson Split', 'For those with extra dog in ''em', '00000000-0000-0000-0000-000000000000'),
    ('Build Night', 'We burnin'' out tonight baby!', '00000000-0000-0000-0000-000000000000');

INSERT INTO timekeeper.time_split_timer (time_split_id, order_idx, len, name, work) VALUES
    -- Pomodoro
    (1, 0, INTERVAL '25 minutes', 'Work', true),
    (1, 1, INTERVAL '5 minutes', 'Break', false),
    (1, 2, INTERVAL '25 minutes', 'Work', true),
    (1, 3, INTERVAL '15 minutes', 'Long Break', false),
    -- Time Magazine
    (2, 0, INTERVAL '52 minutes', 'Work', true),
    (2, 1, INTERVAL '17 minutes', 'Break', false),
    -- Tyson Split
    (3, 0, INTERVAL '90 minutes', 'Work', true),
    (3, 1, INTERVAL '10 minutes', 'Break', false),
    -- Build Night
    (4, 0, INTERVAL '120 minutes', 'Work', true),
    (4, 1, INTERVAL '10 minutes', 'Break', false);

GRANT USAGE ON ALL SEQUENCES IN SCHEMA timekeeper TO anon;
