# Husky Bounce

**One ball. Then chaos.**

A portrait game that mostly plays itself: a ball bounces inside rotating
rings, every bounce plays a note and may add a ball, and three minutes later
hundreds of balls pour out through the gaps. Made to be recorded as a
YouTube Short, TikTok or Instagram Reel: every level is exactly 180 seconds
and is recorded automatically as a 1080 × 1920, 60 fps MP4 with sound.

The rules are in [docs/GAME_RULES.md](docs/GAME_RULES.md).

## Running

Needs Rust 1.87 or later. On Linux, also the X11, ALSA and OpenGL
development packages (Debian/Ubuntu: `sudo apt install libx11-dev libxi-dev
libgl1-mesa-dev libasound2-dev`). For recording, FFmpeg with libx264 in
`PATH` (`sudo apt install ffmpeg`, `brew install ffmpeg`, or an FFmpeg build
for Windows).

```bash
cargo run --release
```

Press Play and watch. Space or a click adds a ball; Esc pauses. After the
level, choose **Save video** or **Discard video**. Videos are saved in your
video folder under `Husky Bounce`.

## Built on RustyHusky

Husky Bounce is built on the [RustyHusky](https://github.com/ricande/RustyHusky)
template (0.2.0 plus unreleased changes), which provides the menus,
settings, scaling, recording and translations. The game itself is
`src/bounce.rs`; the framework's documentation is kept in `docs/` and
`AGENTS.md`.

## License

GPL-3.0-or-later, like RustyHusky; see [LICENSE](LICENSE) and
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md). The notes and the escape
sound in `assets/` were generated with FFmpeg for this game.
