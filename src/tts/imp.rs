use std::str;
use gst::glib;
use gst_base::BaseTransform;
use gst_base::subclass::{BaseTransformMode, base_transform::GenerateOutputSuccess};
use gst_audio::subclass::prelude::*;
use once_cell::sync::Lazy;
use vvcore::*;

static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        "voicevoxtts",
        gst::DebugColorFlags::empty(),
        Some("Text to speech filter using VOICEVOX"),
    )
});

#[derive(Default)]
pub struct TTS {}

impl TTS {
    fn text_to_speech(text: &str) -> Vec<u8> {
        // FIXME: hardcoded path
        let dir = std::ffi::CString::new("/usr/local/lib/open_jtalk_dic_utf_8-1.11").unwrap();
        let vvc = VoicevoxCore::new_from_options(AccelerationMode::Auto, 0, true, dir.as_c_str()).unwrap();

        let speaker: u32 = 1;
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
                "VOICEVOX TTS",
                "Converter/Text/Audio",
                "Text to speech filter using VOICEVOX",
                "Riku Takeuchi <rikuson@users.noreply.github.com>",
            )
        });

        Some(&*ELEMENT_METADATA)
    }

    fn pad_templates() -> &'static [gst::PadTemplate] {
        static PAD_TEMPLATES: Lazy<Vec<gst::PadTemplate>> = Lazy::new(|| {
            let caps = gst_audio::AudioCapsBuilder::new()
                .format(gst_audio::AUDIO_FORMAT_F32)
                .build();
            let src_pad_template =
            gst::PadTemplate::new("src", gst::PadDirection::Src, gst::PadPresence::Always, &caps).unwrap();

            let caps = gst::Caps::builder("text/x-raw").field("format", "utf8").build();
            let sink_pad_template = gst::PadTemplate::new(
                "sink",
                gst::PadDirection::Sink,
                gst::PadPresence::Always,
                &caps,
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

    fn generate_output(&self) -> Result<GenerateOutputSuccess, gst::FlowError> {
        if let Some(buffer) = self.take_queued_buffer() {
            let buffer_reader = buffer
                .as_ref()
                .map_readable()
                .map_err(|_| gst::FlowError::Error)?;
            let text = str::from_utf8(buffer_reader.as_slice()).map_err(|_| gst::FlowError::Error)?;
            let wav = TTS::text_to_speech(&text);
            let mut buffer = gst::Buffer::with_size(wav.len()).map_err(|_| gst::FlowError::Error)?;
            {
                let buffer_ref = buffer.get_mut().ok_or(gst::FlowError::Error)?;
                let mut map = buffer_ref.map_writable().map_err(|_| gst::FlowError::Error)?;

                map.as_mut_slice().copy_from_slice(&wav);
            }
            Ok(GenerateOutputSuccess::Buffer(buffer))
        }
        else {
            println!( "generate_output(): no queued buffers to take");
            Ok(GenerateOutputSuccess::NoOutput)
        }
    }
}
#[glib::object_subclass]
impl ObjectSubclass for TTS {
    const NAME: &'static str = "GstVoiceVoxTTS";
    type Type = super::TTS;
    type ParentType = BaseTransform;
}
