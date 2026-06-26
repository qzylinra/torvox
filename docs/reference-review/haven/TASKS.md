# Haven Project — Source Review Tasks

Generated reference review for the Haven project.

## Summary

| Metric | Value |
|--------|-------|
| Total Files | 595 |
| Total Lines | 166,291 |
| Languages | Kotlin, Rust, C |

### Files by Language

| Language | Files | Lines |
|----------|-------|-------|
| Kotlin | 562 | 151,377 |
| Rust | 21 | 8,284 |
| C | 9 | 4,633 |
| C Header | 3 | 1,997 |

### Files by Module

| Module | Files | Lines |
|--------|-------|-------|
| app/src/debug | 1 | 219 |
| app/src/main | 50 | 23,944 |
| app/src/test | 22 | 4,336 |
| build-proot | 4 | 5,067 |
| core/data | 112 | 14,020 |
| core/et | 2 | 383 |
| core/ffmpeg | 14 | 1,831 |
| core/fido | 14 | 4,245 |
| core/knock | 5 | 452 |
| core/local | 25 | 9,425 |
| core/mail | 9 | 1,697 |
| core/mosh | 3 | 618 |
| core/rclone | 5 | 1,249 |
| core/rdp | 2 | 562 |
| core/reticulum | 11 | 2,237 |
| core/scan | 7 | 496 |
| core/security | 17 | 2,004 |
| core/smb | 2 | 485 |
| core/spa | 6 | 649 |
| core/ssh | 53 | 10,663 |
| core/stepca | 22 | 2,227 |
| core/terminal-haven | 2 | 188 |
| core/toolbar | 2 | 2,187 |
| core/tunnel | 25 | 3,862 |
| core/ui | 8 | 492 |
| core/usb | 12 | 2,048 |
| core/vnc | 14 | 2,378 |
| core/wayland | 3 | 930 |
| feature/connections | 22 | 13,872 |
| feature/editor | 4 | 790 |
| feature/imagetools | 6 | 842 |
| feature/keys | 7 | 3,670 |
| feature/mail | 14 | 3,680 |
| feature/rdp | 3 | 1,993 |
| feature/settings | 9 | 5,394 |
| feature/sftp | 26 | 12,924 |
| feature/terminal | 13 | 6,774 |
| feature/tunnel | 5 | 1,311 |
| feature/vnc | 2 | 2,410 |
| integration-tests | 5 | 1,012 |
| rclone-android | 2 | 221 |
| rdp-kotlin | 22 | 12,037 |
| scratch | 3 | 467 |

---

## app/src/debug

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `app/src/debug/kotlin/sh/haven/app/debug/DebugReceiver.kt` | 219 | NOT REVIEWED | Debug broadcast receiver |

## app/src/main

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `app/src/main/kotlin/sh/haven/app/BiometricLockScreen.kt` | 223 | NOT REVIEWED | UI screen for biometriclock |
| 2 | `app/src/main/kotlin/sh/haven/app/HavenApp.kt` | 403 | NOT REVIEWED | Application class with Hilt setup |
| 3 | `app/src/main/kotlin/sh/haven/app/HavenDocumentsProvider.kt` | 499 | NOT REVIEWED | Android DocumentsProvider for SAF integration |
| 4 | `app/src/main/kotlin/sh/haven/app/MainActivity.kt` | 525 | NOT REVIEWED | Main Android activity entry point |
| 5 | `app/src/main/kotlin/sh/haven/app/agent/AgentAuditRecorder.kt` | 230 | NOT REVIEWED | Agent audit event recorder |
| 6 | `app/src/main/kotlin/sh/haven/app/agent/AppWindowConnectionStore.kt` | 130 | NOT REVIEWED | App window connection state store |
| 7 | `app/src/main/kotlin/sh/haven/app/agent/BiometricGateHost.kt` | 69 | NOT REVIEWED | Biometric gate host component |
| 8 | `app/src/main/kotlin/sh/haven/app/agent/ConsentActionReceiver.kt` | 69 | NOT REVIEWED | Broadcast receiver for consent actions |
| 9 | `app/src/main/kotlin/sh/haven/app/agent/ConsentHost.kt` | 280 | NOT REVIEWED | Agent consent request host |
| 10 | `app/src/main/kotlin/sh/haven/app/agent/EdgeIconDock.kt` | 202 | NOT REVIEWED | Edge icon dock UI component |
| 11 | `app/src/main/kotlin/sh/haven/app/agent/GuestMcpClient.kt` | 142 | NOT REVIEWED | Client for guestmcp |
| 12 | `app/src/main/kotlin/sh/haven/app/agent/HavenUiBridge.kt` | 328 | NOT REVIEWED | Bridge between UI and agent core |
| 13 | `app/src/main/kotlin/sh/haven/app/agent/McpForegroundParticipantModule.kt` | 92 | NOT REVIEWED | Hilt/DI module for mcpforegroundparticipant |
| 14 | `app/src/main/kotlin/sh/haven/app/agent/McpServer.kt` | 1516 | NOT REVIEWED | McpServer implementation |
| 15 | `app/src/main/kotlin/sh/haven/app/agent/McpTools.kt` | 10192 | NOT REVIEWED | McpTools implementation |
| 16 | `app/src/main/kotlin/sh/haven/app/agent/McpTunnelManager.kt` | 597 | NOT REVIEWED | Manager for mcptunnel |
| 17 | `app/src/main/kotlin/sh/haven/app/agent/PipController.kt` | 36 | NOT REVIEWED | PipController implementation |
| 18 | `app/src/main/kotlin/sh/haven/app/agent/PresentationHost.kt` | 766 | NOT REVIEWED | Agent presentation host composable |
| 19 | `app/src/main/kotlin/sh/haven/app/agent/StandingPolicyEnforcer.kt` | 114 | NOT REVIEWED | Standing policy enforcement logic |
| 20 | `app/src/main/kotlin/sh/haven/app/agent/TerminalInputQueue.kt` | 238 | NOT REVIEWED | Terminal input queue management |
| 21 | `app/src/main/kotlin/sh/haven/app/agent/mailrules/MailRuleActionExecutor.kt` | 219 | NOT REVIEWED | Mail rule action executor |
| 22 | `app/src/main/kotlin/sh/haven/app/agent/mailrules/MailRuleEngine.kt` | 212 | NOT REVIEWED | Mail rule evaluation engine |
| 23 | `app/src/main/kotlin/sh/haven/app/agent/mailrules/MailRulesBindingModule.kt` | 24 | NOT REVIEWED | Hilt/DI module for mailrulesbinding |
| 24 | `app/src/main/kotlin/sh/haven/app/agent/mailrules/MailWatchManager.kt` | 100 | NOT REVIEWED | Manager for mailwatch |
| 25 | `app/src/main/kotlin/sh/haven/app/agent/mailrules/RealMailRulePoller.kt` | 65 | NOT REVIEWED | Real mail rule polling implementation |
| 26 | `app/src/main/kotlin/sh/haven/app/desktop/AppWindowLauncher.kt` | 92 | NOT REVIEWED | App window launcher |
| 27 | `app/src/main/kotlin/sh/haven/app/desktop/AppWindowShortcutManager.kt` | 101 | NOT REVIEWED | Manager for appwindowshortcut |
| 28 | `app/src/main/kotlin/sh/haven/app/desktop/DesktopManagerScreen.kt` | 1274 | NOT REVIEWED | UI screen for desktopmanager |
| 29 | `app/src/main/kotlin/sh/haven/app/desktop/DesktopTab.kt` | 116 | NOT REVIEWED | Desktop tab composable |
| 30 | `app/src/main/kotlin/sh/haven/app/desktop/DesktopViewModel.kt` | 1587 | NOT REVIEWED | ViewModel for desktop |
| 31 | `app/src/main/kotlin/sh/haven/app/desktop/InstalledAppsScreen.kt` | 197 | NOT REVIEWED | UI screen for installedapps |
| 32 | `app/src/main/kotlin/sh/haven/app/desktop/RdpDesktopSession.kt` | 58 | NOT REVIEWED | Session handling for rdpdesktop |
| 33 | `app/src/main/kotlin/sh/haven/app/desktop/RemoteDesktopSession.kt` | 61 | NOT REVIEWED | Session handling for remotedesktop |
| 34 | `app/src/main/kotlin/sh/haven/app/desktop/VncDesktopSession.kt` | 52 | NOT REVIEWED | Session handling for vncdesktop |
| 35 | `app/src/main/kotlin/sh/haven/app/mail/MailAttachmentModule.kt` | 57 | NOT REVIEWED | Hilt/DI module for mailattachment |
| 36 | `app/src/main/kotlin/sh/haven/app/navigation/DebugNavEvents.kt` | 20 | NOT REVIEWED | DebugNavEvents implementation |
| 37 | `app/src/main/kotlin/sh/haven/app/navigation/DesktopScreen.kt` | 432 | NOT REVIEWED | UI screen for desktop |
| 38 | `app/src/main/kotlin/sh/haven/app/navigation/HavenNavHost.kt` | 1138 | NOT REVIEWED | Navigation graph for all screens |
| 39 | `app/src/main/kotlin/sh/haven/app/navigation/NavStateViewModel.kt` | 57 | NOT REVIEWED | ViewModel for navstate |
| 40 | `app/src/main/kotlin/sh/haven/app/navigation/Screen.kt` | 4 | NOT REVIEWED | UI screen for  |
| 41 | `app/src/main/kotlin/sh/haven/app/reticulum/NativeReticulumTransport.kt` | 291 | NOT REVIEWED | Native Reticulum transport via JNI |
| 42 | `app/src/main/kotlin/sh/haven/app/reticulum/ReticulumModule.kt` | 17 | NOT REVIEWED | Hilt/DI module for reticulum |
| 43 | `app/src/main/kotlin/sh/haven/app/workspace/WorkspaceLaunchState.kt` | 52 | NOT REVIEWED | Workspace launch state model |
| 44 | `app/src/main/kotlin/sh/haven/app/workspace/WorkspaceLauncher.kt` | 251 | NOT REVIEWED | Workspace launch orchestration |
| 45 | `app/src/main/kotlin/sh/haven/app/workspace/WorkspaceShortcutManager.kt` | 97 | NOT REVIEWED | Manager for workspaceshortcut |
| 46 | `app/src/main/kotlin/sh/haven/app/workspace/WorkspaceViewModel.kt` | 178 | NOT REVIEWED | ViewModel for workspace |
| 47 | `app/src/main/kotlin/sh/haven/app/workspace/ui/SaveWorkspaceDialog.kt` | 181 | NOT REVIEWED | Dialog component for saveworkspace |
| 48 | `app/src/main/kotlin/sh/haven/app/workspace/ui/WorkspaceCard.kt` | 126 | NOT REVIEWED | Workspace card composable |
| 49 | `app/src/main/kotlin/sh/haven/app/workspace/ui/WorkspaceLaunchBanner.kt` | 118 | NOT REVIEWED | Workspace launch banner |
| 50 | `app/src/main/kotlin/sh/haven/app/workspace/ui/WorkspaceSection.kt` | 116 | NOT REVIEWED | Workspace section composable |

