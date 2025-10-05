SET search_path to transport;

DROP TABLE item;
DROP TABLE fluid;
DROP TABLE use_tool;
DROP TYPE STATE;
DROP TYPE ToolType;

CREATE TYPE STATE AS ENUM (
	'pending',
	'processing',
	'completed'
);

CREATE table item (
	id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	date_created TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
	date_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
	begin_node_name CHAR(50) REFERENCES track.nodes(name) NOT NULL,
	end_node_name CHAR(50) REFERENCES track.nodes(name) NOT NULL,
	vehicle_id INT,
	state STATE DEFAULT 'pending',
	CONSTRAINT processing_must_have_vehicle_id CHECK(
		CASE WHEN state = 'processing' THEN
			vehicle_id IS NOT NULL
		ELSE true
		END
	)
);

CREATE table fluid(
	id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	date_created TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
	date_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
	begin_node_name CHAR(50) NOT NULL,
	end_node_name CHAR(50) NOT NULL,
	vehicle_id INT,
	state STATE DEFAULT 'pending',
	CONSTRAINT processing_must_have_vehicle_id CHECK(
		CASE WHEN state = 'processing' THEN
			vehicle_id IS NOT NULL
		ELSE true
		END
	)
);

CREATE TYPE ToolType as ENUM(
	'wrench',
	'solder',
	'crowbar',
	'screwdriver',
	'wire_nipper',
	'soft_hammer'
);

CREATE table use_tool(
	id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	date_created TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
	date_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
	end_node_name CHAR(50) NOT NULL,
	tool_type ToolType NOT NULL, 
	vehicle_id INT,
	state STATE DEFAULT 'pending',
	CONSTRAINT processing_must_have_vehicle_id CHECK(
		CASE WHEN state = 'processing' THEN
			vehicle_id IS NOT NULL
		ELSE true
		END
	)
);

CREATE OR REPLACE FUNCTION update_modified_date()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
	NEW.date_updated = CURRENT_TIMESTAMP;
	RETURN NEW;
END;
$$;

CREATE TRIGGER update_date_trigger
	BEFORE UPDATE ON item
	FOR EACH ROW
	EXECUTE FUNCTION update_modified_date();

CREATE TRIGGER update_date_trigger
	BEFORE UPDATE ON fluid
	FOR EACH ROW
	EXECUTE FUNCTION update_modified_date();

CREATE TRIGGER update_date_trigger
	BEFORE UPDATE ON use_tool
	FOR EACH ROW
	EXECUTE FUNCTION update_modified_date();
