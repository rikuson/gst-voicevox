use gst::{caps::NoFeature, glib, subclass::prelude::*, Caps, CapsIntersectMode, PadDirection};
use gst_audio::subclass::prelude::*;
use gst_audio::{AudioCapsBuilder, AUDIO_FORMAT_F32};
use gst_base::subclass::{base_transform::GenerateOutputSuccess, BaseTransformMode};
use gst_base::BaseTransform;
use once_cell::sync::Lazy;
use std::io::Cursor;
use std::str;
use vvcore::*;
use wavers::{ReadSeek, Wav};

static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        "voicevox",
        gst::DebugColorFlags::empty(),
        Some("Text to speech filter using VOICEVOX"),
    )
});

static SRC_CAPS: Lazy<Caps> = Lazy::new(|| src_caps_builder().build());
static SINK_CAPS: Lazy<Caps> =
    Lazy::new(|| Caps::builder("text/x-raw").field("format", "utf8").build());

fn src_caps_builder() -> AudioCapsBuilder<NoFeature> {
    AudioCapsBuilder::new().format(AUDIO_FORMAT_F32).channels(1)
}

#[derive(Default)]
pub struct TTS {}

impl TTS {
    fn text_to_speech(text: &str) -> Vec<u8> {
        // FIXME: hardcoded path
        let dir = std::ffi::CString::new("/usr/local/lib/open_jtalk_dic_utf_8-1.11").unwrap();
        let vvc = VoicevoxCore::new_from_options(AccelerationMode::Auto, 0, false, dir.as_c_str())
            .unwrap();

        let speaker: u32 = 1;
        vvc.load_model(speaker).unwrap();
        let wav = vvc.tts_simple(text, speaker).unwrap();

        wav.as_slice().to_vec()
    }
}

impl ObjectImpl for TTS {}
impl GstObjectImpl for TTS {}
impl ElementImpl for TTS {
    fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
        static ELEMENT_METADATA: Lazy<gst::subclass::ElementMetadata> = Lazy::new(|| {
            gst::subclass::ElementMetadata::new(
                "VOICEVOX",
                "Converter/Text/Audio",
                "Text to speech filter using VOICEVOX",
                "Riku Takeuchi <rikuson@users.noreply.github.com>",
            )
        });

        Some(&*ELEMENT_METADATA)
    }

    fn pad_templates() -> &'static [gst::PadTemplate] {
        static PAD_TEMPLATES: Lazy<Vec<gst::PadTemplate>> = Lazy::new(|| {
            let src_pad_template = gst::PadTemplate::new(
                "src",
                gst::PadDirection::Src,
                gst::PadPresence::Always,
                &SRC_CAPS,
            )
            .unwrap();

            let sink_pad_template = gst::PadTemplate::new(
                "sink",
                gst::PadDirection::Sink,
                gst::PadPresence::Always,
                &SINK_CAPS,
            )
            .unwrap();

            vec![src_pad_template, sink_pad_template]
        });

        PAD_TEMPLATES.as_ref()
    }
}
impl BaseTransformImpl for TTS {
    const MODE: BaseTransformMode = BaseTransformMode::NeverInPlace;
    const PASSTHROUGH_ON_SAME_CAPS: bool = false;
    const TRANSFORM_IP_ON_PASSTHROUGH: bool = false;

    fn transform_caps(
        &self,
        direction: PadDirection,
        _caps: &Caps,
        maybe_filter: Option<&Caps>,
    ) -> Option<Caps> {
        let mut caps = if direction == PadDirection::Src {
            SINK_CAPS.clone()
        } else {
            let sample_rate = 22050;
            //   src_caps_builder().build()
            src_caps_builder().rate(sample_rate as i32).build()
        };
        if let Some(filter) = maybe_filter {
            caps = filter.intersect_with_mode(&caps, CapsIntersectMode::First);
        }
        Some(caps)
    }

    fn generate_output(&self) -> Result<GenerateOutputSuccess, gst::FlowError> {
        if let Some(buffer) = self.take_queued_buffer() {
            let buffer_reader = buffer
                .as_ref()
                .map_readable()
                .map_err(|_| gst::FlowError::Error)?;
            let text =
                str::from_utf8(buffer_reader.as_slice()).map_err(|_| gst::FlowError::Error)?;
            let wav = TTS::text_to_speech(&text);

            let cursor = Cursor::new(wav.as_slice().to_vec());
            let wav_reader: Box<dyn ReadSeek> = Box::new(std::io::BufReader::new(cursor));
            let mut wav: Wav<f32> = Wav::new(wav_reader).unwrap();

            let samples = wav.read().unwrap();

            let bytes = samples.as_bytes();

            let mut buffer: gst::Buffer =
                gst::Buffer::with_size(bytes.len()).map_err(|_| gst::FlowError::Error)?;
            buffer
                .get_mut()
                .unwrap()
                .copy_from_slice(0, bytes)
                .map_err(|_| gst::FlowError::Error)?;
            Ok(GenerateOutputSuccess::Buffer(buffer))
        } else {
            println!("generate_output(): no queued buffers to take");
            Ok(GenerateOutputSuccess::NoOutput)
        }
    }
}
#[glib::object_subclass]
impl ObjectSubclass for TTS {
    const NAME: &'static str = "GstVoiceVox";
    type Type = super::TTS;
    type ParentType = BaseTransform;
}
