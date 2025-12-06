
mod localization;
mod state;

use crate::state::FormatApperr;
use std::process;
use whitedew_core::{apperr, sys};

fn main() -> process::ExitCode {
    println!("Welcome, WhiteDew!");

    match run() {
        Ok(()) => process::ExitCode::SUCCESS,
        Err(err) => {
            sys::write_stdout(&format!("{}\n", FormatApperr::from(err)));
            process::ExitCode::FAILURE
        }

    }
}

fn run() -> apperr::Result<()>{
   Ok(())
}
