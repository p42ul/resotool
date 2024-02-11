use nih_plug::prelude::{Buffer, *};
use fundsp::hacker32::*;
use voicer::Voicer;
use std::sync::Arc;

mod voicer;

struct Resotool {
    params: Arc<ResotoolParams>,
    bandwidth: Shared<f32>,
    voicer: Voicer,
}


#[derive(Params)]
struct ResotoolParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish.
    #[id = "cutoff"]
    pub bandwidth: FloatParam,
}

impl Default for Resotool {
    fn default() -> Self {
        let params = ResotoolParams::default();
        let bandwidth = shared(params.bandwidth.default_plain_value());
        Self {
            params: Arc::new(params),
            bandwidth: bandwidth,
            voicer: Voicer::new(),
        }
    }
}

impl Default for ResotoolParams {
    fn default() -> Self {
        Self {
            bandwidth: FloatParam::new("Q", 440.0, FloatRange::SymmetricalSkewed { min: 1.0, max: 100.0, factor: 0.75, center: 40.0 }),
        }
    }
}

impl Plugin for Resotool {
    const NAME: &'static str = "Resotool";
    const VENDOR: &'static str = "Paul Buser";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "paul@beepyversion.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(1),
        main_output_channels: NonZeroU32::new(1),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    }];


    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        self.voicer.net.set_sample_rate(_buffer_config.sample_rate.into());
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.bandwidth.set(self.params.bandwidth.value());
        while let Some(event) = context.next_event() {
            match event {
                NoteEvent::NoteOn { note, .. } => {
                    self.voicer.note_on(note);
                }
                NoteEvent::NoteOff { note, .. } => {
                    self.voicer.note_off(note);
                }
                _ => (),
            }
        }
        for channel_samples in buffer.iter_samples() {
            // Smoothing is optionally built into the parameters themselves
            let output_buffer = &mut [0f32; 1];
            for sample in channel_samples {
                self.voicer.net.tick(&[*sample], output_buffer);
                *sample = output_buffer[0];
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Resotool {
    const CLAP_ID: &'static str = "com.beepyversion.resotool";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("MIDI controllable resonators");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for Resotool {
    const VST3_CLASS_ID: [u8; 16] = *b"ResoToolCoolTool";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(Resotool);
nih_export_vst3!(Resotool);
