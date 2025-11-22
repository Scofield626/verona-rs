#pragma once
#include "rust/cxx.h"
#include "verust/src/verona_stubs.rs.h"
#include <cstddef>
#include <cstdint>

void runtime_init(size_t threads);
void runtime_shutdown();
void scheduler_run();
void schedule_task(rust::Box<VeronaTask> task);
