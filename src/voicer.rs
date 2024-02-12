use fundsp::hacker32::*;
use nih_plug::prelude::*;
use pitch_calc::hz_from_step;

const NUM_INPUTS: usize = 1;
const NUM_OUTPUTS: usize = 1;
const NUM_VOICES: usize = 4;

/// A tunable resonator
fn new_resonator(voice: &Voice) -> Box<dyn AudioUnit32> {
    Box::new(
        (var(&voice.wetdry) * pass() | var(&voice.cutoff) | var(&voice.bandwidth)) >> resonator(),
    )
}

pub struct Voicer {
    last_played: u32,
    voices: Box<[Voice; NUM_VOICES]>,
    pub net: Net32,
}

impl Voicer {
    pub fn note_on(&mut self, midi_note: u8) {
        self.last_played += 1;
        for i in 0..NUM_VOICES {
            if !(self.voices[i].sounding) {
                self.voices[i].note_on(midi_note, self.last_played);
                return;
            }
        }
        // All voices are sounding, interrupt the last-used voice
        let mut voice_index: usize = 0;
        for i in 0..NUM_VOICES {
            if self.voices[i].last_played < self.voices[voice_index].last_played {
                voice_index = i;
            }
        }
        self.voices[voice_index].note_on(midi_note, self.last_played);
    }

    pub fn note_off(&mut self, midi_note: u8) {
        for i in 0..NUM_VOICES {
            if self.voices[i].note == midi_note {
                self.voices[i].note_off(midi_note);
            }
        }
    }

    pub fn new() -> Self {
        let voices: [Voice; NUM_VOICES] = std::array::from_fn(|_| Voice::default());
        let mut net = Net32::new(NUM_INPUTS, NUM_OUTPUTS);
        for voice in voices.iter() {
            let resonator = new_resonator(&voice);
            let id = net.push(resonator);
            todo!("have to split() and then join() the filterbank somehow");
        }
        Self {
            last_played: 0,
            voices: Box::new(voices),
            net: net,
        }
    }
}

struct Voice {
    last_played: u32,
    note: u8,
    sounding: bool,
    cutoff: Shared<f32>,
    wetdry: Shared<f32>,
    bandwidth: Shared<f32>,
}

impl Default for Voice {
    fn default() -> Self {
        Self {
            last_played: 0,
            note: 64,
            sounding: false,
            cutoff: Shared::new(440.0),
            wetdry: Shared::new(0.0),
            bandwidth: Shared::new(100.0),
        }
    }
}

impl Voice {
    fn note_on(&mut self, midi_note: u8, last_played: u32) {
        self.wetdry.set(1.0);
        self.note = midi_note;
        self.sounding = true;
        self.cutoff.set(hz_from_step(midi_note.into()));
        self.last_played = last_played;
    }

    fn note_off(&mut self, _midi_note: u8) {
        self.wetdry.set(0.0);
        self.sounding = false;
    }

}
