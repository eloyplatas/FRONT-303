// Front 303 — Aether (Dougal Stewart; MIT), an algorithmic reverb after CloudSeed. The plug-in
// ships no presets, so Front 303 defines eight programs on Aether's own parameters.

#include "reverb.hpp"
#include "DSP/aether_dsp.hpp"
#include "common/parameters.hpp"

namespace f303 {
namespace {

using P = Aether::DSP::Parameters<float>;

struct AetherProgram {
  const char * name;
  float early_level, late_level;
  float taps, tap_length, tap_mix, tap_decay;
  float early_stages, early_delay, early_mod_depth, early_mod_rate, early_feedback;
  float order, lines, line_delay, line_mod_depth, line_mod_rate, line_feedback;
  float late_stages, late_delay, late_mod_depth, late_mod_rate, late_feedback;
  float low_shelf_cutoff, low_shelf_gain, high_shelf_cutoff, high_shelf_gain, high_cut;
};

const AetherProgram PROGRAMS_[PROGRAMS] = {
  // name           early late  taps len  mix  dec  st  dly  md   mr   fb   ord ln  ldly  lmd  lmr  lfb   st dly md   mr   fb   lsf  lsg  hsf   hsg  hc
  {"Velvet Hall",    20, 80,   12, 120, 100, 0.5, 6, 25, 0.3, 0.5, 0.70, 0, 6, 120, 0.3, 0.2, 0.82, 6, 50, 0.3, 0.5, 0.70, 100, -2, 3000, -4, 12000},
  {"Wood Room",      45, 55,   16,  40, 100, 0.7, 5, 15, 0.1, 0.4, 0.60, 0, 4,  35, 0.1, 0.2, 0.62, 4, 20, 0.1, 0.4, 0.60, 150, -3, 2500, -6,  9000},
  {"Stone Chamber",  30, 70,   20,  80, 100, 0.4, 7, 30, 0.2, 0.3, 0.72, 0, 6,  70, 0.2, 0.3, 0.80, 6, 35, 0.2, 0.3, 0.72, 120, -1, 4000, -3, 14000},
  {"Cathedral",      15, 85,   24, 250, 100, 0.3, 8, 45, 0.4, 0.3, 0.75, 1, 12, 250, 0.4, 0.2, 0.90, 7, 70, 0.4, 0.3, 0.75,  90, -1, 2000, -6, 10000},
  {"Bright Plate",    5, 95,    1,   0,   0, 0.0, 8, 10, 0.5, 1.0, 0.78, 1, 4,  60, 0.8, 0.8, 0.86, 8, 12, 0.6, 1.2, 0.78, 200, -6, 6000,  0, 18000},
  {"Slow Bloom",     10, 90,   30, 400, 100, 0.0, 8, 60, 1.0, 0.2, 0.80, 1, 8, 400, 2.0, 0.1, 0.92, 8, 80, 1.2, 0.2, 0.80, 100, -2, 1800, -8,  8000},
  {"Dark Cave",      25, 75,   18, 150, 100, 0.6, 6, 35, 0.2, 0.2, 0.70, 0, 8, 160, 0.3, 0.2, 0.88, 6, 50, 0.3, 0.2, 0.70,  80,  0,  800,-12,  3500},
  {"Endless",         5, 95,   10, 300, 100, 0.0, 8, 50, 0.8, 0.3, 0.75, 1, 12, 300, 1.5, 0.15,0.985,8, 60, 1.0, 0.3, 0.75, 100, -3, 2500, -6, 10000},
};

class Aether_ : public Engine {
  Aether::DSP dsp;
  float values[47];
  int current = 0;
  Shape knobs;

  void set(float P::* field, float v)
  {
    P dummy{};
    std::size_t index = &(dummy.*field) - dummy.data();
    float lo = parameter_infos[index + 6].min, hi = parameter_infos[index + 6].max;
    values[index] = f303::clamp<float>(v, lo, hi);
  }

