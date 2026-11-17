// Front 303 — C interface to the reverb engines. GPL-3.0-only.
#include "reverb.hpp"

using namespace f303;

extern "C" {

void * f303_rev_new(int kind, double rate)
{
  switch(kind)
    {
    case 0: return make_hall(rate);
    case 1: return make_early(rate);
    case 2: return make_aether(rate);
    default: return make_cloud(rate);
    }
}

void f303_rev_free(void * e) { delete static_cast<Engine*>(e); }
void f303_rev_program(void * e, int p) { static_cast<Engine*>(e)->program(f303::clamp<int>(p, 0, PROGRAMS - 1)); }

void f303_rev_shape(void * e, double size, double decay, double damping, double modulation)
{
  Shape s;
  s.size = f303::clamp<double>(size, 0.0, 1.0);
  s.decay = f303::clamp<double>(decay, 0.0, 1.0);
  s.damping = f303::clamp<double>(damping, 0.0, 1.0);
  s.modulation = f303::clamp<double>(modulation, 0.0, 1.0);
  static_cast<Engine*>(e)->shape(s);
}

void f303_rev_clear(void * e) { static_cast<Engine*>(e)->clear(); }

void f303_rev_process(void * e, const float * il, const float * ir, float * ol, float * orr, int n)
{
  static_cast<Engine*>(e)->process(il, ir, ol, orr, std::min(n, MAX_BLOCK));
}

const char * f303_rev_program_name(int kind, int p)
{
  p = f303::clamp<int>(p, 0, PROGRAMS - 1);
  switch(kind)
    {
    case 0: return hall_name(p);
    case 1: return early_name(p);
    case 2: return aether_name(p);
    default: return cloud_name(p);
    }
}

}
