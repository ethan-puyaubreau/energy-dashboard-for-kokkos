import pandas as pd
from modules.utils import load_and_concat_csvs, validate_columns

TIME_WINDOW_MS = 20

def aggregate_gpus(files):
    df = load_and_concat_csvs(files)
    if df.empty or not validate_columns(df, ['timestamp_nanoseconds', 'power_watts'], "GPU data"):
        return pd.DataFrame()
    df['timestamp_ms'] = df['timestamp_nanoseconds'] / 1_000_000
    df['window'] = (df['timestamp_ms'] // TIME_WINDOW_MS) * TIME_WINDOW_MS
    grouped = df.groupby(['window', 'device_id']).agg({
        'timestamp_ms': 'mean',
        'power_watts': 'mean'
    }).reset_index()
    return grouped[['timestamp_ms', 'power_watts', 'device_id']]
