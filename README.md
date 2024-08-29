# gst-voicevox

A GStreamer Text-to-speech element using VOICEVOX

## Installation

```shell
git clone git@github.com:rikuson/gst-voicevox.git
cd gst-voicevox
cargo build
export GST_PLUGIN_PATH=`pwd`/target/release
```

## Example usage

[VOICEVOX CORE](https://github.com/VOICEVOX/voicevox_core) must be installed.

```shell
gst-launch-1.0 --quiet fdsrc ! 'text/x-raw,format=utf8' ! voicevox ! autoaudiosink
```
