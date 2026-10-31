import 'package:dio/dio.dart';

import '../models/user.model.dart';
import '../providers/user.provider.dart';
import '../../core/errors/app_exception.dart';
import '../../core/utils/response_parser.dart';

class UserRepository {
  UserRepository(this._provider);

  final UserProvider _provider;

  Future<User> getMe() async {
    try {
      final response = await _provider.getMe();
      return User.fromJson(unwrapEnvelope(response.data));
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<User> updateProfile({String? name, String? email}) async {
    try {
      final response = await _provider.updateProfile(name: name, email: email);
      return User.fromJson(unwrapEnvelope(response.data));
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<User> uploadAvatar(String filePath) async {
    try {
      final response = await _provider.uploadAvatar(filePath);
      return User.fromJson(unwrapEnvelope(response.data));
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<void> deleteAvatar() async {
    try {
      await _provider.deleteAvatar();
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<void> deleteAccount({String? password}) async {
    try {
      await _provider.deleteAccount(password: password);
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<void> registerDeviceToken({
    required String token,
    required String platform,
  }) async {
    try {
      await _provider.registerDeviceToken(token: token, platform: platform);
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<void> deleteDeviceToken({required String tokenId}) async {
    try {
      await _provider.deleteDeviceToken(tokenId: tokenId);
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }
}
