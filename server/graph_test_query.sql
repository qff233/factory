SET search_path TO track;

SELECT 
	e.id,
	'从 ' || src.name || ' 到 ' || tgt.name as edge_description,
	src.name as begin_name,
	tgt.name as end_name,
	e.cost,
	e.reverse_cost
FROM track.edges e
JOIN track.nodes src ON e.begin_node_id = src.id
JOIN track.nodes tgt ON e.end_node_id = tgt.id
ORDER BY src.name, tgt.name;

WITH path AS (
SELECT * FROM
	track.pgr_astar(
	'SELECT 
        e.id, 
        e.begin_node_id AS source, 
        e.end_node_id AS target, 
        e.cost,
        e.reverse_cost,
        track.ST_X(n_src.geom) AS x1, 
        track.ST_Y(n_src.geom) AS y1,
        track.ST_X(n_tgt.geom) AS x2,
        track.ST_Y(n_tgt.geom) AS y2
    FROM track.edges AS e
    JOIN track.nodes AS n_src ON e.begin_node_id = n_src.id
    JOIN track.nodes AS n_tgt ON e.end_node_id = n_tgt.id',
		6,
		5,
		true)
)
SELECT
	nodes.name
FROM path
JOIN nodes ON path.node = nodes.id;