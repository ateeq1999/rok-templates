//! An admin's work through the client: the users, invitations, the audit
//! log and the job queue — what `users.rs`, `invitations.rs`,
//! `audit_log_entry.rs` and `jobs_api.rs` drive through the router.

use api_client::{
    AuditLogListQuery, InviteUserDto, JobStatus, JobsListQuery, ResetPasswordDto, Role, SortOrder,
    UpdateUserDto, UserSort, UsersListQuery, UsersRemoveQuery,
};

use crate::support::{PASSWORD, TestResult, admin, mailed, mailed_token, user};
use crate::{Api, credentials, session};

#[tokio::test]
async fn an_admin_lists_gets_updates_and_removes_users() -> TestResult {
    let Some(api) = Api::start().await? else {
        return Ok(());
    };
    for email in ["bob@example.com", "alice@example.com"] {
        api.register_verified(email, PASSWORD).await?;
    }
    api.client.set_token(Some(admin(&api.db.pool).await?));

    let query = UsersListQuery {
        q: Some("EXAMPLE.COM".to_owned()),
        sort: Some(UserSort::Email),
        order: Some(SortOrder::Asc),
        per_page: Some(1),
        ..UsersListQuery::default()
    };
    let page = api.client.users().list(&query).await?;
    assert_eq!((page.total, page.page, page.per_page), (2, 1, 1));
    let alice = page.items.first().ok_or("an empty page")?;
    assert_eq!(alice.email, "alice@example.com");
    assert_eq!(api.client.users().get_one(&alice.id).await?, *alice);

    let promoted = UpdateUserDto {
        role: Some(Some(Role::Admin)),
        display_name: Some(Some("Alice".to_owned())),
        ..UpdateUserDto::default()
    };
    let updated = api.client.users().update(&alice.id, &promoted).await?;
    assert_eq!(updated.role, Role::Admin);
    assert_eq!(updated.display_name.as_deref(), Some("Alice"));

    let gone = UsersRemoveQuery::default();
    api.client.users().remove(&alice.id, &gone).await?;
    let err = api
        .client
        .users()
        .get_one(&alice.id)
        .await
        .err()
        .ok_or("a removed user was found")?;
    assert_eq!((err.status, err.code.as_str()), (404, "not_found"));
    api.stop().await
}

#[tokio::test]
async fn the_admin_routes_are_for_admins_only() -> TestResult {
    let Some(api) = Api::start().await? else {
        return Ok(());
    };
    api.client.set_token(Some(user(&api.db.pool).await?));
    let err = api
        .client
        .users()
        .list(&UsersListQuery::default())
        .await
        .err()
        .ok_or("a user listed the users")?;
    assert_eq!((err.status, err.code.as_str()), (403, "forbidden"));
    let err = api
        .client
        .jobs()
        .list(&JobsListQuery::default())
        .await
        .err()
        .ok_or("a user listed the jobs")?;
    assert_eq!(err.status, 403);
    api.stop().await
}

#[tokio::test]
async fn an_invitee_sets_a_password_and_signs_in() -> TestResult {
    let Some(api) = Api::start().await? else {
        return Ok(());
    };
    api.client.set_token(Some(admin(&api.db.pool).await?));
    let invited = api
        .client
        .auth()
        .invite(&InviteUserDto {
            email: " Guest@Example.com ".to_owned(),
            display_name: Some(Some("Guest".to_owned())),
            role: None,
        })
        .await?;
    assert_eq!(invited.email, "guest@example.com");
    assert_eq!(invited.role, Role::User);
    assert!(!invited.has_password);

    // The invitation is in the log.
    let log = api
        .client
        .audit_log()
        .list(&AuditLogListQuery::default())
        .await?;
    let entry = log.items.first().ok_or("an empty log")?;
    assert_eq!(entry.action, "user.invited");
    assert_eq!(entry.target_user_id.as_deref(), Some(invited.id.as_str()));
    assert_eq!(api.client.audit_log().get_one(&entry.id).await?, *entry);

    // The invitation is a job until the worker sends it.
    let pending = JobsListQuery {
        status: Some(JobStatus::Pending),
        ..JobsListQuery::default()
    };
    let queued = api.client.jobs().list(&pending).await?;
    assert!(queued.total >= 1, "{queued:?}");

    let token = mailed(&api.db.pool)
        .await?
        .iter()
        .find(|mail| mail.to == "guest@example.com")
        .and_then(mailed_token)
        .ok_or("no invitation mail")?;
    api.client.set_token(None);
    api.client
        .auth()
        .reset(&ResetPasswordDto {
            token,
            new_password: PASSWORD.to_owned(),
        })
        .await?;
    let login = credentials("guest@example.com", PASSWORD);
    api.client
        .set_token(Some(session(api.client.auth().login(&login).await?)?));
    let me = api.client.auth().me().await?;
    assert_eq!(me.id, invited.id);
    assert!(me.email_verified && me.has_password);
    api.stop().await
}
