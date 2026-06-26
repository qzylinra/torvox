# Reference Review: LetsFLUTssh

**Project path:** `/tmp/reference-projects/LetsFLUTssh/`
**Generated:** 2026-06-25
**Total source files:** 546
**Total lines:** ~296,000

---

## Summary Table

| Directory | Files | Lines | Description |
|-----------|------:|------:|-------------|
| `lib/app/` | 22 | 4,578 | App-level wiring and listeners |
| `lib/core/` | 55 | 13,600 | Core business logic |
| `lib/features/` | 52 | 22,800 | Feature modules (UI + logic) |
| `lib/l10n/` | 16 | 58,300 | Localization (auto-generated) |
| `lib/platform/` | 5 | 554 | Platform-specific code |
| `lib/providers/` | 27 | 5,766 | State providers |
| `lib/src/rust/` | 100 | 40,500 | FRB-generated Rust bindings |
| `lib/theme/` | 1 | 984 | App theming |
| `lib/utils/` | 7 | 2,165 | Utility modules |
| `lib/widgets/` | 70 | 18,400 | Reusable widget library |
| `rust/crates/lfs_core/` | 107 | 38,900 | Core Rust crate |
| `rust/crates/lfs_frb/` | 121 | 68,600 | Flutter-Rust Bridge API |
| `rust/crates/lfs_os_security/` | 46 | 18,500 | OS security integrations |
| `rust/fuzz/` | 12 | 505 | Fuzz testing targets |
| `rust_builder/cargokit/` | 16 | 1,998 | Cargo build tooling |
| `test/` | 273 | 63,600 | Flutter/Dart tests |
| **TOTAL** | **546** | **~296,000** | |

---

## 1. `lib/app/` — App-Level Wiring

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `app_toolbar.dart` | 81 | NOT REVIEWED | App toolbar widget |
| `connection_state_announcer.dart` | 102 | NOT REVIEWED | Connection state announcements |
| `credential_prompt_listener.dart` | 85 | NOT REVIEWED | Credential prompt listener |
| `deep_link_wiring.dart` | 95 | NOT REVIEWED | Deep link initialization |
| `fatal_error_app.dart` | 187 | NOT REVIEWED | Fatal error boundary |
| `global_error_dialog.dart` | 121 | NOT REVIEWED | Global error dialog |
| `hardware_vault_probe_prompt_listener.dart` | 95 | NOT REVIEWED | Hardware vault probe listener |
| `hardware_vault_seal_prompt_listener.dart` | 122 | NOT REVIEWED | Hardware vault seal listener |
| `hardware_vault_unlock_prompt_listener.dart` | 108 | NOT REVIEWED | Hardware vault unlock listener |
| `host_key_prompt_listener.dart` | 111 | NOT REVIEWED | Host key verification listener |
| `import_flow.dart` | 444 | NOT REVIEWED | Import flow orchestration |
| `keychain_probe_prompt_listener.dart` | 96 | NOT REVIEWED | Keychain probe listener |
| `navigator_key.dart` | 64 | NOT REVIEWED | Global navigator key |
| `recovery_prompt_listener.dart` | 151 | NOT REVIEWED | Recovery prompt listener |
| `security_dialog_prompter.dart` | 118 | NOT REVIEWED | Security dialog prompter |
| `security_dialogs.dart` | 58 | NOT REVIEWED | Security dialog definitions |
| `security_init_controller.dart` | 961 | NOT REVIEWED | Security init orchestrator |
| `security_init_controller_first_launch.dart` | 500 | NOT REVIEWED | First launch security flow |
| `security_init_controller_unlock.dart` | 403 | NOT REVIEWED | Unlock security flow |
| `ssh_agent_prompt_listener.dart` | 104 | NOT REVIEWED | SSH agent prompt listener |
| `tier_state_observer.dart` | 53 | NOT REVIEWED | Security tier state observer |
| `tier_unlocked_listener.dart` | 222 | NOT REVIEWED | Tier unlock listener |
| `update_dialog_flow.dart` | 215 | NOT REVIEWED | Update dialog flow |

---

## 2. `lib/core/` — Core Business Logic

### 2.1. `lib/core/bus/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `app_bus.dart` | 273 | NOT REVIEWED | Application event bus |

### 2.2. `lib/core/config/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `app_config.dart` | 567 | NOT REVIEWED | Application configuration |

### 2.3. `lib/core/connection/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `connection.dart` | 541 | NOT REVIEWED | Connection management |
| `connection_extension.dart` | 59 | NOT REVIEWED | Connection extensions |
| `connection_step.dart` | 55 | NOT REVIEWED | Connection step model |
| `connection_step_mappers.dart` | 154 | NOT REVIEWED | Connection step mappers |
| `progress_tracker.dart` | 59 | NOT REVIEWED | Progress tracking |

### 2.4. `lib/core/db/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `mappers.dart` | 178 | NOT REVIEWED | Database mappers |
| `rust_db_init.dart` | 126 | NOT REVIEWED | Rust DB initialization |

### 2.5. `lib/core/deeplink/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `deeplink_handler.dart` | 179 | NOT REVIEWED | Deep link handler |

### 2.6. `lib/core/import/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `export_import.dart` | 422 | NOT REVIEWED | Export/import logic |
| `import_service.dart` | 368 | NOT REVIEWED | Import service |
| `key_file_helper.dart` | 59 | NOT REVIEWED | Key file helper |
| `openssh_config_importer.dart` | 187 | NOT REVIEWED | OpenSSH config importer |
| `ssh_dir_key_scanner.dart` | 83 | NOT REVIEWED | SSH directory key scanner |

### 2.7. `lib/core/logs/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `log_store.dart` | 285 | NOT REVIEWED | Log storage |
| `settings_logging_parser.dart` | 58 | NOT REVIEWED | Logging settings parser |

### 2.8. `lib/core/migration/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `migration_runner.dart` | 34 | NOT REVIEWED | Migration runner |

### 2.9. `lib/core/progress/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `progress_reporter.dart` | 76 | NOT REVIEWED | Progress reporter |

### 2.10. `lib/core/s3/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `s3_fs.dart` | 120 | NOT REVIEWED | S3 filesystem |

### 2.11. `lib/core/security/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `active_dbkey.dart` | 34 | NOT REVIEWED | Active DB key |
| `backup_exclusion.dart` | 65 | NOT REVIEWED | Backup exclusion |
| `biometric_auth.dart` | 321 | NOT REVIEWED | Biometric authentication |
| `biometric_key_vault.dart` | 232 | NOT REVIEWED | Biometric key vault |
| `clipboard_secret.dart` | 139 | NOT REVIEWED | Clipboard secret handling |
| `hardware_tier.dart` | 85 | NOT REVIEWED | Hardware tier definitions |
| `hardware_tier_vault.dart` | 329 | NOT REVIEWED | Hardware tier vault |
| `kdf_params.dart` | 172 | NOT REVIEWED | Key derivation parameters |
| `keychain_password_gate.dart` | 106 | NOT REVIEWED | Keychain password gate |
| `linux_keychain_marker.dart` | 100 | NOT REVIEWED | Linux keychain marker |
| `master_password.dart` | 310 | NOT REVIEWED | Master password management |
| `password_rate_limiter.dart` | 445 | NOT REVIEWED | Password rate limiter |
| `password_strength.dart` | 43 | NOT REVIEWED | Password strength checker |
| `process_hardening.dart` | 100 | NOT REVIEWED | Process hardening |
| `secure_clipboard.dart` | 124 | NOT REVIEWED | Secure clipboard |
| `secure_key_storage.dart` | 212 | NOT REVIEWED | Secure key storage |
| `security_bootstrap.dart` | 192 | NOT REVIEWED | Security bootstrap |
| `security_tier.dart` | 177 | NOT REVIEWED | Security tier definitions |
| `session_credential_cache.dart` | 67 | NOT REVIEWED | Session credential cache |
| `session_lock_listener.dart` | 148 | NOT REVIEWED | Session lock listener |
| `ssh_key.dart` | 417 | NOT REVIEWED | SSH key management |
| `terminal_scrubber.dart` | 85 | NOT REVIEWED | Terminal output scrubbing |
| `threat_vocabulary.dart` | 237 | NOT REVIEWED | Threat vocabulary |
| `tier_unlock_attempt.dart` | 56 | NOT REVIEWED | Tier unlock attempt |
| `wipe_all_service.dart` | 222 | NOT REVIEWED | Wipe all service |

### 2.12. `lib/core/session/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `port_forwards_dao.dart` | 92 | NOT REVIEWED | Port forwards data access |
| `qr_codec.dart` | 320 | NOT REVIEWED | QR code codec |
| `qr_decoded_source.dart` | 34 | NOT REVIEWED | QR decoded source |
| `session.dart` | 799 | NOT REVIEWED | Session model |
| `session_history.dart` | 154 | NOT REVIEWED | Session history |
| `session_recorder.dart` | 433 | NOT REVIEWED | Session recording |
| `session_tree.dart` | 100 | NOT REVIEWED | Session tree |

### 2.13. `lib/core/sftp/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `errors.dart` | 59 | NOT REVIEWED | SFTP error types |
| `file_system.dart` | 147 | NOT REVIEWED | SFTP filesystem |
| `sftp_fs.dart` | 434 | NOT REVIEWED | SFTP filesystem impl |
| `sftp_models.dart` | 112 | NOT REVIEWED | SFTP models |

### 2.14. `lib/core/snippets/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `snippet.dart` | 47 | NOT REVIEWED | Snippet model |
| `snippet_template.dart` | 70 | NOT REVIEWED | Snippet template |

