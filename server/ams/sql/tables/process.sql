CREATE TYPE mes.product_status AS ENUM (
    'queue',
    'hold',
    'running',
    'completed'
);

CREATE TABLE mes.products(
    id BIGSERIAL PRIMARY KEY NOT NULL,
    name VARCHAR(50) NOT NULL UNIQUE,
    priority INT NOT NULL,
    quantity INT NOT NULL,
    process_flow_name VARCHAR(50) NOT NULL,
    status mes.product_status NOT NULL DEFAULT 'queue',
    created_by VARCHAR(50) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_products_name ON mes.products(name);
CREATE INDEX idx_products_process_flow_name ON mes.products(process_flow_name);

CREATE TABLE mes.tasks(
    id BIGSERIAL PRIMARY KEY NOT NULL,
    product_id BIGINT NOT NULL REFERENCES mes.products(id),
    priority INT NOT NULL,
    sequence INT NOT NULL,
    tool_id VARCHAR(50) NOT NULL,
    recipe_name VARCHAR(50) NOT NULL,
    quantity INT NOT NULL,
    status mes.product_status NOT NULL DEFAULT 'queue',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_tasks_product_id ON mes.tasks(product_id);

CREATE OR REPLACE FUNCTION mes.products_add_product()
RETURNS TRIGGER AS $$
BEGIN
IF NOT EXISTS (SELECT 1 FROM mes.process_flows WHERE name = NEW.process_flow_name) THEN
    RAISE EXCEPTION 'process_flow_name does not exist';
END IF;

WITH flow AS (
    SELECT sequence, tool_id, recipe_name
    FROM mes.process_flows
    WHERE name = NEW.process_flow_name
),
tasks AS (
    SELECT NEW.id as product_id, NEW.priority as priority, sequence, tool_id, recipe_name, NEW.quantity as quantity, NEW.status as status
    FROM flow
)
INSERT INTO mes.tasks(product_id, priority, sequence, tool_id, recipe_name, quantity, status)
SELECT * FROM tasks;

return NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER products_add_product_trigger
AFTER INSERT ON mes.products
FOR EACH ROW
EXECUTE FUNCTION mes.products_add_product();

CREATE TRIGGER tasks_update_updated_at_trigger
AFTER UPDATE ON mes.tasks
FOR EACH ROW
EXECUTE FUNCTION mes.update_updated_at();

CREATE OR REPLACE FUNCTION mes.products_update_status()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.status = 'hold' THEN
        UPDATE mes.tasks SET status = 'hold' WHERE product_id = NEW.id AND status = 'queue';
    ELSIF NEW.status = 'queue' THEN
        UPDATE mes.tasks SET status = 'queue' WHERE product_id = NEW.id AND status = 'hold';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER products_update_status_trigger
AFTER UPDATE ON mes.products
FOR EACH ROW
EXECUTE FUNCTION mes.products_update_status();

CREATE OR REPLACE FUNCTION mes.tasks_update_status()
RETURNS TRIGGER AS $$
BEGIN
    -- 如果有task在running，修改products的status为running
    IF NEW.status = 'running' THEN
        IF OLD.status = 'hold' OR OLD.status = 'running' OR OLD.status = 'completed' THEN
            RAISE EXCEPTION 'Cannot change status from hold to running';
        END IF;
        IF EXISTS (SELECT 1 FROM mes.tasks WHERE product_id = NEW.id AND status = 'running') THEN
            UPDATE mes.products SET status = 'running' WHERE id = NEW.id;
        END IF;
    END IF;
    -- 如果所有索引到products的状态都为completed，修改mes.products的状态为completed。
    IF NEW.status = 'completed' THEN
        IF OLD.status = 'hold' OR OLD.status = 'queue' OR OLD.status = 'completed' THEN
            RAISE EXCEPTION 'Cannot change status from hold to running';
        END IF;
        IF NOT EXISTS (SELECT 1 FROM mes.tasks WHERE product_id = NEW.id AND status != 'completed') THEN
            UPDATE mes.products SET status = 'completed' WHERE id = NEW.id;
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER tasks_update_status_trigger
AFTER UPDATE ON mes.tasks
FOR EACH ROW
EXECUTE FUNCTION mes.tasks_update_status();

--------------- USED ---------------
-- query next tasks
SELECT id, tool_id, recipe_name, quantity FROM mes.tasks
WHERE status = 'queue'
-- AND tool_id = 'Tool1'
ORDER BY priority DESC, sequence;

-- update task status
UPDATE mes.tasks
SET status = 'completed'
WHERE id = 7;
--------------- TEST ---------------
INSERT INTO mes.products(name, priority, quantity, process_flow_name, created_by)
VALUES ('Test1', 5, 10, 'Test Process Flow', 'EC2');
INSERT INTO mes.products(name, priority, quantity, process_flow_name, created_by)
VALUES ('Test2', 3, 10, 'Test Process Flow', 'EC2');

-- test for add process
UPDATE mes.tasks SET status = 'running' WHERE product_id = 1;
SELECT * FROM mes.tasks WHERE product_id = 1;
SELECT * FROM mes.products;
UPDATE mes.processes SET status = 'hold' WHERE id = 1;
SELECT * FROM mes.tasks WHERE product_id = 1;
UPDATE mes.processes SET status = 'queue' WHERE id = 1;
SELECT * FROM mes.tasks WHERE product_id = 1;

-- test for task update status
UPDATE mes.processes SET status = 'hold' WHERE id = 1;
UPDATE mes.tasks SET status = 'running' WHERE product_id = 1 AND sequence = 2;
SELECT * FROM mes.processes;
SELECT * FROM mes.tasks WHERE product_id = 1;

UPDATE mes.tasks SET status = 'completed' WHERE product_id = 1;
SELECT * FROM mes.processes;
SELECT * FROM mes.tasks WHERE product_id = 1;
