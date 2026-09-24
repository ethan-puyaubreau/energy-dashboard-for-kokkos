// Loads the connector the way Kokkos Tools does and fires one region around one
// kernel, so CI can analyze the trace it writes. Usage: connector_smoke <libkokkos_energy.so>
#include <chrono>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <dlfcn.h>
#include <thread>

template <typename F>
F symbol(void *lib, const char *name) {
  void *s = dlsym(lib, name);
  if (!s) {
    std::fprintf(stderr, "missing symbol %s\n", name);
    std::exit(1);
  }
  return reinterpret_cast<F>(s);
}

int main(int argc, char **argv) {
  if (argc != 2) {
    std::fprintf(stderr, "usage: %s <libkokkos_energy.so>\n", argv[0]);
    return 2;
  }
  void *lib = dlopen(argv[1], RTLD_NOW);
  if (!lib) {
    std::fprintf(stderr, "%s\n", dlerror());
    return 1;
  }
  auto init = symbol<void (*)(int, uint64_t, uint32_t, void *)>(lib, "kokkosp_init_library");
  auto finalize = symbol<void (*)()>(lib, "kokkosp_finalize_library");
  auto push = symbol<void (*)(const char *)>(lib, "kokkosp_push_profile_region");
  auto pop = symbol<void (*)()>(lib, "kokkosp_pop_profile_region");
  auto begin_for = symbol<void (*)(const char *, uint32_t, uint64_t *)>(lib, "kokkosp_begin_parallel_for");
  auto end_for = symbol<void (*)(uint64_t)>(lib, "kokkosp_end_parallel_for");

  using namespace std::chrono_literals;
  init(0, 0, 0, nullptr);
  push("Solve");
  uint64_t kernel = 0;
  begin_for("Kernel", 0, &kernel);
  std::this_thread::sleep_for(500ms);
  end_for(kernel);
  std::this_thread::sleep_for(100ms);
  pop();
  finalize();
  dlclose(lib);
  return 0;
}
