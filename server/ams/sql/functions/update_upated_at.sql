CREATE OR REPLACE FUNCTION mes.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW() AT TIME ZONE 'Asia/Shanghai';
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
