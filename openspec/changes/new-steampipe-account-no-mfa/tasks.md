# Tasks

All of these are manual steps for the user, outside the codebase.

## 1. Account

- [ ] 1.1 Create a new Gmail address for the build account (the old `lithicrivers@gmail.com` credentials are lost); user check: you can sign in to it
- [ ] 1.2 Recover the old `lithicrivers` Steam account or create a new one on that address, with email Steam Guard (no mobile authenticator); user check: you can sign in at store.steampowered.com
- [ ] 1.3 In Steamworks (Users & Permissions), add the account to the partner group and give it build upload and publishing rights for app 3940720; user check: the account sees the app's SteamPipe Builds page

## 2. Secrets

- [ ] 2.1 Quit the desktop Steam client, then `steamcmd +login <build account> +quit` and approve the email code; user check: running the same command again logs in without asking
- [ ] 2.2 Set `STEAM_USERNAME` and `STEAM_PASSWORD` to the build account's; user check: `gh secret list` shows both updated
- [ ] 2.3 Set `STEAM_CONFIG_VDF` to `config.vdf` without its cache blocks (under 48 KB): `python3 -I scripts/steam/trim_steam_config.py ~/.local/share/Steam/config/config.vdf --sizes` to see them, then `--drop WebStorage,depots,ShaderCacheManager | gh secret set STEAM_CONFIG_VDF`; user check: `gh secret list` shows it updated

## 3. Upload

- [ ] 3.1 `gh run rerun 39716520594 --failed`; verify the "Upload to Steam (SteamPipe)" job passes and the build shows on the `public` branch in Steamworks
- [ ] 3.2 If the job says `Cached credentials not found` with the right username, or `License expired` soon after, propose moving the Steam job to a self-hosted runner or generating Steam Guard codes in CI
