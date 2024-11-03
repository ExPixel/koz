CREATE FUNCTION lol_summoner_rank_updated_or_inserted() RETURNS trigger AS $$
BEGIN
    NEW.updated_at = IFNULL(OLD.updated_at, NOW());
    if OLD.id IS NULL OR OLD IS DISTINCT FROM NEW THEN
        INSERT INTO lol_summoner_rank_history
            (lol_summoner_id, queue_type, tier, division, league_points, wins, losses, mini_series_wins, mini_series_losses, mini_series_target, mini_series_progress, updated_at)
        VALUES
            (NEW.lol_summoner_id, NEW.queue_type, NEW.tier, NEW.division, NEW.league_points, NEW.wins, NEW.losses, NEW.mini_series_wins, NEW.mini_series_losses, NEW.mini_series_target, NEW.mini_series_progress, NEW.updated_at);
    END IF;
    NEW.updated_at = NOW();
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER lol_summoner_rank_updated_trigger
AFTER INSERT OR UPDATE ON lol_summoner_rank
FOR EACH ROW
EXECUTE PROCEDURE lol_summoner_rank_updated_or_inserted();