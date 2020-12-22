BEGIN TRANSACTION;
CREATE TABLE IF NOT EXISTS "triggers" (
	"id"	INTEGER UNIQUE,
	"trigger"	TEXT NOT NULL,
	"code"	TEXT NOT NULL,
	"guild_id"	TEXT NOT NULL COLLATE BINARY,
	PRIMARY KEY("id" AUTOINCREMENT)
);
CREATE TABLE IF NOT EXISTS "databases" (
	"id"	INTEGER,
	"name"	TEXT NOT NULL,
	"guild_id"	TEXT NOT NULL COLLATE BINARY,
	"content"	TEXT NOT NULL COLLATE UTF16,
	PRIMARY KEY("id" AUTOINCREMENT)
);
CREATE UNIQUE INDEX IF NOT EXISTS "triggers_idx" ON "triggers" (
	"trigger",
	"guild_id"
);
CREATE UNIQUE INDEX IF NOT EXISTS "db_names_idx" ON "databases" (
	"name",
	"guild_id"
);
COMMIT;
