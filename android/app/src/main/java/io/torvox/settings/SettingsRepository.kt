package io.torvox.settings

import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.floatPreferencesKey
import androidx.datastore.preferences.core.intPreferencesKey
import androidx.datastore.preferences.core.stringPreferencesKey
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class SettingsRepository
@Inject
constructor(
    private val provider: SettingsDataStoreProvider,
) {
    private object Keys {
        val FONT_SIZE = floatPreferencesKey("font_size")
        val FONT_FAMILY = stringPreferencesKey("font_family")
        val THEME_NAME = stringPreferencesKey("theme_name")
        val DAY_THEME_NAME = stringPreferencesKey("day_theme_name")
        val NIGHT_THEME_NAME = stringPreferencesKey("night_theme_name")
        val THEME_MODE = stringPreferencesKey("theme_mode")
        val SHELL = stringPreferencesKey("shell")
        val SCROLLBACK_LINES = intPreferencesKey("scrollback_lines")
        val APP_THEME_MODE = stringPreferencesKey("app_theme_mode")
        val TOUCH_BEHAVIOR = stringPreferencesKey("touch_behavior")
        val BOOTSTRAP_URL = stringPreferencesKey("bootstrap_url")
        val USE_NERD_FONT_GLYPHS = booleanPreferencesKey("use_nerd_font_glyphs")
        val USE_SEMANTIC_SELECTION = booleanPreferencesKey("use_semantic_selection")
        val SESSION_RESTORE = booleanPreferencesKey("session_restore")
        val KEYBOARD_MODE = stringPreferencesKey("keyboard_mode")
        val USB_SERIAL_ENABLED = booleanPreferencesKey("usb_serial_enabled")
        val MCP_SERVER_ENABLED = booleanPreferencesKey("mcp_server_enabled")
        val VOLUME_KEY_MAP = booleanPreferencesKey("volume_key_map")
    }

    val appThemeMode: Flow<String> = provider.dataStore.data.map { it[Keys.APP_THEME_MODE] ?: "follow_system" }
    val fontSize: Flow<Float> = provider.dataStore.data.map { it[Keys.FONT_SIZE] ?: 18f }
    val fontFamily: Flow<String> = provider.dataStore.data.map { it[Keys.FONT_FAMILY] ?: "" }
    val themeName: Flow<String> = provider.dataStore.data.map { it[Keys.THEME_NAME] ?: "Dracula Plus" }
    val dayThemeName: Flow<String> = provider.dataStore.data.map { it[Keys.DAY_THEME_NAME] ?: "Catppuccin Latte" }
    val nightThemeName: Flow<String> = provider.dataStore.data.map { it[Keys.NIGHT_THEME_NAME] ?: "Dracula Plus" }
    val themeMode: Flow<String> = provider.dataStore.data.map { it[Keys.THEME_MODE] ?: "fixed" }
    val shell: Flow<String> = provider.dataStore.data.map { it[Keys.SHELL] ?: "/system/bin/sh" }
    val scrollbackLines: Flow<Int> = provider.dataStore.data.map { it[Keys.SCROLLBACK_LINES] ?: 50000 }
    val touchBehavior: Flow<String> = provider.dataStore.data.map { it[Keys.TOUCH_BEHAVIOR] ?: "right_click" }
    val bootstrapUrl: Flow<String> = provider.dataStore.data.map { it[Keys.BOOTSTRAP_URL] ?: "" }
    val useNerdFontGlyphs: Flow<Boolean> = provider.dataStore.data.map { it[Keys.USE_NERD_FONT_GLYPHS] ?: false }
    val useSemanticSelection: Flow<Boolean> = provider.dataStore.data.map { it[Keys.USE_SEMANTIC_SELECTION] ?: false }
    val sessionRestore: Flow<Boolean> = provider.dataStore.data.map { it[Keys.SESSION_RESTORE] ?: false }
    val keyboardMode: Flow<String> = provider.dataStore.data.map { it[Keys.KEYBOARD_MODE] ?: "secure" }
    val usbSerialEnabled: Flow<Boolean> = provider.dataStore.data.map { it[Keys.USB_SERIAL_ENABLED] ?: false }
    val mcpServerEnabled: Flow<Boolean> = provider.dataStore.data.map { it[Keys.MCP_SERVER_ENABLED] ?: false }
    val volumeKeyMap: Flow<Boolean> = provider.dataStore.data.map { it[Keys.VOLUME_KEY_MAP] ?: false }

    suspend fun setFontSize(size: Float) {
        provider.dataStore.edit { it[Keys.FONT_SIZE] = size }
    }

    suspend fun setFontFamily(family: String) {
        provider.dataStore.edit { it[Keys.FONT_FAMILY] = family }
    }

    suspend fun setThemeName(name: String) {
        provider.dataStore.edit { it[Keys.THEME_NAME] = name }
    }

    suspend fun setDayThemeName(name: String) {
        provider.dataStore.edit { it[Keys.DAY_THEME_NAME] = name }
    }

    suspend fun setNightThemeName(name: String) {
        provider.dataStore.edit { it[Keys.NIGHT_THEME_NAME] = name }
    }

    suspend fun setThemeMode(mode: String) {
        provider.dataStore.edit { it[Keys.THEME_MODE] = mode }
    }

    suspend fun setAppThemeMode(mode: String) {
        provider.dataStore.edit { it[Keys.APP_THEME_MODE] = mode }
    }

    suspend fun setShell(shell: String) {
        provider.dataStore.edit { it[Keys.SHELL] = shell }
    }

    suspend fun setScrollbackLines(lines: Int) {
        provider.dataStore.edit { it[Keys.SCROLLBACK_LINES] = lines }
    }

    suspend fun setTouchBehavior(behavior: String) {
        provider.dataStore.edit { it[Keys.TOUCH_BEHAVIOR] = behavior }
    }

    suspend fun setBootstrapUrl(url: String) {
        provider.dataStore.edit { it[Keys.BOOTSTRAP_URL] = url }
    }

    suspend fun setUseNerdFontGlyphs(enabled: Boolean) {
        provider.dataStore.edit { it[Keys.USE_NERD_FONT_GLYPHS] = enabled }
    }

    suspend fun setUseSemanticSelection(enabled: Boolean) {
        provider.dataStore.edit { it[Keys.USE_SEMANTIC_SELECTION] = enabled }
    }

    suspend fun setSessionRestore(enabled: Boolean) {
        provider.dataStore.edit { it[Keys.SESSION_RESTORE] = enabled }
    }

    suspend fun setKeyboardMode(mode: String) {
        provider.dataStore.edit { it[Keys.KEYBOARD_MODE] = mode }
    }

    suspend fun setUsbSerialEnabled(enabled: Boolean) {
        provider.dataStore.edit { it[Keys.USB_SERIAL_ENABLED] = enabled }
    }

    suspend fun setMcpServerEnabled(enabled: Boolean) {
        provider.dataStore.edit { it[Keys.MCP_SERVER_ENABLED] = enabled }
    }

    suspend fun setVolumeKeyMap(enabled: Boolean) {
        provider.dataStore.edit { it[Keys.VOLUME_KEY_MAP] = enabled }
    }
}
