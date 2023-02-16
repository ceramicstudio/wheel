# Wheel

Compose DB Start Tool

## Getting Started
Run the following to install wheel

    curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/dbcfd/wheel/main/wheel.sh | bash

Please follow the instructions that follow

## Setting up Postgres
For production ceramic nodes, postgres is required. Visit https://www.postgresql.org/download/ to install postgres.

You will then need to configure postgres for ceramic.

    $ psql postgresql

    CREATE DATABASE ceramic;

    CREATE ROLE ceramic WITH PASSWORD 'password' LOGIN;

    GRANT ALL PRIVILEGES ON DATABASE "ceramic" to ceramic;

The connection string you provide to wheel will then be `postgres://ceramic:password@127.0.0.1:5432/ceramic`
