use atomic_float::AtomicF32;
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::sync::Arc;

mod editor;

/// The time it takes to decay by 12 dB after switching to complete silence
const PEAK_METER_DECAY_MS: f64 = 150.0;

pub struct CleanComp {
    params: Arc<CleanCompParams>,
    sample_rate: f32,

    /// GUI-related items
    peak_meter_decay_weight: f32,
    peak_meter: Arc<AtomicF32>,
}

impl Default for CleanComp {
    fn default() -> Self {
        Self {
            params: Arc::new(CleanCompParams::default()),
            sample_rate: 44100.0,

            peak_meter_decay_weight: 1.0,
            peak_meter: Arc::new(AtomicF32::new(util::MINUS_INFINITY_DB)),
        }
    }
}

#[derive(Params)]
struct CleanCompParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,
    /// The dB threshold for when the compressor sets in
    #[id = "threshold"]
    pub threshold: FloatParam,

    /// The ratio of how much to compress audio above the threshold
    /// e.g. 4.0:1 at 32 dB with a threshold of 28 will return 29 dB
    #[id = "ratio"]
    pub ratio: FloatParam,

    /// If true compress, if false limit (i.e. infinite ratio)
    #[id = "comp_or_limit"]
    pub comp_or_limit: EnumParam<CompType>,

    #[id = "attack"]
    /// How long it takes for the compressor to set in, in milliseconds
    pub attack: FloatParam,

    #[id = "release"]
    /// How long before the compressor stops, in milliseconds
    pub release: FloatParam,
}

#[derive(Enum, PartialEq)]
enum CompType {
    #[id = "compress"]
    Compress,

    #[id = "limit"]
    Limit,
}

impl Default for CleanCompParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),

            threshold: FloatParam::new(
                "Threshold",
                util::db_to_gain(-20.0),
                FloatRange::Linear {
                    min: util::db_to_gain(-60.0),
                    max: util::db_to_gain(0.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(20.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            ratio: FloatParam::new(
                "Ratio",
                2.0,
                FloatRange::Linear {
                    min: 1.0,
                    max: 20.0,
                },
            )
            .with_value_to_string(formatters::v2s_compression_ratio(2))
            .with_string_to_value(formatters::s2v_compression_ratio()),

            comp_or_limit: EnumParam::new("Compress/Limit", CompType::Compress),

            attack: FloatParam::new(
                "Attack",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_unit(" ms")
            .with_value_to_string(Arc::new(|x| format!("{:2}", x)))
            .with_string_to_value(Arc::new(|x: &str| x.parse::<f32>().ok())),

            release: FloatParam::new(
                "Release",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_unit(" ms")
            .with_value_to_string(Arc::new(|x| format!("{:2}", x)))
            .with_string_to_value(Arc::new(|x: &str| x.parse::<f32>().ok())),
        }
    }
}

fn process_comp(sample: f32, comp_or_limit: &CompType, ratio: f32, threshold: f32) -> f32 {
    if sample > threshold {
        match comp_or_limit {
            CompType::Compress => ((sample - threshold) / ratio) + sample,
            CompType::Limit => threshold,
        }
    } else {
        sample
    }
}

impl CleanComp {
    fn set_attack_count(&self) -> i32 {
        let attack = self.params.attack.smoothed.next();

        if attack != 0.0 {
            self.sample_rate as i32 / (1000.0 * attack) as i32
        } else {
            0
        }
    }

    fn set_release_count(&self) -> i32 {
        let release = self.params.release.smoothed.next();

        if release != 0.0 {
            self.sample_rate as i32 / (1000.0 * release) as i32
        } else {
            0
        }
    }
}

impl Plugin for CleanComp {
    const NAME: &'static str = "CleanComp";
    const VENDOR: &'static str = "MSAP Plugins LLC";
    const URL: &'static str = "https://localhost";
    const EMAIL: &'static str = "msap@localhost";

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

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.peak_meter.clone(),
            self.params.editor_state.clone(),
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        nih_log!("Changing sample rate to {}", buffer_config.sample_rate);

        self.peak_meter_decay_weight = 0.25f64
            .powf((buffer_config.sample_rate as f64 * PEAK_METER_DECAY_MS / 1000.0).recip())
            as f32;

        self.sample_rate = buffer_config.sample_rate;

        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            let threshold = self.params.threshold.smoothed.next();
            let ratio = self.params.ratio.value();
            let comp_or_limit = self.params.comp_or_limit.value();

            let default_attack_count = self.set_attack_count();
            let default_release_count = self.set_release_count();

            let mut attack_countdown = default_attack_count;

            let mut release_countdown = default_release_count;

            let mut amplitude = 0.0;
            let num_samples = channel_samples.len();

            for sample in channel_samples {
                nih_log!("sample: {}", *sample);

                if *sample > threshold {
                    attack_countdown -= 1;
                } else {
                    attack_countdown = default_attack_count;
                }

                if attack_countdown <= 0 || release_countdown <= 0 {
                    *sample = process_comp(*sample, &comp_or_limit, ratio, threshold);

                    release_countdown = default_release_count;
                } else if release_countdown > 0 {
                    release_countdown -= 1;
                }
            }

            if self.params.editor_state.is_open() {
                amplitude = (amplitude / num_samples as f32).abs();
                let current_peak_meter = self.peak_meter.load(std::sync::atomic::Ordering::Relaxed);

                let new_peak_meter = if amplitude > current_peak_meter {
                    amplitude
                } else {
                    current_peak_meter * self.peak_meter_decay_weight
                        + amplitude * (1.0 - self.peak_meter_decay_weight)
                };

                self.peak_meter
                    .store(new_peak_meter, std::sync::atomic::Ordering::Relaxed)
            }
        }

        ProcessStatus::Normal
    }
}

impl Vst3Plugin for CleanComp {
    const VST3_CLASS_ID: [u8; 16] = *b"CleanCompisgreat";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Dynamics];
}

nih_export_vst3!(CleanComp);
