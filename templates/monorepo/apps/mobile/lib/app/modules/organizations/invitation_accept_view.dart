import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../data/repositories/organization.repository.dart';
import '../../routes/app_routes.dart';

class InvitationAcceptView extends StatefulWidget {
  const InvitationAcceptView({
    super.key,
    required this.token,
    required this.orgRepository,
  });

  final String token;
  final OrganizationRepository orgRepository;

  @override
  State<InvitationAcceptView> createState() => _InvitationAcceptViewState();
}

class _InvitationAcceptViewState extends State<InvitationAcceptView> {
  _Status _status = _Status.loading;
  String? _error;

  @override
  void initState() {
    super.initState();
    _accept();
  }

  Future<void> _accept() async {
    if (widget.token.isEmpty) {
      setState(() {
        _status = _Status.error;
        _error = 'Invitation token is missing or invalid.';
      });
      return;
    }
    try {
      await widget.orgRepository.acceptInvitation(widget.token);
      if (!mounted) return;
      setState(() => _status = _Status.success);
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _status = _Status.error;
        _error = e.toString();
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Organization Invitation')),
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: switch (_status) {
            _Status.loading => const Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  CircularProgressIndicator(),
                  SizedBox(height: 16),
                  Text('Accepting invitation...'),
                ],
              ),
            _Status.success => Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(Icons.check_circle,
                      size: 64,
                      color: Theme.of(context).colorScheme.primary),
                  const SizedBox(height: 16),
                  const Text('Invitation accepted!'),
                  const SizedBox(height: 24),
                  ElevatedButton(
                    onPressed: () => context.go(AppRoutes.organizations),
                    child: const Text('View Organizations'),
                  ),
                ],
              ),
            _Status.error => Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(Icons.error_outline,
                      size: 64, color: Theme.of(context).colorScheme.error),
                  const SizedBox(height: 16),
                  Text(_error ?? 'Failed to accept invitation'),
                  const SizedBox(height: 24),
                  OutlinedButton(
                    onPressed: () => context.go(AppRoutes.home),
                    child: const Text('Back to Home'),
                  ),
                ],
              ),
          },
        ),
      ),
    );
  }
}

enum _Status { loading, success, error }
