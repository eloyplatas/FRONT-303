// Front 303 — Dragonfly Hall and Dragonfly Early Reflections (Michael Willis, Rob van den Berg;
// GPL-3.0-or-later) on freeverb3 (Teru Kamogashira; GPL-2.0-or-later). The processing follows
// Dragonfly's DSP.cpp; the programs are taken from Dragonfly's own banks.

#include "reverb.hpp"
#include "freeverb/earlyref.hpp"
#include "freeverb/zrev2.hpp"
#include "freeverb/fv3_type_float.h"
#include "freeverb/fv3_rt_pool.hpp"

namespace f303 {
namespace {

// dry, early, late, size, width, delay, diffuse, low cut, low xo, low mult, high cut, high xo,
// high mult, spin, wander, decay, e. send, modulation — rows of Dragonfly Hall's banks
struct HallPreset { const char * name; float p[18]; };
const HallPreset HALL[PROGRAMS] = {
  {"Bright Room",        { 80.0, 10.0, 20.0, 10.0,  90.0,  4.0, 90.0, 4.0, 500, 0.80, 16000, 7900, 0.75, 1.0, 25.0, 0.6, 20.0, 30.0 }},
  {"Large Chamber",      { 80.0, 10.0, 20.0, 20.0,  80.0,  8.0, 90.0, 4.0, 500, 1.30,  7000, 4900, 0.25, 1.8, 12.0, 1.0, 20.0, 20.0 }},
  {"Acoustic Studio",    { 80.0, 10.0, 20.0, 12.0,  90.0,  8.0, 75.0, 4.0, 450, 1.50,  7600, 4900, 0.80, 2.5,  7.0, 0.8, 20.0, 20.0 }},
  {"Small Clear Hall",   { 80.0, 10.0, 20.0, 24.0, 100.0,  4.0, 90.0, 4.0, 500, 1.30,  7600, 5500, 0.50, 3.3, 15.0, 1.3, 20.0, 15.0 }},
  {"Medium Bright Hall", { 80.0, 10.0, 20.0, 30.0, 100.0, 18.0, 90.0, 4.0, 400, 1.25, 10000, 6400, 0.60, 2.9, 15.0, 1.6, 20.0, 15.0 }},
  {"Medium Dark Hall",   { 80.0, 10.0, 20.0, 30.0, 100.0, 18.0, 60.0, 4.0, 500, 1.75,  5800, 4000, 0.40, 2.9, 15.0, 1.8, 20.0, 15.0 }},
  {"Large Clear Hall",   { 80.0, 10.0, 20.0, 40.0, 100.0, 12.0, 80.0, 4.0, 550, 2.00,  8200, 5200, 0.40, 2.1, 20.0, 2.8, 20.0, 15.0 }},
  {"Great Hall",         { 80.0, 10.0, 20.0, 50.0,  90.0, 20.0, 95.0, 4.0, 750, 2.50,  5500, 4000, 0.30, 2.6, 22.0, 3.8, 20.0, 15.0 }},
};
enum { DRY, EARLY, LATE, SIZE, WIDTH, DELAY, DIFFUSE, LOW_CUT, LOW_XO, LOW_MULT, HIGH_CUT,
       HIGH_XO, HIGH_MULT, SPIN, WANDER, DECAY, EARLY_SEND, MODULATION };

// Dragonfly Early Reflections' programs: name and freeverb3 reflection preset
struct EarlyPreset { const char * name; long number; };
const EarlyPreset EARLY_PROGRAMS[PROGRAMS] = {
  {"Abrupt Echo", 2}, {"Backstage Pass", 18}, {"Concert Venue", 0}, {"Damaged Goods", 19},
  {"Elevator Pitch", 1}, {"Floor Thirteen", 13}, {"Garage Band", 14}, {"Home Studio", 21},
};

// Size changes resize freeverb3's delay lines. The recycling pool (fv3_rt_pool) serves them
// from memory freed earlier, so each engine is first taken to its largest size once.
constexpr double MAX_SIZE = 60.0;
constexpr double MIN_SIZE = 10.0;

void setup_early(fv3::earlyref_f & e, double rate)
{
  e.setMuteOnChange(false);
  e.setdryr(0);
  e.setwet(0);
  e.setwidth(0.8);
  e.setLRDelay(0.3);
  e.setLRCrossApFreq(750, 4);
  e.setDiffusionApFreq(150, 4);
  e.setSampleRate(rate);
}

class Hall : public Engine {
  // the first member, so it outlives the reverb's buffers
  fv3_rt_pool pool;
  fv3::earlyref_f early;
  fv3::zrev2_f late;
  float early_out[2][MAX_BLOCK];
  float late_in[2][MAX_BLOCK];
  float late_out[2][MAX_BLOCK];
  float in[2][MAX_BLOCK];
  int current = 3;
  Shape knobs;
  double size = -1, rt60 = -1, high_cut = -1, mod = -1;
  float early_level = 0, late_level = 0, early_send = 0.2f;

