# The manual's prose, in English and Spanish.
B = None  # the build module, set by setup()


def setup(build_module):
    global B
    B = build_module


TEXT = {}

# =============================================================================================
# ENGLISH
# =============================================================================================

EN_VOICE = {
    "model": "Sound engine of the voice: REBIRTH (ReBirth RB-338 2.0 sound, OpenRB-338), VINTAGE (ReBirth's Vintage engine), DW-303 (component-level circuit model) or OPEN303 (Robin Schmidt's Open303). Switching resets the voice the next time the model is chosen.",
    "wave": "Oscillator waveform: sawtooth or square.",
    "tune": "Coarse tuning in semitones, ±2 octaves.",
    "fine": "Fine tuning in cents, ±100.",
    "cutoff": "Filter cut-off. ReBirth and DW-303 use their own curves; Open303 maps the knob to 314–2394 Hz on an exponential taper.",
    "reso": "Filter resonance (Open303: 0–100 %).",
    "envmod": "Depth of the filter envelope (Open303: 0–100 %).",
    "decay": "Filter envelope decay (Open303: 200–2000 ms, exponential).",
    "accent": "How strongly accented steps hit (level and filter).",
    "pattern": "The pattern (1–8) the voice plays and the panel edits. With the transport running, a change waits for the next bar.",
    "on": "Channel on/off. Only 303-1 is on when a new instance is loaded.",
    "level": "Channel level (ReBirth's level law).",
    "pan": "Channel pan (64 = centre).",
    "send": "Send to the delay, after the level (DLY).",
    "dist": "Inserts the distortion on this channel (DST).",
    "density": "Generator: how many steps sound.",
    "accents": "Generator: how often steps are accented (more often off the beat).",
    "slides": "Generator: how often steps slide into the next one.",
    "octaves": "Generator: how often notes jump an octave.",
    "evolve": "Rewrites this share of the playing pattern every time it comes round. 0 = the pattern never changes by itself.",
    "rate": "Step length of this voice's sequencer, from 1/32 to 1/4, with triplets and dotted values. All voices share the tempo.",
    "miditr": "When on, incoming MIDI notes transpose this voice's sequence.",
    "reverb": "Send to the reverb, after level and pan (REV).",
}

EN_GLOBAL = {
    "run": "Plays the sequencers when the host transport is stopped.",
    "tempo": "Tempo used when the host does not provide one (BPM knob).",
    "shuffle": "Delays every second step by up to half a step.",
    "root": "Root note for scale quantisation and the generator.",
    "scale": "Scale: chromatic (no quantisation) and 13 others. Out-of-scale notes sound as the nearest scale note (downwards on a tie); stored patterns are not changed.",
    "master": "Master level (ReBirth's master law).",
    "quality": "DW-303 filter solver: FULL or ECONOMY (slightly cheaper).",
    "output": "STEREO: everything on the main output. MULTI-OUT: each voice and the effect returns on their own outputs; the main output and the master section are silent.",
    "dist_mode": "REBIRTH: ReBirth's distortion. SETEKH: Full FX Media's Setekh.",
    "dist_amount": "REBIRTH: amount. SETEKH: DRIVE (0–5), the blend between the clean signal and tanh(3.5·x).",
    "dist_shape": "REBIRTH: 0 = hard clip, above = waveshaper curves. SETEKH: INPUT gain, 0 to +24 dB.",
    "dist_level": "SETEKH only: OUTPUT gain, −24 to +24 dB (64 = 0 dB).",
    "pcf_channel": "Channel the pattern-controlled filter is inserted on (or off).",
    "pcf_mode": "Low-pass or band-pass.",
    "pcf_pattern": "One of 55 rhythmic modulation patterns, synced to the tempo.",
    "pcf_freq": "Filter frequency.",
    "pcf_reso": "Filter resonance.",
    "pcf_amount": "Depth of the pattern's modulation.",
    "pcf_decay": "Decay of each modulation pulse.",
    "comp_mode": "REBIRTH: ReBirth's compressor. VALENTINE: Tote Bag Labs' Valentine.",
    "comp_route": "Where the compressor sits: off, the master, or one channel.",
    "comp_amount": "REBIRTH: amount. VALENTINE: COMPRESS (input gain), −6 to +36 dB.",
    "comp_threshold": "REBIRTH: threshold. VALENTINE: OUTPUT (make-up), −24 to +12 dB.",
    "comp_ratio": "VALENTINE: ratio 1–21; knee and threshold follow the ratio as in the original.",
    "comp_attack": "VALENTINE: attack 0.02–55 ms (exponential; default 1.3 ms).",
    "comp_release": "VALENTINE: release 2–1100 ms (exponential; default 350 ms).",
    "comp_saturate": "VALENTINE: 0 = off; 1–10 = asymmetric inverse-hyperbolic-sine saturation.",
    "comp_crush": "VALENTINE: 0 = off; 1–10 = sample-and-hold towards 27.5 kHz and bit reduction from 12 to 1 bits.",
    "comp_mix": "VALENTINE: dry/wet, 0–100 %.",
    "delay_on": "Delay on/off.",
    "delay_mode": "REBIRTH, TAPE (IV-X Delay) or INTERVALS.",
    "delay_steps": "Delay time (or interval) in steps of a sixteenth, or of a triplet eighth with TRIPLET.",
    "delay_triplet": "Steps become triplet eighths.",
    "delay_feedback": "REBIRTH / TAPE: feedback (TAPE up to self-oscillation). INTERVALS: LOOP, how much of the last tap returns to the line's input.",
    "delay_pan": "Position of the echoes (TAPE/INTERVALS: equal-power balance).",
    "delay_a": "TAPE: WOW depth. INTERVALS: FALLOFF, level lost at each tap (0–60 %).",
    "delay_b": "TAPE: FLUTTER depth. INTERVALS: TONE (left darker, centre open, right thinner).",
    "delay_c": "TAPE: NOISE (tape hiss and scrape). INTERVALS: SPREAD, from mono to the pattern's own left/right amplitudes.",
    "delay_kind": "TAPE: tape speed and bias (1–4: 15 IPS, 5–8: 7.5 IPS, 9–12: 3.75 IPS; bias 2–5). INTERVALS: the pattern (1–8).",
    "rev_on": "Reverb on/off.",
    "rev_mode": "Engine: HALL (Dragonfly Hall), EARLY (Dragonfly Early Reflections), AETHER, CLOUD (CloudReverb).",
    "rev_program": "One of the engine's eight programs.",
    "rev_predelay": "Pre-delay before the engine, 0–250 ms.",
    "rev_size": "Scales the program's size: ×0.5 … ×2 (centre = as the program).",
    "rev_decay": "Scales the decay time: ×0.35 … ×2.8 (centre = as the program). No effect in EARLY.",
    "rev_damping": "High-frequency damping: the cut-off moves ×2.8 … ×0.35 (centre = as the program).",
    "rev_mod": "Modulation depth: none at 0, the program's at the centre, ×4 at the top. No effect in EARLY.",
    "rev_width": "Stereo width of the return, mono to 150 %.",
    "rev_delay": "Sends the delay's return into the reverb.",
    "rev_level": "Reverb return level.",
}


