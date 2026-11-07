-- The guards' reads: one query per rule, which any feature may call
-- (`db::queries::shared::access`).

-- name: StaffRole :maybe_one
-- The staff role of the account `$1`, if it is staff.
SELECT role
FROM staff_members
WHERE user_id = $1;

-- name: PartnerAccess :maybe_one
-- How the account `$1` may act for the partner `$2`: its role there as a
-- member, its staff role, and the partner's kind; none when there is no
-- such partner.
SELECT p.kind, m.role AS member_role, s.role AS staff_role
FROM partners p
LEFT JOIN partner_members m ON m.partner_id = p.id AND m.user_id = $1
LEFT JOIN staff_members s ON s.user_id = $1
WHERE p.id = $2;
