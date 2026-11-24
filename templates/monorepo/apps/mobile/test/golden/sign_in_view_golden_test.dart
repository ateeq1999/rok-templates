import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';

import 'package:mobile/app/core/config/feature_flags.dart';
import 'package:mobile/app/core/theme/app_theme.dart';
import 'package:mobile/app/core/theme/theme_preferences.dart';
import 'package:mobile/app/modules/auth/auth_bloc.dart';
import 'package:mobile/app/modules/auth/continue_with/continue_with_cubit.dart';
import 'package:mobile/app/modules/auth/sign_in/sign_in_cubit.dart';
import 'package:mobile/app/modules/auth/sign_in/sign_in_view.dart';

class _SignInCubit extends MockCubit<SignInState> implements SignInCubit {
  @override
  bool get isPasswordHidden => true;
}

class _AuthBloc extends MockBloc<AuthEvent, AuthState> implements AuthBloc {}

class _ContinueWithCubit extends MockCubit<List<String>>
    implements ContinueWithCubit {}

/// The sign-in screen as it opens, checked against
/// `goldens/sign_in_view.png`, its design. No API runs: the screen's cubits
/// are stubs. `flutter_test_config.dart` loads the App's bundled fonts and
/// forgives only the glyph-edge shading that differs between operating
/// systems. After a deliberate change to the screen or the theme, redraw
/// the golden with `flutter test --update-goldens test/golden` and review it.
void main() {
  testWidgets('the sign-in screen matches its golden', (tester) async {
    // A phone's screen, 390x844 logical pixels.
    tester.view.physicalSize = const Size(390, 844);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);

    final signIn = _SignInCubit();
    final auth = _AuthBloc();
    final continueWith = _ContinueWithCubit();
    when(() => signIn.state).thenReturn(const SignInInitial());
    when(() => auth.state).thenReturn(const AuthUnauthenticated());
    when(() => continueWith.state).thenReturn(const ['google', 'github']);

    await tester.pumpWidget(
      RepositoryProvider<FeatureFlags>.value(
        value: const FeatureFlags(
          twoFactor: true,
          signUp: true,
          themeCustomization: true,
        ),
        child: MultiBlocProvider(
          providers: [
            BlocProvider<AuthBloc>.value(value: auth),
            BlocProvider<SignInCubit>.value(value: signIn),
            BlocProvider<ContinueWithCubit>.value(value: continueWith),
          ],
          child: MaterialApp(
            debugShowCheckedModeBanner: false,
            theme: AppTheme.build(const ThemePreferences(), Brightness.light),
            home: const SignInView(),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    await expectLater(
      find.byType(SignInView),
      matchesGoldenFile('goldens/sign_in_view.png'),
    );
  });
}
