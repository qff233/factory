CREATE OR REPLACE FUNCTION mes.switch_recipe_version(
    p_tool_type INT,
    p_recipe_name VARCHAR(100),
    p_new_version VARCHAR(20)
)
RETURNS VOID AS $$
DECLARE
    v_old_recipe_id INT;
    v_new_recipe_id INT;
    v_old_version VARCHAR(20);
    v_new_version VARCHAR(20);
    v_update_count INT;
BEGIN
    PERFORM set_config('mes.bypass_trigger', 'true', false);
BEGIN

    SELECT id, version INTO v_old_recipe_id, v_old_version
    FROM mes.recipes
    WHERE tool_type = p_tool_type
        AND name = p_recipe_name
        AND status = 'active';
    -- IF v_old_recipe_id IS NULL THEN
    --     RAISE EXCEPTION '找不到当前active的配方: tool_name=%, recipe_name=%', 
    --         p_tool_name, p_recipe_name;
    -- END IF; 
    
    SELECT id, version INTO v_new_recipe_id, v_new_version
    FROM mes.recipes
    WHERE tool_type = p_tool_type
        AND name = p_recipe_name
        AND version = p_new_version;
    IF v_new_recipe_id IS NULL THEN
        RAISE EXCEPTION '找不到指定版本的配方: tool_type=%, recipe_name=%, version=%', 
            (SELECT name FROM mes.tool_types WHERE id = p_tool_type), p_recipe_name, p_new_version;
    END IF;

    UPDATE mes.recipes
    SET status = 'inactive'
    WHERE id = v_old_recipe_id;

    UPDATE mes.recipes
    SET status = 'active'
    WHERE id = v_new_recipe_id;

    UPDATE mes.recipe_steps
    SET id = v_new_recipe_id
    WHERE id = v_old_recipe_id;

    GET DIAGNOSTICS v_update_count = ROW_COUNT;

    RAISE NOTICE '已切换配方版本：% -> %，更新了 % 个步骤',
        v_old_version, v_new_version, v_update_count;

EXCEPTION
    WHEN OTHERS THEN
        RAISE EXCEPTION '切换配方版本时出错：%', SQLERRM;
END;
    PERFORM set_config('mes.bypass_trigger', 'false', false);
END;
$$ LANGUAGE 'plpgsql';


----- TEST ------

SELECT mes.switch_recipe_version(1, '水', '0.1'); 
SELECT mes.switch_recipe_version(1, '水', '0.2');
SELECT * FROM mes.recipes;
SELECT * FROM mes.recipe_steps;