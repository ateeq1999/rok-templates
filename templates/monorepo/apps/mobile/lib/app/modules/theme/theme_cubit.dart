import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../../core/theme/theme_preferences.dart';

part 'theme_state.dart';

class ThemeCubit extends Cubit<ThemeState> {
  ThemeCubit(this._prefs)
      : super(ThemeState(_loadFromPrefs(_prefs)));

  final SharedPreferences _prefs;

  ThemePreferences get preferences => state.preferences;

  Future<void> setThemeMode(ThemeMode mode) async {
    await _update(preferences.copyWith(themeMode: mode));
  }

  Future<void> setSeedColor(int value) async {
    await _update(preferences.copyWith(seedColorValue: value));
  }

  Future<void> setBorderRadius(double radius) async {
    await _update(preferences.copyWith(borderRadius: radius));
  }

  Future<void> setDensity(AppDensity density) async {
    await _update(preferences.copyWith(density: density));
  }

  Future<void> reset() async {
    await _update(const ThemePreferences());
  }

  Future<void> _update(ThemePreferences next) async {
    emit(ThemeState(next));
    final data = next.toPrefs();
    for (final entry in data.entries) {
      final value = entry.value;
      if (value is int) await _prefs.setInt(entry.key, value);
      if (value is double) await _prefs.setDouble(entry.key, value);
    }
  }

  static ThemePreferences _loadFromPrefs(SharedPreferences prefs) {
    final map = <String, Object?>{};
    for (final key in ThemePreferences.preferenceKeys) {
      map[key] = prefs.get(key);
    }
    return ThemePreferences.fromPrefs(map);
  }
}
