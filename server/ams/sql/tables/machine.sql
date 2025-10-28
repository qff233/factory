CREATE TABLE mes.machines(
    id BIGSERIAL PRIMARY KEY NOT NULL,
    tool_id VARCHAR(50) NOT NULL,
    from_status VARCHAR(20),
    to_status VARCHAR(20),
    product_id REFERENCES mes.products(id),
    recipe_name VARCHAR(50),
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);


