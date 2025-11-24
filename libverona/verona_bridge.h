#pragma once
#include <cstddef>
#include <cstdint>
#include <memory>

// Forward declare for cxx - actual definition comes later
struct VoidCown;

// Now include cxx bridge which will reference VoidCown
#include "rust/cxx.h"
#include "verust/src/verona_stubs.rs.h"

// Runtime functions
void runtime_init(size_t threads);
void runtime_shutdown();
void scheduler_run();
void schedule_task(rust::Box<VeronaTask> task);

// Cown operations - define struct after cxx includes
struct VoidCown {
  void *cown_ptr; // Pointer to verona::cpp::cown_ptr<void>

  VoidCown(void *ptr) : cown_ptr(ptr) {}
  ~VoidCown();
};

std::unique_ptr<VoidCown> make_cown_any(size_t data_ptr);
std::unique_ptr<VoidCown> cown_clone(const VoidCown &cown);
void when_cown(const VoidCown &cown, rust::Box<CownCallback> callback);
void when_cown2(const VoidCown &cown1, const VoidCown &cown2,
                rust::Box<CownCallback> callback);
