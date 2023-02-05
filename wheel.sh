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

REPO=dbcfd/wheel/releases/latest
ARCH=$(uname -m)
OS=unknown-linux-gnu
if [[ $OSTYPE == 'darwin'* ]]
then
  OS=apple-darwin
fi
TARGET=$ARCH-$OS

echo "Downloading wheel for arch "TARGET

VERSION=$(curl https://api.github.com/repos/$REPO -s |  jq .name -r)
TAR_NAME=wheel-$VERSION-$TARGET
OUTPUT_FILE=wheel.tar.xz

curl -L0 https://github.com/$REPO/download/$TAR_NAME.tar.xz --output $OUTPUT_FILE

tar -xvf $OUTPUT_FILE
rm $OUTPUT_FILE

mv ./$TAR_NAME ./wheel

cd wheel

./wheel "$@"