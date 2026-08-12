import os
import sys
from glob import glob
from modules.agg_relative import aggregate_relative
from modules.agg_absolute import aggregate_absolute
from modules.agg_gpus import aggregate_gpus
from modules.agg_kernels import aggregate_kernels
from modules.agg_stats import aggregate_stats
from modules.agg_regions import aggregate_regions
from modules.correlation import create_correlation_data, create_time_series_data, generate_sql_schema

INPUT_DIR = 'input/variorum'
OUTPUT_DIR = 'data/variorum'

PATTERNS = {
    'variorum_relative': '*-variorum-power-relative.csv',
    'variorum_absolute': '*-variorum-power.csv',
    'variorum_gpus': '*-variorum-power-gpus.csv',
    'variorum_kernels': '*-variorum-power-kernels.csv',
    'variorum_stats': '*-variorum-power.dat',
}

OUTPUT_FILES = {
    'variorum_relative': 'variorum_relative.csv',
    'variorum_absolute': 'variorum_absolute.csv',
    'variorum_gpus': 'variorum_gpus.csv',
    'variorum_kernels': 'variorum_kernels.csv',
    'variorum_stats': 'variorum_stats.csv',
    'variorum_regions': 'variorum_regions.csv'
}

def process_standard_aggregation():
    aggregation_map = {
        'variorum_stats': aggregate_stats,
        'variorum_relative': aggregate_relative,
        'variorum_absolute': aggregate_absolute,
        'variorum_gpus': aggregate_gpus,
        'variorum_kernels': aggregate_kernels,
    }
    found_any = False
    for key, pattern in PATTERNS.items():
        files = glob(os.path.join(INPUT_DIR, '**', pattern), recursive=True)
        if not files:
            print(f'WARNING: No files found for {key} in {INPUT_DIR}', file=sys.stderr)
            continue
        found_any = True
        df = aggregation_map[key](files)
        if not df.empty:
            out_path = os.path.join(OUTPUT_DIR, OUTPUT_FILES[key])
            df.to_csv(out_path, index=False)
            print(f'Wrote {out_path} with {len(df)} rows')
        else:
            print(f'INFO: No aggregated data for {key}', file=sys.stderr)
    return found_any

def process_regions_aggregation():
    region_files = glob(os.path.join(INPUT_DIR, '**', '*regions.csv'), recursive=True)
    if region_files:
        df = aggregate_regions(region_files)
        if not df.empty:
            out_path = os.path.join(OUTPUT_DIR, OUTPUT_FILES['variorum_regions'])
            df.to_csv(out_path, index=False)
            print(f'Wrote {out_path} with {len(df)} rows')

def process_correlation_and_series():
    power_files = glob(os.path.join(INPUT_DIR, '**', '*variorum-power.csv'), recursive=True)
    region_files = glob(os.path.join(INPUT_DIR, '**', '*regions.csv'), recursive=True)
    if not power_files or not region_files:
        print("WARNING: No power or region files found for series generation")
        return
    correlation_df = create_correlation_data(power_files, region_files[0])
    if correlation_df.empty:
        print("ERROR: Failed to create correlation data")
        return
    correlation_path = os.path.join(OUTPUT_DIR, 'correlation.csv')
    correlation_df.to_csv(correlation_path, index=False)
    print(f'Wrote {correlation_path}')
    series_df = create_time_series_data(correlation_df)
    if series_df.empty:
        print("ERROR: Failed to create series data")
        return
    series_path = os.path.join(OUTPUT_DIR, 'variorum_series.csv')
    series_df.to_csv(series_path, index=False)
    print(f'Created series file: {series_path}')
    generate_sql_schema(series_df, OUTPUT_DIR)

def main():
    os.makedirs(OUTPUT_DIR, exist_ok=True)
    found_any = process_standard_aggregation()
    process_regions_aggregation()
    if not found_any:
        print(f'WARNING: No input data found in {INPUT_DIR}', file=sys.stderr)
    process_correlation_and_series()

if __name__ == '__main__':
    main()