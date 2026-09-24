cmake_minimum_required(VERSION 3.18)
project(kokkos_energy_connector LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
find_package(Threads REQUIRED)

# Build against a stub NVML that reports one GPU at 100 W, for CI machines without a GPU.
option(ENERGY_CONNECTOR_NVML_STUB "Build against the NVML stub in test/ (CI only)" OFF)

add_library(kokkos_energy SHARED kp_energy_profiler.cpp)

if(ENERGY_CONNECTOR_NVML_STUB)
  add_library(nvml_stub SHARED test/nvml_stub.cpp)
  target_include_directories(nvml_stub PUBLIC test)
  target_link_libraries(kokkos_energy PRIVATE nvml_stub Threads::Threads)

  add_executable(connector_smoke test/smoke.cpp)
  target_link_libraries(connector_smoke PRIVATE ${CMAKE_DL_LIBS})
else()
  find_package(CUDAToolkit REQUIRED)
  target_link_libraries(kokkos_energy PRIVATE CUDA::nvml Threads::Threads)
endif()
