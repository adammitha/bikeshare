CREATE TABLE IF NOT EXISTS cache (
	timestamp TEXT NOT NULL,
	name TEXT,
	longitude REAL,
	latitude REAL,
	total_slots INTEGER,
	free_slots INTEGER,
	avl_bikes INTEGER,
	operative INTEGER,
	style TEXT,
	is_estation INTEGER
);
