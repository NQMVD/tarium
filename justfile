set windows-powershell := true

default:
    just --list

install:
  cargo install --force --path .

build-dev:
  $env:TARIUM_EMBED_CREDENTIALS = "1"; $env:TARIUM_EMBED_GITHUB_APP_ID = "1910665"; $env:TARIUM_EMBED_GITHUB_INSTALLATION_ID = "84660496"; $env:TARIUM_EMBED_GITHUB_PRIVATE_KEY_PATH = "C:\Users\Noah\Documents\tarium-cli.2025-09-06.private-key.pem"; cargo build

build:
  $env:TARIUM_EMBED_CREDENTIALS = "1"; $env:TARIUM_EMBED_GITHUB_APP_ID = "1910665"; $env:TARIUM_EMBED_GITHUB_INSTALLATION_ID = "84660496"; $env:TARIUM_EMBED_GITHUB_PRIVATE_KEY_PATH = "C:\Users\Noah\Documents\tarium-cli.2025-09-06.private-key.pem"; cargo build --release
