Input folder for Variorum Tool

# Data Format:
`variorum-power-gpus.csv` :
```
timestamp_nanoseconds,power_watts,device_id
```

`variorum-power-kernels.csv` :
```
kernel_id,name,type,start_time_ns,end_time_ns,duration_ns
```

`variorum-power-regions.csv` :
```
name,start_time_ns,end_time_ns,duration_ns
```

`variorum-power-relative.csv` :
```
time_relative_ms,power_watts,energy_relative_joules
```
`variorum-power.csv` :
```
timestamp_nanoseconds,variorum_power_watts,variorum_integrated_energy_joules
```

`variorum-power.dat` :
```
# Variorum Power Profiler Results
execution_time_seconds: value
total_integrated_energy_joules: value
final_power_watts: value
average_power_watts: value
min_power_watts: value
max_power_watts: value
average_measured_power_watts: value
num_power_measurements: value
```