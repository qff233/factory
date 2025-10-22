DROP TABLE mes.recipes;
DROP TYPE mes.RECIPE_STATUS;

CREATE TYPE mes.RECIPE_STATUS AS ENUM(
    'active', 'inactive'
);

CREATE TABLE mes.recipes(
    recipe_id SERIAL PRIMARY KEY,
    tool_name VARCHAR(50) NOT NULL,
    recipe_name VARCHAR(100) NOT NULL,
    recipe_version VARCHAR(20) DEFAULT '1.0',
    status mes.RECIPE_STATUS DEFAULT 'inactive',
    inputs VARCHAR(50)[],
    inputbuss VARCHAR(50)[],
    created_by VARCHAR(50) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (tool_name, recipe_name, recipe_version)
);
CREATE INDEX idx_recipes_tool_name ON mes.recipes(tool_name);
CREATE INDEX idx_recipes_recipe_name ON mes.recipes(recipe_name);
CREATE INDEX idx_recipes_status ON mes.recipes(status);

CREATE UNIQUE INDEX unique_active_recipe_per_tool_and_name
ON mes.recipes(tool_name, recipe_name)
WHERE status = 'active';

-------------
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_recipes_updated_at
    BEFORE UPDATE ON mes.recipes
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-------------
CREATE OR REPLACE FUNCTION mes.prevent_direct_status_update()
RETURNS TRIGGER AS $$
BEGIN
    IF current_setting('mes.bypass_trigger', TRUE) = 'true' THEN
        RETURN NEW;
    END IF;

    IF OLD.status IS DISTINCT FROM NEW.status THEN
        RAISE EXCEPTION
            '禁止直接修改配方状态。请使用mes.switch_recipe_version()函数进行切换。';
    END IF;
    return NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER prevent_direct_status_update
    BEFORE UPDATE ON mes.recipes
    FOR EACH ROW
    EXECUTE FUNCTION mes.prevent_direct_status_update();

----- TEST -----

-- ok
INSERT INTO mes.recipes (tool_name, recipe_name, recipe_version, status, inputs, inputbuss, created_by)
VALUES ('CR1','水', '1.0', 'active', ARRAY['氢气 2000', '氧气 1000'], ARRAY[]::VARCHAR(50)[], 'EC2');

-- panic!
INSERT INTO mes.recipes (tool_name, recipe_name, recipe_version, status, inputs, inputbuss, created_by)
VALUES ('CR1','水', '2.0', 'active', ARRAY['氢气 2000', '氧气 1000'], ARRAY[]::VARCHAR(50)[], 'EC2');

-- ok!
INSERT INTO mes.recipes (tool_name, recipe_name, recipe_version, status, inputs, inputbuss, created_by)
VALUES ('CR1','水', '2.0', 'inactive', ARRAY['氢气 2000', '氧气 1000'], ARRAY[]::VARCHAR(50)[], 'EC2');

-- Check update time ok!
UPDATE mes.recipes
SET status = 'active' 
WHERE recipe_id = 5;

SELECT * FROM mes.recipes;
