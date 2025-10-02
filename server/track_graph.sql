DROP TABLE track.edges;
DROP TABLE track.nodes;
DROP TYPE track.NodeType;
DROP TYPE track.Side;
DROP TYPE TRACK.Direction;

CREATE TYPE track.NodeType AS ENUM(
	'fork',
	'charging_station',
	'parking_station',
	'shipping_dock',
	'item_stocker',
	'fluid_stocker'
);

CREATE TYPE track.Side AS ENUM(
	'posx',
	'posy',
	'posz',
	'negx',
	'negy',
	'negz'
);

CREATE TYPE track.Direction AS ENUM(
	'BIDIRECTIONAL',
	'UNIDIRECTIONAL'
);

CREATE TABLE track.nodes(
	id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	name VARCHAR(50) NOT NULL UNIQUE,
	type track.NodeType NOT NULL,
	side track.side,
	geom track.geometry(PointZ) NOT NULL,
	comment TEXT
);

CREATE TABLE track.edges(
	id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	begin_node_id INT REFERENCES track.nodes(id),
	end_node_id INT REFERENCES track.nodes(id),
	direction VARCHAR(15),
	cost FLOAT,
	reverse_cost FLOAT,
	is_lock BOOL DEFAULT false
);

CREATE OR REPLACE FUNCTION track.calculate_manhatten_distance(
	node1_id INT,
	node2_id INT
) 
RETURNS FLOAT
LANGUAGE plpgsql 
AS $$
DECLARE
	node1_geom track.geometry;
	node2_geom track.geometry;
	x1 FLOAT;
	y1 FLOAT;
	z1 FLOAT;
	x2 FLOAT;
	y2 FLOAT;
	z2 FLOAT;
	distance FLOAT;
BEGIN
	SELECT geom INTO node1_geom
	FROM track.nodes
	WHERE id = node1_id;

	SELECT geom INTO node2_geom
	FROM track.nodes
	WHERE id = node2_id;

	x1 := track.ST_X(node1_geom);
	y1 := track.ST_Y(node1_geom);
	z1 := track.ST_Z(node1_geom);

	x2 := track.ST_X(node2_geom);
	y2 := track.ST_Y(node2_geom);
	z2 := track.ST_Z(node2_geom);

	distance := ABS(x1-x2) + ABS(y1-y2) + ABS(z1-z2);
	RETURN distance;
END;
$$;

CREATE OR REPLACE FUNCTION track.update_edge_weight_on_edge_change()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
	NEW.cost := track.calculate_manhatten_distance(NEW.begin_node_id, NEW.end_node_id);
	NEW.reverse_cost := CASE 
		WHEN NEW.direction='bidirectional' THEN NEW.cost
		ELSE -1
	END;
	return NEW;
END;
$$;

CREATE TRIGGER update_edge_weight_trigger
BEFORE INSERT OR UPDATE OF begin_node_id,end_node_id ON track.edges
FOR EACH ROW
EXECUTE FUNCTION track.update_edge_weight_on_edge_change();

CREATE OR REPLACE FUNCTION track.update_edge_weight_on_node_change()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
DECLARE
	cost FLOAT;
BEGIN
	cost := track.calculate_manhatten_distance(begin_node_id, end_node_id);
	UPDATE track.edges 
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
BEFORE UPDATE OF geom ON track.nodes
FOR EACH ROW
EXECUTE FUNCTION track.update_edge_weight_on_node_change();