## app/src/test

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `app/src/test/kotlin/sh/haven/app/BiometricLockStateTest.kt` | 115 | NOT REVIEWED | Unit test for BiometricLockState |
| 2 | `app/src/test/kotlin/sh/haven/app/FileProviderPathsTest.kt` | 87 | NOT REVIEWED | Unit test for FileProviderPaths |
| 3 | `app/src/test/kotlin/sh/haven/app/ScreenTest.kt` | 42 | NOT REVIEWED | Unit test for Screen |
| 4 | `app/src/test/kotlin/sh/haven/app/agent/HavenUiToolsTest.kt` | 178 | NOT REVIEWED | Unit test for HavenUiTools |
| 5 | `app/src/test/kotlin/sh/haven/app/agent/McpChunkedWriteTest.kt` | 167 | NOT REVIEWED | Unit test for McpChunkedWrite |
| 6 | `app/src/test/kotlin/sh/haven/app/agent/McpDragSelectionTest.kt` | 317 | NOT REVIEWED | Unit test for McpDragSelection |
| 7 | `app/src/test/kotlin/sh/haven/app/agent/McpLoopbackTrustTest.kt` | 183 | NOT REVIEWED | Unit test for McpLoopbackTrust |
| 8 | `app/src/test/kotlin/sh/haven/app/agent/McpNoTerminalSessionTest.kt` | 202 | NOT REVIEWED | Unit test for McpNoTerminalSession |
| 9 | `app/src/test/kotlin/sh/haven/app/agent/McpServerConsentTest.kt` | 747 | NOT REVIEWED | Unit test for McpServerConsent |
| 10 | `app/src/test/kotlin/sh/haven/app/agent/McpToolsConsentTest.kt` | 375 | NOT REVIEWED | Unit test for McpToolsConsent |
| 11 | `app/src/test/kotlin/sh/haven/app/agent/McpTunnelProbeTest.kt` | 58 | NOT REVIEWED | Unit test for McpTunnelProbe |
| 12 | `app/src/test/kotlin/sh/haven/app/agent/McpWorkspaceToolsTest.kt` | 300 | NOT REVIEWED | Unit test for McpWorkspaceTools |
| 13 | `app/src/test/kotlin/sh/haven/app/agent/RedactionTest.kt` | 177 | NOT REVIEWED | Unit test for Redaction |
| 14 | `app/src/test/kotlin/sh/haven/app/agent/StandingPolicyEnforcerTest.kt` | 94 | NOT REVIEWED | Unit test for StandingPolicyEnforcer |
| 15 | `app/src/test/kotlin/sh/haven/app/agent/StandingPolicyToolsTest.kt` | 172 | NOT REVIEWED | Unit test for StandingPolicyTools |
| 16 | `app/src/test/kotlin/sh/haven/app/agent/mailrules/MailRuleEngineTest.kt` | 256 | NOT REVIEWED | Unit test for MailRuleEngine |
| 17 | `app/src/test/kotlin/sh/haven/app/desktop/AppWindowLauncherTest.kt` | 94 | NOT REVIEWED | Unit test for AppWindowLauncher |
| 18 | `app/src/test/kotlin/sh/haven/app/desktop/AppWindowShortcutManagerTest.kt` | 45 | NOT REVIEWED | Unit test for AppWindowShortcutManager |
| 19 | `app/src/test/kotlin/sh/haven/app/desktop/RemoteDesktopSessionTest.kt` | 170 | NOT REVIEWED | Unit test for RemoteDesktopSession |
| 20 | `app/src/test/kotlin/sh/haven/app/navigation/NavStateViewModelTest.kt` | 70 | NOT REVIEWED | Unit test for NavStateViewModel |
| 21 | `app/src/test/kotlin/sh/haven/app/workspace/WorkspaceLauncherTest.kt` | 260 | NOT REVIEWED | Unit test for WorkspaceLauncher |
| 22 | `app/src/test/kotlin/sh/haven/app/workspace/WorkspaceViewModelTest.kt` | 227 | NOT REVIEWED | Unit test for WorkspaceViewModel |

## build-proot

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `build-proot/talloc/config.h` | 17 | NOT REVIEWED | config implementation |
| 2 | `build-proot/talloc/lib/replace/replace.h` | 8 | NOT REVIEWED | replace implementation |
| 3 | `build-proot/talloc/talloc.c` | 3070 | NOT REVIEWED | talloc implementation |
| 4 | `build-proot/talloc/talloc.h` | 1972 | NOT REVIEWED | talloc implementation |

## core/data

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `core/data/src/main/kotlin/sh/haven/core/data/agent/AgentActivityHolder.kt` | 32 | NOT REVIEWED | AgentActivityHolder implementation |
| 2 | `core/data/src/main/kotlin/sh/haven/core/data/agent/AgentConsentManager.kt` | 494 | NOT REVIEWED | Manager for agentconsent |
| 3 | `core/data/src/main/kotlin/sh/haven/core/data/agent/AgentPresentationManager.kt` | 243 | NOT REVIEWED | Manager for agentpresentation |
| 4 | `core/data/src/main/kotlin/sh/haven/core/data/agent/AgentUiCommand.kt` | 188 | NOT REVIEWED | AgentUiCommand implementation |
| 5 | `core/data/src/main/kotlin/sh/haven/core/data/agent/AgentUiCommandBus.kt` | 90 | NOT REVIEWED | AgentUiCommandBus implementation |
| 6 | `core/data/src/main/kotlin/sh/haven/core/data/agent/McpStatusHolder.kt` | 38 | NOT REVIEWED | McpStatusHolder implementation |
| 7 | `core/data/src/main/kotlin/sh/haven/core/data/agent/ServedFileTracker.kt` | 84 | NOT REVIEWED | ServedFileTracker implementation |
| 8 | `core/data/src/main/kotlin/sh/haven/core/data/backup/BackupService.kt` | 541 | NOT REVIEWED | BackupService implementation |
| 9 | `core/data/src/main/kotlin/sh/haven/core/data/db/AgeIdentityDao.kt` | 27 | NOT REVIEWED | Data access object for ageidentity |
| 10 | `core/data/src/main/kotlin/sh/haven/core/data/db/AgentAuditEventDao.kt` | 46 | NOT REVIEWED | Data access object for agentauditevent |
| 11 | `core/data/src/main/kotlin/sh/haven/core/data/db/ConnectionDao.kt` | 50 | NOT REVIEWED | Data access object for connection |
| 12 | `core/data/src/main/kotlin/sh/haven/core/data/db/ConnectionGroupDao.kt` | 31 | NOT REVIEWED | Data access object for connectiongroup |
| 13 | `core/data/src/main/kotlin/sh/haven/core/data/db/ConnectionLogDao.kt` | 38 | NOT REVIEWED | Data access object for connectionlog |
| 14 | `core/data/src/main/kotlin/sh/haven/core/data/db/DatabaseModule.kt` | 88 | NOT REVIEWED | Hilt/DI module for database |
| 15 | `core/data/src/main/kotlin/sh/haven/core/data/db/HavenDatabase.kt` | 1167 | NOT REVIEWED | Room database definition with all DAOs |
| 16 | `core/data/src/main/kotlin/sh/haven/core/data/db/KnownHostDao.kt` | 31 | NOT REVIEWED | Data access object for knownhost |
| 17 | `core/data/src/main/kotlin/sh/haven/core/data/db/MailRuleDaos.kt` | 99 | NOT REVIEWED | MailRuleDaos implementation |
| 18 | `core/data/src/main/kotlin/sh/haven/core/data/db/PasteQueueDao.kt` | 57 | NOT REVIEWED | Data access object for pastequeue |
| 19 | `core/data/src/main/kotlin/sh/haven/core/data/db/PortForwardRuleDao.kt` | 30 | NOT REVIEWED | Data access object for portforwardrule |
| 20 | `core/data/src/main/kotlin/sh/haven/core/data/db/ProotInstallLogDao.kt` | 59 | NOT REVIEWED | Data access object for prootinstalllog |
| 21 | `core/data/src/main/kotlin/sh/haven/core/data/db/SshKeyDao.kt` | 27 | NOT REVIEWED | Data access object for sshkey |
| 22 | `core/data/src/main/kotlin/sh/haven/core/data/db/StandingPolicyDao.kt` | 34 | NOT REVIEWED | Data access object for standingpolicy |
| 23 | `core/data/src/main/kotlin/sh/haven/core/data/db/StepCaConfigDao.kt` | 27 | NOT REVIEWED | Data access object for stepcaconfig |
| 24 | `core/data/src/main/kotlin/sh/haven/core/data/db/SyncProfileDao.kt` | 28 | NOT REVIEWED | Data access object for syncprofile |
| 25 | `core/data/src/main/kotlin/sh/haven/core/data/db/TotpSecretDao.kt` | 27 | NOT REVIEWED | Data access object for totpsecret |
| 26 | `core/data/src/main/kotlin/sh/haven/core/data/db/TunnelConfigDao.kt` | 42 | NOT REVIEWED | Data access object for tunnelconfig |
| 27 | `core/data/src/main/kotlin/sh/haven/core/data/db/WorkspaceDao.kt` | 45 | NOT REVIEWED | Data access object for workspace |
| 28 | `core/data/src/main/kotlin/sh/haven/core/data/db/entities/AgeIdentityEntity.kt` | 25 | NOT REVIEWED | Database entity |
| 29 | `core/data/src/main/kotlin/sh/haven/core/data/db/entities/AgentAuditEvent.kt` | 47 | NOT REVIEWED | AgentAuditEvent implementation |
| 30 | `core/data/src/main/kotlin/sh/haven/core/data/db/entities/ConnectionGroup.kt` | 14 | NOT REVIEWED | ConnectionGroup implementation |
| 31 | `core/data/src/main/kotlin/sh/haven/core/data/db/entities/ConnectionLog.kt` | 44 | NOT REVIEWED | ConnectionLog implementation |
| 32 | `core/data/src/main/kotlin/sh/haven/core/data/db/entities/ConnectionLogSummary.kt` | 14 | NOT REVIEWED | ConnectionLogSummary implementation |
| 33 | `core/data/src/main/kotlin/sh/haven/core/data/db/entities/ConnectionProfile.kt` | 425 | NOT REVIEWED | ConnectionProfile implementation |
| 34 | `core/data/src/main/kotlin/sh/haven/core/data/db/entities/KnownHost.kt` | 16 | NOT REVIEWED | KnownHost implementation |
| 35 | `core/data/src/main/kotlin/sh/haven/core/data/db/entities/MailRule.kt` | 36 | NOT REVIEWED | MailRule implementation |
| 36 | `core/data/src/main/kotlin/sh/haven/core/data/db/entities/MailRuleFiring.kt` | 41 | NOT REVIEWED | MailRuleFiring implementation |
| 37 | `core/data/src/main/kotlin/sh/haven/core/data/db/entities/MailRulePendingAction.kt` | 28 | NOT REVIEWED | MailRulePendingAction implementation |
| 38 | `core/data/src/main/kotlin/sh/haven/core/data/db/entities/MailWatermark.kt` | 22 | NOT REVIEWED | MailWatermark implementation |
| 39 | `core/data/src/main/kotlin/sh/haven/core/data/db/entities/PasteQueueEntry.kt` | 82 | NOT REVIEWED | PasteQueueEntry implementation |
| 40 | `core/data/src/main/kotlin/sh/haven/core/data/db/entities/PortForwardRule.kt` | 40 | NOT REVIEWED | PortForwardRule implementation |
| 41 | `core/data/src/main/kotlin/sh/haven/core/data/db/entities/ProotInstallLog.kt` | 48 | NOT REVIEWED | ProotInstallLog implementation |
| 42 | `core/data/src/main/kotlin/sh/haven/core/data/db/entities/ProotInstallLogSummary.kt` | 17 | NOT REVIEWED | ProotInstallLogSummary implementation |
| 43 | `core/data/src/main/kotlin/sh/haven/core/data/db/entities/SshKey.kt` | 100 | NOT REVIEWED | SshKey implementation |
| 44 | `core/data/src/main/kotlin/sh/haven/core/data/db/entities/StandingPolicy.kt` | 45 | NOT REVIEWED | StandingPolicy implementation |
| 45 | `core/data/src/main/kotlin/sh/haven/core/data/db/entities/StepCaConfig.kt` | 82 | NOT REVIEWED | Configuration model |
| 46 | `core/data/src/main/kotlin/sh/haven/core/data/db/entities/SyncProfile.kt` | 39 | NOT REVIEWED | SyncProfile implementation |
| 47 | `core/data/src/main/kotlin/sh/haven/core/data/db/entities/TotpSecret.kt` | 35 | NOT REVIEWED | TotpSecret implementation |
| 48 | `core/data/src/main/kotlin/sh/haven/core/data/db/entities/TunnelConfig.kt` | 70 | NOT REVIEWED | Configuration model |
| 49 | `core/data/src/main/kotlin/sh/haven/core/data/db/entities/WorkspaceItem.kt` | 60 | NOT REVIEWED | WorkspaceItem implementation |
| 50 | `core/data/src/main/kotlin/sh/haven/core/data/db/entities/WorkspaceProfile.kt` | 20 | NOT REVIEWED | WorkspaceProfile implementation |
| 51 | `core/data/src/main/kotlin/sh/haven/core/data/desktop/DesktopSessionRegistry.kt` | 81 | NOT REVIEWED | DesktopSessionRegistry implementation |
| 52 | `core/data/src/main/kotlin/sh/haven/core/data/font/FontBytes.kt` | 97 | NOT REVIEWED | FontBytes implementation |
| 53 | `core/data/src/main/kotlin/sh/haven/core/data/font/TerminalFontInstaller.kt` | 259 | NOT REVIEWED | TerminalFontInstaller implementation |
| 54 | `core/data/src/main/kotlin/sh/haven/core/data/keystore/AgeIdentitySection.kt` | 73 | NOT REVIEWED | AgeIdentitySection implementation |
| 55 | `core/data/src/main/kotlin/sh/haven/core/data/keystore/BiometricGate.kt` | 145 | NOT REVIEWED | BiometricGate implementation |
| 56 | `core/data/src/main/kotlin/sh/haven/core/data/keystore/KeystoreModule.kt` | 20 | NOT REVIEWED | Hilt/DI module for keystore |
| 57 | `core/data/src/main/kotlin/sh/haven/core/data/keystore/ProfileCredentialSection.kt` | 136 | NOT REVIEWED | ProfileCredentialSection implementation |
| 58 | `core/data/src/main/kotlin/sh/haven/core/data/keystore/SshKeySection.kt` | 188 | NOT REVIEWED | SshKeySection implementation |
| 59 | `core/data/src/main/kotlin/sh/haven/core/data/keystore/TotpSecretSection.kt` | 73 | NOT REVIEWED | TotpSecretSection implementation |
| 60 | `core/data/src/main/kotlin/sh/haven/core/data/keystore/UnifiedKeystore.kt` | 92 | NOT REVIEWED | UnifiedKeystore implementation |
| 61 | `core/data/src/main/kotlin/sh/haven/core/data/mailrule/MailRuleApproval.kt` | 27 | NOT REVIEWED | MailRuleApproval implementation |
| 62 | `core/data/src/main/kotlin/sh/haven/core/data/mailrule/MailRuleMatcher.kt` | 68 | NOT REVIEWED | MailRuleMatcher implementation |
| 63 | `core/data/src/main/kotlin/sh/haven/core/data/mailrule/MailRuleModels.kt` | 240 | NOT REVIEWED | MailRuleModels implementation |
| 64 | `core/data/src/main/kotlin/sh/haven/core/data/message/UserMessageBus.kt` | 56 | NOT REVIEWED | UserMessageBus implementation |
| 65 | `core/data/src/main/kotlin/sh/haven/core/data/preferences/AppWindowDefList.kt` | 94 | NOT REVIEWED | AppWindowDefList implementation |
| 66 | `core/data/src/main/kotlin/sh/haven/core/data/preferences/DataStoreModule.kt` | 25 | NOT REVIEWED | Hilt/DI module for datastore |
| 67 | `core/data/src/main/kotlin/sh/haven/core/data/preferences/KeyOrdering.kt` | 37 | NOT REVIEWED | KeyOrdering implementation |
| 68 | `core/data/src/main/kotlin/sh/haven/core/data/preferences/ToolbarKey.kt` | 115 | NOT REVIEWED | ToolbarKey implementation |
| 69 | `core/data/src/main/kotlin/sh/haven/core/data/preferences/ToolbarLayout.kt` | 344 | NOT REVIEWED | ToolbarLayout implementation |
| 70 | `core/data/src/main/kotlin/sh/haven/core/data/preferences/UserPreferencesRepository.kt` | 1559 | NOT REVIEWED | Repository for userpreferences |
| 71 | `core/data/src/main/kotlin/sh/haven/core/data/repository/AgeIdentityRepository.kt` | 61 | NOT REVIEWED | Repository for ageidentity |
| 72 | `core/data/src/main/kotlin/sh/haven/core/data/repository/ConnectionLogRepository.kt` | 62 | NOT REVIEWED | Repository for connectionlog |
| 73 | `core/data/src/main/kotlin/sh/haven/core/data/repository/ConnectionRepository.kt` | 107 | NOT REVIEWED | Repository for connection |
| 74 | `core/data/src/main/kotlin/sh/haven/core/data/repository/MailRuleRepository.kt` | 61 | NOT REVIEWED | Repository for mailrule |
| 75 | `core/data/src/main/kotlin/sh/haven/core/data/repository/PortForwardRepository.kt` | 25 | NOT REVIEWED | Repository for portforward |
| 76 | `core/data/src/main/kotlin/sh/haven/core/data/repository/ProotInstallLogRepository.kt` | 70 | NOT REVIEWED | Repository for prootinstalllog |
| 77 | `core/data/src/main/kotlin/sh/haven/core/data/repository/SshKeyRepository.kt` | 114 | NOT REVIEWED | Repository for sshkey |
| 78 | `core/data/src/main/kotlin/sh/haven/core/data/repository/StandingPolicyRepository.kt` | 24 | NOT REVIEWED | Repository for standingpolicy |
| 79 | `core/data/src/main/kotlin/sh/haven/core/data/repository/StepCaConfigRepository.kt` | 28 | NOT REVIEWED | Repository for stepcaconfig |
| 80 | `core/data/src/main/kotlin/sh/haven/core/data/repository/SyncProfileRepository.kt` | 24 | NOT REVIEWED | Repository for syncprofile |
| 81 | `core/data/src/main/kotlin/sh/haven/core/data/repository/TotpSecretRepository.kt` | 54 | NOT REVIEWED | Repository for totpsecret |
| 82 | `core/data/src/main/kotlin/sh/haven/core/data/repository/TunnelConfigRepository.kt` | 100 | NOT REVIEWED | Repository for tunnelconfig |
| 83 | `core/data/src/main/kotlin/sh/haven/core/data/repository/WorkspaceRepository.kt` | 55 | NOT REVIEWED | Repository for workspace |
| 84 | `core/data/src/main/kotlin/sh/haven/core/data/terminal/ScrollbackRing.kt` | 92 | NOT REVIEWED | ScrollbackRing implementation |
| 85 | `core/data/src/test/kotlin/sh/haven/core/data/ConnectionProfileTest.kt` | 257 | NOT REVIEWED | Unit test for ConnectionProfile |
| 86 | `core/data/src/test/kotlin/sh/haven/core/data/LockTimeoutTest.kt` | 96 | NOT REVIEWED | Unit test for LockTimeout |
| 87 | `core/data/src/test/kotlin/sh/haven/core/data/PasteQueueEntryTest.kt` | 71 | NOT REVIEWED | Unit test for PasteQueueEntry |
| 88 | `core/data/src/test/kotlin/sh/haven/core/data/TunnelConfigTest.kt` | 75 | NOT REVIEWED | Unit test for TunnelConfig |
| 89 | `core/data/src/test/kotlin/sh/haven/core/data/WorkspaceProfileTest.kt` | 57 | NOT REVIEWED | Unit test for WorkspaceProfile |
| 90 | `core/data/src/test/kotlin/sh/haven/core/data/WorkspaceRepositoryTest.kt` | 216 | NOT REVIEWED | Unit test for WorkspaceRepository |
| 91 | `core/data/src/test/kotlin/sh/haven/core/data/agent/AgentConsentManagerTest.kt` | 472 | NOT REVIEWED | Unit test for AgentConsentManager |
| 92 | `core/data/src/test/kotlin/sh/haven/core/data/agent/AgentPresentationManagerTest.kt` | 79 | NOT REVIEWED | Unit test for AgentPresentationManager |
| 93 | `core/data/src/test/kotlin/sh/haven/core/data/agent/AgentUiCommandBusTest.kt` | 193 | NOT REVIEWED | Unit test for AgentUiCommandBus |
| 94 | `core/data/src/test/kotlin/sh/haven/core/data/agent/ServedFileTrackerTest.kt` | 54 | NOT REVIEWED | Unit test for ServedFileTracker |
| 95 | `core/data/src/test/kotlin/sh/haven/core/data/backup/BackupServiceTest.kt` | 820 | NOT REVIEWED | Unit test for BackupService |
| 96 | `core/data/src/test/kotlin/sh/haven/core/data/db/entities/SshKeyTest.kt` | 54 | NOT REVIEWED | Unit test for SshKey |
| 97 | `core/data/src/test/kotlin/sh/haven/core/data/font/FontBytesTest.kt` | 80 | NOT REVIEWED | Unit test for FontBytes |
| 98 | `core/data/src/test/kotlin/sh/haven/core/data/keystore/BiometricGateTest.kt` | 160 | NOT REVIEWED | Unit test for BiometricGate |
| 99 | `core/data/src/test/kotlin/sh/haven/core/data/keystore/ProfileCredentialSectionTest.kt` | 243 | NOT REVIEWED | Unit test for ProfileCredentialSection |
| 100 | `core/data/src/test/kotlin/sh/haven/core/data/keystore/SshKeySectionTest.kt` | 472 | NOT REVIEWED | Unit test for SshKeySection |
| 101 | `core/data/src/test/kotlin/sh/haven/core/data/keystore/UnifiedKeystoreTest.kt` | 211 | NOT REVIEWED | Unit test for UnifiedKeystore |
| 102 | `core/data/src/test/kotlin/sh/haven/core/data/mailrule/MailRuleJsonTest.kt` | 69 | NOT REVIEWED | Unit test for MailRuleJson |
| 103 | `core/data/src/test/kotlin/sh/haven/core/data/mailrule/MailRuleMatcherTest.kt` | 89 | NOT REVIEWED | Unit test for MailRuleMatcher |
| 104 | `core/data/src/test/kotlin/sh/haven/core/data/message/UserMessageBusTest.kt` | 70 | NOT REVIEWED | Unit test for UserMessageBus |
| 105 | `core/data/src/test/kotlin/sh/haven/core/data/preferences/AppWindowDefListTest.kt` | 84 | NOT REVIEWED | Unit test for AppWindowDefList |
| 106 | `core/data/src/test/kotlin/sh/haven/core/data/preferences/AppWindowDefRepoTest.kt` | 66 | NOT REVIEWED | Unit test for AppWindowDefRepo |
| 107 | `core/data/src/test/kotlin/sh/haven/core/data/preferences/EditModeControlsPlacementTest.kt` | 28 | NOT REVIEWED | Unit test for EditModeControlsPlacement |
| 108 | `core/data/src/test/kotlin/sh/haven/core/data/preferences/KeyOrderingTest.kt` | 58 | NOT REVIEWED | Unit test for KeyOrdering |
| 109 | `core/data/src/test/kotlin/sh/haven/core/data/preferences/SnippetOpsTest.kt` | 98 | NOT REVIEWED | Unit test for SnippetOps |
| 110 | `core/data/src/test/kotlin/sh/haven/core/data/preferences/ToolbarEditorOpsTest.kt` | 81 | NOT REVIEWED | Unit test for ToolbarEditorOps |
| 111 | `core/data/src/test/kotlin/sh/haven/core/data/repository/SshKeyRepositoryTest.kt` | 68 | NOT REVIEWED | Unit test for SshKeyRepository |
| 112 | `core/data/src/test/kotlin/sh/haven/core/data/terminal/ScrollbackRingTest.kt` | 80 | NOT REVIEWED | Unit test for ScrollbackRing |

