
<img src="./readme_images/maslow.png" width="400">

UEFIRC is a graphical UEFI IRC client. Connect to an IRC server, chat and read messages, all from the comfort of your motherboard's pre-boot environment.

_50 years of computing, yet no solution for IRC software that can also fry your motherboard. Until now._

UEFIRC is written in Rust and leverages [uefi-rs](https://github.com/rust-osdev/uefi-rs). 

The GUI toolkit comes from [axle OS](https://github.com/codyd51/axle). I think that this is the first time anyone has put TrueType in UEFI.

_NO kernel. NO GUI toolkit. NO scheduler. NO memory protection. Just you, the motherboard firmware, and all your pals across the internet._

<img src="./readme_images/qemu_screenshot.png" width="400">

UEFIRC comes with keyboard and mouse support by leveraging UEFI's [Simple Text Protocol](https://uefi.org/specs/UEFI/2.9_A/12_Protocols_Console_Support.html#efi-simple-text-input-protocol) and [Simple Pointer Protocol](https://uefi.org/specs/UEFI/2.9_A/12_Protocols_Console_Support.html#simple-pointer-protocol).

Most notably, UEFIRC implements a memory-safe wrapper around UEFI's TCP implementation. 

Here's me saying hello to the UEFI development channel from UEFI itself.

<img src="./readme_images/edk_hello.png" width="400">

UEFIRC comes with a [blog post](https://axleos.com/an-irc-client-in-your-motherboard/) with fun animations showing some of the tricky bits it took to get this working. Check it out!

## Configuration

The IRC server, and the identity of the user, are controlled by a configuration file in the EFI filesystem.

Modify `configuration.toml` to change these values.

## Running

UEFIRC uses a small (no-dependency) Python 3.7+ script to coordinate the build. Build and run:

```bash
$ python3 scripts/build.py
```

## Should I use this?

_This should not exist._

## License

MIT license.
