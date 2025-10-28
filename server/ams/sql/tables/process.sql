
CREATE TYPE mes.process_status AS ENUM (
    'waiting',
    'hold',
    'running',
    'completed'
);

CREATE TABLE mes.processes(
    id BIGSERIAL PRIMARY KEY NOT NULL,
    name NOT NULL UNIQUE,
    quantity INT NOT NULL,
    process_flow_name VARCHAR(50) NOT NULL REFERENCES mes.process_flows(name),
    process_flow_index INT NOT NULL REFERENCES mes.process_flows(step),
    status mes.process_status NOT NULL DEFAULT 'waiting',
    created_by VARCHAR(50) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updaetd_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

