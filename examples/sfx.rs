//! This is an example of playing a sound effect preset. For playing your own sound effect file,
//! please see the `sound` example.

use rusty_engine::prelude::*;

rusty_engine::init!();

fn main() {
    let mut game = Game::new();
    let msg = game.add_text(
        "msg",
        "You can play sound effect presets that are included in the asset pack. For example:",
    );
    msg.translation.y = 50.0;

    let msg2 = game.add_text(
        "msg2",
        "engine_state.audio_manager.play_sfx(SfxPreset::Jingle1, 1.0);",
    );
    msg2.translation.y = -50.0;
    msg2.font = "FiraMono-Medium.ttf".to_string();

    game.audio_manager.play_sfx(SfxPreset::Jingle1, 1.0);

    game.run(());
}
