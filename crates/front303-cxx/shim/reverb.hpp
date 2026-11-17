// Front 303 — the shared shape of the four reverb engines. GPL-3.0-only.
#pragma once
#include <cmath>
#include <algorithm>

namespace f303 {

constexpr int PROGRAMS = 8;

// std::clamp is C++17; freeverb3 needs C++14
template<class T> inline T clamp(T v, T lo, T hi) { return v < lo ? lo : (hi < v ? hi : v); }
constexpr int MAX_BLOCK = 64;

// The four character knobs arrive as 0..1 with 0.5 meaning "as the program has it".
struct Shape {
  double size = 0.5, decay = 0.5, damping = 0.5, modulation = 0.5;
  bool operator!=(const Shape & o) const
  { return size != o.size || decay != o.decay || damping != o.damping || modulation != o.modulation; }
};

// ×0.5 … ×2 for the size, ×0.35 … ×2.8 for the decay
inline double size_factor(double k) { return std::pow(2.0, (k - 0.5) * 2.0); }
inline double decay_factor(double k) { return std::pow(2.0, (k - 0.5) * 3.0); }
// a lower cut-off as the damping rises: ×2.8 … ×0.35
inline double damping_factor(double k) { return std::pow(2.0, -(k - 0.5) * 3.0); }
// none at 0, the program's at 0.5, four times it at 1
inline double modulation_factor(double k) { return 4.0 * k * k; }

struct Engine {
  virtual ~Engine() {}
  virtual void program(int p) = 0;
  virtual void shape(const Shape & s) = 0;
  virtual void clear() = 0;
  // wet only, n <= MAX_BLOCK
  virtual void process(const float * il, const float * ir, float * ol, float * orr, int n) = 0;
};

Engine * make_hall(double rate);
Engine * make_early(double rate);
Engine * make_aether(double rate);
Engine * make_cloud(double rate);
const char * hall_name(int p);
const char * early_name(int p);
const char * aether_name(int p);
const char * cloud_name(int p);

}
