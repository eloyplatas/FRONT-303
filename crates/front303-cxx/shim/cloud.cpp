// Front 303 — CloudReverb (xunil-cloud; GPL-3.0), Ghost Note Audio's CloudSeed engine. The
// programs are CloudReverb's presets; values are scaled the way the plug-in's parameters do.

#include "reverb.hpp"
#include "ReverbController.h"
#include "presets.h"

namespace f303 {
namespace {

enum class Law { Linear, Power, Int, Bool };
struct Range { Law law; double start, end, p; };

// CloudReverb's PluginProcessor.cpp, in Parameter order
const Range RANGES[(int)Parameter::Count] = {
  {Law::Linear, 0, 1, 0},          // InputMix
  {Law::Int, 0, 1000, 0},          // PreDelay
  {Law::Power, 20, 1000, 16},      // HighPass
  {Law::Power, 400, 20000, 16},    // LowPass
  {Law::Int, 1, 50, 0},            // TapCount
  {Law::Int, 0, 500, 0},           // TapLength
  {Law::Power, 0, 1, 100},         // TapGain
  {Law::Linear, 0, 1, 0},          // TapDecay
  {Law::Bool, 0, 1, 0},            // DiffusionEnabled
  {Law::Int, 1, 8, 0},             // DiffusionStages
  {Law::Int, 10, 100, 0},          // DiffusionDelay
  {Law::Linear, 0, 1, 0},          // DiffusionFeedback
  {Law::Int, 1, 12, 0},            // LineCount
  {Law::Power, 20, 1000, 100},     // LineDelay
  {Law::Power, 0.05, 60, 1000},    // LineDecay
  {Law::Bool, 0, 1, 0},            // LateDiffusionEnabled
  {Law::Int, 1, 8, 0},             // LateDiffusionStages
  {Law::Int, 10, 100, 0},          // LateDiffusionDelay
  {Law::Linear, 0, 1, 0},          // LateDiffusionFeedback
  {Law::Power, 0, 1, 100},         // PostLowShelfGain
  {Law::Power, 20, 1000, 16},      // PostLowShelfFrequency
  {Law::Power, 0, 1, 100},         // PostHighShelfGain
  {Law::Power, 400, 20000, 16},    // PostHighShelfFrequency
  {Law::Power, 400, 20000, 16},    // PostCutoffFrequency
  {Law::Linear, 0, 2.5, 0},        // EarlyDiffusionModAmount
  {Law::Power, 0, 5, 100},         // EarlyDiffusionModRate
  {Law::Linear, 0, 2.5, 0},        // LineModAmount
  {Law::Power, 0, 5, 100},         // LineModRate
  {Law::Linear, 0, 2.5, 0},        // LateDiffusionModAmount
  {Law::Power, 0, 5, 100},         // LateDiffusionModRate
  {Law::Int, 1, 1000000, 0},       // TapSeed
  {Law::Int, 1, 1000000, 0},       // DiffusionSeed
  {Law::Int, 1, 1000000, 0},       // DelaySeed
  {Law::Int, 1, 1000000, 0},       // PostDiffusionSeed
  {Law::Linear, 0, 1, 0},          // CrossSeed
  {Law::Power, 0, 1, 10},          // DryOut
  {Law::Power, 0, 1, 10},          // PredelayOut
  {Law::Power, 0, 1, 10},          // EarlyOut
  {Law::Power, 0, 1, 10},          // MainOut
  {Law::Bool, 0, 1, 0},            // HiPassEnabled
  {Law::Bool, 0, 1, 0},            // LowPassEnabled
  {Law::Bool, 0, 1, 0},            // LowShelfEnabled
  {Law::Bool, 0, 1, 0},            // HighShelfEnabled
  {Law::Bool, 0, 1, 0},            // CutoffEnabled
  {Law::Linear, 0, 1, 0},          // LateStageTap
  {Law::Bool, 0, 1, 0},            // Interpolation
};

double scale(int i, double v)
{
  const Range & r = RANGES[i];
  switch(r.law)
    {
    case Law::Linear: return r.start + v * (r.end - r.start);
    case Law::Power: return r.start + (std::pow(r.p, v) - 1) / (r.p - 1) * (r.end - r.start);
    case Law::Int: return std::round(r.start + v * (r.end - r.start));
    default: return v >= 0.5 ? 1.0 : 0.0;
    }
}

// CloudReverb's presets but the chorus-delay one, which is not a reverb
const cloudPresets::preset * const PRESETS[PROGRAMS] = {
  &cloudPresets::DullEchoes, &cloudPresets::Hyperplane, &cloudPresets::MediumSpace,
  &cloudPresets::NoiseInTheHallway, &cloudPresets::RubiKaFields, &cloudPresets::SmallRoom,
  &cloudPresets::The90sAreBack, &cloudPresets::ThroughTheLookingGlass,
};
const char * const NAMES[PROGRAMS] = {
  "Dull Echoes", "Hyperplane", "Medium Space", "Noise in the Hallway", "Rubi-Ka Fields",
  "Small Room", "The 90s Are Back", "Through the Looking Glass",
};

class Cloud : public Engine {
  CloudSeed::ReverbController reverb;
  double scaled[(int)Parameter::Count];
  double sent[(int)Parameter::Count];
  int current = 2;
  Shape knobs;

