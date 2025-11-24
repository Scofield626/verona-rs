
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

// Helper - use cown_ptr<void*> instead of cown_ptr<void> since void cannot be
// stored
using CownPtrVoid = cown_ptr<void *>;

// Destructor for VoidCown
VoidCown::~VoidCown() {
  if (cown_ptr) {
    delete static_cast<CownPtrVoid *>(cown_ptr);
  }
}

// Create a cown holding arbitrary Rust data
std::unique_ptr<VoidCown> make_cown_any(size_t data_ptr) {
  // data_ptr is a usize from Rust pointing to Box<dyn Any + Send>
  void *rust_data = reinterpret_cast<void *>(data_ptr);

  // Create a cown holding the Rust data pointer
  // We wrap void* in cown_ptr<void*> since cown_ptr<void> is invalid
  auto cown = make_cown<void *>(rust_data);

  // Wrap in VoidCown for FFI
  auto *cown_heap = new CownPtrVoid(std::move(cown));
  return std::make_unique<VoidCown>(cown_heap);
}

// Clone a cown reference (increments reference count)
std::unique_ptr<VoidCown> cown_clone(const VoidCown &cown) {
  auto *src_cown = static_cast<CownPtrVoid *>(cown.cown_ptr);
  auto *cloned =
      new CownPtrVoid(*src_cown); // Copy constructor does refcount increment
  return std::make_unique<VoidCown>(cloned);
}

// Schedule a when() clause on a single cown
void when_cown(const VoidCown &cown, rust::Box<CownCallback> callback) {
  auto *cown_ptr = static_cast<CownPtrVoid *>(cown.cown_ptr);

  when(*cown_ptr) << [cb = std::move(callback)](auto acquired) mutable {
    // acquired gives us access to the cown's data
    // Dereference to get void** then dereference again to get void*
    void *rust_data_ptr = *acquired;
    size_t ptr_as_usize = reinterpret_cast<size_t>(rust_data_ptr);

    // Call back into Rust with the data pointer
    invoke_cown_callback(std::move(cb), ptr_as_usize);
  };
}

// Schedule a when() clause on two cowns
void when_cown2(const VoidCown &cown1, const VoidCown &cown2,
                rust::Box<CownCallback> callback) {
  auto *ptr1 = static_cast<CownPtrVoid *>(cown1.cown_ptr);
  auto *ptr2 = static_cast<CownPtrVoid *>(cown2.cown_ptr);

  when(*ptr1, *ptr2) <<
      [cb = std::move(callback)](auto acq1, auto acq2) mutable {
        // For multi-cown, we need a different callback signature
        // For now, just pass the first cown's data
        void *rust_data_ptr = *acq1;
        size_t ptr_as_usize = reinterpret_cast<size_t>(rust_data_ptr);

        invoke_cown_callback(std::move(cb), ptr_as_usize);
      };
}
