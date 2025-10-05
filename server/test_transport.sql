SET search_path to transport;

INSERT INTO item(begin_node_name, end_node_name)
VALUES('S2','S1');
SELECT * FROM item; 

SELECT id, begin_node_name, end_node_name
FROM item
WHERE state = 'pending'
LIMIT 20;

INSERT INTO fluid(begin_node_name, end_node_name)
VALUES('S1','S2');
SELECT * FROM fluid;

INSERT INTO use_tool(end_node_name, tool_type)
VALUES('S1', 'solder');
SELECT * FROM use_tool;

SELECT id, begin_node_name, end_node_name, state
FROM item
-- WHERE state = 'pending'
LIMIT 20;

UPDATE item
SET state = 'processing', vehicle_id = 2000
WHERE id = 1;

UPDATE item
SET state = 'completed'
WHERE vehicle_id = 2500
LIMIT 1;
SELECT * FROM item; 