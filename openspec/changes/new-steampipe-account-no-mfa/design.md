# Design

## Context

- `.github/workflows/release.yml`, job "Upload to Steam (SteamPipe)", runs on tags:
  1. installs `steamcmd` with apt
  2. writes the `STEAM_CONFIG_VDF` secret to `~/.local/share/Steam/config/config.vdf`
  3. runs `steamcmd +login $STEAM_USERNAME +info +quit`, then `+run_app_build_http app_build_unified.vdf`
- `app_build_unified.vdf` uploads to app 3940720 and sets the build live on `public`.
- The v0.10.0.0 run (39716520594) failed: first `License expired` (the old cached login had expired), then `Cached credentials not found` after the secret was replaced with `henryfbp1`'s login, because `STEAM_USERNAME` names the build account.
- GitHub secrets hold at most 48 KB. A full desktop `config.vdf` is about 66 KB. Dropping the `WebStorage`, `depots` and `ShaderCacheManager` cache blocks brings it to about 15 KB.

## Decisions

### A separate build account, without a mobile authenticator
- Uploads log in as the build account, never as `henryfbp1`, so CI never prompts the personal account.
- The account uses email Steam Guard on its own Gmail address, so a new login can be approved without the personal phone.

### Cached login, not a password in CI
- steamcmd's cached login (`config.vdf`) lets CI log in without a Steam Guard code. The password secret is only a fallback.
- Log in on a machine where the desktop Steam client isn't signed in to another account, or quit the client first, so `config.vdf` holds only the build account's login.

## Risks / Trade-offs

- [The cached login expires again (`License expired`)] → Repeat the steamcmd login and secret update. If it happens often, run the Steam job on a self-hosted runner, where the login persists, or generate Steam Guard codes in CI.
- [Steam rejects a cached login from another machine (`Cached credentials not found` with the right username)] → The same two fallbacks.
- [Setting builds live on `public` may need extra permissions] → Check the build account's Steamworks permissions cover publishing; otherwise drop `SetLive` and set builds live in Steamworks by hand.
