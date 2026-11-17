// Front 303 — C interface to Open303 (MIT, Robin Schmidt / maddanio).
// GPL-3.0-only as part of Front 303.
//
// Front 303 plays the voice from its own sequencer, so the class's note list (a std::list that
// allocates) and its sequencer are bypassed: steps go straight to the protected triggerNote,
// slideToNote and releaseNote.

#include "rosic_Open303.h"

namespace {
class Voice : public rosic::Open303 {
public:
  void trigger(int note, bool accent) { currentNote = note; triggerNote(note, accent); }
  void slide(int note, bool accent) { currentNote = note; slideToNote(note, accent); }
  void release() { releaseNote(currentNote); }
};
}

extern "C" {

void * f303_o3_new(double sample_rate)
{
  Voice * v = new Voice();
  v->setSampleRate(sample_rate);
  v->setVolume(0.0);
  return v;
}

void f303_o3_free(void * v) { delete static_cast<Voice*>(v); }

// The knob laws of Open303's own plug-in: cutoff 314–2394 Hz and decay 200–2000 ms on
// exponential tapers, the others 0–100 %.
void f303_o3_panel(void * p, double cutoff_hz, double resonance, double env_mod, double decay_ms,
                   double accent, double waveform, double tuning_a4_hz)
{
  Voice * v = static_cast<Voice*>(p);
  if(v->getCutoff() != cutoff_hz) v->setCutoff(cutoff_hz);
  if(v->getResonance() != resonance) v->setResonance(resonance);
  if(v->getEnvMod() != env_mod) v->setEnvMod(env_mod);
  if(v->getDecay() != decay_ms) v->setDecay(decay_ms);
  if(v->getAccent() != accent) v->setAccent(accent);
  if(v->getWaveform() != waveform) v->setWaveform(waveform);
  v->setTuning(tuning_a4_hz);
}

void f303_o3_trigger(void * v, int note, int accent) { static_cast<Voice*>(v)->trigger(note, accent != 0); }
void f303_o3_slide(void * v, int note, int accent) { static_cast<Voice*>(v)->slide(note, accent != 0); }
void f303_o3_release(void * v) { static_cast<Voice*>(v)->release(); }
void f303_o3_silence(void * v) { static_cast<Voice*>(v)->allNotesOff(); }

void f303_o3_render(void * p, float * out, int n)
{
  Voice * v = static_cast<Voice*>(p);
  for(int i = 0; i < n; i++) out[i] = (float)v->getSample();
}

}
