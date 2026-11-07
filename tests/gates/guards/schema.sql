-- The guards recipe's tables, in Karibu Pass's shape: staff with a staff
-- role, and partners of a kind whose members each have a role there.

CREATE TYPE staff_role AS ENUM ('finance', 'support');
CREATE TYPE partner_kind AS ENUM ('organizer', 'venue');
CREATE TYPE partner_role AS ENUM ('owner', 'manager', 'scanner');

CREATE TABLE staff_members (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL UNIQUE REFERENCES users (id),
    role staff_role NOT NULL
);

CREATE TABLE partners (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    kind partner_kind NOT NULL
);

CREATE TABLE partner_members (
    id UUID PRIMARY KEY,
    partner_id UUID NOT NULL REFERENCES partners (id),
    user_id UUID NOT NULL REFERENCES users (id),
    role partner_role NOT NULL,
    UNIQUE (partner_id, user_id)
);
