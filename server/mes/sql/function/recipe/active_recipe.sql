CREATE OR REPLACE FUNCTION mes.active_recipe(
    p_recipe_id INT
)
RETURNS VOID AS $$
DECLARE
    v_recipe RECORD;
    v_old_active_recipe_id INT;
BEGIN
    -- 获取要激活的配方信息
    SELECT tool_name, recipe_name, recipe_version INTO v_recipe
    FROM mes.recipes 
    WHERE recipe_id = p_recipe_id;
    
    IF NOT FOUND THEN
        RAISE EXCEPTION '配方不存在: recipe_id=%', p_recipe_id;
    END IF;
    
    PERFORM mes.switch_recipe_version(
        v_recipe.tool_name,
        v_recipe.recipe_name,
        v_recipe.recipe_version
    );
END;
$$ LANGUAGE plpgsql;

----- TEST -------
SELECT * FROM mes.active_recipe(2);
SELECT * FROM mes.active_recipe(5);

SELECT * FROM mes.inactive_recipe(6);
SELECT * FROM mes.active_recipe(6);

SELECT * FROM mes.recipes;