## core/et

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `core/et/src/main/kotlin/sh/haven/core/et/EtSession.kt` | 114 | NOT REVIEWED | Session handling for et |
| 2 | `core/et/src/main/kotlin/sh/haven/core/et/EtSessionManager.kt` | 269 | NOT REVIEWED | Manager for etsession |

## core/ffmpeg

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `core/ffmpeg/src/main/kotlin/sh/haven/core/ffmpeg/FfmpegExecutor.kt` | 70 | NOT REVIEWED | FFmpeg command executor |
| 2 | `core/ffmpeg/src/main/kotlin/sh/haven/core/ffmpeg/FfmpegJob.kt` | 124 | NOT REVIEWED | FFmpeg job model |
| 3 | `core/ffmpeg/src/main/kotlin/sh/haven/core/ffmpeg/FfmpegProgress.kt` | 49 | NOT REVIEWED | FFmpeg progress tracking |
| 4 | `core/ffmpeg/src/main/kotlin/sh/haven/core/ffmpeg/FfmpegResult.kt` | 14 | NOT REVIEWED | FFmpeg execution result |
| 5 | `core/ffmpeg/src/main/kotlin/sh/haven/core/ffmpeg/FilterPresets.kt` | 41 | NOT REVIEWED | FFmpeg filter presets |
| 6 | `core/ffmpeg/src/main/kotlin/sh/haven/core/ffmpeg/HlsStreamServer.kt` | 575 | NOT REVIEWED | HLS streaming server |
| 7 | `core/ffmpeg/src/main/kotlin/sh/haven/core/ffmpeg/MediaInfo.kt` | 54 | NOT REVIEWED | Media file info model |
| 8 | `core/ffmpeg/src/main/kotlin/sh/haven/core/ffmpeg/ProbeCommand.kt` | 124 | NOT REVIEWED | FFprobe command builder |
| 9 | `core/ffmpeg/src/main/kotlin/sh/haven/core/ffmpeg/TranscodeCommand.kt` | 214 | NOT REVIEWED | FFmpeg transcode command builder |
| 10 | `core/ffmpeg/src/main/kotlin/sh/haven/core/ffmpeg/VideoFilter.kt` | 187 | NOT REVIEWED | Video filter model |
| 11 | `core/ffmpeg/src/test/kotlin/sh/haven/core/ffmpeg/FfmpegProgressTest.kt` | 63 | NOT REVIEWED | Unit test for FfmpegProgress |
| 12 | `core/ffmpeg/src/test/kotlin/sh/haven/core/ffmpeg/FfmpegResultTest.kt` | 52 | NOT REVIEWED | Unit test for FfmpegResult |
| 13 | `core/ffmpeg/src/test/kotlin/sh/haven/core/ffmpeg/TranscodeCommandTest.kt` | 134 | NOT REVIEWED | Unit test for TranscodeCommand |
| 14 | `core/ffmpeg/src/test/kotlin/sh/haven/core/ffmpeg/VideoFilterTest.kt` | 130 | NOT REVIEWED | Unit test for VideoFilter |