### 2.15. `lib/core/ssh/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `errors.dart` | 102 | NOT REVIEWED | SSH error types |
| `openssh_config_parser.dart` | 144 | NOT REVIEWED | OpenSSH config parser |
| `port_forward_rule.dart` | 130 | NOT REVIEWED | Port forward rules |
| `port_forward_runtime.dart` | 154 | NOT REVIEWED | Port forward runtime |
| `ssh_config.dart` | 159 | NOT REVIEWED | SSH config model |
| `transport/rust_transport.dart` | 233 | NOT REVIEWED | Rust-based SSH transport |
| `transport/ssh_transport.dart` | 393 | NOT REVIEWED | SSH transport abstraction |

### 2.16. `lib/core/tags/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `tag.dart` | 55 | NOT REVIEWED | Tag model |

### 2.17. `lib/core/transfer/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `conflict_resolver.dart` | 107 | NOT REVIEWED | Transfer conflict resolver |
| `transfer_task.dart` | 74 | NOT REVIEWED | Transfer task model |
| `unique_name.dart` | 42 | NOT REVIEWED | Unique name generator |

### 2.18. `lib/core/update/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `update_service.dart` | 661 | NOT REVIEWED | Update service |

### 2.19. `lib/core/webdav/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `webdav_fs.dart` | 139 | NOT REVIEWED | WebDAV filesystem |

---

## 3. `lib/features/` — Feature Modules

### 3.1. `lib/features/file_browser/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `breadcrumb_path.dart` | 42 | NOT REVIEWED | Breadcrumb path widget |
| `column_widths.dart` | 29 | NOT REVIEWED | Column width model |
| `file_browser_controller.dart` | 370 | NOT REVIEWED | File browser controller |
| `file_browser_tab.dart` | 480 | NOT REVIEWED | File browser tab |
| `file_pane.dart` | 421 | NOT REVIEWED | File pane widget |
| `file_pane_actions.dart` | 95 | NOT REVIEWED | File pane actions |
| `file_pane_dialogs.dart` | 239 | NOT REVIEWED | File pane dialogs |
| `file_pane_layout.dart` | 664 | NOT REVIEWED | File pane layout |
| `file_row.dart` | 242 | NOT REVIEWED | File row widget |
| `sftp_browser_mixin.dart` | 293 | NOT REVIEWED | SFTP browser mixin |
| `sftp_initializer.dart` | 135 | NOT REVIEWED | SFTP initializer |
| `transfer_helpers.dart` | 362 | NOT REVIEWED | Transfer helpers |
| `transfer_panel.dart` | 887 | NOT REVIEWED | Transfer panel widget |
| `transfer_panel_controller.dart` | 175 | NOT REVIEWED | Transfer panel controller |

### 3.2. `lib/features/key_manager/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `key_manager_dialog.dart` | 1,020 | NOT REVIEWED | Key manager dialog |
| `key_manager_dialog_add.dart` | 282 | NOT REVIEWED | Key add dialog |
| `key_manager_dialog_rows.dart` | 272 | NOT REVIEWED | Key manager rows |
| `key_manager_logic.dart` | 68 | NOT REVIEWED | Key manager logic |

### 3.3. `lib/features/mobile/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `mobile_file_browser.dart` | 831 | NOT REVIEWED | Mobile file browser |
| `mobile_shell.dart` | 890 | NOT REVIEWED | Mobile shell view |
| `mobile_terminal_view.dart` | 688 | NOT REVIEWED | Mobile terminal view |
| `ssh_keyboard_bar.dart` | 456 | NOT REVIEWED | SSH keyboard bar |
| `ssh_keyboard_keys.dart` | 65 | NOT REVIEWED | SSH keyboard keys |
| `terminal_copy_overlay.dart` | 254 | NOT REVIEWED | Terminal copy overlay |

### 3.4. `lib/features/recordings/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `recording_playback_dialog.dart` | 661 | NOT REVIEWED | Recording playback dialog |
| `recording_reader.dart` | 299 | NOT REVIEWED | Recording reader |
| `recordings_browser.dart` | 297 | NOT REVIEWED | Recordings browser |
| `recordings_logic.dart` | 26 | NOT REVIEWED | Recordings logic |

### 3.5. `lib/features/session_manager/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `qr_display_screen.dart` | 197 | NOT REVIEWED | QR display screen |
| `session_connect.dart` | 255 | NOT REVIEWED | Session connect flow |
| `session_details_rows.dart` | 72 | NOT REVIEWED | Session details rows |
| `session_edit_dialog.dart` | 1,163 | NOT REVIEWED | Session edit dialog |
| `session_edit_dialog_auth.dart` | 692 | NOT REVIEWED | Session edit auth tab |
| `session_edit_dialog_connection.dart` | 457 | NOT REVIEWED | Session edit connection tab |
| `session_edit_dialog_options.dart` | 432 | NOT REVIEWED | Session edit options tab |
| `session_edit_dialog_results.dart` | 154 | NOT REVIEWED | Session edit results |
| `session_forwards_logic.dart` | 66 | NOT REVIEWED | Port forwards logic |
| `session_forwards_tab.dart` | 433 | NOT REVIEWED | Port forwards tab |
| `session_panel.dart` | 674 | NOT REVIEWED | Session panel |
| `session_panel_controller.dart` | 248 | NOT REVIEWED | Session panel controller |
| `session_panel_folder_actions.dart` | 438 | NOT REVIEWED | Folder actions |
| `session_panel_session_actions.dart` | 344 | NOT REVIEWED | Session actions |
| `session_panel_widgets.dart` | 429 | NOT REVIEWED | Session panel widgets |
| `session_port_validator.dart` | 20 | NOT REVIEWED | Port validator |
| `session_save_persistence.dart` | 159 | NOT REVIEWED | Session save persistence |
| `session_tree_view.dart` | 356 | NOT REVIEWED | Session tree view |
| `session_tree_view_internals.dart` | 667 | NOT REVIEWED | Session tree internals |
| `session_via_badge.dart` | 86 | NOT REVIEWED | Session via badge |

### 3.6. `lib/features/settings/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `known_hosts_manager.dart` | 240 | NOT REVIEWED | Known hosts manager |
| `known_hosts_manager_logic.dart` | 37 | NOT REVIEWED | Known hosts logic |
| `qr_export_logic.dart` | 85 | NOT REVIEWED | QR export logic |
| `security_section_logic.dart` | 865 | NOT REVIEWED | Security section logic |
| `security_tier_switcher.dart` | 139 | NOT REVIEWED | Security tier switcher |
| `settings_dialogs.dart` | 296 | NOT REVIEWED | Settings dialogs |
| `settings_logging.dart` | 853 | NOT REVIEWED | Settings logging |
| `settings_screen.dart` | 425 | NOT REVIEWED | Settings screen |
| `settings_sections_data.dart` | 473 | NOT REVIEWED | Data settings section |
| `settings_sections_data_export_import.dart` | 766 | NOT REVIEWED | Data export/import section |
| `settings_sections_fido2_broker.dart` | 106 | NOT REVIEWED | FIDO2 broker section |
| `settings_sections_preferences.dart` | 188 | NOT REVIEWED | Preferences section |
| `settings_sections_security.dart` | 696 | NOT REVIEWED | Security section |
| `settings_sections_security_apply.dart` | 327 | NOT REVIEWED | Security apply section |
| `settings_sections_security_biometric.dart` | 301 | NOT REVIEWED | Biometric security section |
| `settings_sections_security_macos.dart` | 191 | NOT REVIEWED | macOS security section |
| `settings_sections_ssh_agent.dart` | 149 | NOT REVIEWED | SSH agent section |
| `settings_sections_sync.dart` | 462 | NOT REVIEWED | Sync settings section |
| `settings_sections_updates.dart` | 439 | NOT REVIEWED | Updates section |
| `settings_widgets.dart` | 682 | NOT REVIEWED | Settings widgets |

### 3.7. `lib/features/snippets/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `snippet_manager_dialog.dart` | 413 | NOT REVIEWED | Snippet manager dialog |
| `snippet_picker.dart` | 300 | NOT REVIEWED | Snippet picker |
| `snippets_logic.dart` | 23 | NOT REVIEWED | Snippets logic |

### 3.8. `lib/features/tabs/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `tab_model.dart` | 42 | NOT REVIEWED | Tab model |
| `welcome_screen.dart` | 49 | NOT REVIEWED | Welcome screen |

### 3.9. `lib/features/tags/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `tag_assign_dialog.dart` | 315 | NOT REVIEWED | Tag assign dialog |
| `tag_manager_dialog.dart` | 247 | NOT REVIEWED | Tag manager dialog |
| `tags_logic.dart` | 35 | NOT REVIEWED | Tags logic |

### 3.10. `lib/features/terminal/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `broadcast_controller.dart` | 181 | NOT REVIEWED | Broadcast controller |
| `pane_recording_registry.dart` | 55 | NOT REVIEWED | Pane recording registry |
| `split_node.dart` | 90 | NOT REVIEWED | Split node model |
| `terminal_pane.dart` | 821 | NOT REVIEWED | Terminal pane widget |
| `terminal_tab.dart` | 168 | NOT REVIEWED | Terminal tab |
| `tiling_view.dart` | 174 | NOT REVIEWED | Tiling view |

### 3.11. `lib/features/tools/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `tools_dialog.dart` | 60 | NOT REVIEWED | Tools dialog |
| `tools_screen.dart` | 69 | NOT REVIEWED | Tools screen |

### 3.12. `lib/features/workspace/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `drop_zone_overlay.dart` | 152 | NOT REVIEWED | Drop zone overlay |
| `panel_tab_bar.dart` | 385 | NOT REVIEWED | Panel tab bar |
| `workspace_controller.dart` | 576 | NOT REVIEWED | Workspace controller |
| `workspace_drop_logic.dart` | 116 | NOT REVIEWED | Workspace drop logic |
| `workspace_node.dart` | 181 | NOT REVIEWED | Workspace node |
| `workspace_view.dart` | 953 | NOT REVIEWED | Workspace view |

