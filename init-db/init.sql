-- VARIORUM TABLES
CREATE TABLE IF NOT EXISTS variorum_relative (
    time_relative_ms DECIMAL(12,3) NOT NULL,
    power_watts DECIMAL(12,6) NOT NULL,
    energy_relative_joules DECIMAL(18,9) NOT NULL
);

CREATE TABLE IF NOT EXISTS variorum_absolute (
    timestamp_system_epoch_ms DECIMAL(18,3) NOT NULL,
    variorum_power_watts DECIMAL(12,6) NOT NULL,
    variorum_integrated_energy_joules DECIMAL(18,9) NOT NULL
);

CREATE TABLE IF NOT EXISTS variorum_gpus (
    timestamp_ms DECIMAL(18,3) NOT NULL,
    power_watts DECIMAL(12,6) NOT NULL,
    device_id INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS variorum_kernels (
    kernel_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    type TEXT NOT NULL,
    start_time_ms DECIMAL(18,3) NOT NULL,
    end_time_ms DECIMAL(18,3) NOT NULL,
    duration_ms DECIMAL(18,3) NOT NULL
);

CREATE TABLE IF NOT EXISTS variorum_stats (
    stat_name TEXT NOT NULL,
    value TEXT NOT NULL
);

DROP TABLE IF EXISTS variorum_regions;
CREATE TABLE IF NOT EXISTS variorum_regions (
    name TEXT NOT NULL,
    start_time_ms DECIMAL(18,3) NOT NULL,
    end_time_ms DECIMAL(18,3) NOT NULL
);

-- COPY commands will be dynamically added by the setup.sh script

-- Fallback replaced by aggregation script when variorum_series.sql is generated
CREATE OR REPLACE FUNCTION select_variorum_nonzero()
RETURNS TABLE (
    time_ns BIGINT,
    power_data TEXT
) AS $$
BEGIN
    RETURN QUERY SELECT
        CAST(v.timestamp_ms * 1000000 AS BIGINT) as time_ns,
        CAST(v.power_watts AS TEXT) as power_data
    FROM variorum_gpus v
    WHERE v.power_watts > 0
    ORDER BY v.timestamp_ms;
END;
$$ LANGUAGE plpgsql;