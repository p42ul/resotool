use fundsp::hacker32::*;
use heapless::{Vec, FnvIndexSet};
use pitch_calc::hz_from_step;


const NUM_INPUTS: usize = 1;
const NUM_OUTPUTS: usize = 1;
const NUM_VOICES: usize = 8;

/// A tunable resonator
pub fn new_resonator(wetdry: Shared<f32>, cutoff: Shared<f32>, bandwidth: Shared<f32>) -> Box<dyn AudioUnit32> {
    Box::new(((var(&wetdry) * pass() | var(&cutoff) | var(&bandwidth)) >> resonator()) & (((1.0 - var(&wetdry)) * pass())))
}

pub struct Voicer {
    voices_off: Vec<Voice, NUM_VOICES>,
    voices_on: Vec<Voice, NUM_VOICES>,
    notes: FnvIndexSet<u8, NUM_VOICES>,
    pub net: Net32,
}

impl Voicer {
    pub fn note_on(&mut self, midi_note: u8) {
        match self.notes.insert(midi_note) {
            Ok(inserted) => {
                if !inserted {
                    return;
                }
            },
            Err(_) => panic!("Couldn't insert MIDI note to FnvHashSet"),
        }
        let mut voice = match self.voices_off.pop() {
            Some(voice) => voice,
            None => {
                let voice = self.voices_on.pop().expect("Stole a voice when there were no voices to steal");
                self.notes.remove(&voice.note);
                voice
            }
        };
        voice.note_on(midi_note);
        if let Err(_) = self.voices_on.insert(0, voice) {
            panic!("Couldn't push voice into voices_on Vec");
        }
    }

    pub fn note_off(&mut self, midi_note: u8) {
        if !self.notes.remove(&midi_note) {
            return;
        }
        for i in 0..NUM_VOICES {
            if let Some(voice) = self.voices_on.get(i) {
                if voice.note == midi_note {
                    voice.note_off(midi_note);
                    self.voices_on.remove(i);
                    // Voicer discards duplicate notes, so a single note
                    // can appear at most once.
                    break;
                }
            }
        }
    }
    
    pub fn new() -> Self {
        let mut voices_off = Vec::<Voice, NUM_VOICES>::new();
        let voices_on = Vec::<Voice, NUM_VOICES>::new();
        let mut net = Net32::new(NUM_INPUTS, NUM_OUTPUTS);
        for _ in 0..NUM_VOICES {
            let voice = Voice::new();
            let id = net.push(new_resonator(voice.wetdry.clone(), voice.cutoff.clone(), voice.bandwidth.clone()));
            match voices_off.push(voice) {
                Ok(_) => {
                    net.pipe_input(id);
                    net.pipe_output(id);
                }
                Err(_) => panic!("Couldn't initialize {} voices", NUM_VOICES),
            }
        }
        Self {
            voices_on: voices_on,
            voices_off: voices_off,
            notes: FnvIndexSet::<u8, NUM_VOICES>::new(),
            net: net,
        }
    }
}

struct Voice {
    note: u8,
    cutoff: Shared<f32>,
    wetdry: Shared<f32>,
    bandwidth: Shared<f32>,
}

impl Voice {
    fn on(&self) {
        self.wetdry.set(1.0);
    }

    fn off(&self) {
        self.wetdry.set(0.0);
    }


    pub fn note_on(&mut self, midi_note: u8) {
        self.on();
        self.note = midi_note;
        self.cutoff.set(hz_from_step(midi_note.into()));
    }

    pub fn note_off(&self, _midi_note: u8) {
        self.off();
    }

    pub fn new() -> Self {
        Self {
            note: 64,
            cutoff: Shared::new(440.0),
            wetdry: Shared::new(0.0),
            bandwidth: Shared::new(100.0),
        }
    }
}