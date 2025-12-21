
use crate::helpers::CoordType;

// On one hand it's disgusting that I wrote this as a global variable, but on the
// other hand, this isn't a public library API, and it makes the code a lot cleaner,
// because we don't need to inject this once-per-process value everywhere.
static mut AMBIGUOUS_WIDTH: usize = 1;

/// Sets the width of "ambiguous" width characters as per "UAX #11: East Asian Width".
///
/// Defaults to 1.
pub fn setup_ambiguous_width(ambiguous_width: CoordType) {
    unsafe { AMBIGUOUS_WIDTH = ambiguous_width as usize };
}
