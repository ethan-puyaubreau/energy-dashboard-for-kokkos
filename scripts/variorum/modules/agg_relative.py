import os
from glob import glob
from modules.utils import load_and_concat_csvs, process_timestamp_column, group_by_time_window
import pandas as pd

TIME_WINDOW_MS = 20

def aggregate_relative(files):
    df = load_and_concat_csvs(files)
    if df.empty:
        return df
    df, actual_time_col = process_timestamp_column(df, 'time_relative_ms')
    if actual_time_col:
        return group_by_time_window(df, actual_time_col, TIME_WINDOW_MS)
    return pd.DataFrame()
