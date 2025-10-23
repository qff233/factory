CREATE TABLE mes.tool_types(
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL
);

INSERT INTO mes.tool_types(name)
VALUES('CR-IV');