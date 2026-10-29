# Tasks

Build a Windows release (`just build-windows`, or the zip from the release workflow) and run it on a real Windows machine, not only a VM if you can: the fast-keystroke bug was seen in a VM. Write down the Windows version and display scaling for each run.

## 1. Launching on Windows

- [ ] 1.1 User check: starting the game (from Steam, the zip, or `just client-windows`) opens only the game window: no console window behind it
- [ ] 1.2 User check: from PowerShell or cmd, `lithicrivers-client --terminal` runs in that console as before, and `just client-blind scripts/agent/smoke.txt` prints its screens and state (the console is attached)
- [ ] 1.3 User check: closing the game window exits the game, and the next launch remembers text size, fullscreen and Display mode

## 2. Input on Windows

- [ ] 2.1 User check: keystrokes are no longer consumed too fast: one press moves one tile, holding a direction repeats at a normal rate, and typing a world name or a search shows each character once
- [ ] 2.2 User check: numpad movement works with NumLock on, and Shift combinations (Shift+Tab, uppercase letters) behave as on Linux
- [ ] 2.3 User check: at 100% and at 150% Windows display scaling, mouse clicks, scrolling and hover land on the cell under the pointer (world tiles, menu rows, Options rows)

## 3. The window and the Display panel on Windows

- [ ] 3.1 User check: docked (the default), floating and hidden all work from Options; the docked strip's buttons respond, and clicks beside it reach the game
- [ ] 3.2 User check: A- and A+ change the text size with no leftover text, and the grid refits after resizing the window
- [ ] 3.3 User check: fullscreen turns on and off from the Display panel and from Options; a floating Display window stays above the fullscreen game
- [ ] 3.4 User check (optional): where no GPU is available (for example Windows Sandbox or Remote Desktop without a GPU), starting the game shows a message box explaining `--terminal` instead of failing silently

## 4. Steam Deck

- [ ] 4.1 User check: in Gaming Mode, the game starts from Steam, fills the screen, and is playable; text size can be changed from Options (the floating Display window may not appear under gamescope, which is expected)
- [ ] 4.2 User check: in Desktop Mode, the window and all three Display modes work as on Linux

## Workflow follow-up

- Each failed check: open a fix branch or change and note it next to the task.
- When all pass: archive this change and tick Steam Deck compatibility in `docs/STEAM-MVP.md`.
