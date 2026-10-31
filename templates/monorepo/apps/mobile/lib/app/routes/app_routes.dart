abstract class AppRoutes {
  static const splash = '/splash';
  static const signIn = '/sign-in';
  static const signUp = '/sign-up';
  static const forgotPassword = '/forgot-password';
  static const resetPassword = '/reset-password';
  static const verifyEmail = '/verify-email';
  static const twoFactor = '/two-factor';
  static const magicLink = '/magic-link';
  static const home = '/home';
  static const profile = '/profile';
  static const settings = '/settings';
  static const appearance = '/settings/appearance';

  // Admin
  static const admin = '/admin';
  static const adminUsers = '/admin/users';
  static const adminAuditLogs = '/admin/audit-logs';
  static String adminUserDetail(String id) => '/admin/users/$id';

  // Organizations
  static const organizations = '/organizations';
  static const orgInvitationAccept = '/invitations/accept';
  static String orgDetail(String id) => '/organizations/$id';
  static String orgInvite(String id) => '/organizations/$id/invite';
}
