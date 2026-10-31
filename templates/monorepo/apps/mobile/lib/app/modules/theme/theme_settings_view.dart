import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../core/theme/theme_preferences.dart';
import 'theme_cubit.dart';

class ThemeSettingsView extends StatelessWidget {
  const ThemeSettingsView({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<ThemeCubit, ThemeState>(
      builder: (context, state) {
        final prefs = state.preferences;
        final cubit = context.read<ThemeCubit>();
        final scheme = Theme.of(context).colorScheme;

        return Scaffold(
          appBar: AppBar(
            title: const Text('Appearance'),
            actions: [
              TextButton(
                onPressed: cubit.reset,
                child: const Text('Reset'),
              ),
            ],
          ),
          body: ListView(
            padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
            children: [
              const _SectionTitle(title:'Mode'),
              _ModeSelector(
                value: prefs.themeMode,
                onChanged: cubit.setThemeMode,
              ),
              const SizedBox(height: 24),
              const _SectionTitle(title:'Accent Color'),
              _ColorPalette(
                selected: prefs.seedColorValue,
                onSelect: cubit.setSeedColor,
              ),
              const SizedBox(height: 24),
              const _SectionTitle(title:'Corner Radius'),
              _RadiusSlider(
                value: prefs.borderRadius,
                onChanged: cubit.setBorderRadius,
              ),
              const SizedBox(height: 24),
              const _SectionTitle(title:'Density'),
              _DensitySelector(
                value: prefs.density,
                onChanged: cubit.setDensity,
              ),
              const SizedBox(height: 32),
              const _SectionTitle(title:'Preview'),
              _Preview(scheme: scheme),
            ],
          ),
        );
      },
    );
  }
}

class _SectionTitle extends StatelessWidget {
  const _SectionTitle({required this.title});
  final String title;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 12, top: 4),
      child: Text(
        title.toUpperCase(),
        style: Theme.of(context).textTheme.labelSmall?.copyWith(
              fontWeight: FontWeight.w700,
              letterSpacing: 1.2,
              color: Theme.of(context).colorScheme.onSurfaceVariant,
            ),
      ),
    );
  }
}

class _ModeSelector extends StatelessWidget {
  const _ModeSelector({required this.value, required this.onChanged});
  final ThemeMode value;
  final ValueChanged<ThemeMode> onChanged;

  @override
  Widget build(BuildContext context) {
    return SegmentedButton<ThemeMode>(
      segments: const [
        ButtonSegment(
            value: ThemeMode.light,
            label: Text('Light'),
            icon: Icon(Icons.light_mode)),
        ButtonSegment(
            value: ThemeMode.system,
            label: Text('System'),
            icon: Icon(Icons.phone_iphone)),
        ButtonSegment(
            value: ThemeMode.dark,
            label: Text('Dark'),
            icon: Icon(Icons.dark_mode)),
      ],
      selected: {value},
      onSelectionChanged: (s) => onChanged(s.first),
    );
  }
}

class _ColorPalette extends StatelessWidget {
  const _ColorPalette({required this.selected, required this.onSelect});
  final int selected;
  final ValueChanged<int> onSelect;

  @override
  Widget build(BuildContext context) {
    return Wrap(
      spacing: 12,
      runSpacing: 12,
      children: ThemeSeed.presets.map((seed) {
        final value = seed.color.toARGB32();
        final isSelected = value == selected;
        return GestureDetector(
          onTap: () => onSelect(value),
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 160),
            width: 56,
            height: 56,
            decoration: BoxDecoration(
              color: seed.color,
              shape: BoxShape.circle,
              border: Border.all(
                color: isSelected
                    ? Theme.of(context).colorScheme.onSurface
                    : Colors.transparent,
                width: 3,
              ),
              boxShadow: [
                BoxShadow(
                  color: seed.color.withValues(alpha: 0.3),
                  blurRadius: isSelected ? 12 : 4,
                  offset: const Offset(0, 2),
                ),
              ],
            ),
            child: isSelected
                ? const Icon(Icons.check, color: Colors.white, size: 24)
                : null,
          ),
        );
      }).toList(),
    );
  }
}

class _RadiusSlider extends StatelessWidget {
  const _RadiusSlider({required this.value, required this.onChanged});
  final double value;
  final ValueChanged<double> onChanged;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        const Icon(Icons.crop_square, size: 20),
        Expanded(
          child: Slider(
            value: value,
            min: 0,
            max: 28,
            divisions: 14,
            label: '${value.round()}',
            onChanged: onChanged,
          ),
        ),
        const Icon(Icons.circle_outlined, size: 20),
        const SizedBox(width: 12),
        SizedBox(
          width: 32,
          child: Text('${value.round()}',
              textAlign: TextAlign.end,
              style: Theme.of(context).textTheme.labelLarge),
        ),
      ],
    );
  }
}

class _DensitySelector extends StatelessWidget {
  const _DensitySelector({required this.value, required this.onChanged});
  final AppDensity value;
  final ValueChanged<AppDensity> onChanged;

  @override
  Widget build(BuildContext context) {
    return SegmentedButton<AppDensity>(
      segments: AppDensity.values
          .map((d) => ButtonSegment(value: d, label: Text(d.label)))
          .toList(),
      selected: {value},
      onSelectionChanged: (s) => onChanged(s.first),
    );
  }
}

class _Preview extends StatelessWidget {
  const _Preview({required this.scheme});
  final ColorScheme scheme;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Card Title',
                style: Theme.of(context).textTheme.titleLarge),
            const SizedBox(height: 4),
            Text(
              'This is a preview card showing how your theme choices look.',
              style: Theme.of(context).textTheme.bodyMedium,
            ),
            const SizedBox(height: 16),
            const TextField(
              decoration: InputDecoration(
                labelText: 'Sample input',
                hintText: 'Type here...',
              ),
            ),
            const SizedBox(height: 16),
            Row(
              children: [
                ElevatedButton(onPressed: () {}, child: const Text('Primary')),
                const SizedBox(width: 8),
                OutlinedButton(onPressed: () {}, child: const Text('Outline')),
                const SizedBox(width: 8),
                TextButton(onPressed: () {}, child: const Text('Text')),
              ],
            ),
            const SizedBox(height: 12),
            Wrap(
              spacing: 8,
              children: [
                const Chip(label: Text('Active')),
                Chip(
                  label: const Text('Selected'),
                  backgroundColor: scheme.primaryContainer,
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
