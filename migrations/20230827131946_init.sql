CREATE TABLE IF NOT EXISTS "users" (
    "user_id" BIGINT PRIMARY KEY,
    "user_name" TEXT UNIQUE,
    "full_name" TEXT NOT NULL
);

CREATE INDEX "idx_users_user_name" on "users" ("user_name");

CREATE TABLE IF NOT EXISTS "chats" (
    "chat_id" BIGINT PRIMARY KEY,
    "chat_name" TEXT
);

CREATE TABLE IF NOT EXISTS "notes" (
    "chat_id" BIGINT,
    "note_id" TEXT,
    "note_content" TEXT NOT NULL,
    PRIMARY KEY("chat_id", "note_id"),
    CONSTRAINT "fk_notes" FOREIGN KEY ("chat_id") REFERENCES "chats" ("chat_id")
);