def en_body(flow):
    f, t = B.fig, B.table
    return f"""
<div class="cover">
  <div style="font-size:11pt;color:#ff8a1c;letter-spacing:2pt;font-weight:700">USER MANUAL · v{B.VERSION}</div>
  <div class="logo">FRONT <b>303</b></div>
  <div class="sub">Four TB-303 bass lines, generative sequencers and ReBirth-style effects — VST3 &amp; CLAP for Linux, Windows and macOS</div>
  <img src="img/panel.png">
  <div class="meta">Eloy Platas · November 2026 · GPL-3.0 · github.com/eloyplatas/FRONT-303<br>
  Built with OpenRB-338, DW-303, Open303, Dragonfly Reverb, Aether, CloudReverb, IV-X Delay, Setekh, Valentine and Delay Intervals.</div>
</div>

<h1 class="brk">Contents</h1>
<div class="toc">
<div>1. What Front 303 is</div><div>2. History of the plugin</div><div>3. Installation</div>
<div>4. Overview of the panel and signal flow</div><div>5. Transport, tempo and host sync</div>
<div>6. The four 303 voices</div><div>7. The sequencers</div><div>8. The pattern generator</div>
<div>9. Scale quantisation</div><div>10. MIDI transposition</div><div>11. The mixer</div>
<div>12. Distortion</div><div>13. PCF (pattern-controlled filter)</div><div>14. Compressor</div>
<div>15. Delay</div><div>16. Reverb</div><div>17. Outputs and multi-out recording</div>
<div>18. Memories</div><div>19. Undo, redo, copy and paste</div><div>20. Using the panel</div>
<div>21. Parameter reference</div><div>22. Technical notes and limits</div><div>23. Credits and licences</div>
</div>

<h1 class="brk">1. What Front 303 is</h1>
<p>Front 303 is a software instrument with <b>four TB-303 bass-line synthesisers</b>, each with its own 16-step sequencer and pattern generator, running through the mixer and effects of <b>ReBirth RB-338</b>, extended with new effect modes and a four-engine reverb. It is a VST3 and CLAP plugin for Linux, Windows and macOS, written in Rust, with a vector-drawn panel in the manner of ReBirth 2.0.</p>
<p>It is inspired by ReBirth but leaves the drum machines out: in their place there are four 303s, and each one can use a different sound engine:</p>
{t(["Model", "What it is", "Origin"], [
 ["<b>REBIRTH</b>", "The 303 of ReBirth RB-338 2.0, block-accurate", "OpenRB-338 (reverse-engineered reimplementation of ReBirth 2.01)"],
 ["<b>VINTAGE</b>", "ReBirth's “Vintage” engine: control every 4 samples, stepped resonance", "OpenRB-338"],
 ["<b>DW-303</b>", "A component-level model of the TB-303 circuit (oscillator, ladder filter, envelopes, accent, VCA)", "DW-303 by DJW Audio"],
 ["<b>OPEN303</b>", "Robin Schmidt's Open303 DSP", "open303 (maddanio's fork)"]], ["k", "", ""])}
<p>Main features:</p>
<ul>
<li>Four voices, model per voice, tune in semitones (±24) and fine tune in cents (±100).</li>
<li>Eight patterns per voice, editable on a 16-step editor with note, accent, slide and rest; pattern length 1–16 (polymeter).</li>
<li>A different <b>step rate</b> per sequencer under one tempo (1/32 to 1/4, triplets and dotted values): polyrhythms.</li>
<li>Random and generative patterns in the 303 style, with mutation and continuous evolution, quantised to a root and one of 14 scales.</li>
<li><b>MIDI transposition</b> of the sequences, selectable per voice.</li>
<li>ReBirth's distortion, PCF, compressor and delay, plus <b>Setekh</b> distortion, <b>Valentine</b> compressor, <b>IV-X tape</b> and <b>interval</b> delays, and a reverb with <b>Dragonfly Hall</b>, <b>Dragonfly Early Reflections</b>, <b>Aether</b> and <b>CloudReverb</b> engines.</li>
<li><b>Separate outputs</b>: one stereo pair per 303 and one for the effect returns, to record each voice on its own track.</li>
<li><b>Ten memories</b> of the whole instrument, <b>undo/redo</b>, and <b>copy/paste</b> of patterns between voices.</li>
<li>142 automatable parameters; patterns and memories are saved with the project.</li>
</ul>

<h1 class="brk">2. History of the plugin</h1>
<p>Front 303 was born in November 2026 from a study of two open-source projects. The first was <b>DW-303</b> (mo0kid), a TB-303 modelled stage by stage from the circuit: a pitch CV and expo converter, the sawtooth/square VCO, the four-pole transistor ladder VCF with its real component values, the MEG and VEG envelopes, the accent circuit with its sweep, and the VCA. The second was <b>OpenRB-338</b> (pingw33n), a reimplementation of Propellerhead's ReBirth RB-338 2.01 obtained by reverse engineering, faithful down to its tables, its block-based processing and its effects — but only its 303 voices and effects were of interest, not the drum machines.</p>
<p>After comparing both — the circuit model sounds closer to a real TB-303, ReBirth sounds like the ReBirth of so many records — the idea was clear: <i>something like OpenRB-338, without drum machines, with four 303s instead, each able to choose its model, with ReBirth's effects and routing, sequencers with random and generative functions that write 303-style patterns quantised to a scale, pitch controls in semitones (±24) with a fine control, and elaborate graphics in the style of ReBirth 2.0</i>. The project was named <b>Front 303</b>, licensed GPL-3.0 and open source for personal use, with Windows and macOS versions planned from the start. Rust and the nice-plug framework were chosen over JUCE because both source projects are Rust crates and because one code base builds for the three systems.</p>
<p>The work went in phases: the engine (four voices, model per voice, tuning, mixer and effects), the sequencers and generator, the VST3/CLAP plugin with host sync and automation, and finally the panel, drawn entirely in code. <b>Version 0.2.0</b> delivered all of that for Linux, checked with Steinberg's VST3 validator.</p>
<p>Next came a study of <b>Open303</b> (maddanio's fork of Robin Schmidt's synth): measured against a TB-303 it has less resonance peak and flatter accents, but it is clean, light and in tune, so it was added as a fourth model. <b>Version 0.3.0</b> also added MIDI transposition of the sequences, step rates per sequencer, separate outputs for every 303 and for the effects, and new modes for the effects ported or compiled from other open projects: Setekh (distortion), Valentine (compressor), IV-X Delay (tape echo), Delay Intervals (interval delay) and four reverbs (Dragonfly Hall and Early Reflections, Aether, CloudReverb). <b>Version 0.4.0</b> brought ten memories in the manner of the author's GenderBender P23, undo/redo and copying patterns between voices, and the Windows and macOS builds, compiled by GitHub Actions.</p>
{t(["Version", "Contents"], [
 ["0.1.0", "Engine: four voices with ReBirth / DW-303 models, tuning, ReBirth mixer and effects; sequencers and generator."],
 ["0.2.0", "VST3/CLAP plugin for Linux, host sync, 104 parameters, ReBirth 2.0-style panel."],
 ["0.3.0", "Open303 model, step rate per voice, MIDI transposition, multi-out, Setekh, Valentine, tape and interval delays, reverb with four engines (142 parameters)."],
 ["0.4.0", "Ten memories, undo/redo, copy/paste of patterns; Windows and macOS builds."]], ["k", ""])}

<h1>3. Installation</h1>
<p>The archives are in the <code>releases</code> folder of the repository <code>github.com/eloyplatas/FRONT-303</code>. Each one contains the VST3 and the CLAP plugin, this manual's Spanish source, the licences and an installation note.</p>
<h3>Linux (x86_64, glibc 2.30 or later)</h3>
<p>Unpack <code>Front303-{B.VERSION}-linux-x86_64.tar.gz</code> and run <code>./install.sh</code>: it copies <code>Front303.vst3</code> to <code>~/.vst3</code> and <code>Front303.clap</code> to <code>~/.clap</code> (<code>./install.sh --uninstall</code> removes them). Needs X11 or XWayland and OpenGL for the panel.</p>
<h3>Windows (64-bit)</h3>
<p>Unzip <code>Front303-{B.VERSION}-windows-x86_64.zip</code>. Copy the <code>Front303.vst3</code> folder to <code>C:\\Program Files\\Common Files\\VST3\\</code> and, optionally, <code>Front303.clap</code> to <code>C:\\Program Files\\Common Files\\CLAP\\</code>. The plugin needs no extra runtime. It is not code-signed, so SmartScreen may warn the first time.</p>
<h3>macOS (10.15 or later, Apple silicon and Intel)</h3>
<p>Unzip <code>Front303-{B.VERSION}-macos-universal.zip</code> and double-click <code>install.command</code> (right-click → Open if macOS refuses). It installs into <code>~/Library/Audio/Plug-Ins/VST3</code> and <code>…/CLAP</code> and removes the quarantine flag. The plugin is ad-hoc signed and not notarised: without removing the quarantine (<code>xattr -dr com.apple.quarantine</code>), macOS reports it as damaged.</p>
<p>Then rescan plugins in the host (in REAPER: <i>Preferences → Plug-ins → VST → Re-scan</i>). Front 303 appears as an instrument.</p>

<h1 class="brk">4. Overview of the panel and signal flow</h1>
{f("panel.png", "The whole panel (1280 × 1058 design pixels, resizable keeping its proportions).")}
<p>From top to bottom: the <b>header</b> (transport, tempo, scale, master, MIDI transposition and output mode); a row with <b>distortion, PCF, compressor and delay</b>; a row with the <b>reverb</b> and the <b>mixer</b>; the four <b>303 units</b>; and the <b>memory bar</b> with undo/redo.</p>
{flow}
<p>Each sequencer plays its voice. Each voice goes through its channel strip (distortion if DST is on, PCF if it is assigned to the channel, compressor if it is routed to the channel, then level and pan). The channels feed the delay (DLY sends) and the reverb (REV sends); the delay can also feed the reverb (DLY → REV). In STEREO mode the channels and the effect returns are summed, go through the master compressor if routed there and the MASTER level, to the main output. In MULTI-OUT mode each channel and the returns leave on their own outputs.</p>

<h1>5. Transport, tempo and host sync</h1>
{f("header.png", "The header.")}
<ul>
<li><b>With the host playing</b>, Front 303 follows it: tempo, position, loops and jumps. Patterns stay aligned to the project's bars (step 1 falls on each bar). The <b>HOST</b> LED lights.</li>
<li><b>With the host stopped</b>, <b>RUN</b> plays the sequencers on their own; the <b>PLAY</b> LED blinks with the beat.</li>
<li>The <b>TEMPO</b> display shows the tempo in use: the host's when it provides one, otherwise the <b>BPM</b> knob's (20–300).</li>
<li><b>SHUFFLE</b> delays every second step by up to half a step.</li>
<li><b>ROOT</b> and <b>SCALE</b> quantise every voice (see section 9).</li>
<li><b>DW-303 SOLVER</b> selects FULL or ECONOMY for the DW-303 filter.</li>
<li><b>MASTER</b> sets the master level, with the stereo meters beside it.</li>
<li><b>MIDI TR</b> shows the current MIDI transposition; <b>OUTPUT</b> selects STEREO or MULTI-OUT.</li>
</ul>

<h1>6. The four 303 voices</h1>
{f("unit.png", "A 303 unit (here with the OPEN303 model).")}
<p>Each unit has a name plate with a LED that follows its level, the <b>model selector</b>, <b>SAW / SQUARE</b>, and seven knobs: <b>TUNE</b> (semitones, display below), <b>FINE</b> (cents, display below), <b>CUT OFF FREQ</b>, <b>RESONANCE</b>, <b>ENV MOD</b>, <b>DECAY</b> and <b>ACCENT</b>.</p>
<h3>The models</h3>
<ul>
<li><b>REBIRTH</b>: the 2.0 sound. The device computes its controls once per 64-sample block, so notes start on block edges (1.45 ms), as in the original. Sustains while the gate is open.</li>
<li><b>VINTAGE</b>: ReBirth's older engine, with control every four samples and stepped resonance.</li>
<li><b>DW-303</b>: the circuit model, run at 4× oversampling (176.4 kHz) with a two-stage half-band decimator. Notes start at their exact sample. Like the real circuit, its VCA decays on very long notes. It costs much more CPU than the other models (around 20 % of a 2.1 GHz core per voice).</li>
<li><b>OPEN303</b>: Robin Schmidt's Open303, with its plug-in's knob laws (cut-off 314–2394 Hz, decay 200–2000 ms). Measured against a TB-303 it has less resonance peak (6–21 dB against 19–43 dB) and accents that do not accumulate; it is included as it is, with two out-of-range memory accesses of the original corrected.</li>
</ul>
<p>Tuning and loudness are matched across the models (A2 = 110 Hz on all of them, riff levels within 1 dB), but each keeps its own knob curves, so the same knob position does not sound identical on every model. Changing the model while playing is clean: the voice left behind is reset.</p>

<h1>7. The sequencers</h1>
<p>Below each unit is its sequencer.</p>
<ul>
<li><b>Steps</b>: click a step to toggle note / rest; drag up and down or turn the wheel to change the note (with a scale chosen, the note moves by scale degrees). The note name is shown (C2 is the 303's reference C); rests show their note dimmed.</li>
<li><b>AC</b>: accent. <b>SL</b>: slide — the step ties into the next one with the 303's glide.</li>
<li><b>LENGTH − / +</b>: pattern length, 1–16 steps. Steps beyond the length are darkened. Different lengths on different voices give polymeter.</li>
<li><b>Step rate</b> (display below LENGTH): 1/32, 1/16T, 1/16, 1/8T, 1/16., 1/8, 1/4T, 1/8. or 1/4. All voices share the tempo; with different rates they cross in polyrhythms (for instance 1/16 against 1/8T, six steps per bar). Rates are counted from the start of the song, so with the host they always land in the same place of the bar.</li>
<li><b>MIDI</b>: whether this voice follows MIDI transposition (section 10).</li>
<li><b>PATTERN 1–8</b>: the pattern the voice plays and the panel edits. With the transport running, a change waits for the next bar line; the red dot marks the pattern being played, the red LED above the steps the step being played.</li>
<li><b>Pattern tools</b>: <b>← →</b> rotate one step; <b>CLR</b> clears; <b>−1 / +1</b> transpose a semitone; <b>OCT</b> raises an octave (notes out of range fold back); <b>COPY / PASTE</b> see section 19.</li>
</ul>

<h1>8. The pattern generator</h1>
<p>The <b>GENERATOR</b> section writes 303-style lines into the selected pattern, in the panel's root and scale.</p>
<ul>
<li><b>GENERATE</b> writes a new pattern. It first writes a motif half the pattern's length and then repeats it with variations, which gives the line its riff character.</li>
<li><b>DENSITY</b>: how many steps sound. <b>ACCENTS</b>: how often steps are accented, more likely off the beat. <b>SLIDES</b>: how often a step slides into the next. <b>OCTAVES</b>: how often notes jump an octave.</li>
<li><b>MUTATE</b> rewrites a quarter of the steps.</li>
<li><b>EVOLVE</b> mutates the playing pattern each time it comes round, by the knob's share. At zero the pattern never changes by itself; “EVOLVING” is shown while it does. The panel shows the changes as they happen.</li>
</ul>

<h1>9. Scale quantisation</h1>
<p><b>ROOT</b> and <b>SCALE</b> quantise what the four voices play and guide the generator. The 14 scales: chromatic (no quantisation), major, minor, dorian, phrygian, lydian, mixolydian, locrian, harmonic minor, phrygian dominant, minor pentatonic, major pentatonic, blues and whole tone. A note outside the scale sounds as the nearest scale note (downwards on a tie). The stored pattern is not changed: switching back to chromatic plays it as written.</p>

<h1>10. MIDI transposition</h1>
<p>Front 303 accepts MIDI notes on its input. Each note-on transposes the sequences of the voices whose <b>MIDI</b> switch is on: <b>C3 (note 60)</b> leaves them as they are, C#3 raises them a semitone, B2 lowers them one, up to ±24 semitones. The transposition stays until the next note (the key need not be held) and starts at the exact sample where the note arrives. Note-offs are ignored. The <b>MIDI TR</b> display shows the current value. It is applied after scale quantisation: a literal transposition, as on a TB-303.</p>
<p>In REAPER, arm the track and choose a MIDI input, or write the notes in a MIDI item on the same track.</p>

<h1>11. The mixer</h1>
{f("mixer.png", "The mixer (here in MULTI-OUT mode, showing each channel's output pair).", "half")}
<p>Per channel: <b>LEVEL</b> with its meter, <b>PAN</b>, <b>DLY</b> (delay send, after the level), <b>REV</b> (reverb send, after level and pan), <b>ON</b> and <b>DST</b> (distortion insert). Gains are smoothed as in ReBirth, so moves never click.</p>

<h1>12. Distortion</h1>
{f("fx_rebirth.png", "The effects row in its ReBirth modes.")}
{f("fx_new.png", "The same row with SETEKH, VALENTINE and INTERVALS selected.")}
<p>The distortion acts on the channels whose <b>DST</b> is on. Two modes:</p>
<ul>
<li><b>REBIRTH</b>: ReBirth's distortion, one unit shared by the channels. <b>AMOUNT</b>; <b>SHAPE</b> at 0 is a hard clip, above it waveshaper curves. It leaves some DC on asymmetric waveforms, as ReBirth's does.</li>
<li><b>SETEKH</b>: the Setekh distortion (Full FX Media): a +6 dB high shelf at 7.5 kHz, the input gain, a blend of the clean signal and tanh(3.5·x) set by <b>DRIVE</b>, the output gain, and the shelf taken back out. <b>INPUT</b> 0 to +24 dB, <b>OUTPUT</b> −24 to +24 dB. Each channel has its own unit.</li>
</ul>

<h1>13. PCF (pattern-controlled filter)</h1>
<p>ReBirth's PCF is inserted on one channel (<b>CHANNEL</b>) and moves its filter rhythmically with one of <b>55 patterns</b> synced to the tempo. <b>MODE</b> low-pass or band-pass; <b>FREQ</b>, <b>RESO</b>, <b>AMOUNT</b> (modulation depth) and <b>DECAY</b> (length of each pulse).</p>

<h1>14. Compressor</h1>
<p>Placed with <b>ON</b> on the master or on one channel. Two modes:</p>
<ul>
<li><b>REBIRTH</b>: ReBirth's compressor, <b>AMOUNT</b> and <b>THRESHOLD</b>.</li>
<li><b>VALENTINE</b>: Tote Bag Labs' Valentine: an input gain (<b>COMPRESS</b>, −6 to +36 dB) into a feed-forward peak compressor whose knee and threshold follow the <b>RATIO</b> (1–21), then an asymmetric inverse-hyperbolic-sine saturator (<b>SATURATE</b> 1–10, 0 = off) and a soft clip, all at 4× the sample rate with antiderivative anti-aliasing; optional <b>CRUSH</b> before it (a sample-and-hold towards 27.5 kHz and a bit crusher from 12 down to 1 bits); <b>ATTACK</b> 0.02–55 ms, <b>RELEASE</b> 2–1100 ms, <b>MIX</b> and <b>OUTPUT</b> (−24 to +12 dB) after. On the master it is stereo-linked.</li>
</ul>

<h1>15. Delay</h1>
<p>Fed by the channels' <b>DLY</b> sends. The time is set in steps (1–32) of a sixteenth, or of a triplet eighth with <b>TRIPLET</b>, synced to the tempo. <b>ON</b> switches it. Three modes:</p>
<h3>REBIRTH</h3>
<p>ReBirth's delay with <b>FEEDBACK</b> and <b>PAN</b>.</p>
<h3>TAPE (IV-X Delay)</h3>
<p>The tape echo of IV-X Delay (Joaquim Broquetas Ruiz), ported to Rust. Every sample the transport creeps towards the set time (changing the time bends the pitch, like a tape machine changing speed; when the mode is switched on the tape “spins up” in under a second); tape noise and the feedback join the input; the record head is IV-X's measured transfer curve and FIR response for the chosen speed and bias; the playback head reads with interpolation at a position swayed by wow, flutter and scrape.</p>
{t(["Control", "Function"], [
 ["FEEDBACK", "0–1.5 in IV-X's units; the top reaches self-oscillation, limited by the tape curve."],
 ["PAN", "Position of the (mono) echo."],
 ["WOW", "Slow pitch sway (1 Hz)."],
 ["FLUTTER", "Fast pitch sway (15 Hz)."],
 ["NOISE", "Tape hiss and head scrape."],
 ["TAPE", "1–4: 15 IPS, 5–8: 7.5 IPS, 9–12: 3.75 IPS, each with bias 2, 3, 4, 5. Slower tape: darker, more saturated, noisier."]], ["k", ""])}
<h3>INTERVALS</h3>
<p>Repeats placed on a pattern of intervals, after Matthew Fecher's Delay Intervals. The input runs down one line; at every multiple of the interval it passes a tap, where the falloff and that tap's filters are applied to the line itself, so every repeat is darker and quieter than the one before. Each tap is heard at its own left and right amplitude, taken from the chosen pattern.</p>
{t(["Pattern", "Taps", "Character"], [
 ["STRAIGHT", "8", "Every interval, both sides."],
 ["PING PONG", "12", "Alternating left and right."],
 ["DOTTED", "13", "Every third interval."],
 ["TRESILLO", "16", "3-3-4-2-4 Afro-Cuban grouping, uneven stereo."],
 ["SWELL", "12", "Repeats that grow instead of fading."],
 ["GALLOP", "12", "Two on, one off."],
 ["SCATTER", "16", "Irregular, spread across the stereo field."],
 ["STUTTER", "16", "Four repeats, a gap, four more."]], ["k", "", ""])}
<p><b>FALLOFF</b> (0–60 % per tap), <b>TONE</b> (left: a low-pass closing to 1 kHz; right: a high-pass opening to 1.5 kHz), <b>SPREAD</b> (mono to the pattern's own stereo), <b>LOOP</b> (the FEEDBACK knob: how much of the last tap returns to the line's input) and <b>PAN</b> (balance). The interval goes up to 1 s.</p>

<h1>16. Reverb</h1>
{f("reverb.png", "The reverb module (CLOUD engine).", "half")}
<p>The reverb receives the channels' <b>REV</b> sends (stereo, after level and pan) and, with <b>DLY → REV</b>, part of the delay's return. <b>ON</b> switches it; <b>ENGINE</b> selects one of four engines; <b>PROGRAM</b> one of eight programs per engine (the display shows the name). The <b>RETURN</b> meter shows the effects' output.</p>
{B.programs_table(["#", "HALL (Dragonfly Hall)", "EARLY (Dragonfly ER)", "AETHER", "CLOUD (CloudReverb)"])}
<ul>
<li><b>HALL</b>: Dragonfly Hall Reverb — freeverb3's early reflections into its zrev2 late reverb — with programs from Dragonfly's banks.</li>
<li><b>EARLY</b>: Dragonfly Early Reflections, with its eight reflection programs. There is no tail, so DECAY and MOD do nothing (their knobs turn grey).</li>
<li><b>AETHER</b>: Dougal Stewart's Aether, an algorithmic reverb after CloudSeed. The original ships no presets; the eight programs are Front 303's.</li>
<li><b>CLOUD</b>: CloudReverb (the CloudSeed engine), with its presets except Chorus Delay, which is not a reverb.</li>
</ul>
<p><b>PREDELAY</b> (0–250 ms), <b>WIDTH</b> (mono to 150 %) and <b>LEVEL</b> are common. <b>SIZE</b>, <b>DECAY</b>, <b>DAMPING</b> and <b>MOD</b> reshape the program: at their centre the program is exactly as designed; to the left smaller, shorter, brighter or still; to the right larger, longer, darker or more modulated. The programs are roughly level-matched. All four engines are built when the plugin loads and only the selected one runs.</p>

<h1>17. Outputs and multi-out recording</h1>
<p>The plugin has a main stereo output and five auxiliary stereo outputs: one per 303 (channels 3–4, 5–6, 7–8, 9–10) and one for the delay and reverb returns (11–12). <b>OUTPUT</b> in the header chooses:</p>
<ul>
<li><b>STEREO</b>: everything mixed on the main output; the auxiliary outputs are silent.</li>
<li><b>MULTI-OUT</b>: each 303 leaves on its pair after its channel strip (distortion, PCF, channel compressor, level, pan); the returns on theirs. The main output is silent and the master section (MASTER and master compressor) is not used. Sends to the effects work as usual.</li>
</ul>
<p>In REAPER: give Front 303's track 12 channels (<i>Track channels: 12</i>), then use the FX window's <i>Build multichannel routing for output of selected FX</i>, or create a track per pair with a receive from Front 303 (<i>audio 3/4 → 1/2</i>, etc.). Each voice and the effects can then be recorded or processed on their own track.</p>
<div class="note">In MULTI-OUT the voices do not go through MASTER: with high channel levels a stem can pass 0 dBFS. In floating point this does not clip; lower LEVEL or the track.</div>

<h1>18. Memories</h1>
{f("memory.png", "The memory bar: memories 1, 2 and 4 used, 4 the current one.")}
<p>The bar at the bottom holds <b>ten memories</b>, as in GenderBender P23:</p>
<ul>
<li><b>A short click</b> on a memory, or the keys <b>1–9 and 0</b>: recalls it.</li>
<li><b>Holding for 0.6 s</b> (mouse or key): saves the current state there. A bar shows the progress and the memory flashes <b>SAVED</b>.</li>
<li>Empty memories are grey (<i>empty</i>), used ones brown (<i>recall</i>), the last saved or recalled orange.</li>
</ul>
<p>A memory stores <b>everything</b>: every control (the voices' sound, step rates, MIDI switches, mixer, effects, reverb, scale, the BPM knob…) and <b>all 32 patterns</b> (8 per voice). Only RUN, OUTPUT and the DW-303 solver are left out, as they belong to the session rather than to the music. Memories are saved with the project, and are stored by parameter name, so they load into later versions. After a recall, pattern changes still wait for the bar line as usual. A recall can be undone.</p>

<h1>19. Undo, redo, copy and paste</h1>
<h3>Undo and redo</h3>
<p><b>UNDO / REDO</b> in the bottom bar, or <b>Ctrl+Z</b> and <b>Ctrl+Shift+Z / Ctrl+Y</b>, take back and redo what is done on the panel: knobs and selectors, step edits, GENERATE, MUTATE, the pattern tools, PASTE and memory recalls. A knob or note drag counts as one step. The last 100 steps are kept. Changes made by host automation or by EVOLVE are not recorded, and the history is not saved with the project.</p>
<h3>Copy and paste of patterns</h3>
<p>In each unit's pattern tools, <b>COPY</b> copies the pattern shown in that voice (its PATTERN number) and <b>PASTE</b> writes it over the pattern shown in the voice where it is pressed. It works between voices and between patterns of the same voice (copy, change PATTERN, paste). The source COPY button stays lit and the bottom bar says what is copied. What is copied is the sequence — notes, accents, slides, rests and length — not the sound.</p>

<h1>20. Using the panel</h1>
<ul>
<li><b>Knobs</b>: drag up and down; Shift or Alt for fine moves; mouse wheel; double-click for the default. Hovering shows the name and value.</li>
<li><b>Displays with arrows</b> (model, rate, scale, effect modes, programs, delay time, output): click the left third to step back, the rest to step forward; wheel; right-click for the default.</li>
<li>The window can be resized keeping its proportions, if the host allows it.</li>
<li><b>Keys</b>: 1–0 for the memories, Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y. In REAPER enable <i>Send all keyboard input to plug-in</i> in the FX window's <b>+</b> menu, otherwise REAPER keeps those keys.</li>
<li>Every control except the patterns' contents is an automatable parameter with names such as “303-2 Cutoff” or “Reverb Size”.</li>
</ul>

<h1 class="brk">21. Parameter reference</h1>
<p>All 142 parameters, as the host sees them. The identifier is stable: hosts store automation under it. Knob-type parameters are shown 0–127 as on ReBirth's panel. Per-voice parameters exist four times (<span class="id">v1_</span> to <span class="id">v4_</span>); voices 2–4 start with ON off.</p>
{B.param_tables(TEXT['en'])}

<h1>22. Technical notes and limits</h1>
<ul>
<li><b>44.1 kHz engine.</b> The engine always runs at 44.1 kHz in 64-sample blocks, because ReBirth's tables and block-rate controls are built for that rate. The plugin resamples to the host rate with a 32-tap Kaiser-windowed sinc and no added latency (the engine renders ahead). The reverbs and other effects also run at 44.1 kHz; the tape echo at 176.4 kHz (IV-X runs at 192 kHz, so its head filters land 8 % lower).</li>
<li><b>Real-time safety.</b> Nothing allocates memory on the audio thread, including the C++ engines: freeverb3's delay lines, resized on size and program changes, are served from a per-reverb pool filled when the plugin loads. This is checked by tests that count every call to malloc.</li>
<li><b>Synthesised tape noise.</b> IV-X's recorded noise tables are not in its published source; Front 303 generates white noise darkened for each tape speed.</li>
<li><b>CPU.</b> A DW-303 voice costs around 20 % of a 2.1 GHz core; the other models almost nothing. The effects and the reverb together are around 10 % of a core.</li>
<li><b>Tests.</b> 51 automated tests (tuning on every model, levels, step rates, MIDI transposition, outputs, every effect mode and reverb program, memories, undo, resampler, no allocation) and Steinberg's VST3 validator (47/47).</li>
<li>The Windows and macOS builds are compiled by GitHub Actions from the same source; the Linux build is made for glibc 2.30 and later.</li>
</ul>

<h1>23. Credits and licences</h1>
<p>Front 303 is free software under the <b>GNU General Public License, version 3</b>. It contains or builds on:</p>
{t(["Project", "Author", "Licence", "Use in Front 303"], [
 ["OpenRB-338", "OpenRB-338 authors (pingw33n)", "GPL-3.0", "ReBirth 303 voices, distortion, PCF, compressor, delay, mixer block"],
 ["DW-303 / RAVET", "DJW Audio (mo0kid)", "GPL-3.0+", "Circuit-model voice, half-band filters"],
 ["Open303", "Robin Schmidt; maddanio", "MIT", "Open303 voice"],
 ["Dragonfly Reverb", "Michael Willis, Rob van den Berg", "GPL-3.0+", "Hall and Early Reflections"],
 ["freeverb3", "Teru Kamogashira", "GPL-2.0+", "Reverb building blocks"],
 ["Aether", "Dougal Stewart", "MIT", "Aether engine"],
 ["CloudReverb / CloudSeed", "xunil-cloud; Valdemar Erlingsson", "GPL-3.0 / MIT", "Cloud engine and presets"],
 ["IV-X Delay", "Joaquim Broquetas Ruiz", "GPL-3.0", "Tape echo, measured tables"],
 ["Setekh", "Full FX Media", "GPL-3.0", "Setekh distortion"],
 ["Valentine", "Tote Bag Labs", "GPL-3.0", "Valentine compressor"],
 ["Delay Intervals", "Matthew Fecher", "MIT", "Interval delay processing"],
 ["nice-plug, egui", "RustAudio; Emil Ernerfeldt", "ISC; MIT/Apache-2.0", "Plugin framework and panel"],
 ["Inter", "The Inter Project Authors", "OFL-1.1", "Panel typeface"]], ["k", "", "", ""])}
<p>ReBirth RB-338 is a trademark of Propellerhead Software (Reason Studios); TB-303 and Roland are trademarks of Roland Corporation. Front 303 is an independent project, not affiliated with or endorsed by any of them, and contains nothing of ReBirth itself. The panel is Front 303's own artwork, drawn in code.</p>
"""


