import 'package:api_client/api_client.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:image_picker/image_picker.dart';

import '../../data/repositories/auth.repository.dart';
import '../../data/repositories/user.repository.dart';
import '../auth/auth_bloc.dart';

part 'profile_state.dart';

class ProfileCubit extends Cubit<ProfileState> {
  ProfileCubit({
    required UserRepository userRepository,
    required AuthRepository authRepository,
    required AuthBloc authBloc,
    ImagePicker? imagePicker,
  })  : _userRepository = userRepository,
        _authRepository = authRepository,
        _authBloc = authBloc,
        _imagePicker = imagePicker ?? ImagePicker(),
        super(const ProfileInitial());

  final UserRepository _userRepository;
  final AuthRepository _authRepository;
  final AuthBloc _authBloc;
  final ImagePicker _imagePicker;

  /// The profile photo's URL, or `null` without one.
  Uri? avatarUri(UserResponse user) => _userRepository.avatarUri(user);

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

      final authState = _authBloc.state;
      if (authState is! AuthAuthenticated) return;
      final contentType = _imageType(pickedFile);
      if (contentType == null) {
        emit(const ProfileFailure('Choose a JPEG, PNG or WebP image.'));
        emit(ProfileLoaded(authState.user));
        return;
      }
      emit(const ProfileLoading());

      final updatedUser = await _userRepository.uploadAvatar(
        await pickedFile.readAsBytes(),
        contentType,
      );
      _changed(updatedUser, authState.token);
    } on ApiException catch (e) {
      _failed(e.message);
    } catch (e) {
      _failed(e.toString());
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
      final updatedUser = await _userRepository.updateDisplayName(trimmed);
      _changed(updatedUser, authState.token);
    } on ApiException catch (e) {
      _failed(e.message);
    } catch (e) {
      _failed(e.toString());
    }
  }

  /// Mails a new link to verify the signed-in user's email.
  Future<void> resendVerification() async {
    final authState = _authBloc.state;
    if (authState is! AuthAuthenticated) return;
    try {
      await _authRepository.sendVerificationEmail();
      emit(const ProfileVerificationSent());
      emit(ProfileLoaded(authState.user));
    } on ApiException catch (e) {
      _failed(e.message);
    } catch (e) {
      _failed(e.toString());
    }
  }

  void _changed(UserResponse user, String token) {
    _authBloc.add(AuthUserChanged(user: user, token: token));
    emit(ProfileLoaded(user));
  }

  /// Reports `message`, then shows the profile again.
  void _failed(String message) {
    emit(ProfileFailure(message));
    final authState = _authBloc.state;
    if (authState is AuthAuthenticated) emit(ProfileLoaded(authState.user));
  }

  /// The media type the API takes for `file`, from its name; `null` when
  /// it takes none (its avatars are JPEG, PNG or WebP).
  static String? _imageType(XFile file) {
    final name = file.name.toLowerCase();
    if (name.endsWith('.png')) return 'image/png';
    if (name.endsWith('.webp')) return 'image/webp';
    if (name.endsWith('.jpg') || name.endsWith('.jpeg')) return 'image/jpeg';
    return file.mimeType;
  }
}
