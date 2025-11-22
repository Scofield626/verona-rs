
#include <cpp/when.h>
#include <cstdio>
#include <iostream>
#include <verona.h>

#include <atomic>

using namespace verona::rt;
using namespace verona::cpp;

static std::atomic<bool> external_source_added{false};

extern "C" {
void poll_future_in_rust(void *future);

void runtime_init(size_t threads) {
  auto &sched = Scheduler::get();
  Scheduler::set_detect_leaks(true);
  sched.set_fair(true);
  sched.init(threads);
  // Prevent the runtime from getting destroyed
  when() << [=]() {
    if (!external_source_added.exchange(true)) {
      Scheduler::add_external_event_source();
    }
  };
}

void runtime_shutdown() {
  when() << [=]() {
    if (external_source_added.exchange(false)) {
      Scheduler::remove_external_event_source();
    }
  };
}

void scheduler_run() {
  auto &sched = Scheduler::get();
  sched.run();
}

void schedule_task(void *task) {
  when() << [=]() { poll_future_in_rust(task); };
}
}
