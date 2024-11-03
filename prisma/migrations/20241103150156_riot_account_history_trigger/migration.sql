CREATE FUNCTION riot_account_updated_or_inserted() RETURNS trigger AS $$
BEGIN
    NEW.updated_at = IFNULL(OLD.updated_at, NOW());
    if OLD.id IS NULL OR OLD IS DISTINCT FROM NEW THEN
        INSERT INTO riot_account_history (riot_account_id, game_name, tag_line, updated_at)
        VALUES (NEW.id, NEW.game_name, NEW.tag_line, NEW.updated_at);
    END IF;
    NEW.updated_at = NOW();
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER riot_account_updated_trigger
AFTER INSERT OR UPDATE ON riot_account
FOR EACH ROW
EXECUTE PROCEDURE riot_account_updated_or_inserted();