SET search_path TO track;

INSERT INTO
nodes(name, type, geom)
VALUES ('P1','parking_station', ST_MakePoint(2,0,0));
INSERT INTO
nodes(name, type, geom)
VALUES ('P2','parking_station', ST_MakePoint(0,0,0));
INSERT INTO
nodes(name, type, geom)
VALUES ('C1','charging_station', ST_MakePoint(1,0,0));

INSERT INTO 
nodes(name, type, side,geom)
VALUES ('S1','item_stocker', 'posz',ST_MakePoint(1,3,0));
INSERT INTO 
nodes(name, type, side,geom)
VALUES ('S2','item_stocker', 'posz',ST_MakePoint(-1,1,0));
INSERT INTO 
nodes(name, type, side,geom)
VALUES ('S3','shipping_dock', 'posz',ST_MakePoint(-1,2,0));

INSERT INTO 
nodes(name, type, geom)
VALUES ('A1','fork',ST_MakePoint(2,1,0));
INSERT INTO 
nodes(name, type, geom)
VALUES ('A2','fork',ST_MakePoint(1,1,0));
INSERT INTO 
nodes(name, type, geom)
VALUES ('A3','fork',ST_MakePoint(1,2,0));
INSERT INTO 
nodes(name, type, geom)
VALUES ('A4','fork',ST_MakePoint(2,2,0));
INSERT INTO 
nodes(name, type, geom)
VALUES ('A5','fork',ST_MakePoint(0,2,0));
INSERT INTO 
nodes(name, type, geom)
VALUES ('A6','fork',ST_MakePoint(0,1,0));

WITH node_pairs AS (
	SELECT
		src.id as begin_id,
		tgt.id as end_id,
		pairs.direction as direction
	FROM ( VALUES
		('P2','A6','bidirectional'),
		('C1','A2','bidirectional'),
		('P1','A1','bidirectional'),
		('S1','A3','bidirectional'),
		('S2','A6','bidirectional'),
		('S3','A5','bidirectional'),
		('A6','A2','unidirectional'),
		('A2','A1','unidirectional'),
		('A1','A4','unidirectional'),
		('A4','A3','unidirectional'),
		('A3','A2','unidirectional'),
		('A3','A5','unidirectional'),
		('A5','A6','unidirectional')
	) AS pairs(begin_name, end_name, direction)
	JOIN nodes src ON src.name = pairs.begin_name
	JOIN nodes tgt ON tgt.name = pairs.end_name
)
INSERT INTO edges(begin_node_id, end_node_id, direction)
SELECT begin_id, end_id, direction FROM node_pairs;