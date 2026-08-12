cmake_minimum_required(VERSION 3.16)
project(energy-bench CXX)

find_package(Kokkos REQUIRED)

add_executable(energy_bench energy_bench.cpp)
target_link_libraries(energy_bench PRIVATE Kokkos::kokkos)
target_compile_features(energy_bench PRIVATE cxx_std_17)
