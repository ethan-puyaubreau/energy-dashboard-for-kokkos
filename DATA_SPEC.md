# Data Specification: kokkos-energy Trace Format (v1)

This document formally specifies the input data format expected by `kokkos-energy` (v0.1.0+).

## 1. Overview

An execution trace consists of a directory containing:
1. `events.csv`: Hierarchical Kokkos execution blocks (regions and kernels).
2. `power_samples.csv`: Periodic hardware telemetry samples (CPU/GPU power).
3. `metadata.json` (optional): Experiment and host environment metadata.

All timestamps are expressed as **64-bit unsigned integers in nanoseconds** (`uint64_t`), referencing UNIX epoch.

---

## 2. File Schemas

### 2.1 `events.csv`
Defines discrete application execution intervals.

| Column | Type | Description |
| :--- | :--- | :--- |
| `id` | `uint64` | Unique event identifier within the run |
| `parent_id` | `uint64` | Identifier of the enclosing region (0 if root event) |
| `name` | `string` | Label given to the region or kernel (e.g. `MatVec`, `cg_solver`) |
| `category` | `string` | Block category: `USER_REGION`, `PARALLEL_FOR`, `PARALLEL_REDUCE`, `PARALLEL_SCAN`, `DEEP_COPY` |
| `start_ns` | `uint64` | Start timestamp in nanoseconds since UNIX epoch |
| `end_ns` | `uint64` | End timestamp in nanoseconds since UNIX epoch |

Example:
```csv
id,parent_id,name,category,start_ns,end_ns
1,0,SolverStep,USER_REGION,1723500000000000000,1723500008000000000
2,1,MatVec,PARALLEL_FOR,1723500000100000000,1723500004100000000
3,1,Reduction,PARALLEL_REDUCE,1723500004200000000,1723500007900000000
```

### 2.2 `power_samples.csv`
Defines the continuous physical telemetry collected by the asynchronous sampling daemon.

| Column | Type | Description |
| :--- | :--- | :--- |
| `timestamp_ns` | `uint64` | Measurement timestamp in nanoseconds since UNIX epoch |
| `domain` | `string` | Subsystem domain: `GPU`, `CPU_PKG`, `CPU_DRAM`, `NODE` |
| `device_id` | `uint32` | Device index (GPU id or CPU socket id) |
| `power_watts` | `float64` | Instantaneous power in Watts |
| `energy_joules`| `float64` | (Optional) Hardware cumulative energy counter in Joules (empty if unavailable) |

Example:
```csv
timestamp_ns,domain,device_id,power_watts,energy_joules
1723500000000000000,GPU,0,250.0,
1723500001000000000,GPU,0,250.0,
1723500002000000000,GPU,0,300.0,
1723500003000000000,GPU,0,300.0,
1723500004000000000,GPU,0,200.0,
```

### 2.3 `metadata.json` (Optional)
```json
{
  "spec_version": "1.0",
  "app_name": "energy_bench",
  "hostname": "compute-node-42",
  "kokkos_backend": "CUDA",
  "start_epoch_ns": 1723500000000000000
}
```
