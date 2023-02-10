# Wheel

Compose DB Start Tool

## Getting Started
Run the following to get up and going

    curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/dbcfd/wheel/main/wheel.sh | bash

What this does
 1. Checks for curl, node, and jq
 1. Grabs the latest release of wheel from github
 1. Runs wheel, which will setup a ceramic and/or composedb environment