  void put(Parameter p, double value)
  {
    int i = (int)p;
    scaled[i] = value;
  }

  void apply()
  {
    const double * n = reinterpret_cast<const double*>(PRESETS[current]);
    for(int i = 0; i < (int)Parameter::Count; i++) scaled[i] = scale(i, n[i]);
    const double sf = size_factor(knobs.size), df = decay_factor(knobs.decay);
    const double mf = modulation_factor(knobs.modulation), kf = damping_factor(knobs.damping);
    auto get = [&](Parameter p) { return scaled[(int)p]; };
    put(Parameter::DryOut, 0.0);
    put(Parameter::PredelayOut, 0.0);
    put(Parameter::LineDelay, f303::clamp<double>(get(Parameter::LineDelay) * sf, 20.0, 1000.0));
    put(Parameter::LateDiffusionDelay, f303::clamp<double>(std::round(get(Parameter::LateDiffusionDelay) * sf), 10.0, 100.0));
    put(Parameter::TapLength, f303::clamp<double>(std::round(get(Parameter::TapLength) * sf), 0.0, 500.0));
    put(Parameter::LineDecay, f303::clamp<double>(get(Parameter::LineDecay) * df, 0.05, 60.0));
    double cutoff = get(Parameter::CutoffEnabled) > 0.5 ? get(Parameter::PostCutoffFrequency) : 20000.0;
    put(Parameter::PostCutoffFrequency, f303::clamp<double>(cutoff * kf, 400.0, 20000.0));
    put(Parameter::CutoffEnabled, 1.0);
    put(Parameter::EarlyDiffusionModAmount, f303::clamp<double>(get(Parameter::EarlyDiffusionModAmount) * mf, 0.0, 2.5));
    put(Parameter::LineModAmount, f303::clamp<double>(get(Parameter::LineModAmount) * mf, 0.0, 2.5));
    put(Parameter::LateDiffusionModAmount, f303::clamp<double>(get(Parameter::LateDiffusionModAmount) * mf, 0.0, 2.5));
    for(int i = 0; i < (int)Parameter::Count; i++)
      if(scaled[i] != sent[i])
        {
          sent[i] = scaled[i];
          reverb.updateParameter((Parameter)i, scaled[i], (float)n[i]);
        }
  }

public:
  explicit Cloud(double rate) : reverb((int)rate)
  {
    reverb.prepare((int)rate, MAX_BLOCK);
    for(int i = 0; i < (int)Parameter::Count; i++) sent[i] = -1e300;
    apply();
    reverb.ClearBuffers();
  }

  void program(int p) override { if(p != current) { current = p; apply(); } }
  void shape(const Shape & s) override { if(s != knobs) { knobs = s; apply(); } }
  void clear() override { reverb.ClearBuffers(); }

  void process(const float * il, const float * ir, float * ol, float * orr, int n) override
  {
    const float * in[2] = { il, ir };
    float * out[2] = { ol, orr };
    reverb.Process(in, out, n);
  }
};

}

Engine * make_cloud(double rate) { return new Cloud(rate); }
const char * cloud_name(int p) { return NAMES[p]; }

}
