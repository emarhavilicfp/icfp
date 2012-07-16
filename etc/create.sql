CREATE TABLE snapshots (id INTEGER PRIMARY KEY AUTOINCREMENT, name varchar, snapshot varchar, runcmd varchar);
CREATE TABLE maps (id INTEGER PRIMARY KEY AUTOINCREMENT, mapname varchar);
CREATE TABLE runs (id INTEGER PRIMARY KEY AUTOINCREMENT, snapid INTEGER, mapid INTEGER, score INTEGER);
