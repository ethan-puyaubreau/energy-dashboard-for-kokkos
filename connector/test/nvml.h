// Minimal stand-in for the NVML header: only what the connector calls.
// Used to build and test the connector on machines without a GPU.
#pragma once

#ifdef __cplusplus
extern "C" {
#endif

typedef enum { NVML_SUCCESS = 0, NVML_ERROR_UNINITIALIZED = 1 } nvmlReturn_t;
typedef struct nvmlDevice_st *nvmlDevice_t;

nvmlReturn_t nvmlInit(void);
nvmlReturn_t nvmlShutdown(void);
nvmlReturn_t nvmlDeviceGetCount(unsigned int *count);
nvmlReturn_t nvmlDeviceGetHandleByIndex(unsigned int index, nvmlDevice_t *device);
nvmlReturn_t nvmlDeviceGetPowerUsage(nvmlDevice_t device, unsigned int *power);

#ifdef __cplusplus
}
#endif
