#!/usr/bin/lua5.1

require"DBI"

db = assert(DBI.Connect("SQLite3", "arena.db"))

lookup_snap = db:prepare"SELECT * FROM snapshots WHERE name=?;"
create_snap = db:prepare"INSERT INTO snapshots (name, snapshot, runcmd) VALUES (?, ?, ?);"
lookup_maps = db:prepare"SELECT * FROM maps WHERE mapname=?;"
create_maps = db:prepare"INSERT INTO maps (mapname) VALUES (?);"

lookup_run = db:prepare"SELECT * FROM runs WHERE snapid=? AND mapid=?;"
function has_run(snap, map)
    lookup_run:execute(snap, map)
    return lookup_run:fetch() and true or false
end
create_run = db:prepare"INSERT INTO runs (snapid, mapid, score) VALUES (?,?,?);"


while true do
    print"update!"
    local snaps = {}
    local maps = {}
    
    while os.execute("cd icfp; git fetch --all") ~= 0 do
        os.execute("sleep 5") -- hahaguy
    end
    
    local lookup_snap = db:prepare"SELECT * FROM snapshots;"
    local lookup_maps = db:prepare"SELECT * FROM maps;"
    
    lookup_snap:execute()
    for r in lookup_snap:rows(true) do
        print("  loaded snap "..r.name)
        snaps[r.id] = r
    end
    
    lookup_maps:execute()
    for r in lookup_maps:rows(true) do
        print("  loaded map "..r.mapname)
        maps[r.id] = r
    end
    
    for _,snap in pairs(snaps) do
        local snap_loaded = false
        for _,map in pairs(maps) do
            if not has_run(snap.id, map.id) then
                if not snap_loaded then
                    print ("  loading snap "..snap.name)
                    if os.execute("cd icfp; git checkout "..snap.snapshot) ~= 0 then
                        print "git checkout failed"
                        break
                    end
                    if os.execute("cd icfp; make clean; make") ~= 0 then
                        print "build failed"
                        break
                    end
                    snap_loaded = true
                end
                print ("  running snap "..snap.snapshot.." with map "..map.mapname)
                local f = io.popen("cat ctl/maps/"..map.mapname..".map | (cd icfp; "..snap.runcmd..") | tr -d '\n'", "r")
                local resp = f:read("*all")
                f:close()
                print ("  result: "..resp)
                local f = io.popen("./testseq ctl/maps/"..map.mapname..".map "..resp, "r")
                local pts = f:read("*all")
                print ("  good for "..pts.." points")
                f:close()
                
                create_run:execute(snap.id, map.id, pts)
                db:commit()
            end
        end
    end
    db:commit()
    print "  done!"
    
    os.execute('curl -d "message=New CARGOFAX results available: http://icfp.nyus.compound.emarhavil.com/~takoyaki/" http://localhost:5651/status')
    
    -- wait for the update
    local f = io.open("update", "r")
    f:read("*all")
    f:close()
end
