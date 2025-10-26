DROP TABLE mes.recipes;
DROP TYPE mes.RECIPE_STATUS;

CREATE TYPE mes.recipe_status AS ENUM(
    'active', 'inactive'
);

DROP TABLE mes.recipe_steps;
DROP TABLE mes.recipes;
CREATE TABLE mes.recipes(
    id SERIAL PRIMARY KEY,
    tool_type INT NOT NULL REFERENCES mes.tool_types(id),
    name VARCHAR(100) NOT NULL,
    version VARCHAR(20) DEFAULT '1.0' NOT NULL,
    status mes.RECIPE_STATUS DEFAULT 'inactive' NOT NULL,
    inputs VARCHAR(50)[],
    inputbuss VARCHAR(50)[],
    created_by VARCHAR(50) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    UNIQUE (tool_type, name, version)
);
CREATE INDEX idx_recipes_tool_type ON mes.recipes(tool_type);
CREATE INDEX idx_recipes_name ON mes.recipes(name);
CREATE INDEX idx_recipes_status ON mes.recipes(status);

CREATE UNIQUE INDEX unique_active_recipe_per_tool_and_name
ON mes.recipes(tool_type, name)
WHERE status = 'active';

-------------
CREATE OR REPLACE FUNCTION mes.update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_recipes_updated_at
    BEFORE UPDATE ON mes.recipes
    FOR EACH ROW
    EXECUTE FUNCTION mes.update_updated_at_column();

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
INSERT INTO mes.recipes (tool_type, name, version, status, inputs, created_by)
VALUES (1,'水', '1.0', 'active', ARRAY['氢气 2000', '氧气 1000'], 'EC2');

-- panic!
INSERT INTO mes.recipes (tool_type, name, version, status, inputs, created_by)
VALUES (1,'水', '2.0', 'active', ARRAY['氢气 2000', '氧气 1000'], 'EC2');

-- ok!
INSERT INTO mes.recipes (tool_type, name, version, status, inputs, created_by)
VALUES (1,'水', '2.0', 'inactive', ARRAY['氢气 2000', '氧气 1000'], 'EC2');

-- Check update time ok!
UPDATE mes.recipes
SET status = 'active' 
WHERE id = 3;

SELECT * FROM mes.recipes;
