DROP TABLE mes.tools;

CREATE TABLE mes.tools(
    id SERIAL PRIMARY KEY,
    tool_id VARCHAR(50),
    tool_type INT REFERENCES mes.tool_types(id)
);