---

## 4. `lib/l10n/` — Localization

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `app_localizations.dart` | 6,523 | NOT REVIEWED | Localization base |
| `app_localizations_ar.dart` | 3,684 | NOT REVIEWED | Arabic |
| `app_localizations_de.dart` | 3,767 | NOT REVIEWED | German |
| `app_localizations_en.dart` | 3,717 | NOT REVIEWED | English |
| `app_localizations_es.dart` | 3,769 | NOT REVIEWED | Spanish |
| `app_localizations_fa.dart` | 3,671 | NOT REVIEWED | Persian |
| `app_localizations_fr.dart` | 3,784 | NOT REVIEWED | French |
| `app_localizations_hi.dart` | 3,682 | NOT REVIEWED | Hindi |
| `app_localizations_id.dart` | 3,680 | NOT REVIEWED | Indonesian |
| `app_localizations_ja.dart` | 3,583 | NOT REVIEWED | Japanese |
| `app_localizations_ko.dart` | 3,578 | NOT REVIEWED | Korean |
| `app_localizations_pt.dart` | 3,756 | NOT REVIEWED | Portuguese |
| `app_localizations_ru.dart` | 3,708 | NOT REVIEWED | Russian |
| `app_localizations_tr.dart` | 3,684 | NOT REVIEWED | Turkish |
| `app_localizations_vi.dart` | 3,667 | NOT REVIEWED | Vietnamese |
| `app_localizations_zh.dart` | 3,541 | NOT REVIEWED | Chinese |

---

## 5. `lib/platform/` — Platform Code

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `android/apk_installer.dart` | 39 | NOT REVIEWED | APK installer |
| `android_storage_permission.dart` | 67 | NOT REVIEWED | Android storage permissions |
| `foreground_service.dart` | 221 | NOT REVIEWED | Foreground service |
| `local_fs.dart` | 191 | NOT REVIEWED | Local filesystem |
| `qr_scanner.dart` | 36 | NOT REVIEWED | QR scanner |

---

## 6. `lib/providers/` — State Providers

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `auto_lock_provider.dart` | 69 | NOT REVIEWED | Auto-lock provider |
| `broadcast_provider.dart` | 21 | NOT REVIEWED | Broadcast provider |
| `config_provider.dart` | 327 | NOT REVIEWED | Config provider |
| `connection_provider.dart` | 258 | NOT REVIEWED | Connection provider |
| `connections_notifier.dart` | 770 | NOT REVIEWED | Connections notifier |
| `connections_notifier_auth.dart` | 284 | NOT REVIEWED | Connections auth notifier |
| `first_launch_banner_provider.dart` | 46 | NOT REVIEWED | First launch banner |
| `focused_pane_provider.dart` | 25 | NOT REVIEWED | Focused pane provider |
| `key_provider.dart` | 325 | NOT REVIEWED | Key provider |
| `known_hosts_provider.dart` | 294 | NOT REVIEWED | Known hosts provider |
| `locale_provider.dart` | 32 | NOT REVIEWED | Locale provider |
| `lock_state.dart` | 150 | NOT REVIEWED | Lock state |
| `log_store_provider.dart` | 10 | NOT REVIEWED | Log store provider |
| `master_password_provider.dart` | 8 | NOT REVIEWED | Master password provider |
| `security_provider.dart` | 447 | NOT REVIEWED | Security provider |
| `security_reinit_provider.dart` | 55 | NOT REVIEWED | Security reinit provider |
| `session_credential_cache_provider.dart` | 29 | NOT REVIEWED | Session credential cache |
| `session_provider.dart` | 924 | NOT REVIEWED | Session provider |
| `snippet_provider.dart` | 179 | NOT REVIEWED | Snippet provider |
| `sync_provider.dart` | 107 | NOT REVIEWED | Sync provider |
| `tag_provider.dart` | 191 | NOT REVIEWED | Tag provider |
| `theme_provider.dart` | 19 | NOT REVIEWED | Theme provider |
| `transfer_provider.dart` | 383 | NOT REVIEWED | Transfer provider |
| `update_provider.dart` | 299 | NOT REVIEWED | Update provider |
| `version_provider.dart` | 20 | NOT REVIEWED | Version provider |

---

## 7. `lib/src/rust/` — FRB-Generated Rust Bindings

### 7.1. `lib/src/rust/api/` — Generated API Bindings (100 files)

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `api.dart` | 12 | NOT REVIEWED | API barrel |
| `app.dart` | 177 | NOT REVIEWED | App API |
| `app.freezed.dart` | — | NOT REVIEWED | (freezed) |
| `archive.dart` | 689 | NOT REVIEWED | Archive API |
| `archive_stage.dart` | 324 | NOT REVIEWED | Archive stage API |
| `auth_compose.dart` | 202 | NOT REVIEWED | Auth compose API |
| `biometric_key_vault.dart` | 36 | NOT REVIEWED | Biometric key vault API |
| `bus.dart` | 678 | NOT REVIEWED | Bus API |
| `capabilities_orchestrator.dart` | 87 | NOT REVIEWED | Capabilities orchestrator API |
| `config.dart` | 429 | NOT REVIEWED | Config API |
| `connection.dart` | 42 | NOT REVIEWED | Connection API |
| `credential_prompt.dart` | 41 | NOT REVIEWED | Credential prompt API |
| `crypto.dart` | 182 | NOT REVIEWED | Crypto API |
| `db.dart` | 1,838 | NOT REVIEWED | Database API |
| `deeplink.dart` | 92 | NOT REVIEWED | Deeplink API |
| `enclave.dart` | 157 | NOT REVIEWED | Enclave API |
| `fido2.dart` | 177 | NOT REVIEWED | FIDO2 API |
| `file_clipboard.dart` | 86 | NOT REVIEWED | File clipboard API |
| `folder_path.dart` | 65 | NOT REVIEWED | Folder path API |
| `format.dart` | 88 | NOT REVIEWED | Format API |
| `forward.dart` | 386 | NOT REVIEWED | Forward API |
| `fprintd.dart` | 22 | NOT REVIEWED | Fprintd API |
| `frb_err.dart` | 114 | NOT REVIEWED | FRB error types |
| `hardware_tier_vault.dart` | 250 | NOT REVIEWED | Hardware tier vault API |
| `hello.dart` | 169 | NOT REVIEWED | Hello API |
| `host_info.dart` | 19 | NOT REVIEWED | Host info API |
| `installer.dart` | 107 | NOT REVIEWED | Installer API |
| `keychain_marker.dart` | 25 | NOT REVIEWED | Keychain marker API |
| `keychain_password_gate.dart` | 88 | NOT REVIEWED | Keychain password gate API |
| `keychain_password_gate_actor.dart` | 76 | NOT REVIEWED | Keychain password gate actor |
| `keys.dart` | 272 | NOT REVIEWED | Keys API |
| `keystore_ssh.dart` | 170 | NOT REVIEWED | Keystore SSH API |
| `known_hosts_parser.dart` | 39 | NOT REVIEWED | Known hosts parser API |
| `local_fs.dart` | 178 | NOT REVIEWED | Local FS API |
| `log_sanitize.dart` | 21 | NOT REVIEWED | Log sanitize API |
| `logger.dart` | 107 | NOT REVIEWED | Logger API |
| `macos_installer.dart` | 44 | NOT REVIEWED | macOS installer API |
| `macos_resign.dart` | 45 | NOT REVIEWED | macOS resign API |
| `master_password.dart` | 226 | NOT REVIEWED | Master password API |
| `migration.dart` | 146 | NOT REVIEWED | Migration API |
| `openssh_config_import.dart` | 180 | NOT REVIEWED | OpenSSH config import API |
| `os_security.dart` | 181 | NOT REVIEWED | OS security API |
| `password_strength.dart` | 16 | NOT REVIEWED | Password strength API |
| `path.dart` | 106 | NOT REVIEWED | Path API |
| `persisted_rate_limit_actor.dart` | 69 | NOT REVIEWED | Rate limit actor API |
| `pkcs11.dart` | 322 | NOT REVIEWED | PKCS#11 API |
| `qr_codec_encode.dart` | 197 | NOT REVIEWED | QR codec encode API |
| `qr_compose.dart` | 22 | NOT REVIEWED | QR compose API |
| `rate_limit.dart` | 66 | NOT REVIEWED | Rate limit API |
| `recorder.dart` | 638 | NOT REVIEWED | Recorder API |
| `recovery.dart` | 202 | NOT REVIEWED | Recovery API |
| `s3.dart` | 217 | NOT REVIEWED | S3 API |
| `secure_key_storage.dart` | 152 | NOT REVIEWED | Secure key storage API |
| `security_capabilities.dart` | 115 | NOT REVIEWED | Security capabilities API |
| `security_config.dart` | 162 | NOT REVIEWED | Security config API |
| `session_history.dart` | 85 | NOT REVIEWED | Session history API |
| `session_tree.dart` | 81 | NOT REVIEWED | Session tree API |
| `sessions.dart` | 626 | NOT REVIEWED | Sessions API |
| `sessions_registry.dart` | 116 | NOT REVIEWED | Sessions registry API |
| `sftp.dart` | 327 | NOT REVIEWED | SFTP API |
| `sftp_models.dart` | 108 | NOT REVIEWED | SFTP models API |
| `snippet_template.dart` | 52 | NOT REVIEWED | Snippet template API |
| `ssh.dart` | 412 | NOT REVIEWED | SSH API |
| `ssh_agent.dart` | 122 | NOT REVIEWED | SSH agent API |
| `ssh_config.dart` | 159 | NOT REVIEWED | SSH config API |
| `ssh_dir_scan.dart` | 36 | NOT REVIEWED | SSH dir scan API |
| `sync.dart` | 230 | NOT REVIEWED | Sync API |
| `terminal.dart` | 709 | NOT REVIEWED | Terminal API |
| `test_hooks.dart` | 78 | NOT REVIEWED | Test hooks API |
| `threat_eval.dart` | 50 | NOT REVIEWED | Threat eval API |
| `tier_machine.dart` | 84 | NOT REVIEWED | Tier machine API |
| `tier_transition_marker.dart` | 24 | NOT REVIEWED | Tier transition marker API |
| `tier_unlock_orchestrator.dart` | 302 | NOT REVIEWED | Tier unlock orchestrator API |
| `tpm.dart` | 88 | NOT REVIEWED | TPM API |
| `tpm_ssh.dart` | 286 | NOT REVIEWED | TPM SSH API |
| `transfer.dart` | 123 | NOT REVIEWED | Transfer API |
| `transfer_conflict.dart` | 51 | NOT REVIEWED | Transfer conflict API |
| `update_http.dart` | 212 | NOT REVIEWED | Update HTTP API |
| `update_metadata.dart` | 140 | NOT REVIEWED | Update metadata API |
| `update_signing.dart` | 20 | NOT REVIEWED | Update signing API |
| `webdav.dart` | 200 | NOT REVIEWED | WebDAV API |
| `winbio.dart` | 10 | NOT REVIEWED | Windows biometric API |
| `wipe.dart` | 46 | NOT REVIEWED | Wipe API |
| `wipe_keychain.dart` | 90 | NOT REVIEWED | Wipe keychain API |
| `wizard_setup.dart` | 71 | NOT REVIEWED | Wizard setup API |