## core/fido

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `core/fido/src/main/kotlin/sh/haven/core/fido/Ctap2Cbor.kt` | 901 | NOT REVIEWED | CTAP2 CBOR encoding/decoding |
| 2 | `core/fido/src/main/kotlin/sh/haven/core/fido/Ctap2PinProtocol.kt` | 218 | NOT REVIEWED | CTAP2 PIN protocol |
| 3 | `core/fido/src/main/kotlin/sh/haven/core/fido/CtapHidTransport.kt` | 202 | NOT REVIEWED | CTAP HID transport layer |
| 4 | `core/fido/src/main/kotlin/sh/haven/core/fido/CtapNfcTransport.kt` | 119 | NOT REVIEWED | CTAP NFC transport layer |
| 5 | `core/fido/src/main/kotlin/sh/haven/core/fido/FidoAuthenticator.kt` | 1676 | NOT REVIEWED | FIDO authenticator interface |
| 6 | `core/fido/src/main/kotlin/sh/haven/core/fido/FidoIdentity.kt` | 135 | NOT REVIEWED | FIDO identity model |
| 7 | `core/fido/src/main/kotlin/sh/haven/core/fido/SkKeyData.kt` | 86 | NOT REVIEWED | Security key data model |
| 8 | `core/fido/src/main/kotlin/sh/haven/core/fido/SkKeyParser.kt` | 290 | NOT REVIEWED | Security key parser |
| 9 | `core/fido/src/test/kotlin/sh/haven/core/fido/Ctap2AssertionStatusTest.kt` | 51 | NOT REVIEWED | Unit test for Ctap2AssertionStatus |
| 10 | `core/fido/src/test/kotlin/sh/haven/core/fido/Ctap2CborTest.kt` | 149 | NOT REVIEWED | Unit test for Ctap2Cbor |
| 11 | `core/fido/src/test/kotlin/sh/haven/core/fido/Ctap2EitherOrTest.kt` | 80 | NOT REVIEWED | Unit test for Ctap2EitherOr |
| 12 | `core/fido/src/test/kotlin/sh/haven/core/fido/Ctap2MakeCredentialTest.kt` | 128 | NOT REVIEWED | Unit test for Ctap2MakeCredential |
| 13 | `core/fido/src/test/kotlin/sh/haven/core/fido/Ctap2PinProtocolTest.kt` | 144 | NOT REVIEWED | Unit test for Ctap2PinProtocol |
| 14 | `core/fido/src/test/kotlin/sh/haven/core/fido/CtapHidInitDrainTest.kt` | 66 | NOT REVIEWED | Unit test for CtapHidInitDrain |

## core/knock

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `core/knock/src/main/kotlin/sh/haven/core/knock/KnockHiltModule.kt` | 15 | NOT REVIEWED | Hilt/DI module for knockhilt |
| 2 | `core/knock/src/main/kotlin/sh/haven/core/knock/KnockSequence.kt` | 88 | NOT REVIEWED | Port knock sequence model |
| 3 | `core/knock/src/main/kotlin/sh/haven/core/knock/PortKnocker.kt` | 100 | NOT REVIEWED | Port knocking implementation |
| 4 | `core/knock/src/test/kotlin/sh/haven/core/knock/KnockSequenceTest.kt` | 101 | NOT REVIEWED | Unit test for KnockSequence |
| 5 | `core/knock/src/test/kotlin/sh/haven/core/knock/PortKnockerTest.kt` | 148 | NOT REVIEWED | Unit test for PortKnocker |

## core/local

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `core/local/src/main/cpp/haven-usb/haven-hidraw-test.c` | 73 | NOT REVIEWED | HID raw test |
| 2 | `core/local/src/main/cpp/haven-usb/haven-usb-probe.c` | 104 | NOT REVIEWED | USB device probe |
| 3 | `core/local/src/main/cpp/haven-usb/haven-usb-serial.c` | 253 | NOT REVIEWED | USB serial communication |
| 4 | `core/local/src/main/cpp/haven-usb/libhaven_usb.c` | 543 | NOT REVIEWED | USB library |
| 5 | `core/local/src/main/cpp/pty_bridge.c` | 123 | NOT REVIEWED | PTY bridge C implementation |
| 6 | `core/local/src/main/kotlin/sh/haven/core/local/AudioBridge.kt` | 272 | NOT REVIEWED | Audio bridge for terminal |
| 7 | `core/local/src/main/kotlin/sh/haven/core/local/DesktopManager.kt` | 1737 | NOT REVIEWED | Manager for desktop |
| 8 | `core/local/src/main/kotlin/sh/haven/core/local/GlCanvasProbe.kt` | 68 | NOT REVIEWED | OpenGL canvas capability probe |
| 9 | `core/local/src/main/kotlin/sh/haven/core/local/GuestAppScanner.kt` | 208 | NOT REVIEWED | Guest app scanner |
| 10 | `core/local/src/main/kotlin/sh/haven/core/local/GuestServiceManager.kt` | 340 | NOT REVIEWED | Manager for guestservice |
| 11 | `core/local/src/main/kotlin/sh/haven/core/local/LocalSession.kt` | 194 | NOT REVIEWED | Session handling for local |
| 12 | `core/local/src/main/kotlin/sh/haven/core/local/LocalSessionManager.kt` | 600 | NOT REVIEWED | Manager for localsession |
| 13 | `core/local/src/main/kotlin/sh/haven/core/local/ProotManager.kt` | 2376 | NOT REVIEWED | Manager for proot |
| 14 | `core/local/src/main/kotlin/sh/haven/core/local/PtyBridge.kt` | 36 | NOT REVIEWED | PTY bridge for terminal |
| 15 | `core/local/src/main/kotlin/sh/haven/core/local/WaylandSocketHelper.kt` | 555 | NOT REVIEWED | Wayland socket helper |
| 16 | `core/local/src/main/kotlin/sh/haven/core/local/proot/Manifest.kt` | 867 | NOT REVIEWED | proot package manifest |
| 17 | `core/local/src/main/kotlin/sh/haven/core/local/proot/MirrorCatalog.kt` | 143 | NOT REVIEWED | proot mirror catalog |
| 18 | `core/local/src/main/kotlin/sh/haven/core/local/proot/PackageOps.kt` | 320 | NOT REVIEWED | proot package operations |
| 19 | `core/local/src/test/kotlin/sh/haven/core/local/DesktopBinaryVerificationTest.kt` | 80 | NOT REVIEWED | Unit test for DesktopBinaryVerification |
| 20 | `core/local/src/test/kotlin/sh/haven/core/local/DesktopManagerResolutionTest.kt` | 71 | NOT REVIEWED | Unit test for DesktopManagerResolution |
| 21 | `core/local/src/test/kotlin/sh/haven/core/local/GlCanvasProbeTest.kt` | 58 | NOT REVIEWED | Unit test for GlCanvasProbe |
| 22 | `core/local/src/test/kotlin/sh/haven/core/local/GuestAppScannerTest.kt` | 119 | NOT REVIEWED | Unit test for GuestAppScanner |
| 23 | `core/local/src/test/kotlin/sh/haven/core/local/LocalSessionCallbackTest.kt` | 61 | NOT REVIEWED | Unit test for LocalSessionCallback |
| 24 | `core/local/src/test/kotlin/sh/haven/core/local/ProotManagerDeleteTest.kt` | 55 | NOT REVIEWED | Unit test for ProotManagerDelete |
| 25 | `core/local/src/test/kotlin/sh/haven/core/local/proot/MirrorCatalogTest.kt` | 169 | NOT REVIEWED | Unit test for MirrorCatalog |

## core/mail

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `core/mail/src/main/kotlin/sh/haven/core/mail/ImapMailClient.kt` | 600 | NOT REVIEWED | Client for imapmail |
| 2 | `core/mail/src/main/kotlin/sh/haven/core/mail/MailClient.kt` | 102 | NOT REVIEWED | Client for mail |
| 3 | `core/mail/src/main/kotlin/sh/haven/core/mail/MailModels.kt` | 218 | NOT REVIEWED | Mail data models |
| 4 | `core/mail/src/main/kotlin/sh/haven/core/mail/MailModule.kt` | 31 | NOT REVIEWED | Hilt/DI module for mail |
| 5 | `core/mail/src/main/kotlin/sh/haven/core/mail/MailSessionManager.kt` | 166 | NOT REVIEWED | Manager for mailsession |
| 6 | `core/mail/src/main/kotlin/sh/haven/core/mail/ProtonMailClient.kt` | 198 | NOT REVIEWED | Client for protonmail |
| 7 | `core/mail/src/test/kotlin/sh/haven/core/mail/ImapMailClientSendTest.kt` | 164 | NOT REVIEWED | Unit test for ImapMailClientSend |
| 8 | `core/mail/src/test/kotlin/sh/haven/core/mail/ImapMailClientTest.kt` | 114 | NOT REVIEWED | Unit test for ImapMailClient |
| 9 | `core/mail/src/test/kotlin/sh/haven/core/mail/MailSessionManagerRoutingTest.kt` | 104 | NOT REVIEWED | Unit test for MailSessionManagerRouting |

## core/mosh

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `core/mosh/src/main/kotlin/sh/haven/core/mosh/MoshSession.kt` | 233 | NOT REVIEWED | Session handling for mosh |
| 2 | `core/mosh/src/main/kotlin/sh/haven/core/mosh/MoshSessionManager.kt` | 281 | NOT REVIEWED | Manager for moshsession |
| 3 | `core/mosh/src/test/kotlin/sh/haven/core/mosh/MoshSessionDecckmTest.kt` | 104 | NOT REVIEWED | Unit test for MoshSessionDecckm |

## core/rclone

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `core/rclone/src/main/kotlin/sh/haven/core/rclone/RcloneClient.kt` | 592 | NOT REVIEWED | Client for rclone |
| 2 | `core/rclone/src/main/kotlin/sh/haven/core/rclone/RcloneConfigParser.kt` | 69 | NOT REVIEWED | Rclone config parser |
| 3 | `core/rclone/src/main/kotlin/sh/haven/core/rclone/RcloneProvider.kt` | 198 | NOT REVIEWED | Rclone provider |
| 4 | `core/rclone/src/main/kotlin/sh/haven/core/rclone/RcloneSessionManager.kt` | 315 | NOT REVIEWED | Manager for rclonesession |
| 5 | `core/rclone/src/test/kotlin/sh/haven/core/rclone/RcloneConfigParserTest.kt` | 75 | NOT REVIEWED | Unit test for RcloneConfigParser |

## core/rdp

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `core/rdp/src/main/kotlin/sh/haven/core/rdp/RdpSession.kt` | 352 | NOT REVIEWED | Session handling for rdp |
| 2 | `core/rdp/src/main/kotlin/sh/haven/core/rdp/RdpSessionManager.kt` | 210 | NOT REVIEWED | Manager for rdpsession |

## core/reticulum

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `core/reticulum/src/main/kotlin/sh/haven/core/reticulum/ReticulumForwardServer.kt` | 254 | NOT REVIEWED | Reticulum forwarding server |
| 2 | `core/reticulum/src/main/kotlin/sh/haven/core/reticulum/ReticulumSession.kt` | 124 | NOT REVIEWED | Session handling for reticulum |
| 3 | `core/reticulum/src/main/kotlin/sh/haven/core/reticulum/ReticulumSessionManager.kt` | 273 | NOT REVIEWED | Manager for reticulumsession |
| 4 | `core/reticulum/src/main/kotlin/sh/haven/core/reticulum/ReticulumTransport.kt` | 164 | NOT REVIEWED | Reticulum transport layer |
| 5 | `core/reticulum/src/main/kotlin/sh/haven/core/reticulum/sftp/ReticulumSftpSession.kt` | 254 | NOT REVIEWED | Session handling for reticulumsftp |
| 6 | `core/reticulum/src/main/kotlin/sh/haven/core/reticulum/sftp/SftpV3Client.kt` | 254 | NOT REVIEWED | Client for sftpv3 |
| 7 | `core/reticulum/src/main/kotlin/sh/haven/core/reticulum/sftp/SftpV3Codec.kt` | 298 | NOT REVIEWED | SFTP v3 codec |
| 8 | `core/reticulum/src/test/kotlin/sh/haven/core/reticulum/ReticulumForwardServerTest.kt` | 85 | NOT REVIEWED | Unit test for ReticulumForwardServer |
| 9 | `core/reticulum/src/test/kotlin/sh/haven/core/reticulum/sftp/SftpV3ClientTest.kt` | 227 | NOT REVIEWED | Unit test for SftpV3Client |
| 10 | `core/reticulum/src/test/kotlin/sh/haven/core/reticulum/sftp/SftpV3CodecTest.kt` | 167 | NOT REVIEWED | Unit test for SftpV3Codec |
| 11 | `core/reticulum/src/test/kotlin/sh/haven/core/reticulum/sftp/SftpV3LocalServerTest.kt` | 137 | NOT REVIEWED | Unit test for SftpV3LocalServer |

