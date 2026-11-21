//! The panel and sequencers as a standalone program (`--features standalone`).
use nice_plug::prelude::*;
use seq666::Seq666;

fn main() {
    nice_export_standalone::<Seq666>();
}
