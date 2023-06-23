CREATE TABLE IF NOT EXISTS cache (
	time        TIMESTAMP WITH TIME ZONE NOT NULL,
	name        TEXT NOT NULL,
	longitude   REAL,
	latitude    REAL,
	total_slots INTEGER NOT NULL,
	free_slots  INTEGER NOT NULL,
	avl_bikes   INTEGER NOT NULL,
	operative   BOOLEAN NOT NULL,
	style       TEXT NOT NULL,
	is_estation BOOLEAN NOT NULL
);
