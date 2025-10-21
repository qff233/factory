CREATE TABLE mes.recipe_steps (
    steps_id SERIAL PRIMARY KEY,
    steps_name VARCHAR(100) NOT NULL,
    recipe_id INT NOT NULL,
    step_number INT NOT NULL,
    step_description TEXT,
    created_by VARCHAR(50) NOT NULL,
    FOREIGN KEY (recipe_id) REFERENCES mes.recipes(recipe_id)
);
CREATE INDEX idx_recipe_steps_name ON mes.recipe_steps(steps_name);

CREATE OR REPLACE FUNCTION mes.check_recipe_active()
RETURNS TRIGGER AS $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM mes.recipes
        WHERE recipe_id = NEW.recipe_id AND status = 'active'
    ) THEN
        RAISE EXCEPTION '只能引用状态为active的配方';
    END IF;
    return NEW;
END;
$$ LANGUAGE 'plpgsql';

DROP TRIGGER ensure_active_recipe;
CREATE TRIGGER ensure_active_recipe
    BEFORE INSERT OR UPDATE ON mes.recipe_steps
    FOR EACH ROW
    EXECUTE FUNCTION mes.check_recipe_active();

---- TEST -----

-- panic!
WITH recipe AS (
    SELECT recipe_id FROM mes.recipes
    WHERE recipe_name = '水' AND status = 'draft'
    LIMIT 1
)
INSERT INTO mes.recipe_steps(
    steps_name,
    recipe_id,
    step_number,
    created_by
)
SELECT '随便', recipe_id, 1, 'EC2' FROM recipe;

WITH recipe AS (
    SELECT recipe_id FROM mes.recipes
    WHERE recipe_name = '水' AND status = 'active'
)
INSERT INTO mes.recipe_steps(
    steps_name,
    recipe_id,
    step_number,
    created_by
)
SELECT '随便', recipe_id, 1, 'EC2' FROM recipe;

SELECT * FROM mes.recipe_steps;
