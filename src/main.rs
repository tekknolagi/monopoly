mod game;
use game::{Action, GameState, PlayerId, RollResult, StateError};
use std::error::Error;
fn main() -> Result<(), StateError> {
    let mut state = GameState::init();
    state.apply(Action::RollDice(PlayerId(0), RollResult(1, 2)))?;
    println!("state: {:?}", state);
    Ok(())
}
