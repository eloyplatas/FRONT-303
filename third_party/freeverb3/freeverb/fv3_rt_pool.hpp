/*  Front 303 addition (GPL-3.0-or-later, like freeverb3).
 *
 *  Delay buffers in freeverb3 are reallocated whenever a size or preset changes, which Dragonfly
 *  does on the audio thread. Front 303 routes those allocations through recycling pools: each
 *  reverb owns one, freed buffers go back to their pool and are handed out again (best fit), so
 *  once a reverb has been warmed up at its largest size, its later size and program changes
 *  reuse memory instead of calling malloc. Code that runs a reverb names its pool with a
 *  fv3_rt_pool_scope; without one, buffers come straight from malloc. */
#ifndef FV3_RT_POOL_HPP
#define FV3_RT_POOL_HPP
#include <atomic>
#include <cstddef>

struct fv3_rt_pool_header;

struct fv3_rt_pool
{
  fv3_rt_pool_header * head = nullptr;
  std::atomic_flag lock = ATOMIC_FLAG_INIT;
  fv3_rt_pool() {}
  fv3_rt_pool(const fv3_rt_pool &) = delete;
  fv3_rt_pool & operator=(const fv3_rt_pool &) = delete;
  ~fv3_rt_pool();
};

/* The pool the calling thread's allocations come from while it lives. */
struct fv3_rt_pool_scope
{
  fv3_rt_pool * previous;
  explicit fv3_rt_pool_scope(fv3_rt_pool & pool);
  ~fv3_rt_pool_scope();
};

void * fv3_rt_pool_take(size_t bytes);
void   fv3_rt_pool_give(void * ptr);

template<class T> inline T * fv3_rt_new(long count)
{
  return static_cast<T*>(fv3_rt_pool_take(sizeof(T) * (size_t)(count > 0 ? count : 1)));
}
template<class T> inline void fv3_rt_delete(T * ptr) { fv3_rt_pool_give(static_cast<void*>(ptr)); }

#endif