  void apply()
  {
    const AetherProgram & a = PROGRAMS_[current];
    double sf = size_factor(knobs.size);
    double df = decay_factor(knobs.decay);
    double mf = modulation_factor(knobs.modulation);
    double kf = damping_factor(knobs.damping);
    set(&P::mix, 100);
    set(&P::dry_level, 0);
    set(&P::predelay_level, 0);
    set(&P::early_level, a.early_level);
    set(&P::late_level, a.late_level);
    set(&P::interpolate, 1);
    set(&P::width, 100);
    set(&P::predelay, 0);
    set(&P::early_low_cut_enabled, 1);
    set(&P::early_low_cut_cutoff, 40);
    set(&P::early_high_cut_enabled, 1);
    set(&P::early_high_cut_cutoff, a.high_cut * kf);
    set(&P::early_taps, a.taps);
    set(&P::early_tap_length, a.tap_length * sf);
    set(&P::early_tap_mix, a.tap_mix);
    set(&P::early_tap_decay, a.tap_decay);
    set(&P::early_diffusion_stages, a.early_stages);
    set(&P::early_diffusion_delay, a.early_delay * sf);
    set(&P::early_diffusion_mod_depth, a.early_mod_depth * mf);
    set(&P::early_diffusion_mod_rate, a.early_mod_rate);
    set(&P::early_diffusion_feedback, a.early_feedback);
    set(&P::late_order, a.order);
    set(&P::late_delay_lines, a.lines);
    set(&P::late_delay, a.line_delay * sf);
    set(&P::late_delay_mod_depth, a.line_mod_depth * mf);
    set(&P::late_delay_mod_rate, a.line_mod_rate);
    // the decay knob scales the time to fall 60 dB: the loop gain is raised to 1/factor
    set(&P::late_delay_line_feedback, std::pow(a.line_feedback, 1.0 / df));
    set(&P::late_diffusion_stages, a.late_stages);
    set(&P::late_diffusion_delay, a.late_delay * sf);
    set(&P::late_diffusion_mod_depth, a.late_mod_depth * mf);
    set(&P::late_diffusion_mod_rate, a.late_mod_rate);
    set(&P::late_diffusion_feedback, a.late_feedback);
    set(&P::late_low_shelf_enabled, 1);
    set(&P::late_low_shelf_cutoff, a.low_shelf_cutoff);
    set(&P::late_low_shelf_gain, a.low_shelf_gain);
    set(&P::late_high_shelf_enabled, 1);
    set(&P::late_high_shelf_cutoff, a.high_shelf_cutoff * kf);
    set(&P::late_high_shelf_gain, a.high_shelf_gain - (knobs.damping - 0.5) * 12.0);
    set(&P::late_high_cut_enabled, 1);
    set(&P::late_high_cut_cutoff, a.high_cut * kf);
    set(&P::seed_crossmix, 80);
    set(&P::tap_seed, 1 + current * 7);
    set(&P::early_diffusion_seed, 2 + current * 7);
    set(&P::delay_seed, 3 + current * 7);
    set(&P::late_diffusion_seed, 4 + current * 7);
    set(&P::early_diffusion_drive, -12);
    set(&P::late_diffusion_drive, -12);
  }

public:
  explicit Aether_(double rate) : dsp((float)rate)
  {
    for(std::size_t i = 0; i < 47; i++)
      {
        values[i] = parameter_infos[i + 6].dflt;
        dsp.param_ports[i] = &values[i];
      }
    dsp.ports.control = nullptr;
    dsp.ports.notify = nullptr;
    apply();
  }

  void program(int p) override { if(p != current) { current = p; apply(); } }
  void shape(const Shape & s) override { if(s != knobs) { knobs = s; apply(); } }
  void clear() override {}

  void process(const float * il, const float * ir, float * ol, float * orr, int n) override
  {
    dsp.ports.audio_in_left = il;
    dsp.ports.audio_in_right = ir;
    dsp.ports.audio_out_left = ol;
    dsp.ports.audio_out_right = orr;
    dsp.process((uint32_t)n);
  }
};

}

Engine * make_aether(double rate) { return new Aether_(rate); }
const char * aether_name(int p) { return PROGRAMS_[p].name; }

}