TEXT["en"] = {
    "title": "Front 303 — User Manual",
    "right": "User manual",
    "flow": {"MODEL": "ReBirth · Vintage · DW-303 · Open303", "STRIP": "CHANNEL STRIP",
             "STEMS": "→ stems (multi-out)", "DLYSEND": "DLY sends", "REVSEND": "REV sends + DLY → REV",
             "MASTER": "master compressor · MASTER", "MAINOUT": "main out 1/2 (STEREO)",
             "MULTI": "main out silent in MULTI-OUT", "MIDI": "MIDI notes → transposition of the voices with MIDI on"},
    "param_head": ["Identifier", "Name", "Range", "Default", "Function"],
    "group_names": {"voice": "Per voice (×4)", "transport": "Transport, scale and master", "dist": "Distortion",
                    "pcf": "PCF", "comp": "Compressor", "delay": "Delay", "reverb": "Reverb"},
    "voice_desc": EN_VOICE,
    "global_desc": EN_GLOBAL,
    "body": en_body,
}

# =============================================================================================
# ESPAÑOL
# =============================================================================================

ES_VOICE = {
    "model": "Motor de sonido de la voz: REBIRTH (sonido 2.0 de ReBirth RB-338, OpenRB-338), VINTAGE (motor Vintage de ReBirth), DW-303 (modelo del circuito componente a componente) u OPEN303 (Open303 de Robin Schmidt). Al cambiar, la voz que se deja se reinicia.",
    "wave": "Forma de onda del oscilador: diente de sierra o cuadrada.",
    "tune": "Afinación gruesa en semitonos, ±2 octavas.",
    "fine": "Afinación fina en cents, ±100.",
    "cutoff": "Corte del filtro. ReBirth y DW-303 usan sus propias curvas; Open303 lleva el mando a 314–2394 Hz en curva exponencial.",
    "reso": "Resonancia del filtro (Open303: 0–100 %).",
    "envmod": "Profundidad de la envolvente del filtro (Open303: 0–100 %).",
    "decay": "Caída de la envolvente del filtro (Open303: 200–2000 ms, exponencial).",
    "accent": "Fuerza de los pasos acentuados (nivel y filtro).",
    "pattern": "El patrón (1–8) que suena y que el panel edita. Con el transporte en marcha, el cambio espera al siguiente compás.",
    "on": "Canal encendido/apagado. Al cargar una instancia solo la 303-1 está encendida.",
    "level": "Nivel del canal (curva de ReBirth).",
    "pan": "Panorama del canal (64 = centro).",
    "send": "Envío al delay, después del nivel (DLY).",
    "dist": "Inserta la distorsión en este canal (DST).",
    "density": "Generador: cuántos pasos suenan.",
    "accents": "Generador: frecuencia de acentos (más probables a contratiempo).",
    "slides": "Generador: frecuencia de slides.",
    "octaves": "Generador: frecuencia de saltos de octava.",
    "evolve": "Reescribe esta proporción del patrón que suena cada vez que da la vuelta. 0 = el patrón no cambia solo.",
    "rate": "Duración del paso del secuenciador de esta voz, de 1/32 a 1/4, con tresillos y puntillos. Todas comparten el tempo.",
    "miditr": "Si está encendido, las notas MIDI transponen la secuencia de esta voz.",
    "reverb": "Envío a la reverb, después de nivel y panorama (REV).",
}

