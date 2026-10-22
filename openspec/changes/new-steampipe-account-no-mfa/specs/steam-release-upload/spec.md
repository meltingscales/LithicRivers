# Spec Delta

## Purpose

How a tagged release reaches Steam: the release workflow logs in to Steam as a dedicated build account and uploads every platform's build.

## ADDED Requirements

### Requirement: Dedicated build account
The release workflow SHALL log in to Steam as a dedicated build account with SteamPipe upload rights for the LithicRivers app, and SHALL NOT use the developer's personal Steam account.

#### Scenario: Uploading a release
- **WHEN** a `v*` tag is pushed and the builds succeed
- **THEN** the Steam job logs in as the build account named in `STEAM_USERNAME` and uploads the Linux, Windows and macOS builds

### Requirement: Login without prompts
The Steam job SHALL log in from a cached login stored in the `STEAM_CONFIG_VDF` secret, so it needs no Steam Guard code and sends no prompt to any phone.

#### Scenario: Cached login works
- **WHEN** `STEAM_CONFIG_VDF` holds a current cached login for the account in `STEAM_USERNAME`
- **THEN** `steamcmd +login $STEAM_USERNAME` logs in without asking for a password or code
