# Changelog

All notable changes to Husky Bounce are documented here.

## [Unreleased]

### Changed

- Levels last 179 s instead of 180 s: a 180 s level gave a 180.05 s video,
  just over the 3:00 limit for Shorts.
- Framework files updated from RustyHusky: videos stop at 3:00
  (`MAX_VIDEO_SECONDS`), the REC marker shows the limit.

### Added

- Started from the RustyHusky template 0.2.0 (plus its unreleased changes).
- The game: balls bounce in rotating rings, bounces play notes and add
  balls, escapes score; 180-second levels that build steadily to chaos;
  levels 1–4 add rings; one fixed seed per level.
- Framework tests use their own test game, so the example could be removed.
- `View::image` and `Input::clicked` are marked as game API, so a game that
  does not use them still passes clippy.
