CREATE TYPE mes.process_status AS ENUM (
    'queue',
    'hold',
    'running',
    'completed'
);

CREATE TABLE mes.processes(
    id BIGSERIAL PRIMARY KEY NOT NULL,
    name VARCHAR(50) NOT NULL UNIQUE,
    priority INT NOT NULL,
    quantity INT NOT NULL,
    process_flow_name VARCHAR(50) NOT NULL,
    status mes.process_status NOT NULL DEFAULT 'queue',
    created_by VARCHAR(50) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_processes_name ON mes.processes(name);
CREATE INDEX idx_processes_process_flow_name ON mes.processes(process_flow_name);

CREATE TABLE mes.tasks(
    id SERIAL PRIMARY KEY NOT NULL,
    process_id BIGINT NOT NULL REFERENCES mes.processes(id),
    priority INT NOT NULL,
    sequence INT NOT NULL,
    tool_id VARCHAR(50) NOT NULL,
    recipe_name VARCHAR(50) NOT NULL,
    quantity INT NOT NULL,
    status mes.process_status NOT NULL DEFAULT 'queue',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_tasks_process_id ON mes.tasks(process_id);

CREATE OR REPLACE FUNCTION mes.processes_add_process()
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
    SELECT NEW.id as process_id, NEW.priority as priority, sequence, tool_id, recipe_name, NEW.quantity as quantity, NEW.status as status
    FROM flow
)
INSERT INTO mes.tasks(process_id, priority, sequence, tool_id, recipe_name, quantity, status)
SELECT * FROM tasks;

return NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER processes_add_process_trigger
AFTER INSERT ON mes.processes
FOR EACH ROW
EXECUTE FUNCTION mes.processes_add_process();

CREATE TRIGGER tasks_update_updated_at_trigger
AFTER UPDATE ON mes.tasks
FOR EACH ROW
EXECUTE FUNCTION mes.update_updated_at();

CREATE OR REPLACE FUNCTION mes.processes_update_status()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.status = 'hold' THEN
        UPDATE mes.tasks SET status = 'hold' WHERE process_id = NEW.id AND status = 'queue';
    ELSIF NEW.status = 'queue' THEN
        UPDATE mes.tasks SET status = 'queue' WHERE process_id = NEW.id AND status = 'hold';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER processes_update_status_trigger
AFTER UPDATE ON mes.processes
FOR EACH ROW
EXECUTE FUNCTION mes.processes_update_status();

CREATE OR REPLACE FUNCTION mes.tasks_update_status()
RETURNS TRIGGER AS $$
BEGIN
    -- 如果有task在running，修改processes的status为running
    IF NEW.status = 'running' THEN
        IF EXISTS (SELECT 1 FROM mes.tasks WHERE process_id = NEW.id AND status = 'running') THEN
            UPDATE mes.processes SET status = 'running' WHERE id = NEW.id;
        END IF;
    END IF;
    -- 如果所有索引到processes的状态都为completed，修改mes.processes的状态为completed。
    IF NEW.status = 'completed' THEN
        IF NOT EXISTS (SELECT 1 FROM mes.tasks WHERE process_id = NEW.id AND status != 'completed') THEN
            UPDATE mes.processes SET status = 'completed' WHERE id = NEW.id;
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
INSERT INTO mes.processes(name, priority, quantity, process_flow_name, created_by)
VALUES ('Test Process1', 5, 10, 'Test Process Flow', 'EC2');
INSERT INTO mes.processes(name, priority, quantity, process_flow_name, created_by)
VALUES ('Test Process2', 3, 10, 'Test Process Flow', 'EC2');

-- test for add process
UPDATE mes.tasks SET status = 'running' WHERE process_id = 1;
UPDATE mes.processes SET status = 'running' WHERE id = 1;
SELECT * FROM mes.tasks WHERE process_id = 1;
UPDATE mes.processes SET status = 'hold' WHERE id = 1;
SELECT * FROM mes.tasks WHERE process_id = 1;
UPDATE mes.processes SET status = 'queue' WHERE id = 1;
SELECT * FROM mes.tasks WHERE process_id = 1;

-- test for task update status
UPDATE mes.processes SET status = 'hold' WHERE id = 1;
UPDATE mes.tasks SET status = 'running' WHERE process_id = 1 AND sequence = 2;
SELECT * FROM mes.processes;
SELECT * FROM mes.tasks WHERE process_id = 1;

UPDATE mes.tasks SET status = 'completed' WHERE process_id = 1;
SELECT * FROM mes.processes;
SELECT * FROM mes.tasks WHERE process_id = 1;
