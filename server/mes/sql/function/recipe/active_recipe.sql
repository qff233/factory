CREATE OR REPLACE FUNCTION mes.active_recipe(
    p_recipe_id INT
)
RETURNS TEXT AS $$
DECLARE
    v_recipe RECORD;
    v_old_active_recipe_id INT;
BEGIN
    SELECT tool_type, name, version INTO v_recipe
    FROM mes.recipes 
    WHERE id = p_recipe_id;
    
    IF NOT FOUND THEN
        RAISE EXCEPTION '配方不存在: recipe_id=%', p_recipe_id;
    END IF;
    
    PERFORM mes.switch_recipe_version(
        v_recipe.tool_type,
        v_recipe.name,
        v_recipe.version
    );

    RETURN FORMAT('%s :[%s]配方active成功！', (SELECT name FROM mes.tool_types WHERE id = v_recipe.tool_type), v_recipe.name);
END;
$$ LANGUAGE plpgsql;


----- TEST -------
SELECT * FROM mes.active_recipe(3);
SELECT * FROM mes.active_recipe(5);
SELECT * FROM mes.active_recipe(1);

SELECT * FROM mes.recipes;