## core/scan

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `core/scan/src/main/kotlin/sh/haven/core/scan/BarcodeDecoder.kt` | 79 | NOT REVIEWED | Barcode decoder |
| 2 | `core/scan/src/main/kotlin/sh/haven/core/scan/ScanModule.kt` | 19 | NOT REVIEWED | Hilt/DI module for scan |
| 3 | `core/scan/src/main/kotlin/sh/haven/core/scan/ScanResult.kt` | 23 | NOT REVIEWED | Scan result model |
| 4 | `core/scan/src/main/kotlin/sh/haven/core/scan/TextRecognizer.kt` | 117 | NOT REVIEWED | Text recognizer |
| 5 | `core/scan/src/main/kotlin/sh/haven/core/scan/TrainedDataManager.kt` | 83 | NOT REVIEWED | Manager for traineddata |
| 6 | `core/scan/src/test/kotlin/sh/haven/core/scan/BarcodeDecoderTest.kt` | 70 | NOT REVIEWED | Unit test for BarcodeDecoder |
| 7 | `core/scan/src/test/kotlin/sh/haven/core/scan/TextRecognizerTest.kt` | 105 | NOT REVIEWED | Unit test for TextRecognizer |

## core/security

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `core/security/src/main/kotlin/sh/haven/core/security/AgeFile.kt` | 379 | NOT REVIEWED | Age file encryption |
| 2 | `core/security/src/main/kotlin/sh/haven/core/security/Bech32.kt` | 113 | NOT REVIEWED | Bech32 encoding |
| 3 | `core/security/src/main/kotlin/sh/haven/core/security/BiometricAuthenticator.kt` | 79 | NOT REVIEWED | Biometric authenticator |
| 4 | `core/security/src/main/kotlin/sh/haven/core/security/CredentialEncryption.kt` | 62 | NOT REVIEWED | Credential encryption |
| 5 | `core/security/src/main/kotlin/sh/haven/core/security/JwtPayload.kt` | 54 | NOT REVIEWED | JWT payload model |
| 6 | `core/security/src/main/kotlin/sh/haven/core/security/KeyEncryption.kt` | 65 | NOT REVIEWED | Key encryption utilities |
| 7 | `core/security/src/main/kotlin/sh/haven/core/security/Keystore.kt` | 232 | NOT REVIEWED | Android Keystore wrapper |
| 8 | `core/security/src/main/kotlin/sh/haven/core/security/OtpAuthUri.kt` | 116 | NOT REVIEWED | OTP auth URI parser |
| 9 | `core/security/src/main/kotlin/sh/haven/core/security/SshKeyGenerator.kt` | 179 | NOT REVIEWED | SSH key generator |
| 10 | `core/security/src/main/kotlin/sh/haven/core/security/Totp.kt` | 132 | NOT REVIEWED | TOTP implementation |
| 11 | `core/security/src/test/kotlin/sh/haven/core/security/AgeFileTest.kt` | 128 | NOT REVIEWED | Unit test for AgeFile |
| 12 | `core/security/src/test/kotlin/sh/haven/core/security/JwtPayloadTest.kt` | 53 | NOT REVIEWED | Unit test for JwtPayload |
| 13 | `core/security/src/test/kotlin/sh/haven/core/security/KeyEncryptionTest.kt` | 55 | NOT REVIEWED | Unit test for KeyEncryption |
| 14 | `core/security/src/test/kotlin/sh/haven/core/security/KeystoreAuditSnapshotTest.kt` | 75 | NOT REVIEWED | Unit test for KeystoreAuditSnapshot |
| 15 | `core/security/src/test/kotlin/sh/haven/core/security/OtpAuthUriTest.kt` | 62 | NOT REVIEWED | Unit test for OtpAuthUri |
| 16 | `core/security/src/test/kotlin/sh/haven/core/security/SshKeyGeneratorTest.kt` | 114 | NOT REVIEWED | Unit test for SshKeyGenerator |
| 17 | `core/security/src/test/kotlin/sh/haven/core/security/TotpTest.kt` | 106 | NOT REVIEWED | Unit test for Totp |

## core/smb

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `core/smb/src/main/kotlin/sh/haven/core/smb/SmbClient.kt` | 303 | NOT REVIEWED | Client for smb |
| 2 | `core/smb/src/main/kotlin/sh/haven/core/smb/SmbSessionManager.kt` | 182 | NOT REVIEWED | Manager for smbsession |

## core/spa

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `core/spa/src/main/kotlin/sh/haven/core/spa/FwknopPacket.kt` | 144 | NOT REVIEWED | FWKNOP packet |
| 2 | `core/spa/src/main/kotlin/sh/haven/core/spa/SpaConfig.kt` | 156 | NOT REVIEWED | Configuration model |
| 3 | `core/spa/src/main/kotlin/sh/haven/core/spa/SpaHiltModule.kt` | 15 | NOT REVIEWED | Hilt/DI module for spahilt |
| 4 | `core/spa/src/main/kotlin/sh/haven/core/spa/SpaSender.kt` | 85 | NOT REVIEWED | SPA packet sender |
| 5 | `core/spa/src/test/kotlin/sh/haven/core/spa/FwknopPacketTest.kt` | 157 | NOT REVIEWED | Unit test for FwknopPacket |
| 6 | `core/spa/src/test/kotlin/sh/haven/core/spa/SpaConfigTest.kt` | 92 | NOT REVIEWED | Unit test for SpaConfig |

## core/ssh

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/CertificateWrappedIdentity.kt` | 53 | NOT REVIEWED | Certificate-wrapped SSH identity |
| 2 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/ConnectionConfig.kt` | 230 | NOT REVIEWED | Configuration model |
| 3 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/DropbearKeyConverter.kt` | 267 | NOT REVIEWED | Dropbear key format converter |
| 4 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/DynamicForwardServer.kt` | 244 | NOT REVIEWED | Dynamic port forwarding (SOCKS) |
| 5 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/ForegroundKeepAlive.kt` | 19 | NOT REVIEWED | Foreground service keep-alive |
| 6 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/ForegroundReviveHook.kt` | 25 | NOT REVIEWED | Foreground service revive hook |
| 7 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/ForegroundSessionParticipant.kt` | 25 | NOT REVIEWED | Foreground session participant |
| 8 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/ForegroundSessionParticipantModule.kt` | 72 | NOT REVIEWED | Hilt/DI module for foregroundsessionparticipant |
| 9 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/HavenProxy.kt` | 20 | NOT REVIEWED | Haven proxy implementation |
| 10 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/HostKeyAuthFailure.kt` | 18 | NOT REVIEWED | Host key auth failure exception |
| 11 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/HostKeyVerifier.kt` | 113 | NOT REVIEWED | SSH host key verifier |
| 12 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/KeyboardInteractivePrompter.kt` | 48 | NOT REVIEWED | Keyboard-interactive auth prompter |
| 13 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/KeyboardInteractiveUserInfo.kt` | 138 | NOT REVIEWED | Keyboard-interactive user info |
| 14 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/KnownHostEntry.kt` | 68 | NOT REVIEWED | Known host entry model |
| 15 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/NetworkMonitor.kt` | 68 | NOT REVIEWED | Network connectivity monitor |
| 16 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/OpenSshCertificate.kt` | 343 | NOT REVIEWED | OpenSSH certificate model |
| 17 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/ProxyJump.kt` | 42 | NOT REVIEWED | ProxyJump (SSH jumphost) support |
| 18 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/ScpClient.kt` | 430 | NOT REVIEWED | Client for scp |
| 19 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/Session.kt` | 20 | NOT REVIEWED | Session handling for  |
| 20 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/SessionManager.kt` | 75 | NOT REVIEWED | Manager for session |
| 21 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/SessionManagerRegistry.kt` | 114 | NOT REVIEWED | Session manager registry |
| 22 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/ShellFileBrowser.kt` | 157 | NOT REVIEWED | Shell-based file browser |
| 23 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/SshCertificateParser.kt` | 339 | NOT REVIEWED | SSH certificate parser |
| 24 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/SshClient.kt` | 854 | NOT REVIEWED | Client for ssh |
| 25 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/SshConnectionService.kt` | 196 | NOT REVIEWED | SSH connection service |
| 26 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/SshIoException.kt` | 16 | NOT REVIEWED | SSH I/O exception |
| 27 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/SshKeyExporter.kt` | 160 | NOT REVIEWED | SSH key exporter |
| 28 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/SshKeyImporter.kt` | 154 | NOT REVIEWED | SSH key importer |
| 29 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/SshOptionsApplier.kt` | 142 | NOT REVIEWED | SSH options applier |
| 30 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/SshSessionManager.kt` | 1531 | NOT REVIEWED | Manager for sshsession |
| 31 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/SshVerboseLogger.kt` | 41 | NOT REVIEWED | SSH verbose logger |
| 32 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/TerminalSession.kt` | 380 | NOT REVIEWED | Session handling for terminal |
| 33 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/sftp/JschSftpSession.kt` | 207 | NOT REVIEWED | Session handling for jschsftp |
| 34 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/sftp/SftpAttrs.kt` | 25 | NOT REVIEWED | SFTP attributes |
| 35 | `core/ssh/src/main/kotlin/sh/haven/core/ssh/sftp/SftpSession.kt` | 95 | NOT REVIEWED | Session handling for sftp |
| 36 | `core/ssh/src/test/kotlin/sh/haven/core/ssh/AuthMethodContainsFidoKeyTest.kt` | 52 | NOT REVIEWED | Unit test for AuthMethodContainsFidoKey |
| 37 | `core/ssh/src/test/kotlin/sh/haven/core/ssh/ConnectionConfigTest.kt` | 311 | NOT REVIEWED | Unit test for ConnectionConfig |
| 38 | `core/ssh/src/test/kotlin/sh/haven/core/ssh/DropbearKeyConverterTest.kt` | 179 | NOT REVIEWED | Unit test for DropbearKeyConverter |
| 39 | `core/ssh/src/test/kotlin/sh/haven/core/ssh/KeyboardInteractiveUserInfoTest.kt` | 318 | NOT REVIEWED | Unit test for KeyboardInteractiveUserInfo |
| 40 | `core/ssh/src/test/kotlin/sh/haven/core/ssh/KnownHostEntryTest.kt` | 108 | NOT REVIEWED | Unit test for KnownHostEntry |
| 41 | `core/ssh/src/test/kotlin/sh/haven/core/ssh/OpenSshCertificateTest.kt` | 322 | NOT REVIEWED | Unit test for OpenSshCertificate |
| 42 | `core/ssh/src/test/kotlin/sh/haven/core/ssh/SshCertificateParserTest.kt` | 194 | NOT REVIEWED | Unit test for SshCertificateParser |
| 43 | `core/ssh/src/test/kotlin/sh/haven/core/ssh/SshClientResolveHostTest.kt` | 57 | NOT REVIEWED | Unit test for SshClientResolveHost |
| 44 | `core/ssh/src/test/kotlin/sh/haven/core/ssh/SshClientShellTest.kt` | 48 | NOT REVIEWED | Unit test for SshClientShell |
| 45 | `core/ssh/src/test/kotlin/sh/haven/core/ssh/SshClientTofuTest.kt` | 164 | NOT REVIEWED | Unit test for SshClientTofu |
| 46 | `core/ssh/src/test/kotlin/sh/haven/core/ssh/SshClientTotpAuthTest.kt` | 148 | NOT REVIEWED | Unit test for SshClientTotpAuth |
| 47 | `core/ssh/src/test/kotlin/sh/haven/core/ssh/SshKeyExporterStressTest.kt` | 68 | NOT REVIEWED | Unit test for SshKeyExporterStress |
| 48 | `core/ssh/src/test/kotlin/sh/haven/core/ssh/SshKeyExporterTest.kt` | 262 | NOT REVIEWED | Unit test for SshKeyExporter |
| 49 | `core/ssh/src/test/kotlin/sh/haven/core/ssh/SshKeyImporterTest.kt` | 288 | NOT REVIEWED | Unit test for SshKeyImporter |
| 50 | `core/ssh/src/test/kotlin/sh/haven/core/ssh/SshOptionsApplierTest.kt` | 191 | NOT REVIEWED | Unit test for SshOptionsApplier |
| 51 | `core/ssh/src/test/kotlin/sh/haven/core/ssh/SshSessionManagerShellOutcomeTest.kt` | 100 | NOT REVIEWED | Unit test for SshSessionManagerShellOutcome |
| 52 | `core/ssh/src/test/kotlin/sh/haven/core/ssh/SshSessionManagerTest.kt` | 848 | NOT REVIEWED | Unit test for SshSessionManager |
| 53 | `core/ssh/src/test/kotlin/sh/haven/core/ssh/TerminalSessionTest.kt` | 276 | NOT REVIEWED | Unit test for TerminalSession |

