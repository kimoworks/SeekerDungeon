# AI Android Build Guide

How the automated Android build pipeline works, and rules for AI assistants using it.

## Overview

The project has two components that enable headless Android builds:

| Component | Path | Purpose |
|-----------|------|---------|
| **Editor script** | `Assets/Editor/AndroidBuilder.cs` | Unity `BuildPipeline` wrapper, callable via `-executeMethod` |
| **PowerShell launcher** | `scripts/android/build-and-run.ps1` | Finds Unity, handles env fixes, invokes batch mode, captures logs |

## How It Works

1. The PS1 script reads `ProjectSettings/ProjectVersion.txt` to auto-detect the correct Unity editor version.
2. It launches Unity in **batch mode** (`-batchmode -quit`), which recompiles scripts automatically and runs the build.
3. Unity must **not** already have the project open. Batch mode and the GUI editor cannot use the same project simultaneously.
4. After a successful build, the APK is deployed to the connected device via `adb` (unless `buildonly` mode is used).
5. Optionally, the script clears `logcat` and captures session-related logs.

## Usage

Run from the repo root in PowerShell:

```powershell
# Full build and run (development), with log capture
.\scripts\android\build-and-run.ps1 -Logs

# Fast incremental patch and run
.\scripts\android\build-and-run.ps1 -Mode patch -Logs

# Build APK only, don't deploy
.\scripts\android\build-and-run.ps1 -Mode buildonly
```

The script also fixes a known issue where the Cursor sandbox overrides `GRADLE_USER_HOME` to an extremely long temp path, which breaks the Gradle build step on Windows.

## Rules for AI Assistants

**Always ask the developer for permission before:**

- Closing or killing the Unity Editor. The developer may have unsaved scene/prefab work, inspector tweaks, or be mid-test. Never close Unity without explicit approval.
- Running a build. Even though the build itself is non-destructive, it requires Unity to not be open with the project, which means the developer loses their current editor state.

**Safe to do without asking:**

- Editing source files (`.cs`, `.uxml`, `.uss`, etc.). Unity will hot-reload these when the developer refocuses the editor.
- Reading logs, running `adb` commands, checking device state.
- Preparing build commands and showing them to the developer to run manually.

**Recommended workflow:**

1. Make all code/asset changes.
2. Tell the developer what changed and ask: "Want me to trigger a build, or do you want to build from the editor?"
3. If they approve an automated build, confirm Unity is closed (or ask to close it), then run the script.
4. After the build, capture logs if needed and report results.

## Unity Menu Items

The `AndroidBuilder.cs` script also registers menu items in the Unity Editor for manual builds:

- **Build > Android - Build and Run (Dev)**
- **Build > Android - Patch and Run (Dev)**
- **Build > Android - Build Only (Dev)**

These are useful when the developer prefers to trigger builds from inside Unity instead of the command line.
