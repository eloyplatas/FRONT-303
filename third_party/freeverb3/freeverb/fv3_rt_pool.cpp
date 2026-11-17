/*  Front 303 addition (GPL-3.0-or-later, like freeverb3). See fv3_rt_pool.hpp. */
#include "fv3_rt_pool.hpp"
#include <cstdlib>
#include <new>

struct fv3_rt_pool_header
{
  size_t capacity;
  fv3_rt_pool_header * next;
  fv3_rt_pool * owner;
  double align;
};

namespace {
  thread_local fv3_rt_pool * current = nullptr;
  struct Guard {
    std::atomic_flag & flag;
    explicit Guard(std::atomic_flag & f) : flag(f) { while(flag.test_and_set(std::memory_order_acquire)) {} }
    ~Guard() { flag.clear(std::memory_order_release); }
  };
}

fv3_rt_pool::~fv3_rt_pool()
{
  while(head != nullptr)
    {
      fv3_rt_pool_header * h = head;
      head = h->next;
      std::free(h);
    }
}

fv3_rt_pool_scope::fv3_rt_pool_scope(fv3_rt_pool & pool) : previous(current) { current = &pool; }
fv3_rt_pool_scope::~fv3_rt_pool_scope() { current = previous; }

void * fv3_rt_pool_take(size_t bytes)
{
  fv3_rt_pool * pool = current;
  if(pool != nullptr)
    {
      Guard guard(pool->lock);
      fv3_rt_pool_header ** best = nullptr;
      for(fv3_rt_pool_header ** it = &pool->head; *it != nullptr; it = &(*it)->next)
        if((*it)->capacity >= bytes && (best == nullptr || (*it)->capacity < (*best)->capacity))
          best = it;
      if(best != nullptr)
        {
          fv3_rt_pool_header * h = *best;
          *best = h->next;
          h->next = nullptr;
          return h + 1;
        }
    }
  fv3_rt_pool_header * h = static_cast<fv3_rt_pool_header*>(std::malloc(sizeof(fv3_rt_pool_header) + bytes));
  if(h == nullptr) throw std::bad_alloc();
  h->capacity = bytes;
  h->next = nullptr;
  h->owner = pool;
  return h + 1;
}

void fv3_rt_pool_give(void * ptr)
{
  if(ptr == nullptr) return;
  fv3_rt_pool_header * h = static_cast<fv3_rt_pool_header*>(ptr) - 1;
  fv3_rt_pool * pool = h->owner;
  if(pool == nullptr)
    {
      std::free(h);
      return;
    }
  Guard guard(pool->lock);
  h->next = pool->head;
  pool->head = h;
}