ES_GLOBAL = {
    "run": "Hace sonar los secuenciadores con el transporte del host parado.",
    "tempo": "Tempo que se usa cuando el host no da uno (knob BPM).",
    "shuffle": "Retrasa uno de cada dos pasos hasta medio paso.",
    "root": "Nota raíz para la cuantización y el generador.",
    "scale": "Escala: cromática (sin cuantizar) y otras 13. Una nota fuera de escala suena como la más cercana de la escala (hacia abajo en caso de empate); el patrón guardado no cambia.",
    "master": "Nivel máster (curva de ReBirth).",
    "quality": "Resolución del filtro del DW-303: FULL o ECONOMY (algo más barato).",
    "output": "STEREO: todo por la salida principal. MULTI-OUT: cada voz y los retornos de efectos por su salida; la principal y la sección máster quedan en silencio.",
    "dist_mode": "REBIRTH: la distorsión de ReBirth. SETEKH: Setekh de Full FX Media.",
    "dist_amount": "REBIRTH: cantidad. SETEKH: DRIVE (0–5), mezcla entre la señal limpia y tanh(3,5·x).",
    "dist_shape": "REBIRTH: 0 = recorte duro, por encima curvas de saturación. SETEKH: ganancia de entrada INPUT, 0 a +24 dB.",
    "dist_level": "Solo SETEKH: ganancia de salida OUTPUT, −24 a +24 dB (64 = 0 dB).",
    "pcf_channel": "Canal en el que se inserta el filtro controlado por patrón (o apagado).",
    "pcf_mode": "Paso bajo o paso banda.",
    "pcf_pattern": "Uno de 55 patrones rítmicos de modulación, sincronizados al tempo.",
    "pcf_freq": "Frecuencia del filtro.",
    "pcf_reso": "Resonancia del filtro.",
    "pcf_amount": "Profundidad de la modulación del patrón.",
    "pcf_decay": "Caída de cada pulso de modulación.",
    "comp_mode": "REBIRTH: el compresor de ReBirth. VALENTINE: Valentine de Tote Bag Labs.",
    "comp_route": "Dónde va el compresor: apagado, en el máster o en un canal.",
    "comp_amount": "REBIRTH: cantidad. VALENTINE: COMPRESS (ganancia de entrada), −6 a +36 dB.",
    "comp_threshold": "REBIRTH: umbral. VALENTINE: OUTPUT (ganancia de compensación), −24 a +12 dB.",
    "comp_ratio": "VALENTINE: ratio 1–21; el knee y el umbral siguen al ratio como en el original.",
    "comp_attack": "VALENTINE: ataque 0,02–55 ms (exponencial; por defecto 1,3 ms).",
    "comp_release": "VALENTINE: relajación 2–1100 ms (exponencial; por defecto 350 ms).",
    "comp_saturate": "VALENTINE: 0 = apagado; 1–10 = saturación asimétrica de seno hiperbólico inverso.",
    "comp_crush": "VALENTINE: 0 = apagado; 1–10 = retención de muestra hacia 27,5 kHz y reducción de 12 a 1 bits.",
    "comp_mix": "VALENTINE: seco/procesado, 0–100 %.",
    "delay_on": "Delay encendido/apagado.",
    "delay_mode": "REBIRTH, TAPE (IV-X Delay) o INTERVALS.",
    "delay_steps": "Tiempo (o intervalo) en pasos de semicorchea, o de corchea de tresillo con TRIPLET.",
    "delay_triplet": "Los pasos pasan a ser corcheas de tresillo.",
    "delay_feedback": "REBIRTH / TAPE: realimentación (TAPE llega a la autooscilación). INTERVALS: LOOP, cuánto de la última toma vuelve a la entrada de la línea.",
    "delay_pan": "Posición de los ecos (TAPE/INTERVALS: balance de igual potencia).",
    "delay_a": "TAPE: profundidad de WOW. INTERVALS: FALLOFF, nivel que se pierde en cada toma (0–60 %).",
    "delay_b": "TAPE: profundidad de FLUTTER. INTERVALS: TONE (izquierda más oscuro, centro abierto, derecha más fino).",
    "delay_c": "TAPE: NOISE (soplido y rascado de cinta). INTERVALS: SPREAD, de mono a las amplitudes izquierda/derecha del patrón.",
    "delay_kind": "TAPE: velocidad y bias (1–4: 15 IPS, 5–8: 7,5 IPS, 9–12: 3,75 IPS; bias 2–5). INTERVALS: el patrón (1–8).",
    "rev_on": "Reverb encendida/apagada.",
    "rev_mode": "Motor: HALL (Dragonfly Hall), EARLY (Dragonfly Early Reflections), AETHER, CLOUD (CloudReverb).",
    "rev_program": "Uno de los ocho programas del motor.",
    "rev_predelay": "Predelay antes del motor, 0–250 ms.",
    "rev_size": "Escala el tamaño del programa: ×0,5 … ×2 (centro = como el programa).",
    "rev_decay": "Escala el tiempo de caída: ×0,35 … ×2,8 (centro = como el programa). Sin efecto en EARLY.",
    "rev_damping": "Amortiguación de agudos: el corte se mueve ×2,8 … ×0,35 (centro = como el programa).",
    "rev_mod": "Profundidad de modulación: nada a 0, la del programa en el centro, ×4 arriba. Sin efecto en EARLY.",
    "rev_width": "Anchura estéreo del retorno, de mono a 150 %.",
    "rev_delay": "Envía el retorno del delay a la reverb.",
    "rev_level": "Nivel de retorno de la reverb.",
}


