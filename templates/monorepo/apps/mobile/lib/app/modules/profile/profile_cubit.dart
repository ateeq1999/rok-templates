import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:image_picker/image_picker.dart';

import '../../data/models/user.model.dart';
import '../../data/repositories/user.repository.dart';
import '../../core/errors/app_exception.dart';
import '../auth/auth_bloc.dart';

part 'profile_state.dart';

class ProfileCubit extends Cubit<ProfileState> {
  ProfileCubit({
    required UserRepository userRepository,
    required AuthBloc authBloc,
  })  : _userRepository = userRepository,
        _authBloc = authBloc,
        super(const ProfileInitial());

  final UserRepository _userRepository;
  final AuthBloc _authBloc;
  final _imagePicker = ImagePicker();

  void loadFromAuthBloc() {
    final authState = _authBloc.state;
    if (authState is AuthAuthenticated) {
      emit(ProfileLoaded(authState.user));
    }
  }

  Future<void> pickAndUploadAvatar() async {
    try {
      final pickedFile = await _imagePicker.pickImage(
        source: ImageSource.gallery,
        maxWidth: 512,
        maxHeight: 512,
        imageQuality: 80,
      );
      if (pickedFile == null) return;

      final current = _currentUser;
      if (current == null) return;
      emit(const ProfileLoading());

      final updatedUser = await _userRepository.uploadAvatar(pickedFile.path);
      _authBloc.add(AuthUserChanged(
        user: updatedUser,
        token: (_authBloc.state as AuthAuthenticated).token,
      ));
      emit(ProfileLoaded(updatedUser));
    } on ApiException catch (e) {
      emit(ProfileFailure(e.message));
    } catch (e) {
      emit(ProfileFailure(e.toString()));
    }
  }

  /// BUG-13 fix: validates that name is non-empty before sending to API.
  Future<void> updateName(String name) async {
    final trimmed = name.trim();
    if (trimmed.isEmpty) return;

    final authState = _authBloc.state;
    if (authState is! AuthAuthenticated) return;

    emit(const ProfileLoading());
    try {
      final updatedUser = await _userRepository.updateProfile(name: trimmed);
      _authBloc.add(AuthUserChanged(user: updatedUser, token: authState.token));
      emit(ProfileLoaded(updatedUser));
    } on ApiException catch (e) {
      emit(ProfileFailure(e.message));
    } catch (e) {
      emit(ProfileFailure(e.toString()));
    }
  }

  User? get _currentUser {
    final s = _authBloc.state;
    return s is AuthAuthenticated ? s.user : null;
  }
}
