CREATE TYPE transport.STATE AS ENUM (
	'pending',
	'processing',
	'completed'
);

CREATE table transport.item (
	id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	date_created TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
	date_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
	begin_node_name CHAR(50) NOT NULL,
	end_node_name CHAR(50) NOT NULL,
	state transport.STATE DEFAULT 'pending'
);

CREATE table transport.fluid(
	id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	date_created TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
	date_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
	begin_node_name CHAR(50) NOT NULL,
	end_node_name CHAR(50) NOT NULL,
	state transport.STATE DEFAULT 'pending'
);

CREATE TYPE transport.ToolType as ENUM(
	'wrench',
	'solder',
	'crowbar',
	'screwdriver',
	'wire_nipper',
	'soft_hammer'
);

CREATE table transport.use_tool(
	id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	date_created TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
	date_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
	end_node_name CHAR(50) NOT NULL,
	tool_type transport.ToolType NOT NULL, 
	state transport.STATE DEFAULT 'pending'
);

CREATE OR REPLACE FUNCTION transport.update_modified_date()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
	NEW.date_updated = CURRENT_TIMESTAMP;
	RETURN NEW;
END;
$$;

CREATE TRIGGER update_date_trigger
	BEFORE UPDATE ON transport.item
	FOR EACH ROW
	EXECUTE FUNCTION transport.update_modified_date();

CREATE TRIGGER update_date_trigger
	BEFORE UPDATE ON transport.fluid
	FOR EACH ROW
	EXECUTE FUNCTION transport.update_modified_date();

CREATE TRIGGER update_date_trigger
	BEFORE UPDATE ON transport.use_tool
	FOR EACH ROW
	EXECUTE FUNCTION transport.update_modified_date();
