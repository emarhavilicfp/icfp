#!/usr/bin/lua5.1

-- First off, pull.
if os.execute('cd ctl; git pull') ~= 0 then
    print"...pull failed..."
    os.exit(1)
end

require"DBI"

db = assert(DBI.Connect("SQLite3", "arena.db"))

lookup_snap = db:prepare"SELECT * FROM snapshots WHERE name=?;"
create_snap = db:prepare"INSERT INTO snapshots (name, snapshot, runcmd) VALUES (?, ?, ?);"
lookup_maps = db:prepare"SELECT * FROM maps WHERE mapname=?;"
create_maps = db:prepare"INSERT INTO maps (mapname) VALUES (?);"

-- Look up all the snapshots.
local f = assert(io.open("ctl/snapshots", "r"))
for ref,name,cmd in f:read("*all"):gfind("([^\t]+)\t([^\t]+)\t([^\n]+)\n") do
    assert(lookup_snap:execute(name))
    if not lookup_snap:fetch() then
        print("created snapshot, name: "..name)
        assert(create_snap:execute(name, ref, cmd))
    end
end
f:close()

-- Look up all the maps.
local f = assert(io.open("ctl/maplist", "r"))
for name in f:read("*all"):gfind("([^\n]+)\n") do
    assert(lookup_maps:execute(name))
    if not lookup_maps:fetch() then
        print("created map, name: "..name)
        create_maps:execute(name)
    end
end

assert(db:commit())
assert(db:close())

print"db closed"

if not arg[1] then
	local f = io.open("update", "w")
	f:write("\n")
	f:close()
end

