[![Build Status](https://travis-ci.org/bfops/playform.svg?branch=master)](https://travis-ci.org/bfops/playform)

## Overview

An interactive, modifiable voxel sandbox project in Rust, inspired in part by [Voxel Farm](http://procworld.blogspot.com/) and Minecraft. I try to keep a dev blog [here](http://playformdev.blogspot.com/).

It's very much a WIP. Hopefully as the Rust ecosystem grows (and, in a perfect world, when Rust gets a story for linking with C++),
the hackiest parts of Playform can be outsourced to other libraries (physics and graphics APIs, threading, networking).

Some picture things (perpetually outdated):

![screenshot 1](/../screenshots/screenshots/screenshot1.png?raw=true)
![screenshot 5](/../screenshots/screenshots/screenshot5.png?raw=true)
![screenshot 2](/../screenshots/screenshots/screenshot2.png?raw=true)
![screenshot 3](/../screenshots/screenshots/screenshot3.png?raw=true)
![screenshot 4](/../screenshots/screenshots/screenshot4.png?raw=true)

## Making it work

Make sure you have:

  * A **nightly** build of the Rust compiler and cargo (if it doesn't build on latest, file an issue)
  * OpenGL 3.3+
  * SDL2
  * SDL2\_ttf
  * libnanomsg
  * portaudio

Playform has a separate server and client, which can be built and run in `server/bin` and `client/bin`,
but there's also a server+client (singleplayer) bundled binary that builds in the root directory.

`cargo build --release` and `cargo run --release` are pretty much required to run Playform with reasonable performance.

## How to play

  * Move: WASD
  * Jump: Space
  * Look around: Mouse
  * Tree tool: Left mouse button (this is slow)
  * Dig tool: Right mouse button
  * Toggle HUD: H

One mob spawns that will play "tag" with you: tag it and it will chase you until it tags you back. If you get too far away from it, it'll probably get lost and fall through the planet. It's a little needy.

## License & Credit

I'm not the most familiar with licensing. If I've done something wrong, let me know. My intent is that Playform itself is MIT licensed (see the LICENSE file).
It includes some code that can be easily found online, and that code usually comes with links to the online source.

Some of the Assets are not mine, and I don't own the rights to them. In particular, thanks to:

  * [http://vector.me/browse/104477/free\_vector\_grass](http://vector.me/browse/104477/free_vector_grass) for the textures used for the grass billboards
  * [http://soundbible.com/1818-Rainforest-Ambience.html](http://soundbible.com/1818-Rainforest-Ambience.html) for the awesome ambient sound
  * [http://soundbible.com/1432-Walking-On-Gravel.html](http://soundbible.com/1432-Walking-On-Gravel.html) for the footstep sounds
