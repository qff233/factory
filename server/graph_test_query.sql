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

UPDATE edges
SET is_lock = false
WHERE id=10;

-- should error
-- INSERT INTO 
-- nodes(name, type,geom)
-- VALUES ('S11','item_stocker',ST_MakePoint(1,3,0));

-- find shortest node
select id, name, ST_X(geom) as x, ST_X(geom) as y, ST_X(geom) as z, ST_Distance(geom, ST_MakePoint(8,8,8)) as dist
FROM nodes
ORDER by dist
LIMIT 1;

WITH path AS (
SELECT * FROM
	track.pgr_astar(
	'SELECT 
        e.id, 
        e.begin_node_id AS source, 
        e.end_node_id AS target, 
        e.cost,
        e.reverse_cost,
        ST_X(n_src.geom) AS x1, 
        ST_Y(n_src.geom) AS y1,
        ST_X(n_tgt.geom) AS x2,
        ST_Y(n_tgt.geom) AS y2
    FROM edges AS e
    JOIN nodes AS n_src ON e.begin_node_id = n_src.id
    JOIN nodes AS n_tgt ON e.end_node_id = n_tgt.id
	WHERE is_lock=false',
		(SELECT id FROM nodes WHERE name = 'S3'),
		(SELECT id FROM nodes WHERE name = 'S2'),
		true)
)

SELECT
	nodes.name
FROM path
JOIN nodes ON path.node = nodes.id;