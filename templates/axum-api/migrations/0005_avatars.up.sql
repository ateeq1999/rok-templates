ALTER TABLE "users" ADD COLUMN "avatar_url" TEXT GENERATED ALWAYS AS ('/api/v1/' || avatar_key) STORED;
