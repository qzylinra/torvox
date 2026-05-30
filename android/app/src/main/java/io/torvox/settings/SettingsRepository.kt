package io.torvox.settings

import android.content.Context
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.Preferences
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
            val SHELL = stringPreferencesKey("shell")
            val SCROLLBACK_LINES = intPreferencesKey("scrollback_lines")
            val TOUCH_BEHAVIOR = stringPreferencesKey("touch_behavior")
        }

        val fontSize: Flow<Float> = context.dataStore.data.map { it[Keys.FONT_SIZE] ?: 14f }
        val fontFamily: Flow<String> = context.dataStore.data.map { it[Keys.FONT_FAMILY] ?: "JetBrains Mono Nerd Font" }
        val themeName: Flow<String> = context.dataStore.data.map { it[Keys.THEME_NAME] ?: "Catppuccin Mocha" }
        val shell: Flow<String> = context.dataStore.data.map { it[Keys.SHELL] ?: "/system/bin/sh" }
        val scrollbackLines: Flow<Int> = context.dataStore.data.map { it[Keys.SCROLLBACK_LINES] ?: 50000 }
        val touchBehavior: Flow<String> = context.dataStore.data.map { it[Keys.TOUCH_BEHAVIOR] ?: "right_click" }

        suspend fun setFontSize(size: Float) {
            context.dataStore.edit { it[Keys.FONT_SIZE] = size }
        }

        suspend fun setFontFamily(family: String) {
            context.dataStore.edit { it[Keys.FONT_FAMILY] = family }
        }

        suspend fun setThemeName(name: String) {
            context.dataStore.edit { it[Keys.THEME_NAME] = name }
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
    }
