# Wheel

Compose DB Start Tool

## Getting Started
Run the following to install wheel

    curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/ceramicstudio/wheel/main/wheel.sh | bash

Please follow the instructions that follow.

If you don't want to step through prompts at all, you can use wheel in "default" mode

    wheel --working-directory <path to setup your work in> --network <one of inmemory|local|dev|clay|mainnet> --no-interactive

Please run `wheel --help` for more options.

## Setting up Postgres
For production ceramic nodes, postgres is required. Visit https://www.postgresql.org/download/ to install postgres.

You will then need to configure postgres for ceramic.

    $ psql postgres

    CREATE DATABASE ceramic;

    CREATE ROLE ceramic WITH PASSWORD 'password' LOGIN;

    GRANT ALL PRIVILEGES ON DATABASE "ceramic" to ceramic;

The connection string you provide to wheel will then be `postgres://ceramic:password@127.0.0.1:5432/ceramic`