### 7.2. `lib/src/rust/` — Generated Bridge Files

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `frb_generated.dart` | 45,604 | NOT REVIEWED | FRB generated main |
| `frb_generated.io.dart` | 6,762 | NOT REVIEWED | FRB IO bindings |
| `frb_generated.web.dart` | 6,682 | NOT REVIEWED | FRB web bindings |

---

## 8. `lib/theme/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `app_theme.dart` | 984 | NOT REVIEWED | App theme definitions |

---

## 9. `lib/utils/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `file_utils.dart` | 60 | NOT REVIEWED | File utilities |
| `format.dart` | 605 | NOT REVIEWED | Formatting utilities |
| `logger.dart` | 914 | NOT REVIEWED | Logger |
| `platform.dart` | 116 | NOT REVIEWED | Platform utilities |
| `sanitize.dart` | 222 | NOT REVIEWED | Sanitization |
| `secret_controller.dart` | 28 | NOT REVIEWED | Secret controller |
| `terminal_clipboard.dart` | 180 | NOT REVIEWED | Terminal clipboard |

---

## 10. `lib/widgets/` — Reusable Widget Library

### 10.1. `lib/widgets/core/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `app_bordered_box.dart` | 55 | NOT REVIEWED | Bordered box widget |
| `app_button.dart` | 389 | NOT REVIEWED | Button widget |
| `app_collection_panel.dart` | 132 | NOT REVIEWED | Collection panel |
| `app_collection_toolbar.dart` | 203 | NOT REVIEWED | Collection toolbar |
| `app_data_row.dart` | 165 | NOT REVIEWED | Data row widget |
| `app_data_search_bar.dart` | 95 | NOT REVIEWED | Data search bar |
| `app_dialog.dart` | 374 | NOT REVIEWED | Dialog widget |
| `app_divider.dart` | 43 | NOT REVIEWED | Divider widget |
| `app_empty_state.dart` | 78 | NOT REVIEWED | Empty state widget |
| `app_icon_button.dart` | 130 | NOT REVIEWED | Icon button |
| `app_info_button.dart` | 50 | NOT REVIEWED | Info button |
| `app_info_dialog.dart` | 166 | NOT REVIEWED | Info dialog |
| `app_picker_chip.dart` | 104 | NOT REVIEWED | Picker chip |
| `app_popup_select.dart` | 146 | NOT REVIEWED | Popup select |
| `app_selection_area.dart` | 39 | NOT REVIEWED | Selection area |
| `app_shell.dart` | 153 | NOT REVIEWED | App shell |
| `clipped_row.dart` | 88 | NOT REVIEWED | Clipped row |
| `column_resize_handle.dart` | 37 | NOT REVIEWED | Column resize handle |
| `confirm_dialog.dart` | 64 | NOT REVIEWED | Confirm dialog |
| `context_menu.dart` | 478 | NOT REVIEWED | Context menu |
| `data_checkboxes.dart` | 200 | NOT REVIEWED | Data checkboxes |
| `dropdown_select_button.dart` | 77 | NOT REVIEWED | Dropdown select |
| `error_state.dart` | 67 | NOT REVIEWED | Error state |
| `form_submit_chain.dart` | 78 | NOT REVIEWED | Form submit chain |
| `hover_region.dart` | 116 | NOT REVIEWED | Hover region |
| `marquee_mixin.dart` | 262 | NOT REVIEWED | Marquee mixin |
| `mobile_selection_bar.dart` | 88 | NOT REVIEWED | Mobile selection bar |
| `mode_button.dart` | 59 | NOT REVIEWED | Mode button |
| `session_kind_icon.dart` | 34 | NOT REVIEWED | Session kind icon |
| `shortcut_registry.dart` | 251 | NOT REVIEWED | Shortcut registry |
| `sidebar_nav_dialog.dart` | 226 | NOT REVIEWED | Sidebar nav dialog |
| `sortable_header_cell.dart` | 96 | NOT REVIEWED | Sortable header cell |
| `split_view.dart` | 74 | NOT REVIEWED | Split view |
| `status_indicator.dart` | 55 | NOT REVIEWED | Status indicator |
| `styled_form_field.dart` | 212 | NOT REVIEWED | Styled form field |
| `tag_color.dart` | 26 | NOT REVIEWED | Tag color |
| `tag_dots.dart` | 79 | NOT REVIEWED | Tag dots |
| `threshold_draggable.dart` | 100 | NOT REVIEWED | Threshold draggable |
| `toast.dart` | 214 | NOT REVIEWED | Toast widget |
| `typed_name_confirm_dialog.dart` | 121 | NOT REVIEWED | Typed name confirm |

### 10.2. `lib/widgets/import_export/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `file_conflict_dialog.dart` | 132 | NOT REVIEWED | File conflict dialog |
| `import_preview_dialog.dart` | 292 | NOT REVIEWED | Import preview dialog |
| `lfs_import_dialog.dart` | 162 | NOT REVIEWED | LFS import dialog |
| `lfs_import_preview_dialog.dart` | 105 | NOT REVIEWED | LFS import preview |
| `link_import_preview_dialog.dart` | 91 | NOT REVIEWED | Link import preview |
| `local_directory_picker.dart` | 184 | NOT REVIEWED | Local directory picker |
| `paste_import_link_dialog.dart` | 189 | NOT REVIEWED | Paste import link dialog |
| `ssh_dir_import_dialog.dart` | 550 | NOT REVIEWED | SSH dir import dialog |
| `unified_export_controller.dart` | 657 | NOT REVIEWED | Unified export controller |
| `unified_export_dialog.dart` | 409 | NOT REVIEWED | Unified export dialog |
| `unified_export_dialog_tree.dart` | 139 | NOT REVIEWED | Export dialog tree |
| `unified_export_models.dart` | 57 | NOT REVIEWED | Export models |

### 10.3. `lib/widgets/security/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `auto_lock_detector.dart` | 365 | NOT REVIEWED | Auto-lock detector |
| `credential_prompt_dialog.dart` | 199 | NOT REVIEWED | Credential prompt dialog |
| `db_corrupt_dialog.dart` | 83 | NOT REVIEWED | DB corrupt dialog |
| `expandable_tier_card.dart` | 663 | NOT REVIEWED | Expandable tier card |
| `expandable_tier_card_header.dart` | 143 | NOT REVIEWED | Tier card header |
| `expandable_tier_card_inputs.dart` | 143 | NOT REVIEWED | Tier card inputs |
| `expandable_tier_card_logic.dart` | 114 | NOT REVIEWED | Tier card logic |
| `expandable_tier_card_threats.dart` | 184 | NOT REVIEWED | Tier card threats |
| `first_launch_security_toast.dart` | 200 | NOT REVIEWED | First launch toast |
| `lock_screen.dart` | 227 | NOT REVIEWED | Lock screen |
| `password_strength_meter.dart` | 107 | NOT REVIEWED | Password strength meter |
| `secure_password_field.dart` | 161 | NOT REVIEWED | Secure password field |
| `secure_screen_scope.dart` | 72 | NOT REVIEWED | Secure screen scope |
| `security_comparison_table.dart` | 292 | NOT REVIEWED | Security comparison table |
| `security_setup_dialog.dart` | 699 | NOT REVIEWED | Security setup dialog |
| `security_setup_dialog_logic.dart` | 86 | NOT REVIEWED | Setup dialog logic |
| `security_setup_dialog_widgets.dart` | 392 | NOT REVIEWED | Setup dialog widgets |
| `security_threat_list.dart` | 129 | NOT REVIEWED | Security threat list |
| `tier_reset_dialog.dart` | 82 | NOT REVIEWED | Tier reset dialog |
| `tier_secret_unlock_dialog.dart` | 489 | NOT REVIEWED | Tier secret unlock dialog |
| `unlock_dialog.dart` | 359 | NOT REVIEWED | Unlock dialog |

