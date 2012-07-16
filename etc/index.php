<html>
<head>
<title>CARGOFAX</title>
<script src="selectme.js"></script>
<style>
.selected { background: #FFFFC0; }
.bestcccc1232123 { background: #80FF80; }
</style>
</head>
<body>
<h1>CARGOFAX</h1>
<table border=1><tbody id="bigtable">
<tr class="header"><td></td>

<?php
$dbh = new PDO('sqlite:/var/run/takoyaki/arena.db'); 
$query = "select * from maps order by id asc";
$maps = [];
foreach ($dbh->query("select * from maps order by id asc") as $row) {
  $maps[] = $row["id"]; ?>
  <th><?=$row["mapname"]?></th>
<?php } ?>
</tr>

<?php
foreach ($dbh->query("select * from snapshots order by id desc") as $snap) {
?><tr><th id="name"><?=$snap["name"]?></th>

<?php
foreach ($maps as $map) {
?><td><?
$q = $dbh->query("select * from runs where snapid=".$snap["id"]." and mapid=".$map);
foreach($q as $res) { echo $res["score"]."<br>"; }
?></td><?
}
?>
</tr><?
}

$dbh = NULL;

?>

</tbody></table>
</body>
</html>