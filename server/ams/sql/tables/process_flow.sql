CREATE TABLE mes.process_flows(
    id SERIAL PRIMARY KEY NOT NULL,
    name VARCHAR(50) NOT NULL,
    sequence INT NOT NULL,
    tool_id VARCHAR(50) NOT NULL,
    recipe_name VARCHAR(50) NOT NULL,
    quantity INT NOT NULL,
    description TEXT,
    created_by VARCHAR(50) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT unique_name_sequence_tool_recipe UNIQUE(name, sequence, tool_id, recipe_name)
);

CREATE INDEX idx_process_flows_name ON mes.process_flows(name);
CREATE INDEX idx_process_flows_tool_id ON mes.process_flows(tool_id);
CREATE INDEX idx_process_flows_recipe_name ON mes.process_flows(recipe_name);

---- TEST -----
INSERT INTO mes.process_flows(name, sequence, tool_id, recipe_name, quantity, description, created_by)
VALUES ('Test Process Flow', 1, 'Tool1', 'Recipe1', 10, 'Test Step Description', 'User1');
INSERT INTO mes.process_flows(name, sequence, tool_id, recipe_name, quantity, description, created_by)
VALUES ('Test Process Flow', 2, 'Tool2', 'Recipe2', 20, 'Test Step Description 2', 'User2');
INSERT INTO mes.process_flows(name, sequence, tool_id, recipe_name, quantity, description, created_by)
VALUES ('Test Process Flow', 3, 'Tool3', 'Recipe3', 30, 'Test Step Description 3', 'User3');
INSERT INTO mes.process_flows(name, sequence, tool_id, recipe_name, quantity, description, created_by)
VALUES ('Test Process Flow', 4, 'Tool4', 'Recipe4', 40, 'Test Step Description 4', 'User4');
INSERT INTO mes.process_flows(name, sequence, tool_id, recipe_name, quantity, description, created_by)
VALUES ('Test Process Flow', 5, 'Tool5', 'Recipe5', 50, 'Test Step Description 5', 'User5');

SELECT * FROM mes.process_flows;