### 10.4. `lib/widgets/ssh_keys/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `agent_signature_request_dialog.dart` | 100 | NOT REVIEWED | Agent signature request |
| `enclave_ssh_dialog.dart` | 324 | NOT REVIEWED | Enclave SSH dialog |
| `hardware_key_badge.dart` | 137 | NOT REVIEWED | Hardware key badge |
| `hardware_key_prompt_dialog.dart` | 132 | NOT REVIEWED | Hardware key prompt |
| `hardware_key_wizard.dart` | 305 | NOT REVIEWED | Hardware key wizard |
| `hello_ssh_dialog.dart` | 356 | NOT REVIEWED | Hello SSH dialog |
| `host_key_dialog.dart` | 225 | NOT REVIEWED | Host key dialog |
| `keystore_ssh_dialog.dart` | 437 | NOT REVIEWED | Keystore SSH dialog |
| `pkcs11_import_dialog.dart` | 966 | NOT REVIEWED | PKCS#11 import dialog |
| `pkcs11_import_dialog_logic.dart` | 108 | NOT REVIEWED | PKCS#11 import logic |
| `tpm_ssh_dialog.dart` | 526 | NOT REVIEWED | TPM SSH dialog |

### 10.5. `lib/widgets/terminal/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `connection_progress.dart` | 91 | NOT REVIEWED | Connection progress |
| `progress_writer.dart` | 110 | NOT REVIEWED | Progress writer |
| `terminal_cell_flags.dart` | 76 | NOT REVIEWED | Terminal cell flags |
| `terminal_cell_metrics.dart` | 79 | NOT REVIEWED | Terminal cell metrics |
| `terminal_controller.dart` | 339 | NOT REVIEWED | Terminal controller |
| `terminal_grid_painter.dart` | 363 | NOT REVIEWED | Terminal grid painter |
| `terminal_key_input.dart` | 168 | NOT REVIEWED | Terminal key input |
| `terminal_palette_theme.dart` | 54 | NOT REVIEWED | Terminal palette theme |
| `terminal_pointer_input.dart` | 228 | NOT REVIEWED | Terminal pointer input |
| `terminal_search_bar.dart` | 141 | NOT REVIEWED | Terminal search bar |
| `terminal_view.dart` | 797 | NOT REVIEWED | Terminal view |
| `update_progress_indicator.dart` | 73 | NOT REVIEWED | Update progress indicator |

---

## 11. `rust/crates/lfs_core/` — Core Rust Crate

### 11.1. `src/` — Source (107 files)

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `lib.rs` | 101 | NOT REVIEWED | Crate root |
| `archive.rs` | 1,651 | NOT REVIEWED | Archive handling |
| `archive_stage.rs` | 324 | NOT REVIEWED | Archive staging |
| `archive_tree.rs` | 328 | NOT REVIEWED | Archive tree |
| `auth_compose.rs` | 712 | NOT REVIEWED | Auth composition |
| `capabilities.rs` | 1,123 | NOT REVIEWED | Capabilities |
| `connection.rs` | 1,052 | NOT REVIEWED | Connection management |
| `credential_prompt.rs` | 67 | NOT REVIEWED | Credential prompt |
| `db.rs` | 2,220 | NOT REVIEWED | Database layer |
| `deeplink.rs` | 335 | NOT REVIEWED | Deeplink handling |
| `enclave.rs` | 248 | NOT REVIEWED | Enclave support |
| `errors.rs` | 85 | NOT REVIEWED | Error types |
| `file_clipboard.rs` | 79 | NOT REVIEWED | File clipboard |
| `folder_path.rs` | 201 | NOT REVIEWED | Folder path |
| `known_hosts_parser.rs` | 297 | NOT REVIEWED | Known hosts parser |
| `local_fs.rs` | 380 | NOT REVIEWED | Local filesystem |
| `log_sanitize.rs` | 94 | NOT REVIEWED | Log sanitization |
| `logger.rs` | 227 | NOT REVIEWED | Logger |
| `openssh_config.rs` | 496 | NOT REVIEWED | OpenSSH config parser |
| `openssh_key_import.rs` | 551 | NOT REVIEWED | OpenSSH key import |
| `pem_certs.rs` | 202 | NOT REVIEWED | PEM certificate handling |
| `pkcs11_uri.rs` | 306 | NOT REVIEWED | PKCS#11 URI |
| `portforward/mod.rs` | 456 | NOT REVIEWED | Port forwarding |
| `qr_codec_decode.rs` | 295 | NOT REVIEWED | QR decode |
| `qr_codec_encode.rs` | 388 | NOT REVIEWED | QR encode |
| `rate_limit.rs` | 202 | NOT REVIEWED | Rate limiting |
| `recorder/` | 7 files | NOT REVIEWED | Session recording subsystem |
| `s3/` | 5 files | NOT REVIEWED | S3 storage subsystem |
| `secrets.rs` | 147 | NOT REVIEWED | Secrets management |
| `security/` | 30 files | NOT REVIEWED | Security subsystem (biometric, keychain, tiers, wipe) |
| `session_history.rs` | 194 | NOT REVIEWED | Session history |
| `session_json.rs` | 310 | NOT REVIEWED | Session JSON serialization |
| `session_tree.rs` | 214 | NOT REVIEWED | Session tree |
| `sessions.rs` | 1,228 | NOT REVIEWED | Sessions management |
| `sftp/` | 1 file | NOT REVIEWED | SFTP module |
| `sftp_models.rs` | 305 | NOT REVIEWED | SFTP models |
| `snippet_template.rs` | 237 | NOT REVIEWED | Snippet templates |
| `ssh/` | 12 files | NOT REVIEWED | SSH subsystem (signers, wire, session connect) |
| `ssh_agent/` | 8 files | NOT REVIEWED | SSH agent subsystem |
| `ssh_config.rs` | 1,139 | NOT REVIEWED | SSH config parser |
| `ssh_dir_scan.rs` | 197 | NOT REVIEWED | SSH directory scanner |
| `storage/` | 5 files | NOT REVIEWED | Storage backends |
| `sync/` | 3 files | NOT REVIEWED | Sync subsystem |
| `terminal/` | 5 files | NOT REVIEWED | Terminal emulator |
| `threat_eval.rs` | 250 | NOT REVIEWED | Threat evaluation |
| `transfer/` | 2 files | NOT REVIEWED | Transfer subsystem |
| `transfer_conflict.rs` | 285 | NOT REVIEWED | Transfer conflict |
| `update/` | 6 files | NOT REVIEWED | Update subsystem |
| `webdav/` | 4 files | NOT REVIEWED | WebDAV subsystem |
| `xml.rs` | 21 | NOT REVIEWED | XML helpers |

### 11.2. `tests/` — Rust Integration Tests

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `sk_signer_test.rs` | 198 | NOT REVIEWED | SK signer test |
| `ssh_agent_endpoint_test.rs` | 157 | NOT REVIEWED | SSH agent endpoint test |
| `webdav_tls_pin_test.rs` | 197 | NOT REVIEWED | WebDAV TLS pin test |

---

## 12. `rust/crates/lfs_frb/` — Flutter-Rust Bridge API

