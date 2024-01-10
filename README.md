# Wheel

## Quick Start

![](./gifs/install.gif)

Run the following to install wheel

    curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/ceramicstudio/wheel/main/wheel.sh | bash
    
Please follow the instructions that follow.

## What is Wheel

Setup Ceramic and ComposeDB in a quick and easy fashion

Wheel can handle "default" behavior for Ceramic and ComposeDB based on your network, or you can customize your configuration by stepping through some or all the available configuration options.

![](./gifs/running.gif)

Wheel can also install Ceramic and run it, as well as install ComposeDB. Both will use the configuration you've defined.

![](./gifs/composedb.gif)

Wheel even provides a quiet mode, for those that know exactly what they want, and don't need to interact with Wheel option
by option.

## Running Wheel in Quiet Mode

If you don't want to step through prompts at all, you can use wheel in "quiet" mode, which will default to clay

    wheel --working-directory <path to setup your work in> quiet  --network <one of in-memory|local|dev|clay|mainnet> generate 

You can also pass an existing DID and PK via the `specify` option instead of `generate`. This requires you to have already setup a DID and [CAS Auth](#cas-auth). Please run `wheel --help` for more options.

### CAS Auth

All networks other than InMemory require CAS authorization. Wheel will walk you through setting up CAS authorization, but
for more information read about [starting your node and copying your DID](https://composedb.js.org/docs/0.4.x/guides/composedb-server/access-mainnet#step-1-start-your-node-and-copy-your-key-did).

## Setting up Postgres

If using Postgres, it will need to be setup. *Note*: For production ceramic nodes, only postgres is supported.

### Option 1: Local Install

Visit <https://www.postgresql.org/download/> to install postgres locally.

You will then need to configure postgres for ceramic.

    $ psql postgres

    CREATE DATABASE ceramic;

    CREATE ROLE ceramic WITH PASSWORD 'password' LOGIN;

    GRANT ALL PRIVILEGES ON DATABASE "ceramic" to ceramic;

The connection string you provide to wheel will then be `postgres://ceramic:password@127.0.0.1:5432/ceramic`

### Option 2: Using Docker

For local development and testing, you can run a postgres in docker rather than installing a postgres server locally. The wheel defaults are to use sqlite, however, this is an option if you want to verify postgres indexing. It is not recommended to run a production node this way! This requires having [Docker](https://docs.docker.com/engine/install/) and [Docker compose](https://docs.docker.com/compose/install/) installed. You can read more about the [official Postgres image](https://www.docker.com/blog/how-to-use-the-postgres-docker-official-image/).

Copy the [compose.yaml](https://github.com/ceramicstudio/wheel/blob/main/compose.yaml) file to your computer. You are welcome to change the values, but by default the connection string for wheel will be `postgres://ceramic:password@127.0.0.1:5432/ceramic`. Start the container:

    docker compose up -d

To stop it

    docker compose down # include -v to delete the data

Postgres data will be stored in the `./ceramic-data` folder using a docker volume.
