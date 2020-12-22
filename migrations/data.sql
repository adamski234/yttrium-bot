BEGIN TRANSACTION;
CREATE TABLE IF NOT EXISTS "guilds" (
	"id"	INTEGER UNIQUE,
	"guild_id"	TEXT NOT NULL UNIQUE COLLATE BINARY,
	PRIMARY KEY("id" AUTOINCREMENT)
);
CREATE TABLE IF NOT EXISTS "triggers" (
	"id"	INTEGER UNIQUE,
	"trigger"	TEXT NOT NULL COLLATE UTF16,
	"code"	TEXT NOT NULL COLLATE UTF16,
	"guild_id"	TEXT NOT NULL COLLATE BINARY,
	FOREIGN KEY("guild_id") REFERENCES "guilds"("guild_id"),
	PRIMARY KEY("id" AUTOINCREMENT)
);
CREATE TABLE IF NOT EXISTS "databases" (
	"id"	INTEGER,
	"name"	TEXT NOT NULL COLLATE UTF16,
	"guild_id"	TEXT NOT NULL COLLATE BINARY,
	"content"	TEXT NOT NULL COLLATE UTF16,
	FOREIGN KEY("guild_id") REFERENCES "guilds"("guild_id"),
	PRIMARY KEY("id" AUTOINCREMENT)
);
CREATE UNIQUE INDEX IF NOT EXISTS "guild_idx" ON "guilds" (
	"guild_id"	DESC
);
CREATE INDEX IF NOT EXISTS "triggers_idx" ON "triggers" (
	"trigger",
	"guild_id"
);
CREATE INDEX IF NOT EXISTS "db_names_idx" ON "databases" (
	"name",
	"guild_id"
);
COMMIT;
