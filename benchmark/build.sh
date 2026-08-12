#!/bin/bash
# Requires Kokkos installed with CUDA support (kokkos_enable_cuda=ON at Kokkos build time).
#
# Minimal usage (Kokkos in default CMake paths):
#   ./build.sh
#
# If Kokkos is in a custom prefix:
#   KOKKOS_DIR=/opt/kokkos/lib/cmake/Kokkos ./build.sh
#
# GPU arch is inferred from the installed Kokkos. If build fails on arch,
# rebuild Kokkos with the right Kokkos_ARCH_* flag for your GPU:
#   RTX 20xx (Turing)   → -DKokkos_ARCH_TURING75=ON
#   RTX 30xx (Ampere)   → -DKokkos_ARCH_AMPERE86=ON
#   RTX 40xx (Ada)      → -DKokkos_ARCH_ADA89=ON
#
# WSL2: install the WSL-specific CUDA toolkit (not the Linux one — it ships
# without the driver). The nvcc compiler must be in PATH.
# See: https://developer.nvidia.com/cuda-downloads → Linux → WSL-Ubuntu

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

cmake -S "$SCRIPT_DIR" -B "$SCRIPT_DIR/build" \
  ${KOKKOS_DIR:+-DKokkos_DIR="$KOKKOS_DIR"} \
  -DCMAKE_BUILD_TYPE=Release

cmake --build "$SCRIPT_DIR/build" -j"$(nproc)"
echo "Binary: $SCRIPT_DIR/build/energy_bench"
