# miditoqwerty-rs

Simple MIDI input to QWERTY output tool for macOS, Linux, and Windows.

Downloads can be found at https://github.com/ArijanJ/miditoqwerty-rs/releases

## Sostenuto

The app presses right bracket (`]`) if it detects sostenuto pedal input. Make sure you bind this button in your game/app if you wish to use a sostenuto pedal.

## Troubleshooting on macOS

If you are on macOS and notes aren't being played but you are [sure your piano works](https://hardwaretester.com/midi), it's probably a permission issue.
You need to grant the app accessibility permissions which allows it to send keystrokes.

If you encounter issues after running a new version of the app, and for general troubleshooting, you can reset all permissions by executing this in your Terminal:
`sudo tccutil reset All dev.arijanj.miditoqwerty`

After this, restart the app and switch your "Output Method" to a different one so that the permission prompt comes back up.

## Alternatives
- [Windows] - https://github.com/ArijanJ/miditoqwerty - predecessor to this app, has more customizability
- [Windows] - https://github.com/Zephkek/MIDIPlusPlus - vibecoded, has a MIDI autoplayer

## License
**MIT** - You are encouraged to reference the output implementations, as they should handle most edge cases properly.
