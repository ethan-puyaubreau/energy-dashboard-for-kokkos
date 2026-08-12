import pandas as pd
from modules.utils import load_and_concat_csvs, process_timestamp_column, group_by_time_window

TIME_WINDOW_MS = 20

def aggregate_absolute(files):
    df = load_and_concat_csvs(files)
    if df.empty:
        return df
    df, actual_time_col = process_timestamp_column(df, 'timestamp_nanoseconds')
    if not actual_time_col:
        df, actual_time_col = process_timestamp_column(df, 'timestamp_system_epoch_ms')
    if actual_time_col:
        return group_by_time_window(df, actual_time_col, TIME_WINDOW_MS)
    return pd.DataFrame()