### 12.1. `src/api/` — API Definitions (121 files)

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `api.rs` | 114 | NOT REVIEWED | API barrel |
| `app.rs` | 405 | NOT REVIEWED | App API |
| `archive.rs` | 828 | NOT REVIEWED | Archive API |
| `archive_stage.rs` | 309 | NOT REVIEWED | Archive stage API |
| `auth_compose.rs` | 453 | NOT REVIEWED | Auth compose API |
| `biometric_key_vault.rs` | 154 | NOT REVIEWED | Biometric key vault API |
| `bus.rs` | 1,305 | NOT REVIEWED | Bus API |
| `capabilities_orchestrator.rs` | 132 | NOT REVIEWED | Capabilities orchestrator |
| `config.rs` | 673 | NOT REVIEWED | Config API |
| `connection.rs` | 216 | NOT REVIEWED | Connection API |
| `credential_prompt.rs` | 84 | NOT REVIEWED | Credential prompt |
| `crypto.rs` | 267 | NOT REVIEWED | Crypto API |
| `db.rs` | 2,386 | NOT REVIEWED | Database API |
| `deeplink.rs` | 161 | NOT REVIEWED | Deeplink API |
| `enclave.rs` | 341 | NOT REVIEWED | Enclave API |
| `fido2.rs` | 168 | NOT REVIEWED | FIDO2 API |
| `file_clipboard.rs` | 95 | NOT REVIEWED | File clipboard API |
| `folder_path.rs` | 185 | NOT REVIEWED | Folder path API |
| `format.rs` | 139 | NOT REVIEWED | Format API |
| `forward.rs` | 604 | NOT REVIEWED | Forward API |
| `fprintd.rs` | 103 | NOT REVIEWED | Fprintd API |
| `frb_err.rs` | 628 | NOT REVIEWED | FRB error types |
| `hardware_tier_vault.rs` | 807 | NOT REVIEWED | Hardware tier vault |
| `hello.rs` | 379 | NOT REVIEWED | Hello API |
| `host_info.rs` | 58 | NOT REVIEWED | Host info |
| `installer.rs` | 152 | NOT REVIEWED | Installer |
| `keychain_marker.rs` | 41 | NOT REVIEWED | Keychain marker |
| `keychain_password_gate.rs` | 133 | NOT REVIEWED | Keychain password gate |
| `keychain_password_gate_actor.rs` | 91 | NOT REVIEWED | Keychain password gate actor |
| `keys.rs` | 382 | NOT REVIEWED | Keys API |
| `keystore_ssh.rs` | 394 | NOT REVIEWED | Keystore SSH |
| `known_hosts_parser.rs` | 57 | NOT REVIEWED | Known hosts parser |
| `local_fs.rs` | 249 | NOT REVIEWED | Local FS |
| `log_sanitize.rs` | 72 | NOT REVIEWED | Log sanitize |
| `logger.rs` | 164 | NOT REVIEWED | Logger |
| `macos_installer.rs` | 117 | NOT REVIEWED | macOS installer |
| `macos_resign.rs` | 168 | NOT REVIEWED | macOS resign |
| `master_password.rs` | 521 | NOT REVIEWED | Master password |
| `migration.rs` | 185 | NOT REVIEWED | Migration |
| `openssh_config_import.rs` | 219 | NOT REVIEWED | OpenSSH config import |
| `os_security.rs` | 282 | NOT REVIEWED | OS security |
| `password_strength.rs` | 90 | NOT REVIEWED | Password strength |
| `path.rs` | 235 | NOT REVIEWED | Path API |
| `persisted_rate_limit_actor.rs` | 106 | NOT REVIEWED | Rate limit actor |
| `pkcs11.rs` | 496 | NOT REVIEWED | PKCS#11 |
| `qr_codec_encode.rs` | 280 | NOT REVIEWED | QR codec encode |
| `qr_compose.rs` | 104 | NOT REVIEWED | QR compose |
| `rate_limit.rs` | 124 | NOT REVIEWED | Rate limit |
| `recorder.rs` | 1,287 | NOT REVIEWED | Recorder |
| `recovery.rs` | 294 | NOT REVIEWED | Recovery |
| `s3.rs` | 374 | NOT REVIEWED | S3 |
| `secure_key_storage.rs` | 256 | NOT REVIEWED | Secure key storage |
| `security_capabilities.rs` | 256 | NOT REVIEWED | Security capabilities |
| `security_config.rs` | 297 | NOT REVIEWED | Security config |
| `session_history.rs` | 154 | NOT REVIEWED | Session history |
| `session_tree.rs` | 119 | NOT REVIEWED | Session tree |
| `sessions.rs` | 708 | NOT REVIEWED | Sessions |
| `sessions_registry.rs` | 220 | NOT REVIEWED | Sessions registry |
| `sftp.rs` | 570 | NOT REVIEWED | SFTP |
| `sftp_models.rs` | 252 | NOT REVIEWED | SFTP models |
| `snippet_template.rs` | 108 | NOT REVIEWED | Snippet template |
| `ssh.rs` | 818 | NOT REVIEWED | SSH |
| `ssh_agent.rs` | 162 | NOT REVIEWED | SSH agent |
| `ssh_config.rs` | 329 | NOT REVIEWED | SSH config |
| `ssh_dir_scan.rs` | 77 | NOT REVIEWED | SSH dir scan |
| `sync.rs` | 294 | NOT REVIEWED | Sync |
| `terminal.rs` | 1,542 | NOT REVIEWED | Terminal |
| `test_hooks.rs` | 114 | NOT REVIEWED | Test hooks |
| `threat_eval.rs` | 131 | NOT REVIEWED | Threat eval |
| `tier_machine.rs` | 237 | NOT REVIEWED | Tier machine |
| `tier_transition_marker.rs` | 38 | NOT REVIEWED | Tier transition marker |
| `tier_unlock_orchestrator.rs` | 444 | NOT REVIEWED | Tier unlock orchestrator |
| `tpm.rs` | 207 | NOT REVIEWED | TPM |
| `tpm_ssh.rs` | 773 | NOT REVIEWED | TPM SSH |
| `transfer.rs` | 278 | NOT REVIEWED | Transfer |
| `transfer_conflict.rs` | 214 | NOT REVIEWED | Transfer conflict |
| `update_http.rs` | 321 | NOT REVIEWED | Update HTTP |
| `update_metadata.rs` | 220 | NOT REVIEWED | Update metadata |
| `update_signing.rs` | 51 | NOT REVIEWED | Update signing |
| `webdav.rs` | 463 | NOT REVIEWED | WebDAV |
| `winbio.rs` | 44 | NOT REVIEWED | Windows biometric |
| `wipe.rs` | 92 | NOT REVIEWED | Wipe |
| `wipe_keychain.rs` | 156 | NOT REVIEWED | Wipe keychain |
| `wizard_setup.rs` | 159 | NOT REVIEWED | Wizard setup |

### 12.2. `src/` — Generated/Support

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `frb_generated.rs` | 46,432 | NOT REVIEWED | FRB generated main |
| `lib.rs` | 13 | NOT REVIEWED | Crate root |

### 12.3. `tests/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `connection_lifecycle.rs` | 354 | NOT REVIEWED | Connection lifecycle test |
| `poison_recovery.rs` | 110 | NOT REVIEWED | Poison recovery test |
| `test_hooks_lifecycle.rs` | 81 | NOT REVIEWED | Test hooks lifecycle |

---

## 13. `rust/crates/lfs_os_security/` — OS Security Integrations

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `src/lib.rs` | 463 | NOT REVIEWED | Crate root |
| `src/android/biometric.rs` | 331 | NOT REVIEWED | Android biometric |
| `src/android/clipboard.rs` | 273 | NOT REVIEWED | Android clipboard |
| `src/android/hardware_vault.rs` | 606 | NOT REVIEWED | Android hardware vault |
| `src/android/jni_bootstrap.rs` | 166 | NOT REVIEWED | JNI bootstrap |
| `src/android/jni_helpers.rs` | 289 | NOT REVIEWED | JNI helpers |
| `src/android/keystore.rs` | 585 | NOT REVIEWED | Android keystore |
| `src/android/keystore_signer.rs` | 462 | NOT REVIEWED | Android keystore signer |
| `src/android/mod.rs` | 23 | NOT REVIEWED | Android module |
| `src/apple_se_ssh.rs` | 904 | NOT REVIEWED | Apple Secure Enclave SSH |
| `src/backup_exclusion.rs` | 92 | NOT REVIEWED | Backup exclusion |
| `src/biometric_auth.rs` | 361 | NOT REVIEWED | Biometric auth |
| `src/fido2_broker.rs` | 912 | NOT REVIEWED | FIDO2 broker |
| `src/hardware_tier_vault.rs` | 1,791 | NOT REVIEWED | Hardware tier vault |
| `src/installer_launch.rs` | 404 | NOT REVIEWED | Installer launch |
| `src/linux/mod.rs` | 34 | NOT REVIEWED | Linux module |
| `src/linux/tpm.rs` | 580 | NOT REVIEWED | Linux TPM |
| `src/linux/tpm_native.rs` | 506 | NOT REVIEWED | Linux TPM native |
| `src/linux/tpm_ssh.rs` | 932 | NOT REVIEWED | Linux TPM SSH |
| `src/linux/tpm_tcg_pem.rs` | 558 | NOT REVIEWED | Linux TPM TCG PEM |
| `src/macos/code_signing.rs` | 490 | NOT REVIEWED | macOS code signing |
| `src/macos/installer.rs` | 342 | NOT REVIEWED | macOS installer |
| `src/macos/mod.rs` | 20 | NOT REVIEWED | macOS module |
| `src/macos/tpm_ssh.rs` | 37 | NOT REVIEWED | macOS TPM SSH |
| `src/path.rs` | 199 | NOT REVIEWED | Path utilities |
| `src/pkcs11/discovery.rs` | 205 | NOT REVIEWED | PKCS#11 discovery |
| `src/pkcs11/error.rs` | 114 | NOT REVIEWED | PKCS#11 error |
| `src/pkcs11/key.rs` | 414 | NOT REVIEWED | PKCS#11 key |
| `src/pkcs11/mod.rs` | 83 | NOT REVIEWED | PKCS#11 module |
| `src/pkcs11/module.rs` | 192 | NOT REVIEWED | PKCS#11 module wrapper |
| `src/pkcs11/session.rs` | 196 | NOT REVIEWED | PKCS#11 session |
| `src/pkcs11/sign.rs` | 342 | NOT REVIEWED | PKCS#11 signing |
| `src/pkcs11/uri.rs` | 587 | NOT REVIEWED | PKCS#11 URI |
| `src/secure_clipboard.rs` | 564 | NOT REVIEWED | Secure clipboard |
| `src/secure_key_storage.rs` | 920 | NOT REVIEWED | Secure key storage |
| `src/session_lock_listener.rs` | 400 | NOT REVIEWED | Session lock listener |
| `src/subprocess_util.rs` | 424 | NOT REVIEWED | Subprocess utility |
| `src/winbio.rs` | 127 | NOT REVIEWED | Windows biometric |
| `src/windows/hardware_vault.rs` | 906 | NOT REVIEWED | Windows hardware vault |
| `src/windows/mod.rs` | 11 | NOT REVIEWED | Windows module |
| `src/windows/ncrypt_ssh.rs` | 1,428 | NOT REVIEWED | Windows NCrypt SSH |
| `tests/pkcs11_softhsm_test.rs` | 77 | NOT REVIEWED | PKCS#11 SoftHSM test |
| `tests/tpm_ssh_swtpm.rs` | 166 | NOT REVIEWED | TPM SSH swtpm test |
| `examples/mint_storage_primary_template_fixture.rs` | 37 | NOT REVIEWED | Mint storage fixture |

---

