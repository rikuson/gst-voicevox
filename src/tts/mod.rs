use gst::glib;
use gst::prelude::*;

mod imp;

glib::wrapper! {
    pub struct TTS(ObjectSubclass<imp::TTS>) @extends gst_base::BaseTransform, gst::Element, gst::Object;
}

// Registers the type for our element, and then registers in GStreamer under
// the name "rsrgb2gray" for being able to instantiate it via e.g.
// gst::ElementFactory::make().
pub fn register(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    gst::Element::register(
        Some(plugin),
        "voicevox",
        gst::Rank::NONE,
        TTS::static_type(),
    )
}
