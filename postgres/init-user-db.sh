#!/usr/bin/env bash
set -e

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
	CREATE ROLE "service";
	CREATE ROLE "diesel";

	CREATE USER auth_service WITH PASSWORD '$POSTGRES_PASSWORD';
	CREATE USER timekeeper_service WITH PASSWORD '$POSTGRES_PASSWORD';
	CREATE USER user_service WITH PASSWORD '$POSTGRES_PASSWORD';

	GRANT diesel TO timekeeper_service;
	GRANT diesel TO user_service;

	GRANT service TO diesel;

	CREATE USER otelu WITH PASSWORD '$POSTGRES_PASSWORD';

	CREATE DATABASE docker;

	-- RAAAAA ORYYYYYYYYYYY!!!!
	GRANT ALL PRIVILEGES ON DATABASE docker TO auth_service;
	GRANT CONNECT ON DATABASE docker TO timekeeper_service;
	GRANT CONNECT ON DATABASE docker TO user_service;

	GRANT pg_monitor TO otelu;

	CREATE ROLE "anon";
	CREATE ROLE "authenticator";
	CREATE ROLE "authenticated";

	GRANT anon TO service;
	GRANT authenticator TO service;
	GRANT authenticated TO service;

	GRANT anon TO authenticated;
EOSQL

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "docker" <<-EOSQL
	CREATE EXTENSION IF NOT EXISTS timescaledb;
	CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

	GRANT USAGE, CREATE ON SCHEMA public TO service;

	CREATE SCHEMA auth;
	ALTER SCHEMA auth OWNER TO auth_service;

	CREATE SCHEMA timekeeper;
	ALTER SCHEMA timekeeper OWNER TO timekeeper_service;

	CREATE SCHEMA "user";
	ALTER SCHEMA "user" OWNER TO user_service;
EOSQL

