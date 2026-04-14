# Roadmap

## Interaction

- **Tap Tempo**: A button or keyboard shortcut to tap a rhythm and derive the base speed from the tap interval.
- **MIDI Output**: Use `navigator.requestMIDIAccess()` to send MIDI note-on/note-off messages, mapping each orbit to a configurable MIDI note and channel.
- **MIDI Input**: Accept MIDI CC messages for real-time parameter control (speed, base frequency, subdivisions).
- **Preset System**: Save and load named configurations via `localStorage`.

## Export

- **Record to WAV/WebM**: Use `MediaRecorder` on `AudioContext.createMediaStreamDestination()` to capture audio output, optionally with canvas video via `canvas.captureStream()`.

## Sequencing

- **Sequence Mode**: Cycle through a list of ratios automatically on each convergence event, creating evolving compositions.
- **Timeline**: A timeline view where ratio changes, speed ramps, and tuning switches can be scheduled and played back as a composition.

## Platform

- **SoundFont / Sample Audio**: Replace or supplement oscillator synthesis with SoundFont or sample-based playback for richer timbres.
- **Mobile Touch**: Touch-friendly controls, swipe gestures for speed, and pinch-to-zoom on the canvas.
- **Web Controls Collapse**: Toggleable controls panel with Tab key and click affordance.
