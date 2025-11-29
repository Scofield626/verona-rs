
#include <cpp/when.h>
#include <cstdio>
#include <iostream>
#include <verona.h>

#include <atomic>

using namespace verona::rt;
using namespace verona::cpp;

static std::atomic<bool> external_source_added{false};

#include "libverona/verona_bridge.h"

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

void schedule_task(rust::Box<VeronaTask> task) {
  when() << [t = std::move(task)]() mutable { poll_future(std::move(t)); };
}

// === Cown Implementation ===

struct RustBox {
  size_t ptr;
  void (*dtor)(size_t);

  RustBox(size_t p, void (*d)(size_t)) : ptr(p), dtor(d) {}
  ~RustBox() { dtor(ptr); }
};

using CownPtrRust = cown_ptr<RustBox>;

// Destructor for VoidCown
VoidCown::~VoidCown() {
  if (cown_ptr) {
    delete static_cast<CownPtrRust *>(cown_ptr);
  }
}

// Create a cown holding arbitrary Rust data
std::unique_ptr<VoidCown> make_cown(size_t data_ptr, size_t dtor_ptr) {
  auto dtor = reinterpret_cast<void (*)(size_t)>(dtor_ptr);
  auto cown = make_cown<RustBox>(data_ptr, dtor);
  auto *cown_heap = new CownPtrRust(std::move(cown));
  return std::make_unique<VoidCown>(cown_heap);
}

// Clone a cown reference (increments reference count)
std::unique_ptr<VoidCown> cown_clone(const VoidCown &cown) {
  auto *src_cown = static_cast<CownPtrRust *>(cown.cown_ptr);
  auto *cloned =
      new CownPtrRust(*src_cown); // Copy constructor does refcount increment
  return std::make_unique<VoidCown>(cloned);
}

// Schedule a when() clause on a single cown
void when_cown(const VoidCown &cown, rust::Box<CownCallback> callback) {
  auto *cown_ptr = static_cast<CownPtrRust *>(cown.cown_ptr);

  when(*cown_ptr) << [cb = std::move(callback)](auto acquired) mutable {
    RustBox &rust_box = *acquired;
    invoke_cown_callback(std::move(cb), rust_box.ptr);
  };
}

// Schedule a when() clause on two cowns
void when_cown2(const VoidCown &cown1, const VoidCown &cown2,
                rust::Box<CownCallback> callback) {
  auto *ptr1 = static_cast<CownPtrRust *>(cown1.cown_ptr);
  auto *ptr2 = static_cast<CownPtrRust *>(cown2.cown_ptr);

  when(*ptr1, *ptr2) <<
      [cb = std::move(callback)](auto acq1, auto acq2) mutable {
        // For multi-cown, we need a different callback signature
        // For now, just pass the first cown's data
        RustBox &rust_box = *acq1;
        invoke_cown_callback(std::move(cb), rust_box.ptr);
      };
}
