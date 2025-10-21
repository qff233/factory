CREATE OR REPLACE FUNCTION mes.inactive_recipe(
    p_recipe_id INT
)
RETURNS VOID AS $$
DECLARE
    v_recipe RECORD;
    v_active_count INT;
    v_used_in_steps BOOLEAN;
BEGIN
    SELECT tool_name, recipe_name, status INTO v_recipe
    FROM mes.recipes 
    WHERE recipe_id = p_recipe_id;
    
    IF NOT FOUND THEN
        RAISE EXCEPTION '配方不存在: recipe_id=%', p_recipe_id;
    END IF;

    SELECT EXISTS (
        SELECT 1 FROM mes.recipe_steps
        WHERE recipe_id = p_recipe_id
    ) INTO v_used_in_steps;
    
    IF v_used_in_steps THEN
        SELECT COUNT(*) INTO v_active_count
        FROM mes.recipes
        WHERE tool_name = v_recipe.tool_name
        AND recipe_name = v_recipe.recipe_name
        AND status = 'active'
        AND recipe_id != p_recipe_id;
        
        IF v_active_count = 0 THEN
            RAISE EXCEPTION 
                '无法停用配方，必须至少保留一个正在使用的active版本。工具: %, 配方: %',
                v_recipe.tool_name, v_recipe.recipe_name;
        END IF;
    END IF;
    

    PERFORM set_config('mes.bypass_trigger', 'true', true);
    BEGIN 
        UPDATE mes.recipes 
        SET status = 'inactive'
        WHERE recipe_id = p_recipe_id;
    END;
    PERFORM set_config('mes.bypass_trigger', 'false', false);
    
    RAISE NOTICE '已停用配方 [%]', v_recipe.recipe_name;
END;
$$ LANGUAGE plpgsql;

----- TEST -------
SELECT * FROM mes.inactive_recipe(2);
SELECT * FROM mes.inactive_recipe(5);

SELECT * FROM mes.active_recipe(6);
SELECT * FROM mes.inactive_recipe(6);

SELECT * FROM mes.recipes;