## 14. `rust/fuzz/` — Fuzz Testing

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `fuzz_targets/deeplink.rs` | 49 | NOT REVIEWED | Deeplink fuzz target |
| `fuzz_targets/known_hosts.rs` | 33 | NOT REVIEWED | Known hosts fuzz target |
| `fuzz_targets/openssh_config.rs` | 32 | NOT REVIEWED | OpenSSH config fuzz target |
| `fuzz_targets/openssh_key_import.rs` | 26 | NOT REVIEWED | OpenSSH key import fuzz |
| `fuzz_targets/pem_certs.rs` | 46 | NOT REVIEWED | PEM certs fuzz target |
| `fuzz_targets/pkcs11_uri.rs` | 38 | NOT REVIEWED | PKCS#11 URI fuzz target |
| `fuzz_targets/ppk_import.rs` | 29 | NOT REVIEWED | PPK import fuzz target |
| `fuzz_targets/qr_codec.rs` | 29 | NOT REVIEWED | QR codec fuzz target |
| `fuzz_targets/sk_key_import.rs` | 25 | NOT REVIEWED | SK key import fuzz |
| `fuzz_targets/ssh_target.rs` | 57 | NOT REVIEWED | SSH target fuzz target |
| `fuzz_targets/terminal_engine.rs` | 87 | NOT REVIEWED | Terminal engine fuzz |
| `fuzz_targets/transfer_entry_name.rs` | 54 | NOT REVIEWED | Transfer entry name fuzz |

---

## 15. `rust_builder/cargokit/` — Cargo Build Tooling

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `build_tool/bin/build_tool.dart` | 8 | NOT REVIEWED | Build tool entry |
| `build_tool/lib/build_tool.dart` | 8 | NOT REVIEWED | Build tool library |
| `build_tool/lib/src/android_environment.dart` | 195 | NOT REVIEWED | Android environment |
| `build_tool/lib/src/artifacts_provider.dart` | 266 | NOT REVIEWED | Artifacts provider |
| `build_tool/lib/src/build_cmake.dart` | 40 | NOT REVIEWED | CMake build |
| `build_tool/lib/src/build_gradle.dart` | 49 | NOT REVIEWED | Gradle build |
| `build_tool/lib/src/build_pod.dart` | 89 | NOT REVIEWED | Pod build |
| `build_tool/lib/src/build_tool.dart` | 276 | NOT REVIEWED | Build tool core |
| `build_tool/lib/src/builder.dart` | 209 | NOT REVIEWED | Builder |
| `build_tool/lib/src/cargo.dart` | 48 | NOT REVIEWED | Cargo interface |
| `build_tool/lib/src/crate_hash.dart` | 124 | NOT REVIEWED | Crate hash |
| `build_tool/lib/src/environment.dart` | 68 | NOT REVIEWED | Environment |
| `build_tool/lib/src/logging.dart` | 52 | NOT REVIEWED | Logging |
| `build_tool/lib/src/options.dart` | 309 | NOT REVIEWED | Build options |
| `build_tool/lib/src/precompile_binaries.dart` | 205 | NOT REVIEWED | Precompile binaries |
| `build_tool/lib/src/rustup.dart` | 149 | NOT REVIEWED | Rustup interface |
| `build_tool/lib/src/target.dart` | 147 | NOT REVIEWED | Target definitions |
| `build_tool/lib/src/util.dart` | 172 | NOT REVIEWED | Utilities |
| `build_tool/lib/src/verify_binaries.dart` | 84 | NOT REVIEWED | Binary verification |
| `ios/Classes/dummy_file.c` | 1 | NOT REVIEWED | iOS dummy |
| `macos/Classes/dummy_file.c` | 1 | NOT REVIEWED | macOS dummy |

---

## 16. `test/` — Flutter/Dart Tests

### 16.1. `test/app/` — App-Level Tests

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `app_toolbar_test.dart` | 122 | NOT REVIEWED | App toolbar test |
| `connection_state_announcer_test.dart` | 343 | NOT REVIEWED | Connection state announcer test |
| `fatal_error_app_test.dart` | 242 | NOT REVIEWED | Fatal error app test |
| `global_error_dialog_test.dart` | 146 | NOT REVIEWED | Global error dialog test |
| `hardware_vault_probe_prompt_listener_test.dart` | 80 | NOT REVIEWED | HW vault probe test |
| `hardware_vault_seal_prompt_listener_test.dart` | 134 | NOT REVIEWED | HW vault seal test |
| `hardware_vault_unlock_prompt_listener_test.dart` | 82 | NOT REVIEWED | HW vault unlock test |
| `host_key_prompt_listener_test.dart` | 267 | NOT REVIEWED | Host key prompt test |
| `import_flow_test.dart` | 1,423 | NOT REVIEWED | Import flow test |
| `keychain_probe_prompt_listener_test.dart` | 82 | NOT REVIEWED | Keychain probe test |
| `navigator_key_test.dart` | 176 | NOT REVIEWED | Navigator key test |
| `overlay_modal_route_observer_test.dart` | 117 | NOT REVIEWED | Overlay modal test |
| `recovery_prompt_listener_test.dart` | 186 | NOT REVIEWED | Recovery prompt test |
| `security_dialogs_test.dart` | 87 | NOT REVIEWED | Security dialogs test |
| `security_init_controller_bootstrap_test.dart` | 83 | NOT REVIEWED | Security init bootstrap test |
| `security_init_controller_orchestration_test.dart` | 500 | NOT REVIEWED | Security init orchestration test |
| `security_init_controller_test.dart` | 59 | NOT REVIEWED | Security init test |
| `ssh_agent_prompt_listener_test.dart` | 319 | NOT REVIEWED | SSH agent prompt test |
| `tier_state_observer_test.dart` | 58 | NOT REVIEWED | Tier state observer test |
| `update_dialog_flow_test.dart` | 343 | NOT REVIEWED | Update dialog flow test |

### 16.2. `test/core/` — Core Logic Tests

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `bus/app_bus_test.dart` | 139 | NOT REVIEWED | App bus test |
| `config/app_config_export_test.dart` | 87 | NOT REVIEWED | Config export test |
| `config/app_config_test.dart` | 913 | NOT REVIEWED | Config test |
| `connection/connection_step_mappers_test.dart` | 325 | NOT REVIEWED | Connection step mappers test |
| `connection/connection_step_test.dart` | 163 | NOT REVIEWED | Connection step test |
| `connection/connection_test.dart` | 182 | NOT REVIEWED | Connection test |
| `connection/progress_tracker_test.dart` | 169 | NOT REVIEWED | Progress tracker test |
| `db/mappers_test.dart` | 135 | NOT REVIEWED | DB mappers test |
| `db/rust_db_init_test.dart` | 108 | NOT REVIEWED | Rust DB init test |
| `deeplink/deeplink_handler_test.dart` | 416 | NOT REVIEWED | Deeplink handler test |
| `error_boundary_test.dart` | 38 | NOT REVIEWED | Error boundary test |
| `import/import_service_test.dart` | 400 | NOT REVIEWED | Import service test |
| `import/key_file_helper_test.dart` | 106 | NOT REVIEWED | Key file helper test |
| `import/openssh_config_importer_test.dart` | 412 | NOT REVIEWED | OpenSSH config importer test |
| `import/ssh_dir_key_scanner_test.dart` | 137 | NOT REVIEWED | SSH dir key scanner test |
| `logs/log_store_test.dart` | 410 | NOT REVIEWED | Log store test |
| `logs/settings_logging_parser_test.dart` | 206 | NOT REVIEWED | Settings logging parser test |
| `migration/migration_runner_test.dart` | 129 | NOT REVIEWED | Migration runner test |
| `no_flutter_in_core_test.dart` | 69 | NOT REVIEWED | No Flutter in core test |
| `progress/progress_reporter_test.dart` | 117 | NOT REVIEWED | Progress reporter test |
| `s3/s3_fs_test.dart` | 237 | NOT REVIEWED | S3 FS test |
| `security/` | 27 files | NOT REVIEWED | Security tests (biometric, hardware tier, keychain, etc.) |
| `session/` | 10 files | NOT REVIEWED | Session tests |
| `sftp/` | 4 files | NOT REVIEWED | SFTP tests |
| `snippets/` | 2 files | NOT REVIEWED | Snippets tests |
| `ssh/` | 8 files | NOT REVIEWED | SSH tests |
| `tags/tag_test.dart` | 112 | NOT REVIEWED | Tag test |
| `transfer/` | 3 files | NOT REVIEWED | Transfer tests |
| `update/update_service_test.dart` | 1,751 | NOT REVIEWED | Update service test |
| `webdav/webdav_fs_test.dart` | 242 | NOT REVIEWED | WebDAV FS test |

### 16.3. `test/features/` — Feature Tests

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `file_browser/` | 12 files | NOT REVIEWED | File browser tests |
| `key_manager/` | 3 files | NOT REVIEWED | Key manager tests |
| `mobile/` | 6 files | NOT REVIEWED | Mobile tests |
| `recordings/` | 5 files | NOT REVIEWED | Recordings tests |
| `session_manager/` | 14 files | NOT REVIEWED | Session manager tests |
| `settings/` | 17 files | NOT REVIEWED | Settings tests |
| `snippets/` | 3 files | NOT REVIEWED | Snippets tests |
| `tabs/` | 2 files | NOT REVIEWED | Tabs tests |
| `tags/` | 3 files | NOT REVIEWED | Tags tests |
| `terminal/` | 5 files | NOT REVIEWED | Terminal tests |
| `tools/` | 2 files | NOT REVIEWED | Tools tests |
| `workspace/` | 6 files | NOT REVIEWED | Workspace tests |

