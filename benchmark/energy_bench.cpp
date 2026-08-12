#include <Kokkos_Core.hpp>
#include <chrono>
#include <cstdio>

using Clock = std::chrono::steady_clock;

static double elapsed_sec(Clock::time_point t0) {
    return std::chrono::duration<double>(Clock::now() - t0).count();
}

// Dense matrix-vector multiply: y[i] = sum_j A[i][j] * x[j]
static void matvec(
    Kokkos::View<double**> A,
    Kokkos::View<double*>  x,
    Kokkos::View<double*>  y,
    int N)
{
    Kokkos::parallel_for("matvec", N, KOKKOS_LAMBDA(int i) {
        double s = 0.0;
        for (int j = 0; j < N; ++j) s += A(i,j) * x(j);
        y(i) = s;
    });
    Kokkos::fence();
}

// Stream: y[i] = alpha * x[i] + y[i]
static void daxpy(
    Kokkos::View<double*> x,
    Kokkos::View<double*> y,
    double alpha, int N)
{
    Kokkos::parallel_for("daxpy", N, KOKKOS_LAMBDA(int i) {
        y(i) += alpha * x(i);
    });
    Kokkos::fence();
}

// Streaming sum
static double reduce_sum(Kokkos::View<double*> v, int N) {
    double result = 0.0;
    Kokkos::parallel_reduce("reduce", N,
        KOKKOS_LAMBDA(int i, double& acc) { acc += v(i); },
        result);
    Kokkos::fence();
    return result;
}

int main(int argc, char** argv) {
    Kokkos::initialize(argc, argv);
    {
        // N=4096 → matrix A ≈ 128 MB VRAM
        // NRED=32M → reduction array ≈ 256 MB VRAM
        const int N    = 4096;
        const int NRED = 32 * 1024 * 1024;

        Kokkos::View<double**> A("A", N, N);
        Kokkos::View<double*>  x("x", N);
        Kokkos::View<double*>  y("y", N);
        Kokkos::View<double*>  v("v", NRED);

        Kokkos::parallel_for("init_A",
            Kokkos::MDRangePolicy<Kokkos::Rank<2>>({0,0},{N,N}),
            KOKKOS_LAMBDA(int i, int j) { A(i,j) = 1.0 / (1.0 + i + j); });
        Kokkos::parallel_for("init_vecs", N, KOKKOS_LAMBDA(int i) {
            x(i) = 1.0;
            y(i) = 0.0;
        });
        Kokkos::parallel_for("init_v", NRED, KOKKOS_LAMBDA(int i) {
            v(i) = static_cast<double>(i % 1024) / 1024.0;
        });
        Kokkos::fence();

        // --- Warmup (2s, no profiling region) ---
        printf("[warmup] 2s DAXPY...\n");
        auto t0 = Clock::now();
        while (elapsed_sec(t0) < 2.0)
            daxpy(x, y, 0.001, N);

        // --- Region 1: MatVec (~4s, compute + memory bound) ---
        printf("[bench]  MatVec region (~4s)...\n");
        Kokkos::Profiling::pushRegion("MatVec");
        t0 = Clock::now();
        while (elapsed_sec(t0) < 4.0)
            matvec(A, x, y, N);
        Kokkos::Profiling::popRegion();

        // --- Region 2: Reduction (~4s, memory bandwidth bound) ---
        printf("[bench]  Reduction region (~4s)...\n");
        double checksum = 0.0;
        Kokkos::Profiling::pushRegion("Reduction");
        t0 = Clock::now();
        while (elapsed_sec(t0) < 4.0)
            checksum += reduce_sum(v, NRED);
        Kokkos::Profiling::popRegion();

        printf("[done]   checksum=%.6e\n", checksum);
    }
    Kokkos::finalize();
    return 0;
}
