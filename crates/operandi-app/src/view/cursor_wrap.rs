//! Free Cursor (Preferences > Input > Remapping and
//! Viewport Properties > Mouse Control; the tester, 2026-11-25, as Blender
//! does): during a drag in a view, a pointer that reaches the view's edge is
//! put back in at the opposite edge, and the drag goes on reading the
//! pointer as if it had kept going. The gestures see one unbroken path:
//! every place the view reads during the drag is the real pointer plus the
//! distance the put-backs have taken off.
#[cfg(test)]
use eframe::egui::vec2;
use eframe::egui::{Event, Pos2, Rect, Vec2};

/// How close to the view's edge (points) a pointer is put back.
const MARGIN: f32 = 3.;

/// One drag's put-backs.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Wrap {
    /// Added to the real pointer to give the place the drag reads.
    offset: Vec2,
    /// A put-back sent but maybe not seen yet: (the place it left, the
    /// place it went, the offset before it). Moves the platform reported
    /// before the put-back took are still near the place it left, and read
    /// with the old offset (otherwise they would jump by the view's width).
    pending: Option<(Pos2, Pos2, Vec2)>,
}

impl Wrap {
    /// Whether a put-back is in force (the drag reads differ from the real
    /// pointer).
    pub(crate) fn active(&self) -> bool {
        self.offset != Vec2::ZERO || self.pending.is_some()
    }

    /// The place the drag reads for real pointer place `p`.
    pub(crate) fn read(&mut self, p: Pos2) -> Pos2 {
        if let Some((left, went, before)) = self.pending {
            if p.distance(left) < p.distance(went) {
                return p + before;
            }
            self.pending = None;
        }
        p + self.offset
    }

    /// Every pointer place in `events` as the drag reads it, up to and
    /// including the release that ends the drag; places after it are real
    /// (a new press starts with no put-backs).
    pub(crate) fn read_events(&mut self, events: &mut [Event]) {
        for e in events {
            if !self.active() {
                return;
            }
            match e {
                Event::PointerMoved(p) => *p = self.read(*p),
                Event::PointerButton { pos, pressed, .. } => {
                    *pos = self.read(*pos);
                    if !*pressed {
                        self.reset();
                    }
                }
                _ => {}
            }
        }
    }

    /// After a frame of the drag: when the real pointer `real` is at (or
    /// past) the edge of view `r`, where to put it back, at the opposite
    /// edge. The offset grows by the jump, so the drag reads on smoothly.
    pub(crate) fn put_back(&mut self, real: Pos2, r: Rect) -> Option<Pos2> {
        if self.pending.is_some() {
            return None;
        }
        let inner = r.shrink(MARGIN + 1.);
        if inner.width() < 8. || inner.height() < 8. {
            return None;
        }
        let across = |v: f32, lo: f32, hi: f32| {
            if v <= lo {
                v + (hi - lo)
            } else if v >= hi {
                v - (hi - lo)
            } else {
                v
            }
        };
        let to = Pos2::new(
            across(real.x, inner.left(), inner.right()),
            across(real.y, inner.top(), inner.bottom()),
        );
        // Still outside after one jump (a very fast flick): into the view.
        let to = inner.clamp(to);
        if to == real {
            return None;
        }
        let before = self.offset;
        self.offset += real - to;
        self.pending = Some((real, to, before));
        Some(to)
    }

    /// The drag ended.
    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }
}

/// The latest real pointer place in `events` (before any reading).
pub(crate) fn latest(events: &[Event]) -> Option<Pos2> {
    events.iter().rev().find_map(|e| match e {
        Event::PointerMoved(p) => Some(*p),
        _ => None,
    })
}

/// `d` points to the right (tests).
#[cfg(test)]
fn right(d: f32) -> Vec2 {
    vec2(d, 0.)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view() -> Rect {
        Rect::from_min_max(Pos2::new(0., 0.), Pos2::new(100., 80.))
    }

    #[test]
    fn a_pointer_at_the_right_edge_comes_back_at_the_left_and_reads_on() {
        let mut w = Wrap::default();
        let r = view();
        assert_eq!(w.put_back(Pos2::new(50., 40.), r), None, "inside");
        let edge = Pos2::new(97., 40.);
        let to = w.put_back(edge, r).expect("put back");
        assert!(to.x < 10., "{to:?}");
        // A move reported before the put-back took reads as before.
        assert_eq!(w.read(edge + right(0.5)), edge + right(0.5));
        // After it: the place it went reads as the edge, and on from there.
        assert_eq!(w.read(to), edge);
        assert_eq!(w.read(to + right(5.)), edge + right(5.));
        assert!(w.active());
        w.reset();
        assert!(!w.active());
    }

    #[test]
    fn up_and_down_wrap_too_and_keep_adding_up() {
        let mut w = Wrap::default();
        let r = view();
        let to = w.put_back(Pos2::new(50., 1.), r).unwrap();
        assert!(to.y > 70.);
        assert_eq!(w.read(to), Pos2::new(50., 1.));
        // A second trip over the top.
        let to2 = w.put_back(Pos2::new(50., 1.), r).unwrap();
        assert_eq!(w.read(to2).y, 1. - (to.y - 1.));
    }

    /// A pen's place can't be moved (Input Device Tablet, 2026-11-26): the
    /// put-back never takes, its moves go on near the edge and past it, and
    /// the drag reads them as they are, with no jump and no second put-back.
    #[test]
    fn a_put_back_that_never_takes_reads_on_unchanged() {
        let mut w = Wrap::default();
        let r = view();
        let edge = Pos2::new(97., 40.);
        assert!(w.put_back(edge, r).is_some());
        for x in [97.5, 98., 99., 105.] {
            let p = Pos2::new(x, 40.);
            assert_eq!(w.read(p), p);
        }
        assert_eq!(w.put_back(Pos2::new(105., 40.), r), None);
    }

    #[test]
    fn the_release_is_read_and_then_the_put_backs_end() {
        let mut w = Wrap::default();
        let r = view();
        let to = w.put_back(Pos2::new(98., 40.), r).unwrap();
        let mut events = vec![
            Event::PointerMoved(to),
            Event::PointerButton {
                pos: to,
                button: eframe::egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            },
            Event::PointerMoved(to),
        ];
        w.read_events(&mut events);
        assert!(matches!(events[0], Event::PointerMoved(p) if p == Pos2::new(98., 40.)));
        assert!(
            matches!(events[1], Event::PointerButton { pos, .. } if pos == Pos2::new(98., 40.))
        );
        assert!(
            matches!(events[2], Event::PointerMoved(p) if p == to),
            "after the release: real"
        );
        assert!(!w.active());
    }
}
