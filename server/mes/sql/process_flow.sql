DROP TABLE mes.process_flow;
DROP TYPE mes.TASK;
CREATE TYPE mes.TASK AS (
    recipe_name TEXT,
    run_count INT
);

CREATE TYPE mes.PROCESS_STEP AS (
    tasks mes.TASK[]
);

CREATE TABLE mes.process_flow (
    id SERIAL PRIMARY KEY,
    steps mes.PROCESS_STEP[],
    update_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE OR REPLACE FUNCTION mes.update_update_date()
RETURNS TRIGGER AS $$
BEGIN
    NEW.update_date = CURRENT_TIMESTAMP;
    return NEW;
END;
$$ LANGUAGE 'plpgsql';

CREATE TRIGGER update_update_date
    BEFORE UPDATE ON mes.process_flow
    FOR EACH ROW
    EXECUTE FUNCTION mes.update_update_date();

----------TEST------------

INSERT INTO mes.process_flow(steps) VALUES(
    ARRAY[
        ROW(
            ARRAY[
                ROW('测试任务1',100)::mes.TASK,
                ROW('测试任务2',200)::mes.TASK,
                ROW('测试任务3',300)::mes.TASK
            ]
        )::mes.process_step,
        ROW(
            ARRAY[
                ROW('测试任务3',100)::mes.TASK,
                ROW('测试任务2',200)::mes.TASK,
                ROW('测试任务1',300)::mes.TASK
            ]
        )::mes.process_step
    ]
);

UPDATE mes.process_flow
SET steps = ARRAY[
    ROW(
        ARRAY[
            ROW('测试任务1',100)::mes.TASK,
            ROW('测试任务2',200)::mes.TASK,
            ROW('测试任务3',300)::mes.TASK
        ]
    )::mes.process_step,
    ROW(
        ARRAY[
            ROW('测试任务3',100)::mes.TASK,
            ROW('测试任务2',200)::mes.TASK,
            ROW('测试任务1',300)::mes.TASK
        ]
    )::mes.process_step
]
WHERE id = 1;

SELECT * from mes.process_flow;
SELECT * from process_log.cr1;