## core/stepca

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `core/stepca/src/main/kotlin/sh/haven/core/stepca/CaFingerprint.kt` | 83 | NOT REVIEWED | CA fingerprint model |
| 2 | `core/stepca/src/main/kotlin/sh/haven/core/stepca/CertRenewalGate.kt` | 134 | NOT REVIEWED | Certificate renewal gate |
| 3 | `core/stepca/src/main/kotlin/sh/haven/core/stepca/CertRenewalWorker.kt` | 84 | NOT REVIEWED | Certificate renewal worker |
| 4 | `core/stepca/src/main/kotlin/sh/haven/core/stepca/KeyIdBuilder.kt` | 19 | NOT REVIEWED | Key ID builder |
| 5 | `core/stepca/src/main/kotlin/sh/haven/core/stepca/OidcAuthClient.kt` | 227 | NOT REVIEWED | Client for oidcauth |
| 6 | `core/stepca/src/main/kotlin/sh/haven/core/stepca/OidcDiscovery.kt` | 61 | NOT REVIEWED | OIDC discovery |
| 7 | `core/stepca/src/main/kotlin/sh/haven/core/stepca/OidcRedirectActivity.kt` | 47 | NOT REVIEWED | OIDC redirect activity |
| 8 | `core/stepca/src/main/kotlin/sh/haven/core/stepca/OidcRedirectBus.kt` | 43 | NOT REVIEWED | OIDC redirect bus |
| 9 | `core/stepca/src/main/kotlin/sh/haven/core/stepca/PinnedTls.kt` | 50 | NOT REVIEWED | Pinned TLS configuration |
| 10 | `core/stepca/src/main/kotlin/sh/haven/core/stepca/Pkce.kt` | 31 | NOT REVIEWED | PKCE implementation |
| 11 | `core/stepca/src/main/kotlin/sh/haven/core/stepca/Provisioners.kt` | 67 | NOT REVIEWED | Step CA provisioners |
| 12 | `core/stepca/src/main/kotlin/sh/haven/core/stepca/RenewalNotifier.kt` | 93 | NOT REVIEWED | Renewal notification |
| 13 | `core/stepca/src/main/kotlin/sh/haven/core/stepca/StepCaApiClient.kt` | 392 | NOT REVIEWED | Client for stepcaapi |
| 14 | `core/stepca/src/main/kotlin/sh/haven/core/stepca/StepCaSignFlow.kt` | 64 | NOT REVIEWED | Step CA sign flow |
| 15 | `core/stepca/src/test/kotlin/sh/haven/core/stepca/CaFingerprintTest.kt` | 85 | NOT REVIEWED | Unit test for CaFingerprint |
| 16 | `core/stepca/src/test/kotlin/sh/haven/core/stepca/CertRenewalGateTest.kt` | 254 | NOT REVIEWED | Unit test for CertRenewalGate |
| 17 | `core/stepca/src/test/kotlin/sh/haven/core/stepca/KeyIdBuilderTest.kt` | 27 | NOT REVIEWED | Unit test for KeyIdBuilder |
| 18 | `core/stepca/src/test/kotlin/sh/haven/core/stepca/OidcAuthClientTest.kt` | 188 | NOT REVIEWED | Unit test for OidcAuthClient |
| 19 | `core/stepca/src/test/kotlin/sh/haven/core/stepca/OidcDiscoveryTest.kt` | 85 | NOT REVIEWED | Unit test for OidcDiscovery |
| 20 | `core/stepca/src/test/kotlin/sh/haven/core/stepca/PkceTest.kt` | 60 | NOT REVIEWED | Unit test for Pkce |
| 21 | `core/stepca/src/test/kotlin/sh/haven/core/stepca/ProvisionersTest.kt` | 78 | NOT REVIEWED | Unit test for Provisioners |
| 22 | `core/stepca/src/test/kotlin/sh/haven/core/stepca/StepCaApiClientTest.kt` | 55 | NOT REVIEWED | Unit test for StepCaApiClient |

## core/terminal-haven

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `core/terminal-haven/src/main/kotlin/sh/haven/core/terminal/HavenKeyboardMode.kt` | 86 | NOT REVIEWED | Haven keyboard mode |
| 2 | `core/terminal-haven/src/main/kotlin/sh/haven/core/terminal/HavenTerminal.kt` | 102 | NOT REVIEWED | Haven terminal implementation |

## core/toolbar

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `core/toolbar/src/main/kotlin/sh/haven/core/toolbar/KeyboardToolbar.kt` | 1975 | NOT REVIEWED | Keyboard toolbar UI |
| 2 | `core/toolbar/src/main/kotlin/sh/haven/core/toolbar/SnippetsBottomSheet.kt` | 212 | NOT REVIEWED | Snippets bottom sheet |

## core/tunnel

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `core/tunnel/src/main/kotlin/sh/haven/core/tunnel/AuthenticatedProxy.kt` | 49 | NOT REVIEWED | Authenticated proxy |
| 2 | `core/tunnel/src/main/kotlin/sh/haven/core/tunnel/CloudflareAccessConfigBlob.kt` | 86 | NOT REVIEWED | Configuration model |
| 3 | `core/tunnel/src/main/kotlin/sh/haven/core/tunnel/CloudflareAccessTunnel.kt` | 344 | NOT REVIEWED | Cloudflare Access tunnel |
| 4 | `core/tunnel/src/main/kotlin/sh/haven/core/tunnel/ProxySocketFactory.kt` | 238 | NOT REVIEWED | Proxy socket factory |
| 5 | `core/tunnel/src/main/kotlin/sh/haven/core/tunnel/TailscaleConfigBlob.kt` | 65 | NOT REVIEWED | Configuration model |
| 6 | `core/tunnel/src/main/kotlin/sh/haven/core/tunnel/TailscaleTunnel.kt` | 196 | NOT REVIEWED | Tailscale tunnel |
| 7 | `core/tunnel/src/main/kotlin/sh/haven/core/tunnel/Tunnel.kt` | 105 | NOT REVIEWED | Tunnel interface |
| 8 | `core/tunnel/src/main/kotlin/sh/haven/core/tunnel/TunnelHiltModule.kt` | 33 | NOT REVIEWED | Hilt/DI module for tunnelhilt |
| 9 | `core/tunnel/src/main/kotlin/sh/haven/core/tunnel/TunnelManager.kt` | 236 | NOT REVIEWED | Manager for tunnel |
| 10 | `core/tunnel/src/main/kotlin/sh/haven/core/tunnel/TunnelProxy.kt` | 50 | NOT REVIEWED | Tunnel proxy |
| 11 | `core/tunnel/src/main/kotlin/sh/haven/core/tunnel/TunnelResolver.kt` | 254 | NOT REVIEWED | Tunnel resolver |
| 12 | `core/tunnel/src/main/kotlin/sh/haven/core/tunnel/TunnelSocketFactory.kt` | 56 | NOT REVIEWED | Tunnel socket factory |
| 13 | `core/tunnel/src/main/kotlin/sh/haven/core/tunnel/TunneledDatagramSocket.kt` | 59 | NOT REVIEWED | Tunneled datagram socket |
| 14 | `core/tunnel/src/main/kotlin/sh/haven/core/tunnel/TunneledSocket.kt` | 233 | NOT REVIEWED | Tunneled socket |
| 15 | `core/tunnel/src/main/kotlin/sh/haven/core/tunnel/WireguardTunnel.kt` | 285 | NOT REVIEWED | WireGuard tunnel |
| 16 | `core/tunnel/src/test/kotlin/sh/haven/core/tunnel/CloudflareAccessConfigBlobTest.kt` | 89 | NOT REVIEWED | Unit test for CloudflareAccessConfigBlob |
| 17 | `core/tunnel/src/test/kotlin/sh/haven/core/tunnel/CloudflareAccessTunnelTest.kt` | 296 | NOT REVIEWED | Unit test for CloudflareAccessTunnel |
| 18 | `core/tunnel/src/test/kotlin/sh/haven/core/tunnel/ProxySocketFactoryTest.kt` | 267 | NOT REVIEWED | Unit test for ProxySocketFactory |
| 19 | `core/tunnel/src/test/kotlin/sh/haven/core/tunnel/TailscaleConfigBlobTest.kt` | 58 | NOT REVIEWED | Unit test for TailscaleConfigBlob |
| 20 | `core/tunnel/src/test/kotlin/sh/haven/core/tunnel/TunnelManagerTest.kt` | 172 | NOT REVIEWED | Unit test for TunnelManager |
| 21 | `core/tunnel/src/test/kotlin/sh/haven/core/tunnel/TunnelProxyTest.kt` | 104 | NOT REVIEWED | Unit test for TunnelProxy |
| 22 | `core/tunnel/src/test/kotlin/sh/haven/core/tunnel/TunnelResolverTest.kt` | 297 | NOT REVIEWED | Unit test for TunnelResolver |
| 23 | `core/tunnel/src/test/kotlin/sh/haven/core/tunnel/TunnelSocketFactoryTest.kt` | 76 | NOT REVIEWED | Unit test for TunnelSocketFactory |
| 24 | `core/tunnel/src/test/kotlin/sh/haven/core/tunnel/TunneledSocketTest.kt` | 114 | NOT REVIEWED | Unit test for TunneledSocket |
| 25 | `core/tunnel/src/test/kotlin/sh/haven/core/tunnel/WireguardKeepaliveTest.kt` | 100 | NOT REVIEWED | Unit test for WireguardKeepalive |

## core/ui

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `core/ui/src/main/kotlin/sh/haven/core/ui/CursorOverlay.kt` | 16 | NOT REVIEWED | Cursor overlay composable |
| 2 | `core/ui/src/main/kotlin/sh/haven/core/ui/KeyEventInterceptor.kt` | 18 | NOT REVIEWED | Key event interceptor |
| 3 | `core/ui/src/main/kotlin/sh/haven/core/ui/PasswordField.kt` | 82 | NOT REVIEWED | Password input field |
| 4 | `core/ui/src/main/kotlin/sh/haven/core/ui/SessionPickerDialog.kt` | 261 | NOT REVIEWED | Dialog component for sessionpicker |
| 5 | `core/ui/src/main/kotlin/sh/haven/core/ui/navigation/Screen.kt` | 27 | NOT REVIEWED | UI screen for  |
| 6 | `core/ui/src/main/kotlin/sh/haven/core/ui/theme/Color.kt` | 11 | NOT REVIEWED | Theme color definitions |
| 7 | `core/ui/src/main/kotlin/sh/haven/core/ui/theme/Theme.kt` | 47 | NOT REVIEWED | Material theme configuration |
| 8 | `core/ui/src/main/kotlin/sh/haven/core/ui/theme/Type.kt` | 30 | NOT REVIEWED | Typography definitions |

## core/usb

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `core/usb/src/main/kotlin/sh/haven/core/usb/UsbAccessGate.kt` | 67 | NOT REVIEWED | USB access gate |
| 2 | `core/usb/src/main/kotlin/sh/haven/core/usb/UsbBroker.kt` | 351 | NOT REVIEWED | USB device broker |
| 3 | `core/usb/src/main/kotlin/sh/haven/core/usb/UsbIpProtocol.kt` | 237 | NOT REVIEWED | USB/IP protocol |
| 4 | `core/usb/src/main/kotlin/sh/haven/core/usb/UsbIpServer.kt` | 382 | NOT REVIEWED | USB/IP server |
| 5 | `core/usb/src/main/kotlin/sh/haven/core/usb/UsbModels.kt` | 57 | NOT REVIEWED | USB data models |
| 6 | `core/usb/src/main/kotlin/sh/haven/core/usb/UsbProxyProtocol.kt` | 189 | NOT REVIEWED | USB proxy protocol |
| 7 | `core/usb/src/main/kotlin/sh/haven/core/usb/UsbProxyServer.kt` | 136 | NOT REVIEWED | USB proxy server |
| 8 | `core/usb/src/test/kotlin/sh/haven/core/usb/UsbAccessGateTest.kt` | 71 | NOT REVIEWED | Unit test for UsbAccessGate |
| 9 | `core/usb/src/test/kotlin/sh/haven/core/usb/UsbBrokerTest.kt` | 162 | NOT REVIEWED | Unit test for UsbBroker |
| 10 | `core/usb/src/test/kotlin/sh/haven/core/usb/UsbIpProtocolTest.kt` | 146 | NOT REVIEWED | Unit test for UsbIpProtocol |
| 11 | `core/usb/src/test/kotlin/sh/haven/core/usb/UsbIpServerTest.kt` | 172 | NOT REVIEWED | Unit test for UsbIpServer |
| 12 | `core/usb/src/test/kotlin/sh/haven/core/usb/UsbProxyProtocolTest.kt` | 78 | NOT REVIEWED | Unit test for UsbProxyProtocol |

