# Proposal

## Why

The release workflow's Steam upload fails. It logs in with steamcmd as the account in the `STEAM_USERNAME` secret, which is a dedicated build account (`lithicrivers`, on `lithicrivers@gmail.com`), not the personal `henryfbp1` account. The credentials for that build account are lost, so neither the cached login in `STEAM_CONFIG_VDF` nor `STEAM_PASSWORD` works.

A dedicated build account is the right setup: uploads don't send Steam Guard prompts to the personal account, and the build account only gets the permissions SteamPipe needs.

## Status

**Blocked, waiting on Google (since 2026-10-22).** `lithicrivers@gmail.com` is under a Google account appeal. Nothing here can start until Google answers: if the appeal restores the address, the old Steam account can be recovered through it; if not, a new Gmail address and a new Steam account are needed.

## What Changes

A manual change, done by the user outside the codebase:

- Create a new Gmail address for the build account (the old `lithicrivers@gmail.com` credentials are lost).
- Create a new Steam account on it, or recover the old one, and give it SteamPipe upload rights for app 3940720 in Steamworks.
- Log in once with steamcmd so it caches the login, and store the cached login (`config.vdf`, trimmed under 48 KB) plus the username and password as GitHub secrets.
- Re-run the failed upload for v0.10.0.0.

The workflow itself stays as it is, unless a step below shows it needs changing (for example, if Steam no longer accepts a cached login from another machine).

## Capabilities

### New Capabilities
- `steam-release-upload`: how the release workflow logs in to Steam and uploads a build, and what the build account needs.

### Modified Capabilities
<!-- None. -->

## Impact

- **GitHub secrets:** `STEAM_USERNAME`, `STEAM_PASSWORD`, `STEAM_CONFIG_VDF` replaced. `STEAM_GUARD_CODE` is unused (its step is commented out).
- **Steamworks:** a new user with build permissions on the LithicRivers app.
- **Code:** none expected.
