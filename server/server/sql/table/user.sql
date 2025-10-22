CREATE TABLE auth.users(
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(20) NOT NULL
);

INSERT INTO auth.users(username, password_hash, role)
VALUES ('qff233', 'test_hash', 'operator');