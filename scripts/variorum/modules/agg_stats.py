import pandas as pd
import sys

def aggregate_stats(files):
    dfs = []
    for f in files:
        try:
            with open(f, 'r') as file:
                lines = file.readlines()[1:]
            data = {}
            for line in lines:
                if ':' in line:
                    key, value = line.split(':', 1)
                    key = key.strip()
                    try:
                        data[key] = float(value.strip())
                    except ValueError:
                        print(f"WARNING: Cannot convert '{value.strip()}' to float for '{key}' in {f}", file=sys.stderr)
                        continue
            if data:
                dfs.append(pd.DataFrame([data]))
        except Exception as e:
            print(f"WARNING: Failed to process {f}: {e}", file=sys.stderr)
    if not dfs:
        return pd.DataFrame(columns=['stat_name', 'value'])
    df_concat = pd.concat(dfs, ignore_index=True)
    df_mean = df_concat.mean(numeric_only=True).reset_index()
    df_mean.columns = ['stat_name', 'value']
    return df_mean
