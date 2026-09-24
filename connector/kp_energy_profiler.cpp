// KokkosP energy connector (trace format v1, see DATA_SPEC.md).
// Developed on the feat/v1-energy-profiler branch of ethan-puyaubreau/kokkos-tools
// (commit ad5d3e8). Kokkos Tools is Apache-2.0 WITH LLVM-exception.

#include <algorithm>
#include <atomic>
#include <chrono>
#include <climits>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <mutex>
#include <nvml.h>
#include <string>
#include <thread>
#include <vector>
#include <unistd.h>

namespace KokkosTools::EnergyProfiler {

void finalize();

namespace {

/**
 * @brief Flag indicating if the profiler is currently active.
 */
std::atomic<bool> g_running{false};

/**
 * @brief Flag indicating if the profiler has been finalized.
 */
std::atomic<bool> g_finalized{false};

/**
 * @brief Emergency exit handler registered with std::atexit.
 */
void emergency_exit_handler() {
  if (g_running.load() && !g_finalized.load()) {
    finalize();
  }
}

/**
 * @brief Background thread for physical telemetry sampling.
 */
std::thread g_sampler_thread;

/**
 * @brief Root output directory for trace artifacts.
 */
std::string g_out_dir = "./";

/**
 * @brief Detected MPI rank, or -1 if not running in an MPI environment.
 */
int g_mpi_rank = -1;

/**
 * @brief Output stream for events.csv.
 */
std::ofstream g_events_file;

/**
 * @brief Output stream for power_samples.csv.
 */
std::ofstream g_power_file;

/**
 * @brief Handles to all available physical GPUs discovered by NVML.
 */
std::vector<nvmlDevice_t> g_nvml_devices;

/**
 * @brief Flag indicating if NVML was successfully initialized.
 */
bool g_nvml_ok = false;

/**
 * @brief Monotonically increasing unique event identifier generator.
 */
std::atomic<uint64_t> g_next_event_id{1};

/**
 * @brief Represents an in-flight region or parallel kernel dispatch.
 */
struct ActiveEvent {
  uint64_t id;
  uint64_t parent_id;
  std::string name;
  const char *category;
  uint64_t start_ns;
};

/**
 * @brief Represents a completed region or kernel ready for batch serialization.
 */
struct FinishedEvent {
  uint64_t id;
  uint64_t parent_id;
  std::string name;
  const char *category;
  uint64_t start_ns;
  uint64_t end_ns;
};

/**
 * @brief Per-thread event storage eliminating critical-path lock contention.
 */
struct ThreadEventBuffer {
  std::vector<ActiveEvent> active_stack;
  std::vector<FinishedEvent> finished_events;

