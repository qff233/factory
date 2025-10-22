CREATE TABLE auth.user_permissions(
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    permission_id INT NOT NULL REFERENCES auth.permissions(id) ON DELETE CASCADE,
    UNIQUE(user_id, permission_id)
);


            SELECT 1
            FROM auth.user_permissions up
            JOIN auth.permissions p ON up.permission_id = p.id
            WHERE up.user_id = 1 AND p.name = '';