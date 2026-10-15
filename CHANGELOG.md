# Changelog

All notable changes to Husky Bounce are documented here.

## [Unreleased]

### Added

- Started from the RustyHusky template 0.2.0 (plus its unreleased changes).
- The game: balls bounce in rotating rings, bounces play notes and add
  balls, escapes score; 180-second levels that build steadily to chaos;
  levels 1–4 add rings; one fixed seed per level.
- Framework tests use their own test game, so the example could be removed.
- `View::image` and `Input::clicked` are marked as game API, so a game that
  does not use them still passes clippy.
