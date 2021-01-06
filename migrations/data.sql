BEGIN TRANSACTION;
CREATE TABLE IF NOT EXISTS "triggers" (
	"trigger"	TEXT NOT NULL,
	"code"	TEXT NOT NULL,
	"guild_id"	TEXT NOT NULL COLLATE BINARY
);
CREATE TABLE IF NOT EXISTS "databases" (
	"name"	TEXT NOT NULL,
	"guild_id"	TEXT NOT NULL COLLATE BINARY,
	"key_name"	TEXT NOT NULL,
	"key_value"	TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS "events" (
	"event"	TEXT NOT NULL,
	"guild_id"	TEXT NOT NULL,
	"code"	TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS "triggers_idx" ON "triggers" (
	"trigger",
	"guild_id"
);
CREATE UNIQUE INDEX IF NOT EXISTS "databases_idx" ON "databases" (
	"name",
	"guild_id",
	"key_name"
);
CREATE UNIQUE INDEX IF NOT EXISTS "events_idx" ON "events" (
	"guild_id",
	"event"
);
COMMIT;
