-- Your SQL goes here
CREATE TABLE jks (
    row_id      INTEGER PRIMARY KEY AUTOINCREMENT,
    id          INTEGER NOT NULL,
    cislo_stofy INTEGER NOT NULL,
    text        TEXT NOT NULL
);

CREATE TABLE song_types (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE jks_types (
    song_id INTEGER NOT NULL,
    type_id INTEGER NOT NULL,

    PRIMARY KEY (song_id, type_id),

    FOREIGN KEY(song_id) REFERENCES jks(row_id) ON DELETE CASCADE,
    FOREIGN KEY(type_id) REFERENCES song_types(id) ON DELETE CASCADE
);