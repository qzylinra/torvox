# Termux-App Reference Review — Task Tracker

**Project:** [termux-app](https://github.com/termux/termux-app)
**Source:** `/tmp/reference-projects/termux-app`
**Generated:** 2026-06-25

---

## Summary

| Module | Source Files | Test Files | Total Lines |
|--------|-------------|-----------|-------------|
| `app` | 34 | 2 | 7,326 |
| `terminal-emulator` | 14 | 19 | 7,853 |
| `terminal-view` | 7 | 0 | 2,833 |
| `termux-shared` | 89 | 1 | 32,108 |
| **TOTAL** | **144** | **22** | **50,120** |

### Language Breakdown

| Language | Files | Total Lines |
|----------|-------|-------------|
| Java | 162 | 49,891 |
| C | 2 | 229 |
| Kotlin | 0 | 0 |
| Rust | 0 | 0 |

### Review Status

| Status | Count |
|--------|-------|
| NOT REVIEWED | 166 |
| IN PROGRESS | 0 |
| REVIEWED | 0 |

---

## Module: `app/`

Core Android application — activities, services, terminal integration, preferences UI.

### `app/src/main/cpp/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `app/src/main/cpp/termux-bootstrap.c` | 11 | NOT REVIEWED | JNI bootstrap — native init for terminal bootstrap |

### `app/src/main/java/com/termux/app/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 2 | `app/src/main/java/com/termux/app/RunCommandService.java` | 287 | NOT REVIEWED | Intent-driven service for running external commands |
| 3 | `app/src/main/java/com/termux/app/TermuxActivity.java` | 1,013 | NOT REVIEWED | Main terminal activity — lifecycle, session mgmt, UI |
| 4 | `app/src/main/java/com/termux/app/TermuxApplication.java` | 85 | NOT REVIEWED | Application subclass — init, crash handler |
| 5 | `app/src/main/java/com/termux/app/TermuxInstaller.java` | 386 | NOT REVIEWED | Bootstrap filesystem installer (APK assets → $PREFIX) |
| 6 | `app/src/main/java/com/termux/app/TermuxOpenReceiver.java` | 235 | NOT REVIEWED | BroadcastReceiver for `termux-open` intents |
| 7 | `app/src/main/java/com/termux/app/TermuxService.java` | 959 | NOT REVIEWED | Foreground service — session lifecycle, notifications |

### `app/src/main/java/com/termux/app/activities/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 8 | `app/src/main/java/com/termux/app/activities/HelpActivity.java` | 77 | NOT REVIEWED | Help/About screen |
| 9 | `app/src/main/java/com/termux/app/activities/SettingsActivity.java` | 169 | NOT REVIEWED | Settings host activity (fragment container) |

### `app/src/main/java/com/termux/app/api/file/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 10 | `app/src/main/java/com/termux/app/api/file/FileReceiverActivity.java` | 287 | NOT REVIEWED | Handles file open intents from other apps |

### `app/src/main/java/com/termux/app/event/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 11 | `app/src/main/java/com/termux/app/event/SystemEventReceiver.java` | 91 | NOT REVIEWED | Boot/system event receiver for auto-start |

### `app/src/main/java/com/termux/app/fragments/settings/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 12 | `app/src/main/java/com/termux/app/fragments/settings/TermuxAPIPreferencesFragment.java` | 49 | NOT REVIEWED | TermuxAPI plugin settings |
| 13 | `app/src/main/java/com/termux/app/fragments/settings/TermuxFloatPreferencesFragment.java` | 49 | NOT REVIEWED | TermuxFloat plugin settings |
| 14 | `app/src/main/java/com/termux/app/fragments/settings/TermuxPreferencesFragment.java` | 49 | NOT REVIEWED | Main Termux app settings |
| 15 | `app/src/main/java/com/termux/app/fragments/settings/TermuxTaskerPreferencesFragment.java` | 49 | NOT REVIEWED | TermuxTasker plugin settings |
| 16 | `app/src/main/java/com/termux/app/fragments/settings/TermuxWidgetPreferencesFragment.java` | 49 | NOT REVIEWED | TermuxWidget plugin settings |

### `app/src/main/java/com/termux/app/fragments/settings/termux/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 17 | `app/src/main/java/com/termux/app/fragments/settings/termux/DebuggingPreferencesFragment.java` | 155 | NOT REVIEWED | Debugging prefs — logs, crash reports |
| 18 | `app/src/main/java/com/termux/app/fragments/settings/termux/TerminalIOPreferencesFragment.java` | 82 | NOT REVIEWED | Terminal IO prefs — bell, extra keys |
| 19 | `app/src/main/java/com/termux/app/fragments/settings/termux/TerminalViewPreferencesFragment.java` | 77 | NOT REVIEWED | Terminal view prefs — font, colors |

### `app/src/main/java/com/termux/app/fragments/settings/termux_api/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 20 | `app/src/main/java/com/termux/app/fragments/settings/termux_api/DebuggingPreferencesFragment.java` | 101 | NOT REVIEWED | TermuxAPI debugging prefs |

### `app/src/main/java/com/termux/app/fragments/settings/termux_float/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 21 | `app/src/main/java/com/termux/app/fragments/settings/termux_float/DebuggingPreferencesFragment.java` | 126 | NOT REVIEWED | TermuxFloat debugging prefs |

### `app/src/main/java/com/termux/app/fragments/settings/termux_tasker/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 22 | `app/src/main/java/com/termux/app/fragments/settings/termux_tasker/DebuggingPreferencesFragment.java` | 101 | NOT REVIEWED | TermuxTasker debugging prefs |

### `app/src/main/java/com/termux/app/fragments/settings/termux_widget/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 23 | `app/src/main/java/com/termux/app/fragments/settings/termux_widget/DebuggingPreferencesFragment.java` | 101 | NOT REVIEWED | TermuxWidget debugging prefs |

### `app/src/main/java/com/termux/app/models/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 24 | `app/src/main/java/com/termux/app/models/UserAction.java` | 18 | NOT REVIEWED | User action model (context menu items) |

### `app/src/main/java/com/termux/app/terminal/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 25 | `app/src/main/java/com/termux/app/terminal/TermuxActivityRootView.java` | 284 | NOT REVIEWED | Root ViewGroup — key dispatch, back handling |
| 26 | `app/src/main/java/com/termux/app/terminal/TermuxSessionsListViewController.java` | 109 | NOT REVIEWED | Sessions list bottom sheet controller |
| 27 | `app/src/main/java/com/termux/app/terminal/TermuxTerminalSessionActivityClient.java` | 528 | NOT REVIEWED | Activity-side terminal session callbacks |
| 28 | `app/src/main/java/com/termux/app/terminal/TermuxTerminalSessionServiceClient.java` | 31 | NOT REVIEWED | Service-side terminal session callbacks |
| 29 | `app/src/main/java/com/termux/app/terminal/TermuxTerminalViewClient.java` | 802 | NOT REVIEWED | TerminalView callbacks — keyboard, clipboard, toast |

### `app/src/main/java/com/termux/app/terminal/io/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 30 | `app/src/main/java/com/termux/app/terminal/io/FullScreenWorkAround.java` | 68 | NOT REVIEWED | Soft keyboard fullscreen workaround |
| 31 | `app/src/main/java/com/termux/app/terminal/io/KeyboardShortcut.java` | 13 | NOT REVIEWED | Keyboard shortcut enum/constants |
| 32 | `app/src/main/java/com/termux/app/terminal/io/TerminalToolbarViewPager.java` | 117 | NOT REVIEWED | Toolbar ViewPager — extra keys, session list |
| 33 | `app/src/main/java/com/termux/app/terminal/io/TermuxTerminalExtraKeys.java` | 108 | NOT REVIEWED | Extra keys button row handler |

### `app/src/main/java/com/termux/filepicker/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 34 | `app/src/main/java/com/termux/filepicker/TermuxDocumentsProvider.java` | 268 | NOT REVIEWED | SAF DocumentsProvider for $HOME access |

### `app/src/test/java/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 35 | `app/src/test/java/com/termux/app/TermuxActivityTest.java` | 32 | NOT REVIEWED | TermuxActivity unit tests |
| 36 | `app/src/test/java/com/termux/app/api/file/FileReceiverActivityTest.java` | 36 | NOT REVIEWED | FileReceiverActivity unit tests |

---

## Module: `terminal-emulator/`

Pure-Java terminal emulator — VT100/xterm parsing, buffer management, character width.

### `terminal-emulator/src/main/java/com/termux/terminal/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 37 | `terminal-emulator/src/main/java/com/termux/terminal/ByteQueue.java` | 108 | NOT REVIEWED | Thread-safe byte queue (PTY ↔ emulator) |
| 38 | `terminal-emulator/src/main/java/com/termux/terminal/JNI.java` | 41 | NOT REVIEWED | JNI bridge for native terminal functions |
| 39 | `terminal-emulator/src/main/java/com/termux/terminal/KeyHandler.java` | 373 | NOT REVIEWED | Key event → escape sequence mapping |
| 40 | `terminal-emulator/src/main/java/com/termux/terminal/Logger.java` | 80 | NOT REVIEWED | Terminal logging utility |
| 41 | `terminal-emulator/src/main/java/com/termux/terminal/TerminalBuffer.java` | 497 | NOT REVIEWED | Screen buffer — lines, scrollback, cursor |
| 42 | `terminal-emulator/src/main/java/com/termux/terminal/TerminalColorScheme.java` | 126 | NOT REVIEWED | Color scheme definitions (16 ANSI + fg/bg) |
| 43 | `terminal-emulator/src/main/java/com/termux/terminal/TerminalColors.java` | 96 | NOT REVIEWED | RGB color storage for 256 palette |
| 44 | `terminal-emulator/src/main/java/com/termux/terminal/TerminalEmulator.java` | 2,617 | NOT REVIEWED | **Core VT parser/emulator** — CSI, OSC, DCS handling |
| 45 | `terminal-emulator/src/main/java/com/termux/terminal/TerminalOutput.java` | 32 | NOT REVIEWED | Output interface (write bytes/strings) |
| 46 | `terminal-emulator/src/main/java/com/termux/terminal/TerminalRow.java` | 283 | NOT REVIEWED | Single row — chars, styles, selection |
| 47 | `terminal-emulator/src/main/java/com/termux/terminal/TerminalSession.java` | 373 | NOT REVIEWED | Session lifecycle — process exec, I/O threads |
| 48 | `terminal-emulator/src/main/java/com/termux/terminal/TerminalSessionClient.java` | 51 | NOT REVIEWED | Session callbacks interface |
| 49 | `terminal-emulator/src/main/java/com/termux/terminal/TextStyle.java` | 90 | NOT REVIEWED | Style encoding (bold, italic, fg, bg, etc.) |
| 50 | `terminal-emulator/src/main/java/com/termux/terminal/WcWidth.java` | 566 | NOT REVIEWED | Unicode character width (East Asian Width) |

### `terminal-emulator/src/main/jni/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 51 | `terminal-emulator/src/main/jni/termux.c` | 218 | NOT REVIEWED | JNI native — process exec, PTY setup |

### `terminal-emulator/src/test/java/com/termux/terminal/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 52 | `terminal-emulator/src/test/java/com/termux/terminal/ApcTest.java` | 21 | NOT REVIEWED | APC sequence tests |
| 53 | `terminal-emulator/src/test/java/com/termux/terminal/ByteQueueTest.java` | 54 | NOT REVIEWED | ByteQueue unit tests |
| 54 | `terminal-emulator/src/test/java/com/termux/terminal/ControlSequenceIntroducerTest.java` | 131 | NOT REVIEWED | CSI sequence parsing tests |
| 55 | `terminal-emulator/src/test/java/com/termux/terminal/CursorAndScreenTest.java` | 266 | NOT REVIEWED | Cursor movement + screen operations |
| 56 | `terminal-emulator/src/test/java/com/termux/terminal/DecSetTest.java` | 78 | NOT REVIEWED | DECSET/DECRST mode tests |
| 57 | `terminal-emulator/src/test/java/com/termux/terminal/DeviceControlStringTest.java` | 53 | NOT REVIEWED | DCS sequence tests |
| 58 | `terminal-emulator/src/test/java/com/termux/terminal/HistoryTest.java` | 33 | NOT REVIEWED | Scrollback history tests |
| 59 | `terminal-emulator/src/test/java/com/termux/terminal/KeyHandlerTest.java` | 203 | NOT REVIEWED | Key mapping tests |
| 60 | `terminal-emulator/src/test/java/com/termux/terminal/OperatingSystemControlTest.java` | 196 | NOT REVIEWED | OSC sequence tests |
| 61 | `terminal-emulator/src/test/java/com/termux/terminal/RectangularAreasTest.java` | 117 | NOT REVIEWED | DECERA/DECFRA rectangular erase |
| 62 | `terminal-emulator/src/test/java/com/termux/terminal/ResizeTest.java` | 212 | NOT REVIEWED | Terminal resize behavior tests |
| 63 | `terminal-emulator/src/test/java/com/termux/terminal/ScreenBufferTest.java` | 65 | NOT REVIEWED | Primary/alternate buffer switching |
| 64 | `terminal-emulator/src/test/java/com/termux/terminal/ScrollRegionTest.java` | 167 | NOT REVIEWED | Scroll region (DECSTBM) tests |
| 65 | `terminal-emulator/src/test/java/com/termux/terminal/TerminalRowTest.java` | 432 | NOT REVIEWED | Row operations — insert, delete, erase |
| 66 | `terminal-emulator/src/test/java/com/termux/terminal/TerminalTest.java` | 350 | NOT REVIEWED | General terminal emulation tests |
| 67 | `terminal-emulator/src/test/java/com/termux/terminal/TerminalTestCase.java` | 319 | NOT REVIEWED | Base test class with helpers |
| 68 | `terminal-emulator/src/test/java/com/termux/terminal/TextStyleTest.java` | 65 | NOT REVIEWED | TextStyle encoding tests |
| 69 | `terminal-emulator/src/test/java/com/termux/terminal/UnicodeInputTest.java` | 136 | NOT REVIEWED | Unicode input handling tests |
| 70 | `terminal-emulator/src/test/java/com/termux/terminal/WcWidthTest.java` | 81 | NOT REVIEWED | Character width lookup tests |

---

## Module: `terminal-view/`

Android View layer for rendering terminal output on screen.

### `terminal-view/src/main/java/com/termux/view/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 71 | `terminal-view/src/main/java/com/termux/view/GestureAndScaleRecognizer.java` | 112 | NOT REVIEWED | Multi-touch gesture recognition |
| 72 | `terminal-view/src/main/java/com/termux/view/TerminalRenderer.java` | 249 | NOT REVIEWED | Canvas rendering — glyphs, cursor, selection |
| 73 | `terminal-view/src/main/java/com/termux/view/TerminalView.java` | 1,500 | NOT REVIEWED | **Main View** — SurfaceView, touch,IME, clipboard |
| 74 | `terminal-view/src/main/java/com/termux/view/TerminalViewClient.java` | 83 | NOT REVIEWED | View callbacks interface |

### `terminal-view/src/main/java/com/termux/view/support/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 75 | `terminal-view/src/main/java/com/termux/view/support/PopupWindowCompatGingerbread.java` | 75 | NOT REVIEWED | PopupWindow compat for older API levels |

### `terminal-view/src/main/java/com/termux/view/textselection/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 76 | `terminal-view/src/main/java/com/termux/view/textselection/CursorController.java` | 55 | NOT REVIEWED | Text selection cursor controller interface |
| 77 | `terminal-view/src/main/java/com/termux/view/textselection/TextSelectionCursorController.java` | 407 | NOT REVIEWED | Handles cursor drag for text selection |
| 78 | `terminal-view/src/main/java/com/termux/view/textselection/TextSelectionHandleView.java` | 352 | NOT REVIEWED | Selection handle handles (start/mid/end) |

---

## Module: `termux-shared/`

Shared library used by all Termux plugin apps — utilities, shell integration, settings, extra keys.

### `termux-shared/src/main/java/com/termux/shared/activities/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 79 | `termux-shared/src/main/java/com/termux/shared/activities/ReportActivity.java` | 476 | NOT REVIEWED | Crash/error report display activity |
| 80 | `termux-shared/src/main/java/com/termux/shared/activities/TextIOActivity.java` | 278 | NOT REVIEWED | Text input/output dialog activity |

### `termux-shared/src/main/java/com/termux/shared/activity/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 81 | `termux-shared/src/main/java/com/termux/shared/activity/ActivityErrno.java` | 20 | NOT REVIEWED | Activity error codes |
| 82 | `termux-shared/src/main/java/com/termux/shared/activity/ActivityUtils.java` | 137 | NOT REVIEWED | Activity lifecycle utilities |
| 83 | `termux-shared/src/main/java/com/termux/shared/activity/media/AppCompatActivityUtils.java` | 120 | NOT REVIEWED | AppCompat activity utilities |

### `termux-shared/src/main/java/com/termux/shared/android/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 84 | `termux-shared/src/main/java/com/termux/shared/android/AndroidUtils.java` | 270 | NOT REVIEWED | Android system utilities |
| 85 | `termux-shared/src/main/java/com/termux/shared/android/FeatureFlagUtils.java` | 169 | NOT REVIEWED | Feature flag management |
| 86 | `termux-shared/src/main/java/com/termux/shared/android/PackageUtils.java` | 830 | NOT REVIEWED | Package manager utilities (install, query, perms) |
| 87 | `termux-shared/src/main/java/com/termux/shared/android/PermissionUtils.java` | 573 | NOT REVIEWED | Runtime permission handling |
| 88 | `termux-shared/src/main/java/com/termux/shared/android/PhantomProcessUtils.java` | 115 | NOT REVIEWED | Phantom process killer detection/mitigation |
| 89 | `termux-shared/src/main/java/com/termux/shared/android/ProcessUtils.java` | 58 | NOT REVIEWED | Process utilities |
| 90 | `termux-shared/src/main/java/com/termux/shared/android/SELinuxUtils.java` | 96 | NOT REVIEWED | SELinux context utilities |
| 91 | `termux-shared/src/main/java/com/termux/shared/android/SettingsProviderUtils.java` | 99 | NOT REVIEWED | Settings.Secure provider access |
| 92 | `termux-shared/src/main/java/com/termux/shared/android/UserUtils.java` | 143 | NOT REVIEWED | Android user management |
| 93 | `termux-shared/src/main/java/com/termux/shared/android/resource/ResourceUtils.java` | 136 | NOT REVIEWED | Resource/dimen/attr lookup |

### `termux-shared/src/main/java/com/termux/shared/crash/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 94 | `termux-shared/src/main/java/com/termux/shared/crash/CrashHandler.java` | 158 | NOT REVIEWED | Uncaught exception handler + report |

### `termux-shared/src/main/java/com/termux/shared/data/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 95 | `termux-shared/src/main/java/com/termux/shared/data/DataUtils.java` | 258 | NOT REVIEWED | Data conversion utilities |
| 96 | `termux-shared/src/main/java/com/termux/shared/data/IntentUtils.java` | 166 | NOT REVIEWED | Intent construction/parsing |

### `termux-shared/src/main/java/com/termux/shared/errors/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 97 | `termux-shared/src/main/java/com/termux/shared/errors/Errno.java` | 118 | NOT REVIEWED | Base error code enum |
| 98 | `termux-shared/src/main/java/com/termux/shared/errors/Error.java` | 298 | NOT REVIEWED | Typed error wrapper |
| 99 | `termux-shared/src/main/java/com/termux/shared/errors/FunctionErrno.java` | 22 | NOT REVIEWED | Function-level error codes |

### `termux-shared/src/main/java/com/termux/shared/file/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 100 | `termux-shared/src/main/java/com/termux/shared/file/FileUtils.java` | 2,044 | NOT REVIEWED | **File operations** — copy, move, permissions, Symlinks |
| 101 | `termux-shared/src/main/java/com/termux/shared/file/FileUtilsErrno.java` | 111 | NOT REVIEWED | File operation error codes |

### `termux-shared/src/main/java/com/termux/shared/file/filesystem/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 102 | `termux-shared/src/main/java/com/termux/shared/file/filesystem/FileAttributes.java` | 418 | NOT REVIEWED | File attribute model (stat-like) |
| 103 | `termux-shared/src/main/java/com/termux/shared/file/filesystem/FileKey.java` | 68 | NOT REVIEWED | File identity key (inode/dev) |
| 104 | `termux-shared/src/main/java/com/termux/shared/file/filesystem/FilePermission.java` | 88 | NOT REVIEWED | Unix permission bits |
| 105 | `termux-shared/src/main/java/com/termux/shared/file/filesystem/FilePermissions.java` | 145 | NOT REVIEWED | Permission set operations |
| 106 | `termux-shared/src/main/java/com/termux/shared/file/filesystem/FileTime.java` | 156 | NOT REVIEWED | File timestamp model |
| 107 | `termux-shared/src/main/java/com/termux/shared/file/filesystem/FileType.java` | 32 | NOT REVIEWED | File type enum (regular, dir, symlink, etc.) |
| 108 | `termux-shared/src/main/java/com/termux/shared/file/filesystem/FileTypes.java` | 119 | NOT REVIEWED | File type utilities |
| 109 | `termux-shared/src/main/java/com/termux/shared/file/filesystem/NativeDispatcher.java` | 58 | NOT REVIEWED | JNI dispatch for filesystem ops |
| 110 | `termux-shared/src/main/java/com/termux/shared/file/filesystem/UnixConstants.java` | 158 | NOT REVIEWED | Unix errno/fcntl constants |

### `termux-shared/src/main/java/com/termux/shared/file/tests/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 111 | `termux-shared/src/main/java/com/termux/shared/file/tests/FileUtilsTests.java` | 396 | NOT REVIEWED | FileUtils test helpers |

### `termux-shared/src/main/java/com/termux/shared/interact/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 112 | `termux-shared/src/main/java/com/termux/shared/interact/MessageDialogUtils.java` | 99 | NOT REVIEWED | Dialog/message display utilities |
| 113 | `termux-shared/src/main/java/com/termux/shared/interact/ShareUtils.java` | 225 | NOT REVIEWED | Share intent utilities |

### `termux-shared/src/main/java/com/termux/shared/jni/models/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 114 | `termux-shared/src/main/java/com/termux/shared/jni/models/JniResult.java` | 109 | NOT REVIEWED | JNI call result model |

### `termux-shared/src/main/java/com/termux/shared/logger/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 115 | `termux-shared/src/main/java/com/termux/shared/logger/Logger.java` | 502 | NOT REVIEWED | Logging framework — levels, file output |

### `termux-shared/src/main/java/com/termux/shared/markdown/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 116 | `termux-shared/src/main/java/com/termux/shared/markdown/MarkdownUtils.java` | 207 | NOT REVIEWED | Markdown parsing for reports |

### `termux-shared/src/main/java/com/termux/shared/models/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 117 | `termux-shared/src/main/java/com/termux/shared/models/ReportInfo.java` | 119 | NOT REVIEWED | Report data model |
| 118 | `termux-shared/src/main/java/com/termux/shared/models/TextIOInfo.java` | 254 | NOT REVIEWED | Text I/O dialog config model |

### `termux-shared/src/main/java/com/termux/shared/net/socket/local/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 119 | `termux-shared/src/main/java/com/termux/shared/net/socket/local/ILocalSocketManager.java` | 72 | NOT REVIEWED | Socket manager interface |
| 120 | `termux-shared/src/main/java/com/termux/shared/net/socket/local/LocalClientSocket.java` | 483 | NOT REVIEWED | Local Unix socket client |
| 121 | `termux-shared/src/main/java/com/termux/shared/net/socket/local/LocalServerSocket.java` | 303 | NOT REVIEWED | Local Unix socket server |
| 122 | `termux-shared/src/main/java/com/termux/shared/net/socket/local/LocalSocketErrno.java` | 43 | NOT REVIEWED | Socket error codes |
| 123 | `termux-shared/src/main/java/com/termux/shared/net/socket/local/LocalSocketManager.java` | 450 | NOT REVIEWED | Socket lifecycle manager |
| 124 | `termux-shared/src/main/java/com/termux/shared/net/socket/local/LocalSocketManagerClientBase.java` | 47 | NOT REVIEWED | Client base class |
| 125 | `termux-shared/src/main/java/com/termux/shared/net/socket/local/LocalSocketRunConfig.java` | 265 | NOT REVIEWED | Socket run configuration |
| 126 | `termux-shared/src/main/java/com/termux/shared/net/socket/local/PeerCred.java` | 142 | NOT REVIEWED | Unix peer credentials (pid/uid/gid) |

### `termux-shared/src/main/java/com/termux/shared/net/uri/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 127 | `termux-shared/src/main/java/com/termux/shared/net/uri/UriScheme.java` | 28 | NOT REVIEWED | URI scheme constants |
| 128 | `termux-shared/src/main/java/com/termux/shared/net/uri/UriUtils.java` | 102 | NOT REVIEWED | URI parsing utilities |

### `termux-shared/src/main/java/com/termux/shared/net/url/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 129 | `termux-shared/src/main/java/com/termux/shared/net/url/UrlUtils.java` | 113 | NOT REVIEWED | URL validation/parsing |

### `termux-shared/src/main/java/com/termux/shared/notification/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 130 | `termux-shared/src/main/java/com/termux/shared/notification/NotificationUtils.java` | 148 | NOT REVIEWED | Notification channel/build utilities |

### `termux-shared/src/main/java/com/termux/shared/reflection/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 131 | `termux-shared/src/main/java/com/termux/shared/reflection/ReflectionUtils.java` | 282 | NOT REVIEWED | Reflective method/field access |

### `termux-shared/src/main/java/com/termux/shared/settings/preferences/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 132 | `termux-shared/src/main/java/com/termux/shared/settings/preferences/AppSharedPreferences.java` | 49 | NOT REVIEWED | SharedPreferences base wrapper |
| 133 | `termux-shared/src/main/java/com/termux/shared/settings/preferences/SharedPreferenceUtils.java` | 432 | NOT REVIEWED | SharedPrefs read/write/listener utilities |

### `termux-shared/src/main/java/com/termux/shared/settings/properties/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 134 | `termux-shared/src/main/java/com/termux/shared/settings/properties/SharedProperties.java` | 645 | NOT REVIEWED | properties file parser/loader |
| 135 | `termux-shared/src/main/java/com/termux/shared/settings/properties/SharedPropertiesParser.java` | 37 | NOT REVIEWED | Low-level properties parser |

### `termux-shared/src/main/java/com/termux/shared/shell/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 136 | `termux-shared/src/main/java/com/termux/shared/shell/ArgumentTokenizer.java` | 229 | NOT REVIEWED | Shell argument tokenization |
| 137 | `termux-shared/src/main/java/com/termux/shared/shell/ShellUtils.java` | 76 | NOT REVIEWED | Shell path/exec utilities |
| 138 | `termux-shared/src/main/java/com/termux/shared/shell/StreamGobbler.java` | 325 | NOT REVIEWED | Stream reader thread (stdout/stderr) |

### `termux-shared/src/main/java/com/termux/shared/shell/am/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 139 | `termux-shared/src/main/java/com/termux/shared/shell/am/AmSocketServer.java` | 258 | NOT REVIEWED | Activity Manager socket server |
| 140 | `termux-shared/src/main/java/com/termux/shared/shell/am/AmSocketServerErrno.java` | 18 | NOT REVIEWED | AM socket error codes |
| 141 | `termux-shared/src/main/java/com/termux/shared/shell/am/AmSocketServerRunConfig.java` | 108 | NOT REVIEWED | AM socket run configuration |

### `termux-shared/src/main/java/com/termux/shared/shell/command/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 142 | `termux-shared/src/main/java/com/termux/shared/shell/command/ExecutionCommand.java` | 691 | NOT REVIEWED | Shell command execution model |
| 143 | `termux-shared/src/main/java/com/termux/shared/shell/command/ShellCommandConstants.java` | 75 | NOT REVIEWED | Shell command constants |

### `termux-shared/src/main/java/com/termux/shared/shell/command/environment/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 144 | `termux-shared/src/main/java/com/termux/shared/shell/command/environment/AndroidShellEnvironment.java` | 100 | NOT REVIEWED | Android-specific shell env |
| 145 | `termux-shared/src/main/java/com/termux/shared/shell/command/environment/IShellEnvironment.java` | 52 | NOT REVIEWED | Shell environment interface |
| 146 | `termux-shared/src/main/java/com/termux/shared/shell/command/environment/ShellCommandShellEnvironment.java` | 62 | NOT REVIEWED | Command-specific env overrides |
| 147 | `termux-shared/src/main/java/com/termux/shared/shell/command/environment/ShellEnvironmentUtils.java` | 180 | NOT REVIEWED | Env variable utilities |
| 148 | `termux-shared/src/main/java/com/termux/shared/shell/command/environment/ShellEnvironmentVariable.java` | 28 | NOT REVIEWED | Env variable model |
| 149 | `termux-shared/src/main/java/com/termux/shared/shell/command/environment/UnixShellEnvironment.java` | 83 | NOT REVIEWED | Unix shell env defaults |

### `termux-shared/src/main/java/com/termux/shared/shell/command/result/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 150 | `termux-shared/src/main/java/com/termux/shared/shell/command/result/ResultConfig.java` | 170 | NOT REVIEWED | Result reporting configuration |
| 151 | `termux-shared/src/main/java/com/termux/shared/shell/command/result/ResultData.java` | 258 | NOT REVIEWED | Command result data model |
| 152 | `termux-shared/src/main/java/com/termux/shared/shell/command/result/ResultSender.java` | 349 | NOT REVIEWED | Result sender (socket/broadcast) |
| 153 | `termux-shared/src/main/java/com/termux/shared/shell/command/result/ResultSenderErrno.java` | 22 | NOT REVIEWED | Result sender error codes |

### `termux-shared/src/main/java/com/termux/shared/shell/command/runner/app/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 154 | `termux-shared/src/main/java/com/termux/shared/shell/command/runner/app/AppShell.java` | 349 | NOT REVIEWED | App shell command runner |

### `termux-shared/src/main/java/com/termux/shared/termux/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 155 | `termux-shared/src/main/java/com/termux/shared/termux/TermuxBootstrap.java` | 219 | NOT REVIEWED | Bootstrap type detection (apt/distro) |
| 156 | `termux-shared/src/main/java/com/termux/shared/termux/TermuxConstants.java` | 1,338 | NOT REVIEWED | **Core constants** — paths, packages, intents |
| 157 | `termux-shared/src/main/java/com/termux/shared/termux/TermuxUtils.java` | 730 | NOT REVIEWED | Termux-wide utilities |

### `termux-shared/src/main/java/com/termux/shared/termux/crash/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 158 | `termux-shared/src/main/java/com/termux/shared/termux/crash/TermuxCrashUtils.java` | 411 | NOT REVIEWED | Termux crash reporting |

### `termux-shared/src/main/java/com/termux/shared/termux/data/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 159 | `termux-shared/src/main/java/com/termux/shared/termux/data/TermuxUrlUtils.java` | 104 | NOT REVIEWED | URL intent handling |

### `termux-shared/src/main/java/com/termux/shared/termux/extrakeys/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 160 | `termux-shared/src/main/java/com/termux/shared/termux/extrakeys/ExtraKeyButton.java` | 151 | NOT REVIEWED | Extra key button model |
| 161 | `termux-shared/src/main/java/com/termux/shared/termux/extrakeys/ExtraKeysConstants.java` | 212 | NOT REVIEWED | Extra key constants/labels |
| 162 | `termux-shared/src/main/java/com/termux/shared/termux/extrakeys/ExtraKeysInfo.java` | 213 | NOT REVIEWED | Extra keys layout parser |
| 163 | `termux-shared/src/main/java/com/termux/shared/termux/extrakeys/ExtraKeysView.java` | 681 | NOT REVIEWED | Extra keys rendering + touch handling |
| 164 | `termux-shared/src/main/java/com/termux/shared/termux/extrakeys/SpecialButton.java` | 52 | NOT REVIEWED | Special button type enum |
| 165 | `termux-shared/src/main/java/com/termux/shared/termux/extrakeys/SpecialButtonState.java` | 51 | NOT REVIEWED | Special button toggle state |

### `termux-shared/src/main/java/com/termux/shared/termux/file/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 166 | `termux-shared/src/main/java/com/termux/shared/termux/file/TermuxFileUtils.java` | 414 | NOT REVIEWED | Termux file path utilities |

### `termux-shared/src/main/java/com/termux/shared/termux/interact/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 167 | `termux-shared/src/main/java/com/termux/shared/termux/interact/TextInputDialogUtils.java` | 72 | NOT REVIEWED | Text input dialog builder |

### `termux-shared/src/main/java/com/termux/shared/termux/models/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 168 | `termux-shared/src/main/java/com/termux/shared/termux/models/UserAction.java` | 18 | NOT REVIEWED | Shared user action model |

### `termux-shared/src/main/java/com/termux/shared/termux/notification/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 169 | `termux-shared/src/main/java/com/termux/shared/termux/notification/TermuxNotificationUtils.java` | 109 | NOT REVIEWED | Termux notification builders |

### `termux-shared/src/main/java/com/termux/shared/termux/plugins/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 170 | `termux-shared/src/main/java/com/termux/shared/termux/plugins/TermuxPluginUtils.java` | 469 | NOT REVIEWED | Plugin lifecycle utilities |

### `termux-shared/src/main/java/com/termux/shared/termux/settings/preferences/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 171 | `termux-shared/src/main/java/com/termux/shared/termux/settings/preferences/TermuxAPIAppSharedPreferences.java` | 84 | NOT REVIEWED | TermuxAPI preferences |
| 172 | `termux-shared/src/main/java/com/termux/shared/termux/settings/preferences/TermuxAppSharedPreferences.java` | 261 | NOT REVIEWED | Main app shared preferences |
| 173 | `termux-shared/src/main/java/com/termux/shared/termux/settings/preferences/TermuxBootAppSharedPreferences.java` | 75 | NOT REVIEWED | TermuxBoot preferences |
| 174 | `termux-shared/src/main/java/com/termux/shared/termux/settings/preferences/TermuxFloatAppSharedPreferences.java` | 161 | NOT REVIEWED | TermuxFloat preferences |
| 175 | `termux-shared/src/main/java/com/termux/shared/termux/settings/preferences/TermuxPreferenceConstants.java` | 319 | NOT REVIEWED | Preference key constants |
| 176 | `termux-shared/src/main/java/com/termux/shared/termux/settings/preferences/TermuxStylingAppSharedPreferences.java` | 75 | NOT REVIEWED | TermuxStyling preferences |
| 177 | `termux-shared/src/main/java/com/termux/shared/termux/settings/preferences/TermuxTaskerAppSharedPreferences.java` | 85 | NOT REVIEWED | TermuxTasker preferences |
| 178 | `termux-shared/src/main/java/com/termux/shared/termux/settings/preferences/TermuxWidgetAppSharedPreferences.java` | 94 | NOT REVIEWED | TermuxWidget preferences |

### `termux-shared/src/main/java/com/termux/shared/termux/settings/properties/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 179 | `termux-shared/src/main/java/com/termux/shared/termux/settings/properties/TermuxAppSharedProperties.java` | 42 | NOT REVIEWED | App shared properties |
| 180 | `termux-shared/src/main/java/com/termux/shared/termux/settings/properties/TermuxPropertyConstants.java` | 481 | NOT REVIEWED | Property key constants |
| 181 | `termux-shared/src/main/java/com/termux/shared/termux/settings/properties/TermuxSharedProperties.java` | 721 | NOT REVIEWED | Termux properties loader |

### `termux-shared/src/main/java/com/termux/shared/termux/shell/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 182 | `termux-shared/src/main/java/com/termux/shared/termux/shell/TermuxShellManager.java` | 123 | NOT REVIEWED | Shell process manager |
| 183 | `termux-shared/src/main/java/com/termux/shared/termux/shell/TermuxShellUtils.java` | 122 | NOT REVIEWED | Shell utility functions |

### `termux-shared/src/main/java/com/termux/shared/termux/shell/am/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 184 | `termux-shared/src/main/java/com/termux/shared/termux/shell/am/TermuxAmSocketServer.java` | 232 | NOT REVIEWED | Termux-specific AM socket server |

### `termux-shared/src/main/java/com/termux/shared/termux/shell/command/environment/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 185 | `termux-shared/src/main/java/com/termux/shared/termux/shell/command/environment/TermuxAPIShellEnvironment.java` | 43 | NOT REVIEWED | TermuxAPI env setup |
| 186 | `termux-shared/src/main/java/com/termux/shared/termux/shell/command/environment/TermuxAppShellEnvironment.java` | 172 | NOT REVIEWED | Termux app env setup |
| 187 | `termux-shared/src/main/java/com/termux/shared/termux/shell/command/environment/TermuxShellCommandShellEnvironment.java` | 48 | NOT REVIEWED | Shell command env setup |
| 188 | `termux-shared/src/main/java/com/termux/shared/termux/shell/command/environment/TermuxShellEnvironment.java` | 117 | NOT REVIEWED | Base Termux shell environment |

### `termux-shared/src/main/java/com/termux/shared/termux/shell/command/runner/terminal/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 189 | `termux-shared/src/main/java/com/termux/shared/termux/shell/command/runner/terminal/TermuxSession.java` | 296 | NOT REVIEWED | Terminal session runner |

### `termux-shared/src/main/java/com/termux/shared/termux/terminal/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 190 | `termux-shared/src/main/java/com/termux/shared/termux/terminal/TermuxTerminalSessionClientBase.java` | 94 | NOT REVIEWED | Base terminal session callbacks |
| 191 | `termux-shared/src/main/java/com/termux/shared/termux/terminal/TermuxTerminalViewClientBase.java` | 127 | NOT REVIEWED | Base terminal view callbacks |

### `termux-shared/src/main/java/com/termux/shared/termux/terminal/io/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 192 | `termux-shared/src/main/java/com/termux/shared/termux/terminal/io/BellHandler.java` | 79 | NOT REVIEWED | Terminal bell (audible/vibrate) handler |
| 193 | `termux-shared/src/main/java/com/termux/shared/termux/terminal/io/TerminalExtraKeys.java` | 85 | NOT REVIEWED | Terminal extra keys base |

### `termux-shared/src/main/java/com/termux/shared/termux/theme/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 194 | `termux-shared/src/main/java/com/termux/shared/termux/theme/TermuxThemeUtils.java` | 25 | NOT REVIEWED | Theme utilities |

### `termux-shared/src/main/java/com/termux/shared/theme/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 195 | `termux-shared/src/main/java/com/termux/shared/theme/NightMode.java` | 91 | NOT REVIEWED | Night mode enum/handling |
| 196 | `termux-shared/src/main/java/com/termux/shared/theme/ThemeUtils.java` | 86 | NOT REVIEWED | Theme resource utilities |

### `termux-shared/src/main/java/com/termux/shared/view/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 197 | `termux-shared/src/main/java/com/termux/shared/view/KeyboardUtils.java` | 198 | NOT REVIEWED | Soft keyboard show/hide |
| 198 | `termux-shared/src/main/java/com/termux/shared/view/ViewUtils.java` | 246 | NOT REVIEWED | View utilities (dimensions, insets) |

### `termux-shared/src/androidTest/java/`

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 199 | `termux-shared/src/androidTest/java/com/termux/shared/ExampleInstrumentedTest.java` | 26 | NOT REVIEWED | Placeholder instrumented test |

---

## Top 10 Largest Files

| # | File | Lines |
|---|------|-------|
| 1 | `terminal-emulator/.../TerminalEmulator.java` | 2,617 |
| 2 | `termux-shared/.../FileUtils.java` | 2,044 |
| 3 | `terminal-view/.../TerminalView.java` | 1,500 |
| 4 | `termux-shared/.../TermuxConstants.java` | 1,338 |
| 5 | `app/.../TermuxActivity.java` | 1,013 |
| 6 | `app/.../TermuxService.java` | 959 |
| 7 | `termux-shared/.../PackageUtils.java` | 830 |
| 8 | `app/.../TermuxTerminalViewClient.java` | 802 |
| 9 | `termux-shared/.../TermuxUtils.java` | 730 |
| 10 | `termux-shared/.../TermuxSharedProperties.java` | 721 |

---

## Review Checklist

For each file reviewed, evaluate:

- [ ] **Architecture**: How does this fit in the module hierarchy?
- [ ] **Public API**: What interfaces/methods are exposed?
- [ ] **State management**: How is mutable state handled?
- [ ] **Concurrency**: Thread safety, synchronization, race conditions?
- [ ] **Error handling**: Exception propagation, error recovery?
- [ ] **Platform coupling**: Android API usage, JNI boundaries?
- [ ] **Security**: Permission checks, input validation, injection?
- [ ] **Performance**: Allocation hotspots, O(n²) loops, IPC overhead?
- [ ] **Testing**: Coverage, testability, mock friendliness?
- [ ] **Comparison to torvox**: What patterns can we adopt/avoid?