## core/vnc

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `core/vnc/src/main/kotlin/sh/haven/core/vnc/VncClient.kt` | 322 | NOT REVIEWED | Client for vnc |
| 2 | `core/vnc/src/main/kotlin/sh/haven/core/vnc/VncConfig.kt` | 52 | NOT REVIEWED | Configuration model |
| 3 | `core/vnc/src/main/kotlin/sh/haven/core/vnc/VncSession.kt` | 184 | NOT REVIEWED | Session handling for vnc |
| 4 | `core/vnc/src/main/kotlin/sh/haven/core/vnc/io/CountingInputStream.kt` | 36 | NOT REVIEWED | Counting input stream |
| 5 | `core/vnc/src/main/kotlin/sh/haven/core/vnc/io/ThroughputTracker.kt` | 51 | NOT REVIEWED | Throughput tracker |
| 6 | `core/vnc/src/main/kotlin/sh/haven/core/vnc/protocol/Handshaker.kt` | 340 | NOT REVIEWED | VNC protocol handshaker |
| 7 | `core/vnc/src/main/kotlin/sh/haven/core/vnc/protocol/Initializer.kt` | 50 | NOT REVIEWED | VNC protocol initializer |
| 8 | `core/vnc/src/main/kotlin/sh/haven/core/vnc/protocol/Messages.kt` | 270 | NOT REVIEWED | VNC protocol messages |
| 9 | `core/vnc/src/main/kotlin/sh/haven/core/vnc/rendering/Framebuffer.kt` | 583 | NOT REVIEWED | VNC framebuffer |
| 10 | `core/vnc/src/test/kotlin/sh/haven/core/vnc/VncClientEventLoopTest.kt` | 64 | NOT REVIEWED | Unit test for VncClientEventLoop |
| 11 | `core/vnc/src/test/kotlin/sh/haven/core/vnc/VncSessionInputWakeTest.kt` | 81 | NOT REVIEWED | Unit test for VncSessionInputWake |
| 12 | `core/vnc/src/test/kotlin/sh/haven/core/vnc/VncSessionTest.kt` | 79 | NOT REVIEWED | Unit test for VncSession |
| 13 | `core/vnc/src/test/kotlin/sh/haven/core/vnc/rendering/FramebufferCursorTest.kt` | 92 | NOT REVIEWED | Unit test for FramebufferCursor |
| 14 | `core/vnc/src/test/kotlin/sh/haven/core/vnc/rendering/FramebufferZrleTest.kt` | 174 | NOT REVIEWED | Unit test for FramebufferZrle |

## core/wayland

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `core/wayland/src/main/kotlin/sh/haven/core/wayland/WaylandBridge.kt` | 73 | NOT REVIEWED | Wayland bridge |
| 2 | `core/wayland/src/main/kotlin/sh/haven/core/wayland/WaylandDesktopView.kt` | 756 | NOT REVIEWED | Wayland desktop view |
| 3 | `core/wayland/src/main/kotlin/sh/haven/core/wayland/WaylandToolbar.kt` | 101 | NOT REVIEWED | Wayland toolbar |

## feature/connections

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `feature/connections/src/main/kotlin/sh/haven/feature/connections/AgentActiveChip.kt` | 206 | NOT REVIEWED | Agent active chip composable |
| 2 | `feature/connections/src/main/kotlin/sh/haven/feature/connections/ConnectionEditDialog.kt` | 3812 | NOT REVIEWED | Dialog component for connectionedit |
| 3 | `feature/connections/src/main/kotlin/sh/haven/feature/connections/ConnectionsScreen.kt` | 2066 | NOT REVIEWED | UI screen for connections |
| 4 | `feature/connections/src/main/kotlin/sh/haven/feature/connections/ConnectionsViewModel.kt` | 4490 | NOT REVIEWED | ViewModel for connections |
| 5 | `feature/connections/src/main/kotlin/sh/haven/feature/connections/DeployKeyDialog.kt` | 110 | NOT REVIEWED | Dialog component for deploykey |
| 6 | `feature/connections/src/main/kotlin/sh/haven/feature/connections/EmailProviderPreset.kt` | 64 | NOT REVIEWED | Email provider presets |
| 7 | `feature/connections/src/main/kotlin/sh/haven/feature/connections/EmbeddedCloudflareTunnelInput.kt` | 25 | NOT REVIEWED | Embedded Cloudflare tunnel input |
| 8 | `feature/connections/src/main/kotlin/sh/haven/feature/connections/FidoTouchPromptDialog.kt` | 155 | NOT REVIEWED | Dialog component for fidotouchprompt |
| 9 | `feature/connections/src/main/kotlin/sh/haven/feature/connections/HostKeyDialog.kt` | 113 | NOT REVIEWED | Dialog component for hostkey |
| 10 | `feature/connections/src/main/kotlin/sh/haven/feature/connections/ImportRcloneConfigDialog.kt` | 165 | NOT REVIEWED | Dialog component for importrcloneconfig |
| 11 | `feature/connections/src/main/kotlin/sh/haven/feature/connections/KeyboardInteractiveDialog.kt` | 136 | NOT REVIEWED | Dialog component for keyboardinteractive |
| 12 | `feature/connections/src/main/kotlin/sh/haven/feature/connections/LinuxVmSetupDialog.kt` | 322 | NOT REVIEWED | Dialog component for linuxvmsetup |
| 13 | `feature/connections/src/main/kotlin/sh/haven/feature/connections/NetworkDiscovery.kt` | 519 | NOT REVIEWED | Network service discovery |
| 14 | `feature/connections/src/main/kotlin/sh/haven/feature/connections/PasswordDialog.kt` | 235 | NOT REVIEWED | Dialog component for password |
| 15 | `feature/connections/src/main/kotlin/sh/haven/feature/connections/PortForwardDialog.kt` | 586 | NOT REVIEWED | Dialog component for portforward |
| 16 | `feature/connections/src/main/kotlin/sh/haven/feature/connections/RcloneConfigViewModel.kt` | 233 | NOT REVIEWED | ViewModel for rcloneconfig |
| 17 | `feature/connections/src/main/kotlin/sh/haven/feature/connections/mosh/TunneledUdpAdapter.kt` | 38 | NOT REVIEWED | Tunneled UDP adapter for mosh |
| 18 | `feature/connections/src/main/kotlin/sh/haven/feature/connections/usb/UsbForegroundParticipantModule.kt` | 41 | NOT REVIEWED | Hilt/DI module for usbforegroundparticipant |
| 19 | `feature/connections/src/main/kotlin/sh/haven/feature/connections/usb/UsbipConnectionForwarder.kt` | 165 | NOT REVIEWED | USB/IP connection forwarder |
| 20 | `feature/connections/src/test/kotlin/sh/haven/feature/connections/ConnectionsViewModelSessionTest.kt` | 224 | NOT REVIEWED | Unit test for ConnectionsViewModelSession |
| 21 | `feature/connections/src/test/kotlin/sh/haven/feature/connections/EmailProviderPresetTest.kt` | 46 | NOT REVIEWED | Unit test for EmailProviderPreset |
| 22 | `feature/connections/src/test/kotlin/sh/haven/feature/connections/usb/UsbipConnectionForwarderTest.kt` | 121 | NOT REVIEWED | Unit test for UsbipConnectionForwarder |

## feature/editor

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `feature/editor/src/main/kotlin/sh/haven/feature/editor/EditorScreen.kt` | 418 | NOT REVIEWED | UI screen for editor |
| 2 | `feature/editor/src/main/kotlin/sh/haven/feature/editor/EditorViewModel.kt` | 80 | NOT REVIEWED | ViewModel for editor |
| 3 | `feature/editor/src/main/kotlin/sh/haven/feature/editor/FileContentProvider.kt` | 10 | NOT REVIEWED | File content provider |
| 4 | `feature/editor/src/main/kotlin/sh/haven/feature/editor/TextMateSupport.kt` | 282 | NOT REVIEWED | TextMate grammar support |

## feature/imagetools

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `feature/imagetools/src/main/kotlin/sh/haven/feature/imagetools/CropOverlay.kt` | 201 | NOT REVIEWED | Image crop overlay |
| 2 | `feature/imagetools/src/main/kotlin/sh/haven/feature/imagetools/ImageToolState.kt` | 23 | NOT REVIEWED | Image tool state |
| 3 | `feature/imagetools/src/main/kotlin/sh/haven/feature/imagetools/ImageToolsScreen.kt` | 265 | NOT REVIEWED | UI screen for imagetools |
| 4 | `feature/imagetools/src/main/kotlin/sh/haven/feature/imagetools/PerspectiveScreen.kt` | 212 | NOT REVIEWED | UI screen for perspective |
| 5 | `feature/imagetools/src/main/kotlin/sh/haven/feature/imagetools/RotateOverlay.kt` | 93 | NOT REVIEWED | Image rotate overlay |
| 6 | `feature/imagetools/src/main/kotlin/sh/haven/feature/imagetools/ZoomableImage.kt` | 48 | NOT REVIEWED | Zoomable image composable |

## feature/keys

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `feature/keys/src/main/kotlin/sh/haven/feature/keys/DiscoverFromSecurityKeyDialogs.kt` | 281 | NOT REVIEWED | Security key discovery dialogs |
| 2 | `feature/keys/src/main/kotlin/sh/haven/feature/keys/GenerateStepCaDialog.kt` | 131 | NOT REVIEWED | Dialog component for generatestepca |
| 3 | `feature/keys/src/main/kotlin/sh/haven/feature/keys/KeysScreen.kt` | 1423 | NOT REVIEWED | UI screen for keys |
| 4 | `feature/keys/src/main/kotlin/sh/haven/feature/keys/KeysViewModel.kt` | 931 | NOT REVIEWED | ViewModel for keys |
| 5 | `feature/keys/src/main/kotlin/sh/haven/feature/keys/StepCaConfigsSection.kt` | 716 | NOT REVIEWED | Step CA configs section |
| 6 | `feature/keys/src/main/kotlin/sh/haven/feature/keys/StepCaConfigsViewModel.kt` | 94 | NOT REVIEWED | ViewModel for stepcaconfigs |
| 7 | `feature/keys/src/main/kotlin/sh/haven/feature/keys/StepCaFileImport.kt` | 94 | NOT REVIEWED | Step CA file import |

## feature/mail

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `feature/mail/src/main/kotlin/sh/haven/feature/mail/AttachmentResolver.kt` | 15 | NOT REVIEWED | Mail attachment resolver |
| 2 | `feature/mail/src/main/kotlin/sh/haven/feature/mail/ComposeDrafting.kt` | 153 | NOT REVIEWED | Mail compose drafting |
| 3 | `feature/mail/src/main/kotlin/sh/haven/feature/mail/MailBackend.kt` | 73 | NOT REVIEWED | Mail backend interface |
| 4 | `feature/mail/src/main/kotlin/sh/haven/feature/mail/MailRuleCurated.kt` | 58 | NOT REVIEWED | Curated mail rules |
| 5 | `feature/mail/src/main/kotlin/sh/haven/feature/mail/MailRulesScreen.kt` | 883 | NOT REVIEWED | UI screen for mailrules |
| 6 | `feature/mail/src/main/kotlin/sh/haven/feature/mail/MailRulesViewModel.kt` | 128 | NOT REVIEWED | ViewModel for mailrules |
| 7 | `feature/mail/src/main/kotlin/sh/haven/feature/mail/MailScreen.kt` | 1020 | NOT REVIEWED | UI screen for mail |
| 8 | `feature/mail/src/main/kotlin/sh/haven/feature/mail/MailTransportSelector.kt` | 26 | NOT REVIEWED | Mail transport selector |
| 9 | `feature/mail/src/main/kotlin/sh/haven/feature/mail/MailViewModel.kt` | 640 | NOT REVIEWED | ViewModel for mail |
| 10 | `feature/mail/src/main/kotlin/sh/haven/feature/mail/MimeParser.kt` | 224 | NOT REVIEWED | MIME message parser |
| 11 | `feature/mail/src/main/kotlin/sh/haven/feature/mail/RfcMailBackend.kt` | 69 | NOT REVIEWED | RFC-compliant mail backend |
| 12 | `feature/mail/src/test/kotlin/sh/haven/feature/mail/ComposeDraftingTest.kt` | 139 | NOT REVIEWED | Unit test for ComposeDrafting |
| 13 | `feature/mail/src/test/kotlin/sh/haven/feature/mail/MailRuleCuratedTest.kt` | 105 | NOT REVIEWED | Unit test for MailRuleCurated |
| 14 | `feature/mail/src/test/kotlin/sh/haven/feature/mail/MimeParserTest.kt` | 147 | NOT REVIEWED | Unit test for MimeParser |

## feature/rdp

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `feature/rdp/src/main/kotlin/sh/haven/feature/rdp/RdpKeyMapping.kt` | 98 | NOT REVIEWED | RDP key mapping |
| 2 | `feature/rdp/src/main/kotlin/sh/haven/feature/rdp/RdpScreen.kt` | 1481 | NOT REVIEWED | UI screen for rdp |
| 3 | `feature/rdp/src/main/kotlin/sh/haven/feature/rdp/RdpViewModel.kt` | 414 | NOT REVIEWED | ViewModel for rdp |

