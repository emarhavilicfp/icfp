function setup() {
  var table = document.getElementById('bigtable');
  for (r in table.childNodes) {
    if (table.childNodes[r].className == "header") {
      continue;
    }
    if (table.childNodes[r].nodeType != 1) continue;
    function genselect(rp) {
      return function() { table.childNodes[rp].className = (table.childNodes[rp].className == "selected") ? null : "selected"; recalculate(); };
    }
    table.childNodes[r].addEventListener('click', genselect(r), false);
  }
  recalculate();
}

function recalculate() {
  var table = document.getElementById('bigtable');
  var best = [];
  var sels = false;
  var besttot = 0;
  
  for (r in table.childNodes) {
    var row = table.childNodes[r];
    if (row.className == "selected") {
      sels = true;
      break;
    }
  }

  for (r in table.childNodes) {
    var row = table.childNodes[r];
    var tot = 0;
    if (row.className == "header") {
      continue;
    }
    if (sels && row.className != "selected") continue;
    for (c in row.childNodes) {
      var col = row.childNodes[c];
      var val = parseInt(row.childNodes[c].innerHTML);
      
      if (c == 0) continue;
      if (best[c] == undefined) best[c] = 0;
      if (val > best[c]) best[c] = val;
      if (!isNaN(val)) tot += val;
    }
    if (tot > besttot) besttot = tot;
  }
  console.log(besttot);
  for (r in table.childNodes) {
    var row = table.childNodes[r];
    var live = !sels || (row.className == "selected");
    var tot = 0;
    if (row.className == "header") {
      continue;
    }
    for (c in row.childNodes) {
      var col = row.childNodes[c];
      var val = parseInt(row.childNodes[c].innerHTML);
      
      if (c == 0) continue;
      if (!isNaN(val)) tot += val;
      col.className = (best[c] == val && live) ? "bestcccc1232123" : "";
    }
    console.log(tot, besttot);
    for (c in row.childNodes) { 
      if (row.childNodes[c].id == "name") 
        row.childNodes[c].className = (tot == besttot) ? "bestcccc1232123" : 0;
    }
  }
}


window.onload = setup;