  ThreadEventBuffer() {
    active_stack.reserve(64);
    finished_events.reserve(4096);
  }
};

/**
 * @brief Mutex protecting registration of thread-local buffers.
 */
std::mutex g_registry_mutex;

/**
 * @brief Master registry of all thread buffers for flush on finalization.
 */
std::vector<ThreadEventBuffer*> g_thread_buffers;

/**
 * @brief Pointer to the current thread's event buffer.
 */
thread_local ThreadEventBuffer* t_buffer = nullptr;

/**
 * @brief Retrieves or registers the thread-local event buffer.
 * @return Reference to the current thread's ThreadEventBuffer.
 */
inline ThreadEventBuffer& get_thread_buffer() {
  if (!t_buffer) {
    t_buffer = new ThreadEventBuffer();
    std::lock_guard<std::mutex> lock(g_registry_mutex);
    g_thread_buffers.push_back(t_buffer);
  }
  return *t_buffer;
}

/**
 * @brief System clock epoch baseline in nanoseconds.
 */
static const uint64_t g_epoch_system_ns = std::chrono::duration_cast<std::chrono::nanoseconds>(
    std::chrono::system_clock::now().time_since_epoch()).count();

/**
 * @brief Monotonic steady clock baseline at initialization.
 */
static const auto g_epoch_steady = std::chrono::steady_clock::now();

/**
 * @brief Returns the current timestamp in nanoseconds since UNIX epoch with guaranteed monotonicity.
 * @return Monotonically increasing time in nanoseconds.
 */
inline uint64_t now_ns() {
  auto elapsed = std::chrono::duration_cast<std::chrono::nanoseconds>(
      std::chrono::steady_clock::now() - g_epoch_steady).count();
  return g_epoch_system_ns + static_cast<uint64_t>(elapsed);
}

/**
 * @brief Discovers the local host name via gethostname.
 * @return Current hostname string.
 */
std::string get_current_hostname() {
  char buf[256] = {0};
  if (gethostname(buf, sizeof(buf) - 1) == 0) {
    return std::string(buf);
  }
  return "unknown_host";
}

/**
 * @brief Discovers the application executable basename via readlink.
 * @return Binary name string.
 */
std::string get_current_app_name() {
  char buf[PATH_MAX] = {0};
  ssize_t len = readlink("/proc/self/exe", buf, sizeof(buf) - 1);
  if (len > 0) {
    std::string path(buf, len);
    auto pos = path.find_last_of('/');
    return (pos != std::string::npos) ? path.substr(pos + 1) : path;
  }
  return "kokkos_app";
}

/**
 * @brief Discovers the MPI process rank from standard environment variables.
 * @return Rank integer if running under MPI, or -1 otherwise.
 */
int detect_mpi_rank() {
  const char *rank_vars[] = {
      "OMPI_COMM_WORLD_RANK",
      "PMI_RANK",
      "MV2_COMM_WORLD_RANK",
      "SLURM_PROCID",
  };
  for (const char *var : rank_vars) {
    const char *val = std::getenv(var);
    if (val && *val) {
      try {
        return std::stoi(val);
      } catch (...) {
        return -1;
      }
    }
  }
  return -1;
}

/**
 * @brief Discovers active execution backend.
 * @return String representation of active backend.
 */
std::string detect_backend() {
  const char *backend_env = std::getenv("KOKKOS_BACKEND");
  if (backend_env && *backend_env) {
    return std::string(backend_env);
  }
  if (g_nvml_ok && !g_nvml_devices.empty()) {
    return "CUDA";
  }
  if (std::getenv("OMP_NUM_THREADS") || std::getenv("OMP_PROC_BIND")) {
    return "OPENMP";
  }
  return "HOST";
}

/**
 * @brief Telemetry sampling loop capturing physical GPU power without cumulative drift.
 */
void sampler_loop() {
  const auto interval = std::chrono::milliseconds(20); // 50 Hz
  auto next_tick = std::chrono::steady_clock::now();

  while (g_running) {
    next_tick += interval;

    if (g_nvml_ok && g_power_file.is_open()) {
      uint64_t ts = now_ns();
      for (size_t i = 0; i < g_nvml_devices.size(); ++i) {
        unsigned int power_mw = 0;
        if (nvmlDeviceGetPowerUsage(g_nvml_devices[i], &power_mw) == NVML_SUCCESS) {
          double watts = static_cast<double>(power_mw) / 1000.0;
          g_power_file << ts << ",GPU," << i << "," << watts << ",\n";
        }
      }
      g_power_file.flush();
    }

    std::this_thread::sleep_until(next_tick);
  }
}

/**
 * @brief Writes session metadata JSON according to V1 specification.
 */
void write_metadata() {
  std::ofstream meta(g_out_dir + "/metadata.json");
  if (meta.is_open()) {
    meta << "{\n"
         << "  \"spec_version\": \"1.0\",\n"
         << "  \"app_name\": \"" << get_current_app_name() << "\",\n"
         << "  \"hostname\": \"" << get_current_hostname() << "\",\n"
         << "  \"kokkos_backend\": \"" << detect_backend() << "\",\n"
         << "  \"device_count\": " << g_nvml_devices.size() << ",\n"
         << "  \"start_epoch_ns\": " << now_ns();
    if (g_mpi_rank >= 0) {
      meta << ",\n  \"mpi_rank\": " << g_mpi_rank;
    }
    meta << "\n}\n";
  }
}

/**
 * @brief Escapes quotes and commas in strings according to RFC 4180 CSV standard.
 * @param field Raw input field string.
 * @return Escaped string safe for CSV serialization.
 */
std::string escape_csv_field(const std::string &field) {
  bool needs_quotes = false;
  std::string escaped;
  escaped.reserve(field.size() + 8);

  for (char c : field) {
    if (c == '"') {
      needs_quotes = true;
      escaped += "\"\"";
    } else {
      if (c == ',' || c == '\n' || c == '\r') {
        needs_quotes = true;
      }
      escaped += c;
    }
  }

  if (needs_quotes) {
    return "\"" + escaped + "\"";
  }
  return escaped;
}

} // namespace

/**
 * @brief Records entering a region or kernel into thread-local buffer.
 * @param name Event label or kernel name.
 * @param cat Event category (e.g. USER_REGION, PARALLEL_FOR).
 * @param kID Pointer to receive generated unique kernel ID.
 */
void push_event(const char *name, const char *cat, uint64_t *kID) {
  if (!g_running) return;

  uint64_t id = g_next_event_id.fetch_add(1, std::memory_order_relaxed);
  if (kID) *kID = id;

  ThreadEventBuffer &buf = get_thread_buffer();
  uint64_t parent_id = buf.active_stack.empty() ? 0 : buf.active_stack.back().id;
  uint64_t t0 = now_ns();

  buf.active_stack.push_back({id, parent_id, name ? name : "unnamed", cat ? cat : "", t0});
}

/**
 * @brief Records leaving a region or kernel and moves it to finished buffer.
 * @param kID Target kernel ID to close, or 0 for top of stack.
 */
void pop_event(uint64_t kID) {
  if (!g_running) return;

  uint64_t t1 = now_ns();
  ThreadEventBuffer &buf = get_thread_buffer();

  if (buf.active_stack.empty()) return;

  if (kID == 0 || buf.active_stack.back().id == kID) {
    ActiveEvent ev = std::move(buf.active_stack.back());
    buf.active_stack.pop_back();
    buf.finished_events.push_back({ev.id, ev.parent_id, std::move(ev.name), ev.category, ev.start_ns, t1});
    return;
  }

  for (auto it = buf.active_stack.rbegin(); it != buf.active_stack.rend(); ++it) {
    if (it->id == kID) {
      ActiveEvent ev = std::move(*it);
      auto forward_it = it.base() - 1;
      buf.active_stack.erase(forward_it);
      buf.finished_events.push_back({ev.id, ev.parent_id, std::move(ev.name), ev.category, ev.start_ns, t1});
      break;
    }
  }
}

/**
 * @brief Initializes output paths, streams, NVML devices, and sampling thread.
 */
void init() {
  const char *out_env = std::getenv("KOKKOS_TOOLS_OUTPUT_PATH");
  if (out_env && *out_env) {
    g_out_dir = out_env;
  }

  g_mpi_rank = detect_mpi_rank();
  if (g_mpi_rank >= 0) {
    g_out_dir = (std::filesystem::path(g_out_dir) / ("rank_" + std::to_string(g_mpi_rank))).string();
  }

  std::error_code ec;
  std::filesystem::create_directories(g_out_dir, ec);
  if (ec) {
    std::cerr << "[energy-dashboard-connector] Error creating output directory "
              << g_out_dir << ": " << ec.message() << "\n";
  }

  g_events_file.open(g_out_dir + "/events.csv");
  g_power_file.open(g_out_dir + "/power_samples.csv");

  if (g_events_file.is_open()) {
    g_events_file << "id,parent_id,name,category,start_ns,end_ns\n";
  }
  if (g_power_file.is_open()) {
    g_power_file << "timestamp_ns,domain,device_id,power_watts,energy_joules\n";
  }

  // Init NVML before generating metadata
  if (nvmlInit() == NVML_SUCCESS) {
    unsigned int dev_count = 0;
    if (nvmlDeviceGetCount(&dev_count) == NVML_SUCCESS && dev_count > 0) {
      for (unsigned int i = 0; i < dev_count; ++i) {
        nvmlDevice_t dev;
        if (nvmlDeviceGetHandleByIndex(i, &dev) == NVML_SUCCESS) {
          g_nvml_devices.push_back(dev);
        }
      }
      if (!g_nvml_devices.empty()) {
        g_nvml_ok = true;
        std::cout << "[energy-dashboard-connector] NVML initialized with "
                  << g_nvml_devices.size() << " GPU device(s)\n";
      }
    }
  } else {
    std::cerr << "[energy-dashboard-connector] Warning: NVML init failed, running without GPU telemetry\n";
  }

  write_metadata();

  std::atexit(emergency_exit_handler);

  g_running = true;
  g_sampler_thread = std::thread(sampler_loop);
}

/**
 * @brief Stops telemetry thread, flushes buffered thread events, and closes file handles.
 */
void finalize() {
  bool expected = false;
  if (!g_finalized.compare_exchange_strong(expected, true)) {
    return;
  }

  g_running = false;
  if (g_sampler_thread.joinable()) {
    g_sampler_thread.join();
  }

  if (g_nvml_ok) {
    nvmlShutdown();
  }

  // Flush all buffered thread events to events.csv
  if (g_events_file.is_open()) {
    std::lock_guard<std::mutex> lock(g_registry_mutex);
    uint64_t t_now = now_ns();

    // Close any unclosed active events in each buffer
    size_t total_events = 0;
    for (ThreadEventBuffer *buf : g_thread_buffers) {
      if (!buf) continue;
      while (!buf->active_stack.empty()) {
        ActiveEvent &ev = buf->active_stack.back();
        buf->finished_events.push_back({ev.id, ev.parent_id, std::move(ev.name), ev.category, ev.start_ns, t_now});
        buf->active_stack.pop_back();
      }
      total_events += buf->finished_events.size();
    }

    // Collect and sort all events chronologically across threads
    std::vector<const FinishedEvent*> all_events;
    all_events.reserve(total_events);
    for (ThreadEventBuffer *buf : g_thread_buffers) {
      if (!buf) continue;
      for (const auto &ev : buf->finished_events) {
        all_events.push_back(&ev);
      }
    }

    std::stable_sort(all_events.begin(), all_events.end(),
                     [](const FinishedEvent *a, const FinishedEvent *b) {
                       return a->start_ns < b->start_ns;
                     });

    for (const FinishedEvent *ev : all_events) {
      g_events_file << ev->id << "," << ev->parent_id << ","
                    << escape_csv_field(ev->name) << ","
                    << ev->category << "," << ev->start_ns << ","
                    << ev->end_ns << "\n";
    }

    for (ThreadEventBuffer *buf : g_thread_buffers) {
      delete buf;
    }
    g_thread_buffers.clear();
    g_events_file.close();
  }

  if (g_power_file.is_open()) {
    g_power_file.flush();
    g_power_file.close();
  }

  std::cout << "[energy-dashboard-connector] Profiling complete. Traces written to: " << g_out_dir << "\n";
}

} // namespace KokkosTools::EnergyProfiler

