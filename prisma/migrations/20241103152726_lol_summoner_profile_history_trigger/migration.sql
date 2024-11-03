CREATE FUNCTION lol_summoner_profile_updated_or_inserted() RETURNS trigger AS $$
BEGIN
    NEW.updated_at = IFNULL(OLD.updated_at, NOW());
    if OLD.id IS NULL OR OLD IS DISTINCT FROM NEW THEN
        INSERT INTO lol_summoner_profile_history (lol_summoner_id, profile_icon_id, revision_date, summoner_level, updated_at)
        VALUES (NEW.id, NEW.profile_icon_id, NEW.revision_date, NEW.summoner_level, NEW.updated_at);
    END IF;
    NEW.updated_at = NOW();
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER lol_summoner_profile_updated_trigger
AFTER INSERT OR UPDATE ON lol_summoner
FOR EACH ROW
EXECUTE PROCEDURE lol_summoner_profile_updated_or_inserted();