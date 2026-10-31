import 'package:flutter/material.dart';

/// Preset seed colors available in the theme customization UI.
class ThemeSeed {
  const ThemeSeed(this.name, this.color);
  final String name;
  final Color color;

  static const presets = <ThemeSeed>[
    ThemeSeed('Indigo', Color(0xFF5B6CFF)),
    ThemeSeed('Violet', Color(0xFF7C3AED)),
    ThemeSeed('Teal', Color(0xFF0D9488)),
    ThemeSeed('Emerald', Color(0xFF059669)),
    ThemeSeed('Amber', Color(0xFFF59E0B)),
    ThemeSeed('Rose', Color(0xFFE11D48)),
  ];

  static ThemeSeed byValue(int value) => presets.firstWhere(
        (s) => s.color.toARGB32() == value,
        orElse: () => presets[0],
      );
}

enum AppDensity { compact, standard, comfortable }

extension AppDensityX on AppDensity {
  VisualDensity get visualDensity => switch (this) {
        AppDensity.compact => VisualDensity.compact,
        AppDensity.standard => VisualDensity.standard,
        AppDensity.comfortable =>
          const VisualDensity(horizontal: 2, vertical: 2),
      };

  String get label => switch (this) {
        AppDensity.compact => 'Compact',
        AppDensity.standard => 'Standard',
        AppDensity.comfortable => 'Comfortable',
      };
}

/// User-editable theme preferences. Persisted via SharedPreferences.
class ThemePreferences {
  const ThemePreferences({
    this.themeMode = ThemeMode.system,
    this.seedColorValue = 0xFF5B6CFF,
    this.borderRadius = 14,
    this.density = AppDensity.standard,
  });

  final ThemeMode themeMode;
  final int seedColorValue;
  final double borderRadius;
  final AppDensity density;

  Color get seedColor => Color(seedColorValue);

  ThemePreferences copyWith({
    ThemeMode? themeMode,
    int? seedColorValue,
    double? borderRadius,
    AppDensity? density,
  }) {
    return ThemePreferences(
      themeMode: themeMode ?? this.themeMode,
      seedColorValue: seedColorValue ?? this.seedColorValue,
      borderRadius: borderRadius ?? this.borderRadius,
      density: density ?? this.density,
    );
  }

  static const _kThemeMode = 'theme.mode';
  static const _kSeedColor = 'theme.seed';
  static const _kRadius = 'theme.radius';
  static const _kDensity = 'theme.density';

  Map<String, Object> toPrefs() => {
        _kThemeMode: themeMode.index,
        _kSeedColor: seedColorValue,
        _kRadius: borderRadius,
        _kDensity: density.index,
      };

  static ThemePreferences fromPrefs(Map<String, Object?> p) {
    return ThemePreferences(
      themeMode: ThemeMode
          .values[(p[_kThemeMode] as int?) ?? ThemeMode.system.index],
      seedColorValue: (p[_kSeedColor] as int?) ?? 0xFF5B6CFF,
      borderRadius: (p[_kRadius] as num?)?.toDouble() ?? 14,
      density: AppDensity
          .values[(p[_kDensity] as int?) ?? AppDensity.standard.index],
    );
  }

  static const preferenceKeys = <String>[_kThemeMode, _kSeedColor, _kRadius, _kDensity];
}