// C Interface expected by KokkosP runtime
extern "C" {

/**
 * @brief KokkosP initialization callback.
 */
void kokkosp_init_library(const int, const uint64_t, const uint32_t, void*) {
  KokkosTools::EnergyProfiler::init();
}

/**
 * @brief KokkosP finalization callback.
 */
void kokkosp_finalize_library() {
  KokkosTools::EnergyProfiler::finalize();
}

/**
 * @brief KokkosP push user region callback.
 * @param name Region name label.
 */
void kokkosp_push_profile_region(const char *name) {
  KokkosTools::EnergyProfiler::push_event(name, "USER_REGION", nullptr);
}

/**
 * @brief KokkosP pop user region callback.
 */
void kokkosp_pop_profile_region() {
  KokkosTools::EnergyProfiler::pop_event(0);
}

/**
 * @brief KokkosP begin parallel_for callback.
 * @param name Kernel name label.
 * @param devID Device identifier.
 * @param kID Pointer to receive kernel ID.
 */
void kokkosp_begin_parallel_for(const char *name, uint32_t, uint64_t *kID) {
  KokkosTools::EnergyProfiler::push_event(name, "PARALLEL_FOR", kID);
}

/**
 * @brief KokkosP end parallel_for callback.
 * @param kID Kernel ID returned during begin callback.
 */
void kokkosp_end_parallel_for(uint64_t kID) {
  KokkosTools::EnergyProfiler::pop_event(kID);
}

/**
 * @brief KokkosP begin parallel_reduce callback.
 * @param name Kernel name label.
 * @param devID Device identifier.
 * @param kID Pointer to receive kernel ID.
 */
void kokkosp_begin_parallel_reduce(const char *name, uint32_t, uint64_t *kID) {
  KokkosTools::EnergyProfiler::push_event(name, "PARALLEL_REDUCE", kID);
}

/**
 * @brief KokkosP end parallel_reduce callback.
 * @param kID Kernel ID returned during begin callback.
 */
void kokkosp_end_parallel_reduce(uint64_t kID) {
  KokkosTools::EnergyProfiler::pop_event(kID);
}

/**
 * @brief KokkosP begin parallel_scan callback.
 * @param name Kernel name label.
 * @param devID Device identifier.
 * @param kID Pointer to receive kernel ID.
 */
void kokkosp_begin_parallel_scan(const char *name, uint32_t, uint64_t *kID) {
  KokkosTools::EnergyProfiler::push_event(name, "PARALLEL_SCAN", kID);
}

/**
 * @brief KokkosP end parallel_scan callback.
 * @param kID Kernel ID returned during begin callback.
 */
void kokkosp_end_parallel_scan(uint64_t kID) {
  KokkosTools::EnergyProfiler::pop_event(kID);
}

struct Kokkos_Profiling_SpaceHandle {
  char name[64];
};

/**
 * @brief KokkosP begin deep copy callback for data transfers.
 * @param dst_handle Destination memory space handle.
 * @param dst_name Destination label.
 * @param dst_ptr Destination data pointer.
 * @param src_handle Source memory space handle.
 * @param src_name Source label.
 * @param src_ptr Source data pointer.
 * @param size Transfer size in bytes.
 */
void kokkosp_begin_deep_copy(
    struct Kokkos_Profiling_SpaceHandle dst_handle, const char* dst_name, const void*,
    struct Kokkos_Profiling_SpaceHandle src_handle, const char* src_name, const void*,
    uint64_t size) {
  std::string label = "deep_copy [";
  label += (src_name && *src_name) ? src_name : src_handle.name;
  label += " -> ";
  label += (dst_name && *dst_name) ? dst_name : dst_handle.name;
  label += "] (";
  label += std::to_string(size);
  label += " B)";
  KokkosTools::EnergyProfiler::push_event(label.c_str(), "DEEP_COPY", nullptr);
}

/**
 * @brief KokkosP end deep copy callback.
 */
void kokkosp_end_deep_copy() {
  KokkosTools::EnergyProfiler::pop_event(0);
}

} // extern "C"
