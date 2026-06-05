package io.torvox.settings

import android.content.Context
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.floatPreferencesKey
import androidx.datastore.preferences.core.intPreferencesKey
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject
import javax.inject.Singleton

private val Context.dataStore: DataStore<Preferences> by preferencesDataStore(name = "settings")

@Singleton
class SettingsRepository
    @Inject
    constructor(
        @ApplicationContext private val context: Context,
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
            val TOUCH_BEHAVIOR = stringPreferencesKey("touch_behavior")
            val MATERIAL_YOU = booleanPreferencesKey("material_you")
        }

        val fontSize: Flow<Float> = context.dataStore.data.map { it[Keys.FONT_SIZE] ?: 18f }
        val fontFamily: Flow<String> = context.dataStore.data.map { it[Keys.FONT_FAMILY] ?: "" }
        val themeName: Flow<String> = context.dataStore.data.map { it[Keys.THEME_NAME] ?: "Catppuccin Mocha" }
        val dayThemeName: Flow<String> = context.dataStore.data.map { it[Keys.DAY_THEME_NAME] ?: "Catppuccin Mocha" }
        val nightThemeName: Flow<String> = context.dataStore.data.map { it[Keys.NIGHT_THEME_NAME] ?: "Catppuccin Mocha" }
        val themeMode: Flow<String> = context.dataStore.data.map { it[Keys.THEME_MODE] ?: "follow_system" }
        val shell: Flow<String> = context.dataStore.data.map { it[Keys.SHELL] ?: "/system/bin/sh" }
        val scrollbackLines: Flow<Int> = context.dataStore.data.map { it[Keys.SCROLLBACK_LINES] ?: 50000 }
        val touchBehavior: Flow<String> = context.dataStore.data.map { it[Keys.TOUCH_BEHAVIOR] ?: "right_click" }
        val materialYouEnabled: Flow<Boolean> = context.dataStore.data.map { it[Keys.MATERIAL_YOU] ?: false }

        suspend fun setFontSize(size: Float) {
            context.dataStore.edit { it[Keys.FONT_SIZE] = size }
        }

        suspend fun setFontFamily(family: String) {
            context.dataStore.edit { it[Keys.FONT_FAMILY] = family }
        }

        suspend fun setThemeName(name: String) {
            context.dataStore.edit { it[Keys.THEME_NAME] = name }
        }

        suspend fun setDayThemeName(name: String) {
            context.dataStore.edit { it[Keys.DAY_THEME_NAME] = name }
        }

        suspend fun setNightThemeName(name: String) {
            context.dataStore.edit { it[Keys.NIGHT_THEME_NAME] = name }
        }

        suspend fun setThemeMode(mode: String) {
            context.dataStore.edit { it[Keys.THEME_MODE] = mode }
        }

        suspend fun setShell(shell: String) {
            context.dataStore.edit { it[Keys.SHELL] = shell }
        }

        suspend fun setScrollbackLines(lines: Int) {
            context.dataStore.edit { it[Keys.SCROLLBACK_LINES] = lines }
        }

        suspend fun setTouchBehavior(behavior: String) {
            context.dataStore.edit { it[Keys.TOUCH_BEHAVIOR] = behavior }
        }

        suspend fun setMaterialYouEnabled(enabled: Boolean) {
            context.dataStore.edit { it[Keys.MATERIAL_YOU] = enabled }
        }
    }