## feature/settings

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `feature/settings/src/main/kotlin/sh/haven/feature/settings/AgentActivityScreen.kt` | 367 | NOT REVIEWED | UI screen for agentactivity |
| 2 | `feature/settings/src/main/kotlin/sh/haven/feature/settings/AgentActivityViewModel.kt` | 77 | NOT REVIEWED | ViewModel for agentactivity |
| 3 | `feature/settings/src/main/kotlin/sh/haven/feature/settings/AuditLogScreen.kt` | 286 | NOT REVIEWED | UI screen for auditlog |
| 4 | `feature/settings/src/main/kotlin/sh/haven/feature/settings/AuditLogViewModel.kt` | 93 | NOT REVIEWED | ViewModel for auditlog |
| 5 | `feature/settings/src/main/kotlin/sh/haven/feature/settings/PairedClientsScreen.kt` | 153 | NOT REVIEWED | UI screen for pairedclients |
| 6 | `feature/settings/src/main/kotlin/sh/haven/feature/settings/ProotInstallLogScreen.kt` | 239 | NOT REVIEWED | UI screen for prootinstalllog |
| 7 | `feature/settings/src/main/kotlin/sh/haven/feature/settings/ProotInstallLogViewModel.kt` | 93 | NOT REVIEWED | ViewModel for prootinstalllog |
| 8 | `feature/settings/src/main/kotlin/sh/haven/feature/settings/SettingsScreen.kt` | 3268 | NOT REVIEWED | UI screen for settings |
| 9 | `feature/settings/src/main/kotlin/sh/haven/feature/settings/SettingsViewModel.kt` | 818 | NOT REVIEWED | ViewModel for settings |

## feature/sftp

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/CompressionSection.kt` | 405 | NOT REVIEWED | SFTP compression section |
| 2 | `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/FilterSection.kt` | 342 | NOT REVIEWED | SFTP filter section |
| 3 | `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/LocalPasteIO.kt` | 83 | NOT REVIEWED | Local paste I/O |
| 4 | `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/MediaActions.kt` | 550 | NOT REVIEWED | Media actions |
| 5 | `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/SftpScreen.kt` | 3135 | NOT REVIEWED | UI screen for sftp |
| 6 | `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/SftpStreamServer.kt` | 283 | NOT REVIEWED | SFTP stream server |
| 7 | `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/SftpViewModel.kt` | 5483 | NOT REVIEWED | ViewModel for sftp |
| 8 | `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/attach/ShellQuote.kt` | 11 | NOT REVIEWED | Shell quoting utility |
| 9 | `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/attach/TerminalAttachCoordinator.kt` | 202 | NOT REVIEWED | Terminal attach coordinator |
| 10 | `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/transport/FileBackend.kt` | 103 | NOT REVIEWED | File backend interface |
| 11 | `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/transport/LocalFileBackend.kt` | 171 | NOT REVIEWED | Local file backend |
| 12 | `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/transport/RcloneFileBackend.kt` | 180 | NOT REVIEWED | Rclone file backend |
| 13 | `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/transport/RemoteFileTransport.kt` | 89 | NOT REVIEWED | Remote file transport |
| 14 | `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/transport/ReticulumFileBackend.kt` | 354 | NOT REVIEWED | Reticulum file backend |
| 15 | `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/transport/ReticulumSftpFileBackend.kt` | 55 | NOT REVIEWED | Reticulum SFTP file backend |
| 16 | `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/transport/ScpTransport.kt` | 195 | NOT REVIEWED | SCP transport |
| 17 | `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/transport/SftpTransport.kt` | 131 | NOT REVIEWED | SFTP transport |
| 18 | `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/transport/SmbFileBackend.kt` | 83 | NOT REVIEWED | SMB file backend |
| 19 | `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/transport/TransportSelector.kt` | 192 | NOT REVIEWED | Transport selector |
| 20 | `feature/sftp/src/test/kotlin/sh/haven/feature/sftp/LocalPasteIOTest.kt` | 123 | NOT REVIEWED | Unit test for LocalPasteIO |
| 21 | `feature/sftp/src/test/kotlin/sh/haven/feature/sftp/PermissionsParseTest.kt` | 55 | NOT REVIEWED | Unit test for PermissionsParse |
| 22 | `feature/sftp/src/test/kotlin/sh/haven/feature/sftp/SftpStreamServerTest.kt` | 236 | NOT REVIEWED | Unit test for SftpStreamServer |
| 23 | `feature/sftp/src/test/kotlin/sh/haven/feature/sftp/attach/ShellQuoteTest.kt` | 36 | NOT REVIEWED | Unit test for ShellQuote |
| 24 | `feature/sftp/src/test/kotlin/sh/haven/feature/sftp/transport/LocalFileBackendStreamingTest.kt` | 89 | NOT REVIEWED | Unit test for LocalFileBackendStreaming |
| 25 | `feature/sftp/src/test/kotlin/sh/haven/feature/sftp/transport/LocalFileBackendTest.kt` | 162 | NOT REVIEWED | Unit test for LocalFileBackend |
| 26 | `feature/sftp/src/test/kotlin/sh/haven/feature/sftp/transport/ReticulumFileBackendTest.kt` | 176 | NOT REVIEWED | Unit test for ReticulumFileBackend |

## feature/terminal

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `feature/terminal/src/main/kotlin/sh/haven/feature/terminal/MouseModeTracker.kt` | 142 | NOT REVIEWED | Terminal mouse mode tracker |
| 2 | `feature/terminal/src/main/kotlin/sh/haven/feature/terminal/OscHandler.kt` | 411 | NOT REVIEWED | OSC escape sequence handler |
| 3 | `feature/terminal/src/main/kotlin/sh/haven/feature/terminal/SelectionToolbar.kt` | 606 | NOT REVIEWED | Selection toolbar |
| 4 | `feature/terminal/src/main/kotlin/sh/haven/feature/terminal/TerminalNotifications.kt` | 62 | NOT REVIEWED | Terminal notifications |
| 5 | `feature/terminal/src/main/kotlin/sh/haven/feature/terminal/TerminalScreen.kt` | 1613 | NOT REVIEWED | UI screen for terminal |
| 6 | `feature/terminal/src/main/kotlin/sh/haven/feature/terminal/TerminalViewModel.kt` | 2422 | NOT REVIEWED | ViewModel for terminal |
| 7 | `feature/terminal/src/main/kotlin/sh/haven/feature/terminal/agent/TerminalSessionRegistry.kt` | 119 | NOT REVIEWED | Terminal session registry |
| 8 | `feature/terminal/src/main/kotlin/sh/haven/feature/terminal/attach/AttachOptionsSheet.kt` | 115 | NOT REVIEWED | Attach options sheet |
| 9 | `feature/terminal/src/test/kotlin/sh/haven/feature/terminal/MouseModeTrackerTest.kt` | 89 | NOT REVIEWED | Unit test for MouseModeTracker |
| 10 | `feature/terminal/src/test/kotlin/sh/haven/feature/terminal/OscHandlerTest.kt` | 541 | NOT REVIEWED | Unit test for OscHandler |
| 11 | `feature/terminal/src/test/kotlin/sh/haven/feature/terminal/SmartCopyTest.kt` | 455 | NOT REVIEWED | Unit test for SmartCopy |
| 12 | `feature/terminal/src/test/kotlin/sh/haven/feature/terminal/TerminalRecorderTest.kt` | 67 | NOT REVIEWED | Unit test for TerminalRecorder |
| 13 | `feature/terminal/src/test/kotlin/sh/haven/feature/terminal/TerminalViewModelTest.kt` | 132 | NOT REVIEWED | Unit test for TerminalViewModel |

## feature/tunnel

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `feature/tunnel/src/main/kotlin/sh/haven/feature/tunnel/CloudflareAccessLoginActivity.kt` | 230 | NOT REVIEWED | Cloudflare Access login activity |
| 2 | `feature/tunnel/src/main/kotlin/sh/haven/feature/tunnel/CloudflareAccessLoginContract.kt` | 61 | NOT REVIEWED | Cloudflare Access login contract |
| 3 | `feature/tunnel/src/main/kotlin/sh/haven/feature/tunnel/CloudflareInlineFields.kt` | 252 | NOT REVIEWED | Cloudflare inline fields |
| 4 | `feature/tunnel/src/main/kotlin/sh/haven/feature/tunnel/TunnelViewModel.kt` | 282 | NOT REVIEWED | ViewModel for tunnel |
| 5 | `feature/tunnel/src/main/kotlin/sh/haven/feature/tunnel/TunnelsScreen.kt` | 486 | NOT REVIEWED | UI screen for tunnels |

## feature/vnc

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `feature/vnc/src/main/kotlin/sh/haven/feature/vnc/VncScreen.kt` | 2039 | NOT REVIEWED | UI screen for vnc |
| 2 | `feature/vnc/src/main/kotlin/sh/haven/feature/vnc/VncViewModel.kt` | 371 | NOT REVIEWED | ViewModel for vnc |

## integration-tests

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `integration-tests/src/test/kotlin/sh/haven/integration/EtTransportTest.kt` | 329 | NOT REVIEWED | Unit test for EtTransport |
| 2 | `integration-tests/src/test/kotlin/sh/haven/integration/MoshConnectivityTest.kt` | 161 | NOT REVIEWED | Unit test for MoshConnectivity |
| 3 | `integration-tests/src/test/kotlin/sh/haven/integration/MoshTransportTest.kt` | 254 | NOT REVIEWED | Unit test for MoshTransport |
| 4 | `integration-tests/src/test/kotlin/sh/haven/integration/SessionManagerTest.kt` | 179 | NOT REVIEWED | Unit test for SessionManager |
| 5 | `integration-tests/src/test/kotlin/sh/haven/integration/TestServer.kt` | 89 | NOT REVIEWED | Unit test for Server |

## rclone-android

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `rclone-android/kotlin/sh/haven/mail/bridge/MailBridge.kt` | 111 | NOT REVIEWED | Mail bridge for rclone |
| 2 | `rclone-android/kotlin/sh/haven/rclone/bridge/RcloneBridge.kt` | 110 | NOT REVIEWED | Rclone JNI bridge |

## rdp-kotlin

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `rdp-kotlin/kotlin/sh/haven/rdp/rdp_transport.kt` | 3753 | NOT REVIEWED | RDP transport (Kotlin/JNA) |
| 2 | `rdp-kotlin/rust/build.rs` | 4 | NOT REVIEWED | Rust build script |
| 3 | `rdp-kotlin/rust/src/bin/clear-trace.rs` | 108 | NOT REVIEWED | Clear trace CLI tool |
| 4 | `rdp-kotlin/rust/src/bin/rdp-cli.rs` | 207 | NOT REVIEWED | RDP CLI tool |
| 5 | `rdp-kotlin/rust/src/bin/replay-egfx-pdu.rs` | 315 | NOT REVIEWED | Replay eGFX PDU tool |
| 6 | `rdp-kotlin/rust/src/egfx/clear.rs` | 806 | NOT REVIEWED | eGFX clear operations |
| 7 | `rdp-kotlin/rust/src/egfx/mod.rs` | 594 | NOT REVIEWED | eGFX module root |
| 8 | `rdp-kotlin/rust/src/egfx/progressive.rs` | 1028 | NOT REVIEWED | eGFX progressive codec |
| 9 | `rdp-kotlin/rust/src/egfx/surface.rs` | 329 | NOT REVIEWED | eGFX surface management |
| 10 | `rdp-kotlin/rust/src/lib.rs` | 1499 | NOT REVIEWED | IronRDP connector lib |
| 11 | `rdp-kotlin/rust/src/redirection.rs` | 328 | NOT REVIEWED | RDP redirection support |
| 12 | `rdp-kotlin/rust/uniffi-bindgen.rs` | 3 | NOT REVIEWED | UniFFI binding generator |
| 13 | `rdp-kotlin/rust/vendor/ironrdp-connector/src/channel_connection.rs` | 262 | NOT REVIEWED | IronRDP channel connection |
| 14 | `rdp-kotlin/rust/vendor/ironrdp-connector/src/connection.rs` | 803 | NOT REVIEWED | IronRDP connection |
| 15 | `rdp-kotlin/rust/vendor/ironrdp-connector/src/connection_activation.rs` | 401 | NOT REVIEWED | IronRDP connection activation |
| 16 | `rdp-kotlin/rust/vendor/ironrdp-connector/src/connection_finalization.rs` | 237 | NOT REVIEWED | IronRDP connection finalization |
| 17 | `rdp-kotlin/rust/vendor/ironrdp-connector/src/credssp.rs` | 277 | NOT REVIEWED | IronRDP CredSSP |
| 18 | `rdp-kotlin/rust/vendor/ironrdp-connector/src/legacy.rs` | 192 | NOT REVIEWED | legacy implementation |
| 19 | `rdp-kotlin/rust/vendor/ironrdp-connector/src/lib.rs` | 436 | NOT REVIEWED | IronRDP connector lib |
| 20 | `rdp-kotlin/rust/vendor/ironrdp-connector/src/license_exchange.rs` | 365 | NOT REVIEWED | IronRDP license exchange |
| 21 | `rdp-kotlin/rust/vendor/ironrdp-connector/src/macros.rs` | 38 | NOT REVIEWED | IronRDP macros |
| 22 | `rdp-kotlin/rust/vendor/ironrdp-connector/src/server_name.rs` | 52 | NOT REVIEWED | IronRDP server name |

## scratch

| # | File | Lines | Status | Description |
|---|------|-------|--------|-------------|
| 1 | `scratch/c0probe.c` | 127 | NOT REVIEWED | C probe utility 0 |
| 2 | `scratch/c1probe.c` | 179 | NOT REVIEWED | C probe utility 1 |
| 3 | `scratch/subprobe.c` | 161 | NOT REVIEWED | C sub-probe utility |
