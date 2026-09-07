//! Reading sittings out of the Kindle's own system log: the
//! `ReadingTimerController` lines, the `fastmetrics` reader-shell records, and
//! `powerd`'s state changes. [`session::parse_sessions`] reads all three.

pub mod line;
pub mod metric;
pub mod power;
pub mod session;
pub mod source;

pub use metric::METRIC_MARKERS;
pub use power::POWER_MARKERS;
pub use session::parse_sessions;

/// Every tag worth keeping a line for; the prefilter ahead of everything else.
/// One device-day is some 76 KB of these, against a syslog two orders of
/// magnitude larger.
pub const MARKERS: [&str; 13] = {
    let (m, p) = (METRIC_MARKERS, POWER_MARKERS);
    [
        line::TIMER_MARKER,
        m[0],
        m[1],
        m[2],
        m[3],
        m[4],
        m[5],
        m[6],
        m[7],
        p[0],
        p[1],
        p[2],
        p[3],
    ]
};

/// The pass ahead of [`MARKERS`]: every one of the thirteen carries one of
/// these three.
///
/// A `contains` costs a walk of the line whether it matches or nothing, and
/// [`source::collect_from`] walks a syslog two orders of magnitude larger than
/// the part of it this reads — on a first run, every daily snapshot the device
/// still holds. Three walks settle the lines carrying nothing; the thirteen
/// then run on what is left, which is almost all of what is worth keeping.
///
/// `ereader_` covers the eight [`METRIC_MARKERS`] and
/// `ereader_powerd_state_change` with them. A test holds the containment,
/// which is the whole of what makes this safe.
const TELLS: [&str; 3] = [line::TIMER_MARKER, "ereader_", "lipc:evts:name="];

/// Whether a line is one worth keeping, [`TELLS`] deciding the great majority
/// of them before [`MARKERS`] is consulted.
pub fn marked(line: &str) -> bool {
    TELLS.iter().any(|tell| line.contains(tell))
        && MARKERS.iter().any(|marker| line.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`marked`] answers for [`MARKERS`] and nothing else, which holds only
    /// while every marker carries a tell.
    #[test]
    fn every_marker_carries_a_tell() {
        for marker in MARKERS {
            assert!(
                TELLS.iter().any(|tell| marker.contains(tell)),
                "{marker} carries no tell — `marked` would pass it over"
            );
            assert!(marked(&format!("260807:101501 cvm[1]: I {marker} x")));
        }
    }

    #[test]
    fn a_line_carrying_none_of_them_is_not_marked() {
        assert!(!marked(
            "260807:101502 kernel: I mmc0: something entirely unrelated"
        ));
        // A tell on its own is not a marker.
        assert!(!marked(
            "260807:101502 fastmetrics[1]: D SchemaName[ereader_nothing]"
        ));
    }
}
