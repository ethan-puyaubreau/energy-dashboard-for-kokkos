"""!
@file correlation.py
@brief Correlation of Variorum power measurements with profiled execution regions.
"""

import os
import pandas as pd
from modules.utils import load_and_concat_csvs, validate_columns


def prepare_power_data(power_files):
    """!
    @brief Prepare power DataFrame from list of power CSV files.
    @param power_files List of paths to power CSV files.
    @return Cleaned DataFrame with timestamp_ns and power_watts.
    """
    df = load_and_concat_csvs(power_files)
    if df.empty:
        return pd.DataFrame()
    if 'timestamp_system_epoch_ms' in df.columns:
        df['timestamp_ns'] = (df['timestamp_system_epoch_ms'] * 1_000_000).astype('int64')
    elif 'timestamp_nanoseconds' in df.columns:
        df['timestamp_ns'] = df['timestamp_nanoseconds'].astype('int64')
    else:
        print("ERROR: No timestamp column found in power data")
        return pd.DataFrame()
    if 'variorum_power_watts' in df.columns:
        df['power_watts'] = df['variorum_power_watts']
    else:
        power_cols = [col for col in df.columns if 'power' in col.lower() and 'watts' in col.lower()]
        if power_cols:
            df['power_watts'] = df[power_cols[0]]
        else:
            print("ERROR: No power column found in data")
            return pd.DataFrame()
    return df


def prepare_regions_data(region_file):
    """!
    @brief Prepare execution regions DataFrame.
    @param region_file Path to CSV file containing region intervals.
    @return DataFrame containing start_time_ns, end_time_ns, and unique region names.
    """
    try:
        regions_df = pd.read_csv(region_file)
    except Exception as e:
        print(f"ERROR: Failed to read region file {region_file}: {e}")
        return pd.DataFrame()
    required_cols = ['start_time_ns', 'end_time_ns', 'name', 'duration_ns']
    if not validate_columns(regions_df, required_cols, "regions data"):
        return pd.DataFrame()
    regions_df['start_time_ns'] = regions_df['start_time_ns'].astype('int64')
    regions_df['end_time_ns'] = regions_df['end_time_ns'].astype('int64')
    regions_df['duration_ns'] = regions_df['duration_ns'].astype('int64')
    region_counts = {}
    unique_region_names = []
    for _, row in regions_df.iterrows():
        base_name = row['name']
        region_counts[base_name] = region_counts.get(base_name, 0) + 1
        unique_name = f"{base_name}_{region_counts[base_name]}"
        unique_region_names.append(unique_name)
    regions_df['unique_name'] = unique_region_names
    return regions_df


def create_correlation_data(power_files, region_file):
    """!
    @brief Match power samples with active execution regions.
    @param power_files List of power CSV files.
    @param region_file Path to regions CSV file.
    @return DataFrame associating timestamps with region names and power values.
    """
    power_df = prepare_power_data(power_files)
    regions_df = prepare_regions_data(region_file)
    if power_df.empty or regions_df.empty:
        return pd.DataFrame()
    correlation_data = []
    for _, row in power_df.iterrows():
        timestamp = row['timestamp_ns']
        power_val = row['power_watts']
        active_regions = regions_df[(regions_df['start_time_ns'] <= timestamp) & (regions_df['end_time_ns'] >= timestamp)]
        region_name = active_regions.iloc[0]['unique_name'] if len(active_regions) > 0 else "Unknown Region"
        correlation_data.append({
            'timestamp_nanoseconds': int(timestamp),
            'power_watts': power_val,
            'region_name': region_name
        })
    return pd.DataFrame(correlation_data)


def create_time_series_data(correlation_df):
    """!
    @brief Pivot correlation data into a multi-column time series table.
    @param correlation_df DataFrame produced by create_correlation_data.
    @return Pivoted DataFrame with time_ns and columns per region.
    """
    if correlation_df.empty:
        return pd.DataFrame()
    pivoted = correlation_df.pivot_table(
        index='timestamp_nanoseconds', columns='region_name', values='power_watts', aggfunc='first'
    ).reset_index().rename(columns={'timestamp_nanoseconds': 'time_ns'})
    pivoted['time_ns'] = pivoted['time_ns'].astype('int64')
    pivoted.columns.name = None
    return pivoted.sort_values('time_ns').reset_index(drop=True)


def generate_sql_schema(series_df, output_dir):
    """!
    @brief Generate PostgreSQL schema and copy commands for the time series table.
    @param series_df Pivoted DataFrame of series data.
    @param output_dir Destination directory for variorum_series.sql.
    """
    if series_df.empty:
        return
    col_defs = []
    for col in series_df.columns:
        if col == 'time_ns':
            col_defs.append("    time_ns BIGINT")
        else:
            safe = col.replace(' ', '_').replace('-', '_').replace('.', '_')
            col_defs.append(f'    "{safe}" DOUBLE PRECISION')
    sql = (
        "DROP TABLE IF EXISTS variorum_series;\n"
        "CREATE TABLE variorum_series (\n"
        + ",\n".join(col_defs) + "\n);\n\n"
        "\\COPY variorum_series FROM '/csv_data/variorum/variorum_series.csv' WITH (FORMAT csv, HEADER true);\n\n"
        "CREATE OR REPLACE FUNCTION select_variorum_nonzero()\n"
        "RETURNS SETOF variorum_series AS $$\n"
        "DECLARE\n"
        "    col_list text;\n"
        "    dyn_sql text;\n"
        "BEGIN\n"
        "    SELECT string_agg(format('%I IS NOT NULL AND %I != 0', column_name, column_name), ' OR ')\n"
        "    INTO col_list\n"
        "    FROM information_schema.columns\n"
        "    WHERE table_name = 'variorum_series'\n"
        "      AND column_name != 'time_ns';\n"
        "    IF col_list IS NULL THEN\n"
        "        RETURN QUERY SELECT * FROM variorum_series;\n"
        "    ELSE\n"
        "        dyn_sql := format('SELECT * FROM variorum_series WHERE %s', col_list);\n"
        "        RETURN QUERY EXECUTE dyn_sql;\n"
        "    END IF;\n"
        "END;\n"
        "$$ LANGUAGE plpgsql;\n"
    )
    sql_path = os.path.join(output_dir, 'variorum_series.sql')
    with open(sql_path, 'w') as f:
        f.write(sql)
    print(f'Created SQL file: {sql_path}')
