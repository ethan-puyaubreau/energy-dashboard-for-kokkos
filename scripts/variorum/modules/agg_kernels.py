import pandas as pd
from modules.utils import load_and_concat_csvs, validate_columns, convert_ns_to_ms

def aggregate_kernels(files):
    df = load_and_concat_csvs(files)
    if df.empty:
        return df
    ns_cols = ['start_time_ns', 'end_time_ns', 'duration_ns']
    if not validate_columns(df, ns_cols, "kernel data"):
        return pd.DataFrame()
    df = convert_ns_to_ms(df, ns_cols)
    return df[['kernel_id', 'name', 'type', 'start_time_ms', 'end_time_ms', 'duration_ms']]