def es_body(flow):
    f, t = B.fig, B.table
    return f"""
<div class="cover">
  <div style="font-size:11pt;color:#ff8a1c;letter-spacing:2pt;font-weight:700">MANUAL DE USUARIO · v{B.VERSION}</div>
  <div class="logo">FRONT <b>303</b></div>
  <div class="sub">Cuatro líneas de bajo TB-303, secuenciadores generativos y efectos al estilo ReBirth — VST3 y CLAP para Linux, Windows y macOS</div>
  <img src="img/panel.png">
  <div class="meta">Eloy Platas · noviembre de 2026 · GPL-3.0 · github.com/eloyplatas/FRONT-303<br>
  Construido con OpenRB-338, DW-303, Open303, Dragonfly Reverb, Aether, CloudReverb, IV-X Delay, Setekh, Valentine y Delay Intervals.</div>
</div>

<h1 class="brk">Índice</h1>
<div class="toc">
<div>1. Qué es Front 303</div><div>2. Historia del plugin</div><div>3. Instalación</div>
<div>4. El panel y el recorrido de la señal</div><div>5. Transporte, tempo y sincronía con el host</div>
<div>6. Las cuatro 303</div><div>7. Los secuenciadores</div><div>8. El generador de patrones</div>
<div>9. Cuantización a escala</div><div>10. Transposición por MIDI</div><div>11. El mezclador</div>
<div>12. Distorsión</div><div>13. PCF (filtro controlado por patrón)</div><div>14. Compresor</div>
<div>15. Delay</div><div>16. Reverb</div><div>17. Salidas y grabación multipista</div>
<div>18. Memorias</div><div>19. Deshacer, rehacer, copiar y pegar</div><div>20. Uso del panel</div>
<div>21. Referencia de parámetros</div><div>22. Notas técnicas y límites</div><div>23. Créditos y licencias</div>
</div>

<h1 class="brk">1. Qué es Front 303</h1>
<p>Front 303 es un instrumento de software con <b>cuatro sintetizadores de bajo TB-303</b>, cada uno con su secuenciador de 16 pasos y su generador de patrones, que pasan por el mezclador y los efectos de <b>ReBirth RB-338</b>, ampliados con modos nuevos y una reverb de cuatro motores. Es un plugin VST3 y CLAP para Linux, Windows y macOS, escrito en Rust, con un panel dibujado en vectores a la manera de ReBirth 2.0.</p>
<p>Se inspira en ReBirth pero deja fuera las cajas de ritmo: en su lugar hay cuatro 303, y cada una puede usar un motor de sonido distinto:</p>
{t(["Modelo", "Qué es", "Origen"], [
 ["<b>REBIRTH</b>", "La 303 de ReBirth RB-338 2.0, exacta a nivel de bloque", "OpenRB-338 (reimplementación por ingeniería inversa de ReBirth 2.01)"],
 ["<b>VINTAGE</b>", "El motor «Vintage» de ReBirth: control cada 4 muestras, resonancia escalonada", "OpenRB-338"],
 ["<b>DW-303</b>", "Modelo del circuito de la TB-303 componente a componente (oscilador, filtro en escalera, envolventes, acento, VCA)", "DW-303 de DJW Audio"],
 ["<b>OPEN303</b>", "El DSP Open303 de Robin Schmidt", "open303 (versión de maddanio)"]], ["k", "", ""])}
<p>Funciones principales:</p>
<ul>
<li>Cuatro voces, modelo por voz, afinación en semitonos (±24) y afinación fina en cents (±100).</li>
<li>Ocho patrones por voz, editables en un editor de 16 pasos con nota, acento, slide y silencio; longitud de 1 a 16 pasos (polimetría).</li>
<li>Una <b>métrica</b> distinta para cada secuenciador bajo el mismo tempo (de 1/32 a 1/4, con tresillos y puntillos): polirritmos.</li>
<li>Patrones aleatorios y generativos al estilo 303, con mutación y evolución continua, cuantizados a una raíz y una de 14 escalas.</li>
<li><b>Transposición por MIDI</b> de las secuencias, elegible por voz.</li>
<li>Distorsión, PCF, compresor y delay de ReBirth, más la distorsión <b>Setekh</b>, el compresor <b>Valentine</b>, los delays de <b>cinta IV-X</b> e <b>intervalos</b> y una reverb con los motores <b>Dragonfly Hall</b>, <b>Dragonfly Early Reflections</b>, <b>Aether</b> y <b>CloudReverb</b>.</li>
<li><b>Salidas separadas</b>: un par estéreo por 303 y otro para los retornos de efectos, para grabar cada voz en su pista.</li>
<li><b>Diez memorias</b> del instrumento completo, <b>deshacer/rehacer</b> y <b>copiar/pegar</b> patrones entre voces.</li>
<li>142 parámetros automatizables; patrones y memorias se guardan con el proyecto.</li>
</ul>

<h1 class="brk">2. Historia del plugin</h1>
<p>Front 303 nació en noviembre de 2026 a partir del estudio de dos proyectos de código abierto. El primero fue <b>DW-303</b> (mo0kid), una TB-303 modelada etapa a etapa desde el circuito: el CV de tono y el convertidor exponencial, el VCO de diente de sierra y cuadrada, el VCF en escalera de transistores de cuatro polos con sus valores de componentes reales, las envolventes MEG y VEG, el circuito de acento con su barrido y el VCA. El segundo fue <b>OpenRB-338</b> (pingw33n), una reimplementación del ReBirth RB-338 2.01 de Propellerhead hecha por ingeniería inversa, fiel hasta en sus tablas, su proceso por bloques y sus efectos; de él interesaban solo las 303 y los efectos, no las cajas de ritmo.</p>
<p>Tras comparar los dos —el modelo de circuito suena más cerca de una TB-303 real, ReBirth suena al ReBirth de tantos discos— la idea quedó clara: <i>algo como OpenRB-338, sin cajas de ritmo y con cuatro 303, cada una con su modelo elegible, con los efectos y el ruteo de ReBirth, secuenciadores con funciones aleatorias y generativas que saquen patrones estilo 303 cuantizados a escala, controles de tono en semitonos (±24) con un control fino, y gráficos elaborados al estilo de ReBirth 2.0</i>. El proyecto se llamó <b>Front 303</b>, con licencia GPL-3.0 y código abierto, para uso personal, con versiones de Windows y macOS previstas desde el principio. Se eligió Rust y el framework nice-plug en lugar de JUCE porque los dos proyectos de partida son crates de Rust y porque un único código compila para los tres sistemas.</p>
<p>El trabajo avanzó por fases: el motor (cuatro voces, modelo por voz, afinación, mezclador y efectos), los secuenciadores y el generador, el plugin VST3/CLAP con sincronía al host y automatización, y por último el panel, dibujado entero en código. La <b>versión 0.2.0</b> reunió todo eso para Linux, comprobado con el validador VST3 de Steinberg.</p>
<p>Después vino el estudio de <b>Open303</b> (la versión de maddanio del sinte de Robin Schmidt): medido frente a una TB-303 tiene menos pico de resonancia y acentos más planos, pero es limpio, ligero y afinado, así que entró como cuarto modelo. La <b>versión 0.3.0</b> añadió además la transposición MIDI de las secuencias, la métrica por secuenciador, salidas separadas para cada 303 y para los efectos, y modos nuevos para los efectos portados o compilados desde otros proyectos abiertos: Setekh (distorsión), Valentine (compresor), IV-X Delay (eco de cinta), Delay Intervals (delay por intervalos) y cuatro reverbs (Dragonfly Hall y Early Reflections, Aether, CloudReverb). La <b>versión 0.4.0</b> trajo diez memorias a la manera de GenderBender P23 —otro plugin del autor—, deshacer/rehacer y copiar patrones entre voces, y las versiones para Windows y macOS, compiladas en GitHub Actions.</p>
{t(["Versión", "Contenido"], [
 ["0.1.0", "Motor: cuatro voces con modelos ReBirth / DW-303, afinación, mezclador y efectos de ReBirth; secuenciadores y generador."],
 ["0.2.0", "Plugin VST3/CLAP para Linux, sincronía con el host, 104 parámetros, panel al estilo ReBirth 2.0."],
 ["0.3.0", "Modelo Open303, métrica por voz, transposición MIDI, salidas separadas, Setekh, Valentine, delays de cinta e intervalos, reverb de cuatro motores (142 parámetros)."],
 ["0.4.0", "Diez memorias, deshacer/rehacer, copiar/pegar patrones; versiones para Windows y macOS."]], ["k", ""])}

<h1>3. Instalación</h1>
<p>Los archivos están en la carpeta <code>releases</code> del repositorio <code>github.com/eloyplatas/FRONT-303</code>. Cada uno trae el plugin VST3 y el CLAP, el manual en texto, las licencias y una nota de instalación.</p>
<h3>Linux (x86_64, glibc 2.30 o posterior)</h3>
<p>Descomprime <code>Front303-{B.VERSION}-linux-x86_64.tar.gz</code> y ejecuta <code>./install.sh</code>: copia <code>Front303.vst3</code> en <code>~/.vst3</code> y <code>Front303.clap</code> en <code>~/.clap</code> (<code>./install.sh --uninstall</code> los quita). Necesita X11 o XWayland y OpenGL para el panel.</p>
<h3>Windows (64 bits)</h3>
<p>Descomprime <code>Front303-{B.VERSION}-windows-x86_64.zip</code>. Copia la carpeta <code>Front303.vst3</code> en <code>C:\\Program Files\\Common Files\\VST3\\</code> y, si quieres, <code>Front303.clap</code> en <code>C:\\Program Files\\Common Files\\CLAP\\</code>. No necesita ninguna librería adicional. No está firmado, así que SmartScreen puede avisar la primera vez.</p>
<h3>macOS (10.15 o posterior, Apple Silicon e Intel)</h3>
<p>Descomprime <code>Front303-{B.VERSION}-macos-universal.zip</code> y haz doble clic en <code>install.command</code> (clic derecho → Abrir si macOS no deja). Lo instala en <code>~/Library/Audio/Plug-Ins/VST3</code> y <code>…/CLAP</code> y quita la marca de cuarentena. El plugin lleva una firma ad hoc y no está notarizado: sin quitar la cuarentena (<code>xattr -dr com.apple.quarantine</code>), macOS dice que está dañado.</p>
<p>Después, vuelve a escanear plugins en el host (en REAPER: <i>Preferences → Plug-ins → VST → Re-scan</i>). Front 303 aparece como instrumento.</p>

<h1 class="brk">4. El panel y el recorrido de la señal</h1>
{f("panel.png", "El panel completo (1280 × 1058 píxeles de diseño; se puede redimensionar manteniendo la proporción).")}
<p>De arriba abajo: la <b>cabecera</b> (transporte, tempo, escala, máster, transposición MIDI y modo de salida); una fila con <b>distorsión, PCF, compresor y delay</b>; una fila con la <b>reverb</b> y el <b>mezclador</b>; las cuatro <b>unidades 303</b>; y la <b>barra de memorias</b> con deshacer/rehacer.</p>
{flow}
<p>Cada secuenciador toca su voz. Cada voz pasa por su tira de canal (distorsión si DST está encendido, PCF si está asignado a ese canal, compresor si está en ese canal, y después nivel y panorama). Los canales alimentan el delay (envíos DLY) y la reverb (envíos REV); el delay puede alimentar también la reverb (DLY → REV). En modo STEREO los canales y los retornos de efectos se suman, pasan por el compresor del máster si está ahí y por el nivel MASTER, y salen por la salida principal. En MULTI-OUT cada canal y los retornos salen por su propia salida.</p>

<h1>5. Transporte, tempo y sincronía con el host</h1>
{f("header.png", "La cabecera.")}
<ul>
<li><b>Con el host en marcha</b>, Front 303 lo sigue: tempo, posición, bucles y saltos. Los patrones quedan alineados a los compases del proyecto (el paso 1 cae en cada compás). Se enciende el LED <b>HOST</b>.</li>
<li><b>Con el host parado</b>, <b>RUN</b> hace sonar los secuenciadores por su cuenta; el LED <b>PLAY</b> parpadea con el pulso.</li>
<li>El display <b>TEMPO</b> muestra el tempo en uso: el del host si lo da, si no el del knob <b>BPM</b> (20–300).</li>
<li><b>SHUFFLE</b> retrasa uno de cada dos pasos hasta medio paso.</li>
<li><b>ROOT</b> y <b>SCALE</b> cuantizan todas las voces (sección 9).</li>
<li><b>DW-303 SOLVER</b> elige FULL o ECONOMY para el filtro del DW-303.</li>
<li><b>MASTER</b> fija el nivel máster, con los vúmetros estéreo al lado.</li>
<li><b>MIDI TR</b> muestra la transposición MIDI actual; <b>OUTPUT</b> elige STEREO o MULTI-OUT.</li>
</ul>

<h1>6. Las cuatro 303</h1>
{f("unit.png", "Una unidad 303 (aquí con el modelo OPEN303).")}
<p>Cada unidad tiene una placa con un LED que sigue su nivel, el <b>selector de modelo</b>, <b>SAW / SQUARE</b> y siete knobs: <b>TUNE</b> (semitonos, con display debajo), <b>FINE</b> (cents, con display), <b>CUT OFF FREQ</b>, <b>RESONANCE</b>, <b>ENV MOD</b>, <b>DECAY</b> y <b>ACCENT</b>.</p>
<h3>Los modelos</h3>
<ul>
<li><b>REBIRTH</b>: el sonido 2.0. El aparato calcula sus controles una vez por bloque de 64 muestras, así que las notas empiezan en los bordes de bloque (1,45 ms), como en el original. Sostiene mientras la puerta está abierta.</li>
<li><b>VINTAGE</b>: el motor antiguo de ReBirth, con control cada cuatro muestras y resonancia escalonada.</li>
<li><b>DW-303</b>: el modelo de circuito, a 4× de sobremuestreo (176,4 kHz) con un diezmador de media banda en dos etapas. Las notas empiezan en su muestra exacta. Como el circuito real, su VCA decae en notas muy largas. Cuesta mucha más CPU que los otros modelos (en torno al 20 % de un núcleo a 2,1 GHz por voz).</li>
<li><b>OPEN303</b>: el Open303 de Robin Schmidt, con las curvas de su propio plugin (corte 314–2394 Hz, decay 200–2000 ms). Medido frente a una TB-303 tiene menos pico de resonancia (6–21 dB frente a 19–43 dB) y acentos que no se acumulan; se incluye tal cual, con dos accesos a memoria fuera de rango del original corregidos.</li>
</ul>
<p>Afinación y volumen están igualados entre modelos (La2 = 110 Hz en todos, nivel de un riff a menos de 1 dB), pero cada uno conserva sus curvas de knob, así que la misma posición no suena idéntica en todos. Cambiar de modelo mientras suena es limpio: la voz que se deja se reinicia.</p>

<h1>7. Los secuenciadores</h1>
<p>Bajo cada unidad está su secuenciador.</p>
<ul>
<li><b>Pasos</b>: clic para alternar nota / silencio; arrastrar arriba/abajo o girar la rueda para cambiar la nota (con una escala elegida, la nota se mueve por grados de la escala). Se muestra el nombre de la nota (C2 es el Do de referencia de la 303); los silencios muestran su nota atenuada.</li>
<li><b>AC</b>: acento. <b>SL</b>: slide, el paso se liga al siguiente con el glide de la 303.</li>
<li><b>LENGTH − / +</b>: longitud del patrón, 1–16 pasos. Los pasos fuera de la longitud se oscurecen. Longitudes distintas en voces distintas dan polimetría.</li>
<li><b>Métrica</b> (display bajo LENGTH): 1/32, 1/16T, 1/16, 1/8T, 1/16., 1/8, 1/4T, 1/8. o 1/4. Todas comparten el tempo; con métricas distintas se cruzan en polirritmos (por ejemplo 1/16 contra 1/8T, seis pasos por compás). Se cuentan desde el inicio de la canción, así que con el host caen siempre en el mismo sitio del compás.</li>
<li><b>MIDI</b>: si esta voz sigue la transposición MIDI (sección 10).</li>
<li><b>PATTERN 1–8</b>: el patrón que suena y que el panel edita. Con el transporte en marcha el cambio espera a la barra de compás; el punto rojo marca el patrón que suena y el LED rojo sobre los pasos, el paso que suena.</li>
<li><b>Herramientas de patrón</b>: <b>← →</b> desplazan un paso; <b>CLR</b> borra; <b>−1 / +1</b> transponen un semitono; <b>OCT</b> sube una octava (las notas que se salen se pliegan); <b>COPY / PASTE</b> en la sección 19.</li>
</ul>

<h1>8. El generador de patrones</h1>
<p>La sección <b>GENERATOR</b> escribe líneas al estilo 303 en el patrón seleccionado, en la raíz y la escala del panel.</p>
<ul>
<li><b>GENERATE</b> escribe un patrón nuevo. Primero escribe un motivo de medio patrón y luego lo repite con variaciones, lo que da a la línea su aire de riff.</li>
<li><b>DENSITY</b>: cuántos pasos suenan. <b>ACCENTS</b>: frecuencia de acentos, más probables a contratiempo. <b>SLIDES</b>: frecuencia de slides. <b>OCTAVES</b>: frecuencia de saltos de octava.</li>
<li><b>MUTATE</b> reescribe una cuarta parte de los pasos.</li>
<li><b>EVOLVE</b> muta el patrón que suena cada vez que da la vuelta, en la proporción del knob. A cero el patrón no cambia solo; mientras evoluciona se lee «EVOLVING». El panel muestra los cambios en el momento.</li>
</ul>

<h1>9. Cuantización a escala</h1>
<p><b>ROOT</b> y <b>SCALE</b> cuantizan lo que tocan las cuatro voces y guían al generador. Las 14 escalas: cromática (sin cuantizar), mayor, menor, dórica, frigia, lidia, mixolidia, locria, menor armónica, frigia dominante, pentatónica menor, pentatónica mayor, blues y tonos enteros. Una nota fuera de escala suena como la más cercana de la escala (hacia abajo en caso de empate). El patrón guardado no cambia: al volver a cromática suena como está escrito.</p>

<h1>10. Transposición por MIDI</h1>
<p>Front 303 acepta notas MIDI en su entrada. Cada note-on transpone las secuencias de las voces que tengan <b>MIDI</b> encendido: <b>C3 (nota 60)</b> las deja como están, C#3 las sube un semitono, B2 las baja uno, hasta ±24 semitonos. La transposición se queda hasta la siguiente nota (no hay que mantener la tecla) y empieza en la muestra exacta en que llega la nota. Los note-off se ignoran. El display <b>MIDI TR</b> muestra el valor actual. Se aplica después de cuantizar a la escala: es una transposición literal, como en una TB-303.</p>
<p>En REAPER, arma la pista y elige una entrada MIDI, o escribe las notas en un ítem MIDI de la misma pista.</p>

<h1>11. El mezclador</h1>
{f("mixer.png", "El mezclador (aquí en MULTI-OUT, con el par de salida de cada canal).", "half")}
<p>Por canal: <b>LEVEL</b> con su vúmetro, <b>PAN</b>, <b>DLY</b> (envío al delay, después del nivel), <b>REV</b> (envío a la reverb, después de nivel y panorama), <b>ON</b> y <b>DST</b> (inserción de la distorsión). Las ganancias se suavizan como en ReBirth, así que los movimientos no hacen clic.</p>

<h1>12. Distorsión</h1>
{f("fx_rebirth.png", "La fila de efectos en sus modos ReBirth.")}
{f("fx_new.png", "La misma fila con SETEKH, VALENTINE e INTERVALS.")}
<p>La distorsión actúa en los canales con <b>DST</b> encendido. Dos modos:</p>
<ul>
<li><b>REBIRTH</b>: la distorsión de ReBirth, una sola unidad compartida por los canales. <b>AMOUNT</b>; <b>SHAPE</b> a 0 es un recorte duro y por encima curvas de saturación. Deja algo de continua con ondas asimétricas, como la de ReBirth.</li>
<li><b>SETEKH</b>: la distorsión Setekh (Full FX Media): realce de agudos de +6 dB a 7,5 kHz, ganancia de entrada, mezcla de la señal limpia con tanh(3,5·x) según <b>DRIVE</b>, ganancia de salida, y el realce deshecho al final. <b>INPUT</b> de 0 a +24 dB, <b>OUTPUT</b> de −24 a +24 dB. Cada canal tiene la suya.</li>
</ul>

<h1>13. PCF (filtro controlado por patrón)</h1>
<p>El PCF de ReBirth se inserta en un canal (<b>CHANNEL</b>) y mueve su filtro rítmicamente con uno de <b>55 patrones</b> sincronizados al tempo. <b>MODE</b> paso bajo o paso banda; <b>FREQ</b>, <b>RESO</b>, <b>AMOUNT</b> (profundidad de la modulación) y <b>DECAY</b> (duración de cada pulso).</p>

<h1>14. Compresor</h1>
<p>Se coloca con <b>ON</b> en el máster o en un canal. Dos modos:</p>
<ul>
<li><b>REBIRTH</b>: el compresor de ReBirth, <b>AMOUNT</b> y <b>THRESHOLD</b>.</li>
<li><b>VALENTINE</b>: el Valentine de Tote Bag Labs: una ganancia de entrada (<b>COMPRESS</b>, −6 a +36 dB) hacia un compresor de picos feed-forward cuyo knee y umbral siguen al <b>RATIO</b> (1–21), luego un saturador asimétrico de seno hiperbólico inverso (<b>SATURATE</b> 1–10, 0 = apagado) y un recorte suave, todo a 4× la frecuencia de muestreo con antialiasing por antiderivada; <b>CRUSH</b> opcional delante (retención de muestra hacia 27,5 kHz y reducción de bits de 12 a 1); <b>ATTACK</b> 0,02–55 ms, <b>RELEASE</b> 2–1100 ms, <b>MIX</b> y <b>OUTPUT</b> (−24 a +12 dB) detrás. En el máster trabaja en estéreo enlazado.</li>
</ul>

<h1>15. Delay</h1>
<p>Lo alimentan los envíos <b>DLY</b> de los canales. El tiempo se fija en pasos (1–32) de semicorchea, o de corchea de tresillo con <b>TRIPLET</b>, sincronizado al tempo. <b>ON</b> lo enciende. Tres modos:</p>
<h3>REBIRTH</h3>
<p>El delay de ReBirth con <b>FEEDBACK</b> y <b>PAN</b>.</p>
<h3>TAPE (IV-X Delay)</h3>
<p>El eco de cinta de IV-X Delay (Joaquim Broquetas Ruiz), portado a Rust. En cada muestra el transporte se acerca poco a poco al tiempo fijado (al cambiar el tiempo el tono se dobla, como una máquina de cinta cambiando de velocidad; al activar el modo la cinta «arranca» en menos de un segundo); el ruido de cinta y la realimentación se suman a la entrada; el cabezal de grabación es la curva de transferencia y la respuesta FIR medidas por IV-X para la velocidad y el bias elegidos; el de reproducción lee con interpolación en una posición que mueven el wow, el flutter y el rascado.</p>
{t(["Control", "Función"], [
 ["FEEDBACK", "0–1,5 en unidades de IV-X; arriba llega a la autooscilación, limitada por la curva de la cinta."],
 ["PAN", "Posición del eco (mono)."],
 ["WOW", "Oscilación lenta del tono (1 Hz)."],
 ["FLUTTER", "Oscilación rápida del tono (15 Hz)."],
 ["NOISE", "Soplido de cinta y rascado del cabezal."],
 ["TAPE", "1–4: 15 IPS, 5–8: 7,5 IPS, 9–12: 3,75 IPS, cada una con bias 2, 3, 4, 5. Cinta más lenta: más oscura, más saturada y con más ruido."]], ["k", ""])}
<h3>INTERVALS</h3>
<p>Repeticiones colocadas sobre un patrón de intervalos, a la manera de Delay Intervals de Matthew Fecher. La entrada recorre una línea; en cada múltiplo del intervalo pasa por una toma, donde la caída y los filtros de esa toma se aplican a la propia línea, así que cada repetición es más oscura y más baja que la anterior. Cada toma suena con su amplitud izquierda y derecha, tomadas del patrón elegido.</p>
{t(["Patrón", "Tomas", "Carácter"], [
 ["STRAIGHT", "8", "Todos los intervalos, a los dos lados."],
 ["PING PONG", "12", "Alterna izquierda y derecha."],
 ["DOTTED", "13", "Uno de cada tres intervalos."],
 ["TRESILLO", "16", "Agrupación afrocubana 3-3-4-2-4, estéreo desigual."],
 ["SWELL", "12", "Repeticiones que crecen en lugar de apagarse."],
 ["GALLOP", "12", "Dos sí, uno no."],
 ["SCATTER", "16", "Irregular, repartido por el estéreo."],
 ["STUTTER", "16", "Cuatro repeticiones, un hueco, cuatro más."]], ["k", "", ""])}
<p><b>FALLOFF</b> (0–60 % por toma), <b>TONE</b> (a la izquierda un paso bajo que cierra hasta 1 kHz; a la derecha un paso alto que abre hasta 1,5 kHz), <b>SPREAD</b> (de mono al estéreo propio del patrón), <b>LOOP</b> (el knob FEEDBACK: cuánto de la última toma vuelve a la entrada de la línea) y <b>PAN</b> (balance). El intervalo llega hasta 1 s.</p>

<h1>16. Reverb</h1>
{f("reverb.png", "El módulo de reverb (motor CLOUD).", "half")}
<p>La reverb recibe los envíos <b>REV</b> de los canales (en estéreo, después de nivel y panorama) y, con <b>DLY → REV</b>, parte del retorno del delay. <b>ON</b> la enciende; <b>ENGINE</b> elige uno de cuatro motores; <b>PROGRAM</b> uno de ocho programas por motor (el display muestra el nombre). El vúmetro <b>RETURN</b> muestra la salida de los efectos.</p>
{B.programs_table(["#", "HALL (Dragonfly Hall)", "EARLY (Dragonfly ER)", "AETHER", "CLOUD (CloudReverb)"])}
<ul>
<li><b>HALL</b>: Dragonfly Hall Reverb —reflexiones tempranas de freeverb3 hacia su cola zrev2— con programas de los bancos de Dragonfly.</li>
<li><b>EARLY</b>: Dragonfly Early Reflections, con sus ocho programas de reflexiones. No tiene cola, así que DECAY y MOD no actúan (sus knobs se ven grises).</li>
<li><b>AETHER</b>: el Aether de Dougal Stewart, una reverb algorítmica a partir de CloudSeed. El original no trae presets; los ocho programas son de Front 303.</li>
<li><b>CLOUD</b>: CloudReverb (el motor CloudSeed), con sus presets menos Chorus Delay, que no es una reverb.</li>
</ul>
<p><b>PREDELAY</b> (0–250 ms), <b>WIDTH</b> (de mono a 150 %) y <b>LEVEL</b> son comunes. <b>SIZE</b>, <b>DECAY</b>, <b>DAMPING</b> y <b>MOD</b> modifican el programa: en el centro el programa queda tal como está diseñado; a la izquierda, más pequeño, corto, brillante o quieto; a la derecha, más grande, largo, oscuro o modulado. Los programas están nivelados entre sí aproximadamente. Los cuatro motores se construyen al cargar el plugin y solo funciona el elegido.</p>

<h1>17. Salidas y grabación multipista</h1>
<p>El plugin tiene una salida estéreo principal y cinco salidas estéreo auxiliares: una por 303 (canales 3–4, 5–6, 7–8, 9–10) y una para los retornos de delay y reverb (11–12). <b>OUTPUT</b> en la cabecera elige:</p>
<ul>
<li><b>STEREO</b>: todo mezclado por la salida principal; las auxiliares quedan en silencio.</li>
<li><b>MULTI-OUT</b>: cada 303 sale por su par después de su tira de canal (distorsión, PCF, compresor de canal, nivel, panorama); los retornos por el suyo. La salida principal queda en silencio y la sección máster (MASTER y compresor en el máster) no actúa. Los envíos a los efectos funcionan igual.</li>
</ul>
<p>En REAPER: pon 12 canales en la pista de Front 303 (<i>Track channels: 12</i>) y usa en la ventana del FX <i>Build multichannel routing for output of selected FX</i>, o crea una pista por par con un receive desde Front 303 (<i>audio 3/4 → 1/2</i>, etc.). Así cada voz y los efectos se graban o procesan en su propia pista.</p>
<div class="note">En MULTI-OUT las voces no pasan por MASTER: con niveles de canal altos una pista puede superar 0 dBFS. En coma flotante no recorta; baja LEVEL o la pista.</div>

<h1>18. Memorias</h1>
{f("memory.png", "La barra de memorias: memorias 1, 2 y 4 usadas, la 4 es la actual.")}
<p>La barra inferior tiene <b>diez memorias</b>, como en GenderBender P23:</p>
<ul>
<li><b>Clic corto</b> en una memoria, o las teclas <b>1–9 y 0</b>: la recupera.</li>
<li><b>Mantener 0,6 s</b> (ratón o tecla): guarda ahí el estado actual. Una barrita muestra el progreso y la memoria parpadea con <b>SAVED</b>.</li>
<li>Vacías en gris (<i>empty</i>), usadas en marrón (<i>recall</i>) y la última guardada o recuperada en naranja.</li>
</ul>
<p>Una memoria guarda <b>todo</b>: todos los controles (sonido de las voces, métricas, interruptores MIDI, mezclador, efectos, reverb, escala, knob BPM…) y <b>los 32 patrones</b> (8 por voz). Solo quedan fuera RUN, OUTPUT y el DW-303 SOLVER, que son de la sesión y no de la música. Se guardan con el proyecto, por nombre de parámetro, así que cargan también en versiones posteriores. Tras recuperar una memoria, los cambios de patrón siguen esperando a la barra de compás. Una recuperación se puede deshacer.</p>

<h1>19. Deshacer, rehacer, copiar y pegar</h1>
<h3>Deshacer y rehacer</h3>
<p><b>UNDO / REDO</b> en la barra inferior, o <b>Ctrl+Z</b> y <b>Ctrl+Mayús+Z / Ctrl+Y</b>, deshacen y rehacen lo hecho en el panel: knobs y selectores, ediciones de pasos, GENERATE, MUTATE, las herramientas de patrón, PASTE y las recuperaciones de memoria. Un arrastre de knob o de nota cuenta como un solo paso. Se guardan los últimos 100 pasos. Lo que cambian la automatización del host o EVOLVE no entra en el historial, y el historial no se guarda con el proyecto.</p>
<h3>Copiar y pegar patrones</h3>
<p>En las herramientas de patrón de cada unidad, <b>COPY</b> copia el patrón que se ve en esa voz (su número de PATTERN) y <b>PASTE</b> lo escribe sobre el patrón que se ve en la voz donde se pulsa. Funciona entre voces y entre patrones de la misma voz (copia, cambia de PATTERN y pega). El COPY de origen queda iluminado y la barra inferior dice qué hay copiado. Se copia la secuencia —notas, acentos, slides, silencios y longitud—, no el sonido.</p>

<h1>20. Uso del panel</h1>
<ul>
<li><b>Knobs</b>: arrastrar arriba/abajo; con Mayúsculas o Alt, ajuste fino; rueda del ratón; doble clic para el valor por defecto. Al pasar por encima se ven nombre y valor.</li>
<li><b>Displays con flechas</b> (modelo, métrica, escala, modos de efecto, programas, tiempo del delay, salida): clic en el tercio izquierdo para retroceder y en el resto para avanzar; rueda; clic derecho para el valor por defecto.</li>
<li>La ventana se puede redimensionar manteniendo la proporción, si el host lo permite.</li>
<li><b>Teclas</b>: 1–0 para las memorias, Ctrl+Z / Ctrl+Mayús+Z / Ctrl+Y. En REAPER activa <i>Send all keyboard input to plug-in</i> en el menú <b>+</b> de la ventana del FX; si no, REAPER se queda esas teclas.</li>
<li>Todos los controles, salvo el contenido de los patrones, son parámetros automatizables con nombres como «303-2 Cutoff» o «Reverb Size».</li>
</ul>

<h1 class="brk">21. Referencia de parámetros</h1>
<p>Los 142 parámetros tal como los ve el host. El identificador es estable: los hosts guardan la automatización con él. Los parámetros tipo knob se muestran 0–127 como en el panel de ReBirth. Los parámetros por voz existen cuatro veces (<span class="id">v1_</span> a <span class="id">v4_</span>); las voces 2–4 empiezan con ON apagado.</p>
{B.param_tables(TEXT['es'])}

<h1>22. Notas técnicas y límites</h1>
<ul>
<li><b>Motor a 44,1 kHz.</b> El motor trabaja siempre a 44,1 kHz en bloques de 64 muestras, porque las tablas y los controles por bloque de ReBirth están hechos para esa frecuencia. El plugin remuestrea a la del proyecto con un sinc de 32 tomas con ventana de Kaiser y sin latencia añadida (el motor se adelanta). Las reverbs y demás efectos también van a 44,1 kHz; el eco de cinta a 176,4 kHz (IV-X va a 192 kHz, así que su filtro de cabezal queda un 8 % más grave).</li>
<li><b>Tiempo real.</b> Nada reserva memoria en el hilo de audio, tampoco los motores en C++: las líneas de retardo de freeverb3, que se redimensionan al cambiar tamaño o programa, salen de una reserva por reverb que se llena al cargar el plugin. Lo comprueban tests que cuentan cada llamada a malloc.</li>
<li><b>Ruido de cinta sintético.</b> Las tablas de ruido grabadas de IV-X no están en su código publicado; Front 303 genera ruido blanco oscurecido según la velocidad de cinta.</li>
<li><b>CPU.</b> Una voz DW-303 cuesta en torno al 20 % de un núcleo a 2,1 GHz; los demás modelos, casi nada. Efectos y reverb juntos, en torno al 10 % de un núcleo.</li>
<li><b>Pruebas.</b> 51 tests automáticos (afinación en todos los modelos, niveles, métricas, transposición MIDI, salidas, todos los modos de efecto y programas de reverb, memorias, deshacer, remuestreo, ausencia de reservas de memoria) y el validador VST3 de Steinberg (47/47).</li>
<li>Las versiones de Windows y macOS las compila GitHub Actions desde el mismo código; la de Linux se hace para glibc 2.30 o posterior.</li>
</ul>

<h1>23. Créditos y licencias</h1>
<p>Front 303 es software libre bajo la <b>Licencia Pública General de GNU, versión 3</b>. Contiene o se basa en:</p>
{t(["Proyecto", "Autor", "Licencia", "Uso en Front 303"], [
 ["OpenRB-338", "Autores de OpenRB-338 (pingw33n)", "GPL-3.0", "Voces 303 de ReBirth, distorsión, PCF, compresor, delay, mezclador"],
 ["DW-303 / RAVET", "DJW Audio (mo0kid)", "GPL-3.0+", "Voz de modelo de circuito, filtros de media banda"],
 ["Open303", "Robin Schmidt; maddanio", "MIT", "Voz Open303"],
 ["Dragonfly Reverb", "Michael Willis, Rob van den Berg", "GPL-3.0+", "Hall y Early Reflections"],
 ["freeverb3", "Teru Kamogashira", "GPL-2.0+", "Bloques de reverb"],
 ["Aether", "Dougal Stewart", "MIT", "Motor Aether"],
 ["CloudReverb / CloudSeed", "xunil-cloud; Valdemar Erlingsson", "GPL-3.0 / MIT", "Motor Cloud y presets"],
 ["IV-X Delay", "Joaquim Broquetas Ruiz", "GPL-3.0", "Eco de cinta, tablas medidas"],
 ["Setekh", "Full FX Media", "GPL-3.0", "Distorsión Setekh"],
 ["Valentine", "Tote Bag Labs", "GPL-3.0", "Compresor Valentine"],
 ["Delay Intervals", "Matthew Fecher", "MIT", "Proceso del delay por intervalos"],
 ["nice-plug, egui", "RustAudio; Emil Ernerfeldt", "ISC; MIT/Apache-2.0", "Framework del plugin y panel"],
 ["Inter", "The Inter Project Authors", "OFL-1.1", "Tipografía del panel"]], ["k", "", "", ""])}
<p>ReBirth RB-338 es marca de Propellerhead Software (Reason Studios); TB-303 y Roland son marcas de Roland Corporation. Front 303 es un proyecto independiente, sin relación con ninguna de ellas ni respaldo suyo, y no contiene nada del propio ReBirth. El panel es arte propio de Front 303, dibujado en código.</p>
"""


TEXT["es"] = {
    "title": "Front 303 — Manual de usuario",
    "right": "Manual de usuario",
    "flow": {"MODEL": "ReBirth · Vintage · DW-303 · Open303", "STRIP": "TIRA DE CANAL",
             "STEMS": "→ pistas (multi-out)", "DLYSEND": "envíos DLY", "REVSEND": "envíos REV + DLY → REV",
             "MASTER": "compresor máster · MASTER", "MAINOUT": "salida principal 1/2 (STEREO)",
             "MULTI": "la principal calla en MULTI-OUT", "MIDI": "notas MIDI → transposición de las voces con MIDI encendido"},
    "param_head": ["Identificador", "Nombre", "Rango", "Defecto", "Función"],
    "group_names": {"voice": "Por voz (×4)", "transport": "Transporte, escala y máster", "dist": "Distorsión",
                    "pcf": "PCF", "comp": "Compresor", "delay": "Delay", "reverb": "Reverb"},
    "voice_desc": ES_VOICE,
    "global_desc": ES_GLOBAL,
    "body": es_body,
}
