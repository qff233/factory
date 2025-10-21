CREATE TYPE mes.TASK_STATUS AS ENUM(
    'pending', 'running', 'paused', 'completed', 'cancelled'
);

CREATE TABLE mes.tasks(
    task_id SERIAL PRIMARY KEY,
    task_name VARCHAR(50) UNIQUE NOT NULL,
    priority INT DEFAULT 10,
    quantity INT NOT NULL,
    status mes.TASK_STATUS,
    steps_name VARCHAR(50) NOT NULL,
    current_step_id INT NOT NULL,
    start_time TIMESTAMP,
    end_time TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (steps_name) REFERENCES mes.recipe_steps(recipe_name)
);


