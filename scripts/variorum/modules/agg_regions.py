import pandas as pd
from modules.utils import load_and_concat_csvs, validate_columns, convert_ns_to_ms

def aggregate_regions(files):
    df = load_and_concat_csvs(files)
    if df.empty:
        return df
    required_cols = ['start_time_ns', 'end_time_ns', 'name']
    if not validate_columns(df, required_cols, "region data"):
        return pd.DataFrame()
    df = convert_ns_to_ms(df, ['start_time_ns', 'end_time_ns'])
    return df[['name', 'start_time_ms', 'end_time_ms']]