  void apply(bool all)
  {
    const float * p = HALL[current].p;
    double s = f303::clamp<double>(p[SIZE] * size_factor(knobs.size), MIN_SIZE, MAX_SIZE);
    if(all || std::fabs(s - size) > 0.2)
      {
        size = s;
        early.setRSFactor(s / 10.0);
        late.setRSFactor(s / 80.0);
      }
    double r = f303::clamp<double>(p[DECAY] * decay_factor(knobs.decay), 0.1, 20.0);
    if(all || r != rt60) { rt60 = r; late.setrt60(r); }
    double hc = f303::clamp<double>(p[HIGH_CUT] * damping_factor(knobs.damping), 1000.0, 18000.0);
    if(all || hc != high_cut)
      {
        high_cut = hc;
        early.setoutputlpf(hc);
        late.setoutputlpf(hc);
        late.setxover_high(std::min((double)p[HIGH_XO], hc));
        late.setrt60_factor_high(f303::clamp<double>(p[HIGH_MULT] * damping_factor(knobs.damping), 0.2, 1.2));
      }
    double m = f303::clamp<double>(p[MODULATION] * modulation_factor(knobs.modulation), 0.1, 100.0) / 100.0;
    if(all || m != mod) { mod = m; late.setspinfactor(m); late.setlfofactor(m); }
    if(all)
      {
        early.setwidth(p[WIDTH] / 100.0);
        late.setwidth(p[WIDTH] / 100.0);
        late.setPreDelay(0.1);
        late.setidiffusion1(p[DIFFUSE] / 140.0);
        late.setapfeedback(p[DIFFUSE] / 140.0);
        early.setoutputhpf(p[LOW_CUT]);
        late.setoutputhpf(p[LOW_CUT]);
        late.setxover_low(p[LOW_XO]);
        late.setrt60_factor_low(p[LOW_MULT]);
        late.setspin(p[SPIN]);
        late.setwander(p[WANDER]);
        early_send = p[EARLY_SEND] / 100.0f;
        // the programs mix early and late 1:2; Front 303 keeps the ratio with the late at unity
        early_level = p[EARLY] / p[LATE];
        late_level = 1.0f;
      }
  }

public:
  explicit Hall(double rate)
  {
    fv3_rt_pool_scope scope(pool);
    early.loadPresetReflection(FV3_EARLYREF_PRESET_1);
    setup_early(early, rate);
    late.setMuteOnChange(false);
    late.setwet(0);
    late.setdryr(0);
    late.setwidth(1.0);
    late.setSampleRate(rate);
    // every program at the largest size once (the programs' wander sets the modulated
    // lines' lengths), so that later changes find their memory in the pool
    knobs.size = 1.0;
    for(int p = 0; p < PROGRAMS; p++) { current = p; apply(true); }
    knobs.size = 0.5;
    current = 3;
    apply(true);
    clear();
  }

  void program(int p) override
  {
    fv3_rt_pool_scope scope(pool);
    if(p != current) { current = p; apply(true); }
  }
  void shape(const Shape & s) override
  {
    fv3_rt_pool_scope scope(pool);
    if(s != knobs) { knobs = s; apply(false); }
  }
  void clear() override { early.mute(); late.mute(); }

  void process(const float * il, const float * ir, float * ol, float * orr, int n) override
  {
    fv3_rt_pool_scope scope(pool);
    for(int i = 0; i < n; i++) { in[0][i] = il[i]; in[1][i] = ir[i]; }
    early.processreplace(in[0], in[1], early_out[0], early_out[1], n);
    for(int i = 0; i < n; i++)
      {
        late_in[0][i] = early_send * early_out[0][i] + in[0][i];
        late_in[1][i] = early_send * early_out[1][i] + in[1][i];
      }
    late.processreplace(late_in[0], late_in[1], late_out[0], late_out[1], n);
    for(int i = 0; i < n; i++)
      {
        ol[i] = early_level * early_out[0][i] + late_level * late_out[0][i];
        orr[i] = early_level * early_out[1][i] + late_level * late_out[1][i];
      }
  }
};

// Dragonfly Early Reflections: the program's reflections, a size of 20 m, 100 % width,
// 50 Hz low cut and 10 kHz high cut, as the plug-in's defaults
class Early : public Engine {
  fv3_rt_pool pool;
  fv3::earlyref_f model;
  float in[2][MAX_BLOCK];
  int current = 2;
  Shape knobs;
  double size = -1, high_cut = -1;

  void apply(bool all)
  {
    double s = f303::clamp<double>(20.0 * size_factor(knobs.size), MIN_SIZE, MAX_SIZE);
    if(all || std::fabs(s - size) > 0.2) { size = s; model.setRSFactor(s / 10.0); }
    double hc = f303::clamp<double>(10000.0 * damping_factor(knobs.damping), 1000.0, 18000.0);
    if(all || hc != high_cut) { high_cut = hc; model.setoutputlpf(hc); }
  }

public:
  explicit Early(double rate)
  {
    fv3_rt_pool_scope scope(pool);
    setup_early(model, rate);
    model.setRSFactor(MAX_SIZE / 10.0);
    // every program's reflection table once, so later program changes reuse that memory
    for(int p = 0; p < PROGRAMS; p++) model.loadPresetReflection(EARLY_PROGRAMS[p].number);
    model.loadPresetReflection(EARLY_PROGRAMS[current].number);
    model.setwidth(1.0);
    model.setoutputhpf(50.0);
    apply(true);
    clear();
  }

  void program(int p) override
  {
    fv3_rt_pool_scope scope(pool);
    if(p == current) return;
    current = p;
    model.loadPresetReflection(EARLY_PROGRAMS[p].number);
    apply(true);
  }
  void shape(const Shape & s) override
  {
    fv3_rt_pool_scope scope(pool);
    if(s != knobs) { knobs = s; apply(false); }
  }
  void clear() override { model.mute(); }

  void process(const float * il, const float * ir, float * ol, float * orr, int n) override
  {
    fv3_rt_pool_scope scope(pool);
    for(int i = 0; i < n; i++) { in[0][i] = il[i]; in[1][i] = ir[i]; }
    model.processreplace(in[0], in[1], ol, orr, n);
  }
};

}

Engine * make_hall(double rate) { return new Hall(rate); }
Engine * make_early(double rate) { return new Early(rate); }
const char * hall_name(int p) { return HALL[p].name; }
const char * early_name(int p) { return EARLY_PROGRAMS[p].name; }

}
