//! The panel and engine as a standalone program (`--features standalone`), for trying them
//! without a host.
use front303::Front303;
use nice_plug::prelude::*;

fn main() {
    nice_export_standalone::<Front303>();
}
