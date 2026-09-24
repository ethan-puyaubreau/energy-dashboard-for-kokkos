// NVML stub: one GPU drawing a constant 100 W.
#include "nvml.h"

extern "C" {
nvmlReturn_t nvmlInit(void) { return NVML_SUCCESS; }
nvmlReturn_t nvmlShutdown(void) { return NVML_SUCCESS; }
nvmlReturn_t nvmlDeviceGetCount(unsigned int *count) {
  *count = 1;
  return NVML_SUCCESS;
}
nvmlReturn_t nvmlDeviceGetHandleByIndex(unsigned int, nvmlDevice_t *device) {
  *device = nullptr;
  return NVML_SUCCESS;
}
nvmlReturn_t nvmlDeviceGetPowerUsage(nvmlDevice_t, unsigned int *power) {
  *power = 100000;  // milliwatts
  return NVML_SUCCESS;
}
}
