#!/usr/bin/env bash

if ! command -v curl &> /dev/null
then
    echo "curl was not found, please install curl"
    exit
fi

if ! command -v npx &> /dev/null
then
    echo "npx was not found, please install node.js"
    exit
fi

if ! command -v cargo &> /dev/null
then
    echo "Cargo was not found, installing Rust"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
fi

cargo install --force wheel-3bl

wheel