### 16.4. `test/widgets/` — Widget Tests (100+ files)

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `agent_signature_request_dialog_test.dart` | 136 | NOT REVIEWED | Agent signature dialog test |
| `app_bordered_box_test.dart` | 106 | NOT REVIEWED | Bordered box test |
| `app_button_test.dart` | 146 | NOT REVIEWED | Button test |
| `app_collection_toolbar_test.dart` | 136 | NOT REVIEWED | Toolbar test |
| `app_data_row_test.dart` | 118 | NOT REVIEWED | Data row test |
| `app_data_search_bar_test.dart` | 58 | NOT REVIEWED | Search bar test |
| `app_dialog_test.dart` | 459 | NOT REVIEWED | Dialog test |
| `app_divider_test.dart` | 45 | NOT REVIEWED | Divider test |
| `app_empty_state_test.dart` | 100 | NOT REVIEWED | Empty state test |
| `app_icon_button_test.dart` | 321 | NOT REVIEWED | Icon button test |
| `app_info_button_test.dart` | 90 | NOT REVIEWED | Info button test |
| `app_info_dialog_test.dart` | 99 | NOT REVIEWED | Info dialog test |
| `app_picker_chip_test.dart` | 88 | NOT REVIEWED | Picker chip test |
| `app_popup_select_test.dart` | 184 | NOT REVIEWED | Popup select test |
| `app_selection_area_test.dart` | 139 | NOT REVIEWED | Selection area test |
| `app_shell_test.dart` | 272 | NOT REVIEWED | App shell test |
| `auto_lock_detector_test.dart` | 405 | NOT REVIEWED | Auto-lock detector test |
| `clipped_row_test.dart` | 76 | NOT REVIEWED | Clipped row test |
| `column_resize_handle_test.dart` | 80 | NOT REVIEWED | Column resize test |
| `confirm_dialog_test.dart` | 147 | NOT REVIEWED | Confirm dialog test |
| `connection_progress_test.dart` | 121 | NOT REVIEWED | Connection progress test |
| `context_menu_test.dart` | 494 | NOT REVIEWED | Context menu test |
| `core/app_collection_panel_test.dart` | 121 | NOT REVIEWED | Collection panel test |
| `core/sidebar_nav_dialog_test.dart` | 181 | NOT REVIEWED | Sidebar nav test |
| `core/typed_name_confirm_dialog_test.dart` | 88 | NOT REVIEWED | Typed name test |
| `data_checkboxes_test.dart` | 145 | NOT REVIEWED | Data checkboxes test |
| `db_corrupt_dialog_test.dart` | 113 | NOT REVIEWED | DB corrupt test |
| `dropdown_select_button_test.dart` | 66 | NOT REVIEWED | Dropdown select test |
| `enclave_ssh_dialog_test.dart` | 222 | NOT REVIEWED | Enclave SSH test |
| `error_state_test.dart` | 103 | NOT REVIEWED | Error state test |
| `expandable_tier_card_logic_test.dart` | 341 | NOT REVIEWED | Tier card logic test |
| `expandable_tier_card_test.dart` | 635 | NOT REVIEWED | Tier card test |
| `file_conflict_dialog_test.dart` | 160 | NOT REVIEWED | File conflict test |
| `first_launch_security_toast_test.dart` | 181 | NOT REVIEWED | First launch toast test |
| `form_submit_chain_test.dart` | 92 | NOT REVIEWED | Form submit test |
| `hello_ssh_dialog_test.dart` | 386 | NOT REVIEWED | Hello SSH test |
| `host_key_dialog_test.dart` | 389 | NOT REVIEWED | Host key test |
| `hover_region_test.dart` | 264 | NOT REVIEWED | Hover region test |
| `import_export/` | 4 files | NOT REVIEWED | Import/export widget tests |
| `import_preview_dialog_test.dart` | 381 | NOT REVIEWED | Import preview test |
| `keystore_ssh_dialog_test.dart` | 568 | NOT REVIEWED | Keystore SSH test |
| `lfs_import_dialog_test.dart` | 399 | NOT REVIEWED | LFS import test |
| `lfs_import_preview_dialog_test.dart` | 383 | NOT REVIEWED | LFS import preview test |
| `local_directory_picker_test.dart` | 155 | NOT REVIEWED | Local directory picker test |
| `lock_focus_exclusion_test.dart` | 85 | NOT REVIEWED | Lock focus test |
| `lock_screen_test.dart` | 315 | NOT REVIEWED | Lock screen test |
| `marquee_mixin_test.dart` | 283 | NOT REVIEWED | Marquee test |
| `mobile_selection_bar_test.dart` | 169 | NOT REVIEWED | Mobile selection test |
| `mode_button_test.dart` | 105 | NOT REVIEWED | Mode button test |
| `password_strength_meter_test.dart` | 87 | NOT REVIEWED | Password strength test |
| `pkcs11_import_dialog_logic_test.dart` | 173 | NOT REVIEWED | PKCS#11 import logic test |
| `pkcs11_import_dialog_test.dart` | 1,809 | NOT REVIEWED | PKCS#11 import test |
| `progress_writer_test.dart` | 208 | NOT REVIEWED | Progress writer test |
| `secure_password_field_test.dart` | 93 | NOT REVIEWED | Secure password test |
| `secure_screen_scope_test.dart` | 68 | NOT REVIEWED | Secure screen test |
| `security_comparison_table_test.dart` | 94 | NOT REVIEWED | Security comparison test |
| `security_setup_dialog_logic_test.dart` | 195 | NOT REVIEWED | Security setup logic test |
| `security_setup_dialog_test.dart` | 682 | NOT REVIEWED | Security setup test |
| `security_threat_list_test.dart` | 90 | NOT REVIEWED | Security threat test |
| `session_kind_icon_test.dart` | 43 | NOT REVIEWED | Session kind icon test |
| `shortcut_registry_test.dart` | 412 | NOT REVIEWED | Shortcut registry test |
| `sortable_header_cell_test.dart` | 159 | NOT REVIEWED | Sortable header test |
| `split_view_test.dart` | 100 | NOT REVIEWED | Split view test |
| `ssh_dir_import_dialog_test.dart` | 671 | NOT REVIEWED | SSH dir import test |
| `ssh_keys/hardware_key_badge_test.dart` | 61 | NOT REVIEWED | HW key badge test |
| `ssh_keys/hardware_key_prompt_dialog_test.dart` | 197 | NOT REVIEWED | HW key prompt test |
| `ssh_keys/hardware_key_wizard_test.dart` | 175 | NOT REVIEWED | HW key wizard test |
| `status_indicator_test.dart` | 73 | NOT REVIEWED | Status indicator test |
| `styled_form_field_test.dart` | 279 | NOT REVIEWED | Styled form field test |
| `tag_color_test.dart` | 64 | NOT REVIEWED | Tag color test |
| `tag_dots_test.dart` | 181 | NOT REVIEWED | Tag dots test |
| `terminal/terminal_cell_flags_test.dart` | 61 | NOT REVIEWED | Terminal cell flags test |
| `terminal/terminal_controller_test.dart` | 479 | NOT REVIEWED | Terminal controller test |
| `terminal/terminal_grid_painter_test.dart` | 472 | NOT REVIEWED | Terminal grid painter test |
| `terminal/terminal_key_input_test.dart` | 133 | NOT REVIEWED | Terminal key input test |
| `terminal/terminal_palette_theme_test.dart` | 86 | NOT REVIEWED | Terminal palette test |
| `terminal/terminal_pointer_input_test.dart` | 294 | NOT REVIEWED | Terminal pointer input test |
| `terminal/terminal_search_bar_test.dart` | 106 | NOT REVIEWED | Terminal search test |
| `terminal/terminal_view_test.dart` | 1,025 | NOT REVIEWED | Terminal view test |
| `terminal_cell_metrics_test.dart` | 44 | NOT REVIEWED | Terminal cell metrics test |
| `threshold_draggable_test.dart` | 84 | NOT REVIEWED | Threshold draggable test |
| `tier_reset_dialog_test.dart` | 71 | NOT REVIEWED | Tier reset test |
| `tier_secret_unlock_dialog_test.dart` | 879 | NOT REVIEWED | Tier secret unlock test |
| `toast_test.dart` | 435 | NOT REVIEWED | Toast test |
| `tpm_ssh_dialog_test.dart` | 767 | NOT REVIEWED | TPM SSH test |
| `unified_export_controller_test.dart` | 1,038 | NOT REVIEWED | Unified export controller test |
| `unified_export_dialog_test.dart` | 992 | NOT REVIEWED | Unified export dialog test |
| `unlock_dialog_test.dart` | 729 | NOT REVIEWED | Unlock dialog test |
| `update_progress_indicator_test.dart` | 110 | NOT REVIEWED | Update progress test |

### 16.5. Other Tests

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `main_test.dart` | 1,449 | NOT REVIEWED | Main test entry |
| `flutter_test_config.dart` | 84 | NOT REVIEWED | Test config |
| `fuzz/` | 7 files | NOT REVIEWED | Fuzz test wrappers |
| `helpers/` | 11 files | NOT REVIEWED | Test helpers |
| `integration/` | 20 files | NOT REVIEWED | Integration tests |
| `platform/` | 3 files | NOT REVIEWED | Platform tests |
| `providers/` | 17 files | NOT REVIEWED | Provider tests |
| `theme/app_theme_test.dart` | 215 | NOT REVIEWED | Theme test |
| `utils/` | 10 files | NOT REVIEWED | Utility tests |

---

## 17. `windows/` — Windows Headers

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `flutter/generated_plugin_registrant.h` | 15 | NOT REVIEWED | Plugin registrant |
| `runner/flutter_window.h` | 33 | NOT REVIEWED | Flutter window |
| `runner/resource.h` | 16 | NOT REVIEWED | Resource header |
| `runner/utils.h` | 15 | NOT REVIEWED | Utilities header |
| `runner/win32_window.h` | 102 | NOT REVIEWED | Win32 window |

---

## Review Checklist

- [ ] All files marked NOT REVIEWED above
- [ ] FRB (flutter_rust_bridge) integration patterns documented
- [ ] Security tier architecture assessed
- [ ] Terminal emulation approach compared to torvox
- [ ] SFTP/transfer patterns noted
- [ ] PKCS#11/hardware key integration documented
- [ ] Widget library patterns identified
