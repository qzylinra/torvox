# Torvox Maestro E2E Tests

[Maestro](https://maestro.mobile.dev/) is a mobile E2E testing framework that
uses declarative YAML flow files. These flows exercise the torvox Android
terminal emulator through real UI interactions.

## Requirements

- Android emulator (API 33+) or physical device running a debug build of torvox
  (`com.termux` package name)
- `maestro` CLI installed (see below)
- `adb` debugging enabled and device/emulator connected (`adb devices`)

## Install Maestro

```bash
curl -Ls "https://get.maestro.mobile.dev" | bash
```

Or via Homebrew:

```bash
brew install maestro
```

Verify the installation:

```bash
maestro --version
```

## Build and Install the App

From the project root:

```bash
cd android
./gradlew assembleDebug
adb install -r app/build/outputs/apk/debug/app-debug.apk
```

## Running the Flows

Run all flows:

```bash
maestro test android/maestro/flows/
```

Run a specific flow:

```bash
maestro test android/maestro/flows/terminal-basic.yaml
```

Run a flow with verbose output:

```bash
maestro test android/maestro/flows/terminal-basic.yaml --format junit
```

## Continuous Integration

In CI, start an emulator first, wait for boot, then run Maestro flows.
Example snippet for a GitHub Actions workflow step:

```yaml
- name: Run Maestro E2E tests
  run: |
    maestro test android/maestro/flows/ \
      --env APP_ID=com.termux \
      --format junit \
      --output report.xml
```

## Troubleshooting

- **App not found**: Ensure the app is installed with `adb install` before
  running Maestro.
- **Element not visible**: Increase the `timeout` in
  `extendedWaitUntil` or check that the terminal finished bootstrapping.
- **Swipe not scrolling**: The terminal content area may need more output
  generated first. Add additional `echo` commands if needed.

## Notes

- There is also a root-level `maestro/` directory with additional flow files.
  The flows in `android/maestro/` are designed as minimal standalone templates.
- The `com.termux` package name is intentional (see AGENTS.md under Known
  Pitfalls #16).
