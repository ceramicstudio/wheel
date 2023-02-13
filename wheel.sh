#!/usr/bin/env bash

if ! command -v curl &> /dev/null
then
    echo "curl was not found, please install curl"
    exit
fi

if ! command -v jq &> /dev/null
then
    echo "jq was not found, please install jq"
    exit
fi

if ! command -v npx &> /dev/null
then
    echo "npx was not found, please install node.js"
    exit
fi

REPO=3box/wheel/releases
ARCH=$(uname -m)
if [[ $ARCH == 'arm64' ]]
then
  ARCH=aarch64
fi
OS=unknown-linux-gnu
if [[ $OSTYPE == 'darwin'* ]]
then
  OS=apple-darwin
fi
TARGET=$ARCH-$OS

VERSION=$(curl https://api.github.com/repos/$REPO/latest -s |  jq .name -r)
TAR_NAME=wheel_$TARGET.tar.gz
OUTPUT_FILE=wheel.tar.gz
DOWNLOAD_URL=https://github.com/$REPO/download/$VERSION/$TAR_NAME

echo "Downloading wheel for target "$TARGET" from "$DOWNLOAD_URL

curl -LJ0 --output $OUTPUT_FILE $DOWNLOAD_URL

tar -xvf $OUTPUT_FILE
rm $OUTPUT_FILE

./wheel "$@"
