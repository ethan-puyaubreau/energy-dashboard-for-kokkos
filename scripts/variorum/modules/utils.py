import pandas as pd
import sys

def load_and_concat_csvs(files):
    if not files:
        return pd.DataFrame()
    try:
        dfs = [pd.read_csv(f) for f in files]
        return pd.concat(dfs, ignore_index=True)
    except Exception as e:
        print(f"ERROR: Failed to load CSV files: {e}", file=sys.stderr)
        return pd.DataFrame()

def validate_columns(df, required_cols, context="data"):
    missing = [col for col in required_cols if col not in df.columns]
    if missing:
        print(f"ERROR: Missing columns {missing} in {context}")
        return False
    return True

def convert_ns_to_ms(df, ns_cols):
    for ns_col in ns_cols:
        if ns_col in df.columns:
            ms_col = ns_col.replace('_ns', '_ms')
            df[ms_col] = df[ns_col] / 1_000_000
    return df

def group_by_time_window(df, time_col, window_ms):
    df = df.copy()
    df['window'] = (df[time_col] // window_ms) * window_ms
    grouped = df.groupby('window').agg({
        time_col: 'mean',
        **{col: 'mean' for col in df.columns if col not in [time_col, 'window']}
    }).reset_index(drop=True)
    cols = [time_col] + [c for c in grouped.columns if c != time_col]
    return grouped[cols]

def process_timestamp_column(df, preferred_col):
    if preferred_col in df.columns:
        return df, preferred_col
    elif 'timestamp_nanoseconds' in df.columns:
        print(f"INFO: Using timestamp_nanoseconds instead of {preferred_col}")
        df['timestamp_ms'] = df['timestamp_nanoseconds'] / 1_000_000
        return df, 'timestamp_ms'
    else:
        print(f"ERROR: No timestamp column found. Available: {df.columns.tolist()}")
        return df, None
