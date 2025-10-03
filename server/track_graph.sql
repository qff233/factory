SET search_path TO track;

DROP TABLE edges;
DROP TABLE nodes;
DROP TYPE NodeType;
DROP TYPE Side;
DROP TYPE Direction;

CREATE TYPE NodeType AS ENUM(
	'fork',
	'charging_station',
	'parking_station',
	'shipping_dock',
	'item_stocker',
	'fluid_stocker'
);

CREATE TYPE Side AS ENUM(
	'posx',
	'posy',
	'posz',
	'negx',
	'negy',
	'negz'
);

CREATE TYPE Direction AS ENUM(
	'BIDIRECTIONAL',
	'UNIDIRECTIONAL'
);

CREATE TABLE nodes(
	id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	name VARCHAR(50) NOT NULL UNIQUE,
	type NodeType NOT NULL,
	side side,
	geom geometry(PointZ) NOT NULL,
	comment TEXT,
	CONSTRAINT side_should_is_not_null_in_some_type
	CHECK(
		CASE
			WHEN type in (
				'shipping_dock',
				'item_stocker',
				'fluid_stocker') 
			THEN side IS NOT NULL
			ELSE true
		END
	)
);

CREATE TABLE edges(
	id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	begin_node_id INT REFERENCES nodes(id),
	end_node_id INT REFERENCES nodes(id),
	direction VARCHAR(15),
	cost FLOAT,
	reverse_cost FLOAT,
	is_lock BOOL DEFAULT false
);

CREATE OR REPLACE FUNCTION calculate_manhatten_distance(
	node1_id INT,
	node2_id INT
) 
RETURNS FLOAT
LANGUAGE plpgsql 
AS $$
DECLARE
	node1_geom geometry;
	node2_geom geometry;
	x1 FLOAT;
	y1 FLOAT;
	z1 FLOAT;
	x2 FLOAT;
	y2 FLOAT;
	z2 FLOAT;
	distance FLOAT;
BEGIN
	SELECT geom INTO node1_geom
	FROM nodes
	WHERE id = node1_id;

	SELECT geom INTO node2_geom
	FROM nodes
	WHERE id = node2_id;

	x1 := ST_X(node1_geom);
	y1 := ST_Y(node1_geom);
	z1 := ST_Z(node1_geom);

	x2 := ST_X(node2_geom);
	y2 := ST_Y(node2_geom);
	z2 := ST_Z(node2_geom);

	distance := ABS(x1-x2) + ABS(y1-y2) + ABS(z1-z2);
	RETURN distance;
END;
$$;

CREATE OR REPLACE FUNCTION update_edge_weight_on_edge_change()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
	NEW.cost := calculate_manhatten_distance(NEW.begin_node_id, NEW.end_node_id);
	NEW.reverse_cost := CASE 
		WHEN NEW.direction='bidirectional' THEN NEW.cost
		ELSE -1
	END;
	return NEW;
END;
$$;

CREATE TRIGGER update_edge_weight_trigger
BEFORE INSERT OR UPDATE OF begin_node_id,end_node_id ON edges
FOR EACH ROW
EXECUTE FUNCTION update_edge_weight_on_edge_change();

CREATE OR REPLACE FUNCTION update_edge_weight_on_node_change()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
DECLARE
	cost FLOAT;
BEGIN
	cost := calculate_manhatten_distance(begin_node_id, end_node_id);
	UPDATE edges 
	SET
		cost = cost,
		reverse_cost = CASE 
			WHEN direction='bidirectional' THEN cost
			ELSE -1
		END
	WHERE begin_node_id = NEW.id OR end_node_id = NEW.id;
	return NEW;
END;
$$;

CREATE TRIGGER update_edge_weight_trigger
BEFORE UPDATE OF geom ON nodes
FOR EACH ROW
EXECUTE FUNCTION update_edge_weight_on_node_change();
