import 'package:dio/dio.dart';

class UserProvider {
  UserProvider(this._dio);

  final Dio _dio;

  // BUG-U6: was /api/users/me
  Future<Response<dynamic>> getMe() => _dio.get('/v1/api/users/me');

  Future<Response<dynamic>> updateProfile({String? name, String? email}) {
    final data = <String, dynamic>{};
    if (name != null) data['name'] = name;
    if (email != null) data['email'] = email;
    return _dio.patch('/v1/api/users/me', data: data);
  }

  Future<Response<dynamic>> uploadAvatar(String filePath) async {
    final formData = FormData.fromMap({
      'avatar': await MultipartFile.fromFile(filePath),
    });
    return _dio.post(
      '/v1/api/users/me/avatar',
      data: formData,
      options: Options(contentType: 'multipart/form-data'),
    );
  }

  Future<Response<dynamic>> deleteAvatar() =>
      _dio.delete('/v1/api/users/me/avatar');

  Future<Response<dynamic>> deleteAccount({String? password}) => _dio.delete(
        '/v1/api/users/me',
        data: password == null ? null : {'password': password},
      );

  // BUG-U5: was /api/users/me/devices
  Future<Response<dynamic>> registerDeviceToken({
    required String token,
    required String platform,
  }) =>
      _dio.post(
        '/v1/api/users/me/device-tokens',
        data: {'token': token, 'platform': platform},
      );

  // BUG-U5: was /api/users/me/devices
  Future<Response<dynamic>> deleteDeviceToken({required String tokenId}) =>
      _dio.delete('/v1/api/users/me/device-tokens/$tokenId');
}
