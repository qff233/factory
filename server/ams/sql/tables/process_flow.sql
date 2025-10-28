CREATE TABLE mes.process_flows(
    id SERIAL PRIMARY KEY NOT NULL,
    name VARCHAR(50) NOT NULL UNIQUE,
    step INT NOT NULL,
    tool_id VARCHAR(50) NOT NULL,
    recipe_name VARCHAR(50) NOT NULL,
    quantity INT NOT NULL,
    step_description TEXT,
    created_by VARCHAR(50) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_recipe_steps_name ON mes.recipe_steps(name);

---- TEST -----
SELECT * FROM mes.process_flows;
