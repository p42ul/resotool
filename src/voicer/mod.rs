use fundsp::hacker32::*;
use nih_plug::util::midi_note_to_freq;

mod adsr;

// Make sure to change these together
const NUM_VOICES: usize = 8;
type VoiceSize = U8;

pub struct Voicer {
    last_played: u32,
    voices: Box<[Voice; NUM_VOICES]>,
    pub audio: Box<dyn AudioUnit32>,
}

impl Voicer {
    pub fn set_bandwidth(&self, bandwidth: f32) {
        for voice in self.voices.iter() {
            voice.bandwidth.set(bandwidth);
        }
    }

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
        let filterbank = stack::<VoiceSize, _, _>(|i| {
            let voice = &voices[i as usize];
            (
                pass() *
                (
                    var(&voice.trigger) >>
                    (adsr::adsr_shared(voice.adsr.attack.clone(),
                                       voice.adsr.decay.clone(),
                                       voice.adsr.sustain.clone(),
                                       voice.adsr.release.clone())))
                    | var(&voice.cutoff) | var(&voice.bandwidth
                )
            )
                >> resonator()
        });
        let audio = Box::new(split() >> filterbank >> join());
        Self {
            last_played: 0,
            voices: Box::new(voices),
            audio: audio,
        }
    }
}

struct Voice {
    last_played: u32,
    note: u8,
    sounding: bool,
    trigger: Shared<f32>,
    adsr: adsr::Adsr,
    cutoff: Shared<f32>,
    bandwidth: Shared<f32>,
}

impl Default for Voice {
    fn default() -> Self {
        Self {
            last_played: 0,
            note: 64,
            sounding: false,
            trigger: Shared::new(0.0),
            cutoff: Shared::new(440.0),
            bandwidth: Shared::new(100.0),
            adsr: adsr::Adsr::default(),
        }
    }
}

impl Voice {
    fn note_on(&mut self, midi_note: u8, last_played: u32) {
        self.trigger.set(1.0);
        self.note = midi_note;
        self.sounding = true;
        self.cutoff.set(midi_note_to_freq(midi_note));
        self.last_played = last_played;
    }

    fn note_off(&mut self, _midi_note: u8) {
        self.trigger.set(0.0);
        self.sounding = false;
    }
}
