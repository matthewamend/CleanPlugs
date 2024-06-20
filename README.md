# CleanPlugs

CleanPlugs is a series of related VST3 plugins, created to help learn how to make audio plugins with Rust and nih-plug, as well as DSP in general. All plugins are free and open source. 

## Current Plugins

* CleanLimit - A Simple Limiter, made as a test. It doesn't have many features, nor is it likely to. It is superseded by CleanComp (which has a limiting feature), and it contains no GUI.

* CleanComp - A simple compressor with your basic threshold, ratio, attack, and release. It features no fancy DSP bells and whistles.

## Installation

Currently, each plugin is its own crate. For right now, you can simply follow these instructions, substituting `plugin_name` with the actual plugin:

```bash
cd plugin_name
cargo xtask bundle plugin_name --release
```
These build steps will supply the VST3 plugins into the target/bundled folder within the plugin crate. You can simply move these to whatever directory you use to store plugins, your call.

## Disclaimer

As stated above, these plugins will be of little use to most, since your garden variety DAW will have built-in plugins that do the same exact thing, but better. Also don't expect active maintenance; this is not a project meant for serious, high-quality use.

## Contributing

If you feel like contributing, feel free to open a pull request. These are meant mostly as a learning tool, but I would still appreciate it :-)

## License
[GNU GPLv3](https://choosealicense.com/licenses/gpl-3.0/)