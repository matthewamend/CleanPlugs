use nih_plug::prelude::*;
use std::sync::Arc;

struct CleanLimit {
    params: Arc<CleanLimitParams>,
}

impl Default for CleanLimit {
    fn default() -> Self {
        Self {
            params: Arc::new(CleanLimitParams::default()),
        }
    }
}

#[derive(Params)]
struct CleanLimitParams {
    #[id = "threshold"]
    pub threshold: FloatParam,

    #[id = "attack"]
    pub attack: FloatParam,

    #[id = "release"]
    pub release: FloatParam,
}

impl Default for CleanLimitParams {
    fn default() -> Self {
        Self {
            threshold: FloatParam::new(
                "Threshold",
                util::db_to_gain(-30.0),
                FloatRange::Linear {
                    min: util::db_to_gain(-60.0),
                    max: util::db_to_gain(0.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(20.0))
            .with_unit(" db")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            attack: FloatParam::new(
                "Attack",
                3.0,
                FloatRange::Linear {
                    min: util::db_to_gain(-)
                }
            )
        }
    }
}

impl Plugin for CleanLimit {
    const NAME: &'static str = "CleanLimit";
    const VENDOR: &'static str = "Matt Amend";
    const URL: &'static str = "https://youtube.com";
    const EMAIL: &'static str = "email@mail.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),

            aux_input_ports: &[],
            aux_output_ports: &[],

            names: PortNames::const_default(),
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();

    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            let threshold = self.params.threshold.smoothed.next();

            for sample in channel_samples {
                if *sample > threshold {
                    *sample = threshold;
                }
            }
        }

        ProcessStatus::Normal
    }

    fn deactivate(&mut self) {}
}

impl Vst3Plugin for CleanLimit {
    const VST3_CLASS_ID: [u8; 16] = *b"CleanLimitPlugin";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Dynamics];
}

nih_export_vst3!(CleanLimit);
