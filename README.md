This is a very basic TS3 client plugin that mutes everyone except one specific person.

Example use-case: you want to record a gameplay video but also want to have the voice of the commander in it
so that the actions of you and the group have some context.
However, you might only want to record the commander and not any other players.
This could be for clarity but also most likely for consent reasons.

This plugin is currently NOT ready for easy use.
The UID of the person that is not muted is hardcoded and therefore the plugin needs to be compiled specifically.
Also, errors are not handled properly and flood protection might fuck it up.

The plugin basically just monitors client movements.
If a client joins your channel, it is muted (via the normal TS3 mute function).
If you join another channel, every client is muted there (might break because of flood protection).

I currently only built and tested the plugin on Linux.

To build it, rust needs to be installed including the compiler you need for you platform.
Just calling
```shell
cargo build
```
or
```shell
cargo build --release
```
should suffice.
You can find the `.so` or `.dll` in the `target/debug` or `target/release` folder.

Since using this plugin mutes pretty much everyone, your TS3 profile will be fucked.
MAKE SURE YOU USE A SEPARATE TS3 INSTANCE AND CONFIG!
So you would have you normal instance and using it normally and run the special one alongside.

I start the special instance with:
```shell
(cd ~/Programs/TeamSpeak3-capture2;./ts3client_runscript.sh -nosingleinstance -homeconfig ~/Programs/TeamSpeak3-capture2/config)
```

Considerations for configuring it:
* Microphone: to be sure, make an invalid config or at least use push to talk with a weird/unset key.
* Playback: leave as is because it probably worked for you.
* Hotkeys: disable them all because you do not want to interact with this TS3 instance anyway.
* Notifications: disable them all (Soundpack: "Sounds deactivated" should work). Otherwise, you will get all "user joined" notifications and stuff.


