use std::cmp::Ordering;

use crate::Bar;

/// Inferred four-point historical path for a standard OHLC bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HistoricalPathKind {
    OpenHighLowClose,
    OpenLowHighClose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PathPointKind {
    Open,
    High,
    Low,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PathLegDirection {
    Rising,
    Falling,
    Flat,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PathPoint {
    pub index: u8,
    pub kind: PathPointKind,
    pub price: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PathLeg {
    pub index: u8,
    pub from: PathPoint,
    pub to: PathPoint,
    pub direction: PathLegDirection,
}

impl PathLeg {
    fn new(index: u8, from: PathPoint, to: PathPoint) -> Self {
        let direction = match from.price.total_cmp(&to.price) {
            Ordering::Less => PathLegDirection::Rising,
            Ordering::Greater => PathLegDirection::Falling,
            Ordering::Equal => PathLegDirection::Flat,
        };
        Self {
            index,
            from,
            to,
            direction,
        }
    }

    /// Rank used to visit crossings along this monotonic leg.
    ///
    /// Lower rank is visited first: lower prices on a rising leg, higher
    /// prices on a falling leg. A flat leg has a single mark; every finite
    /// price shares the same rank so a zero-length leg cannot order two
    /// distinct crossings.
    pub(crate) fn crossing_rank(self, price: f64) -> f64 {
        match self.direction {
            PathLegDirection::Rising => price,
            PathLegDirection::Falling => -price,
            PathLegDirection::Flat => 0.0,
        }
    }

    pub(crate) fn cmp_crossing_prices(self, left: f64, right: f64) -> Ordering {
        self.crossing_rank(left)
            .total_cmp(&self.crossing_rank(right))
    }

    pub(crate) fn contains_price(self, price: f64) -> bool {
        if !price.is_finite() {
            return false;
        }
        let low = self.from.price.min(self.to.price);
        let high = self.from.price.max(self.to.price);
        price >= low && price <= high
    }

    /// Remaining prices on this monotonic leg, including the current mark.
    pub(crate) fn contains_unconsumed(self, mark: f64, price: f64) -> bool {
        if !self.contains_price(price) {
            return false;
        }
        match self.direction {
            PathLegDirection::Rising => price.total_cmp(&mark) != Ordering::Less,
            PathLegDirection::Falling => price.total_cmp(&mark) != Ordering::Greater,
            PathLegDirection::Flat => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct HistoricalPath {
    pub kind: HistoricalPathKind,
    pub points: [PathPoint; 4],
}

impl HistoricalPath {
    /// Build the inferred path from a validated-style OHLC tuple.
    ///
    /// Non-finite prices and bars that violate `low <= open,close <= high`
    /// return `None`. Existing bar-input validation remains the public
    /// failure boundary; this constructor does not invent a second one.
    ///
    /// Equal open-to-high and open-to-low distance selects open-low-high-close
    /// from the 2026-09-03 ADAUSDT sample lock (LIM then STP). That is not a
    /// claim about equal-distance dojis or close-direction variants.
    pub(crate) fn from_ohlc(open: f64, high: f64, low: f64, close: f64) -> Option<Self> {
        if ![open, high, low, close]
            .iter()
            .all(|price| price.is_finite())
        {
            return None;
        }
        if high < open || high < close || low > open || low > close {
            return None;
        }

        let to_high = (open - high).abs();
        let to_low = (open - low).abs();
        let kind = match to_high.total_cmp(&to_low) {
            Ordering::Less => HistoricalPathKind::OpenHighLowClose,
            Ordering::Greater | Ordering::Equal => HistoricalPathKind::OpenLowHighClose,
        };

        let kinds = match kind {
            HistoricalPathKind::OpenHighLowClose => [
                PathPointKind::Open,
                PathPointKind::High,
                PathPointKind::Low,
                PathPointKind::Close,
            ],
            HistoricalPathKind::OpenLowHighClose => [
                PathPointKind::Open,
                PathPointKind::Low,
                PathPointKind::High,
                PathPointKind::Close,
            ],
        };
        let prices = match kind {
            HistoricalPathKind::OpenHighLowClose => [open, high, low, close],
            HistoricalPathKind::OpenLowHighClose => [open, low, high, close],
        };
        let points = [
            PathPoint {
                index: 0,
                kind: kinds[0],
                price: prices[0],
            },
            PathPoint {
                index: 1,
                kind: kinds[1],
                price: prices[1],
            },
            PathPoint {
                index: 2,
                kind: kinds[2],
                price: prices[2],
            },
            PathPoint {
                index: 3,
                kind: kinds[3],
                price: prices[3],
            },
        ];
        Some(Self { kind, points })
    }

    pub(crate) fn from_bar(bar: &Bar) -> Option<Self> {
        Self::from_ohlc(bar.open, bar.high, bar.low, bar.close)
    }

    /// Infer a path for a bar that already passed public validation.
    ///
    /// Inconsistent high/low extremes are clamped onto `open`/`close` so the
    /// broker still walks four points instead of skipping fills. `from_ohlc`
    /// stays strict for unit tests of the pure constructor.
    pub(crate) fn from_validated_bar(bar: &Bar) -> Option<Self> {
        Self::from_bar(bar).or_else(|| {
            let high = bar.high.max(bar.open);
            let low = bar.low.min(bar.open);
            Self::from_ohlc(bar.open, high, low, bar.close)
        })
    }

    pub(crate) fn legs(self) -> [PathLeg; 3] {
        [
            PathLeg::new(0, self.points[0], self.points[1]),
            PathLeg::new(1, self.points[1], self.points[2]),
            PathLeg::new(2, self.points[2], self.points[3]),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(path: &HistoricalPath) -> [PathPointKind; 4] {
        path.points.map(|point| point.kind)
    }

    fn prices(path: &HistoricalPath) -> [f64; 4] {
        path.points.map(|point| point.price)
    }

    fn assert_monotonic_legs(path: &HistoricalPath) {
        for (index, leg) in path.legs().iter().enumerate() {
            assert_eq!(leg.index as usize, index);
            assert_eq!(leg.from, path.points[index]);
            assert_eq!(leg.to, path.points[index + 1]);
            let expected = match leg.from.price.total_cmp(&leg.to.price) {
                Ordering::Less => PathLegDirection::Rising,
                Ordering::Greater => PathLegDirection::Falling,
                Ordering::Equal => PathLegDirection::Flat,
            };
            assert_eq!(leg.direction, expected);
        }
    }

    #[test]
    fn high_first_selects_open_high_low_close() {
        let path = HistoricalPath::from_ohlc(10.0, 11.0, 8.0, 9.0).expect("path");
        assert_eq!(path.kind, HistoricalPathKind::OpenHighLowClose);
        assert_eq!(
            kinds(&path),
            [
                PathPointKind::Open,
                PathPointKind::High,
                PathPointKind::Low,
                PathPointKind::Close,
            ]
        );
        assert_eq!(prices(&path), [10.0, 11.0, 8.0, 9.0]);
        assert_monotonic_legs(&path);
        let legs = path.legs();
        assert_eq!(legs[0].direction, PathLegDirection::Rising);
        assert_eq!(legs[1].direction, PathLegDirection::Falling);
        assert_eq!(legs[2].direction, PathLegDirection::Rising);
    }

    #[test]
    fn low_first_selects_open_low_high_close() {
        let path = HistoricalPath::from_ohlc(10.0, 12.0, 9.0, 11.0).expect("path");
        assert_eq!(path.kind, HistoricalPathKind::OpenLowHighClose);
        assert_eq!(
            kinds(&path),
            [
                PathPointKind::Open,
                PathPointKind::Low,
                PathPointKind::High,
                PathPointKind::Close,
            ]
        );
        assert_eq!(prices(&path), [10.0, 9.0, 12.0, 11.0]);
        assert_monotonic_legs(&path);
        let legs = path.legs();
        assert_eq!(legs[0].direction, PathLegDirection::Falling);
        assert_eq!(legs[1].direction, PathLegDirection::Rising);
        assert_eq!(legs[2].direction, PathLegDirection::Falling);
    }

    #[test]
    fn equal_distance_selects_sample_locked_open_low_high_close() {
        let synthetic = HistoricalPath::from_ohlc(10.0, 12.0, 8.0, 10.0).expect("path");
        assert_eq!(synthetic.kind, HistoricalPathKind::OpenLowHighClose);
        assert_eq!(prices(&synthetic), [10.0, 8.0, 12.0, 10.0]);

        let analogue = HistoricalPath::from_ohlc(0.1939, 0.1941, 0.1937, 0.1939).expect("path");
        assert_eq!(analogue.kind, HistoricalPathKind::OpenLowHighClose);
        assert_eq!(prices(&analogue), [0.1939, 0.1937, 0.1941, 0.1939]);
        let [open_to_low, low_to_high, _] = analogue.legs();
        assert_eq!(open_to_low.direction, PathLegDirection::Falling);
        assert_eq!(
            open_to_low.cmp_crossing_prices(0.1938, 0.19375),
            Ordering::Less
        );
        assert_eq!(low_to_high.direction, PathLegDirection::Rising);
        assert_eq!(
            low_to_high.cmp_crossing_prices(0.1938, 0.1940),
            Ordering::Less,
            "after the low, 0.1938 is visited before STP at 0.1940"
        );
    }

    #[test]
    fn bullish_and_bearish_closes_do_not_change_path_kind() {
        let bearish_high_first = HistoricalPath::from_ohlc(10.0, 11.0, 8.0, 9.0).expect("path");
        let bullish_high_first = HistoricalPath::from_ohlc(10.0, 11.0, 8.0, 10.5).expect("path");
        assert_eq!(
            bearish_high_first.kind,
            HistoricalPathKind::OpenHighLowClose
        );
        assert_eq!(
            bullish_high_first.kind,
            HistoricalPathKind::OpenHighLowClose
        );

        let bearish_low_first = HistoricalPath::from_ohlc(10.0, 12.0, 9.0, 9.5).expect("path");
        let bullish_low_first = HistoricalPath::from_ohlc(10.0, 12.0, 9.0, 11.0).expect("path");
        assert_eq!(bearish_low_first.kind, HistoricalPathKind::OpenLowHighClose);
        assert_eq!(bullish_low_first.kind, HistoricalPathKind::OpenLowHighClose);
    }

    #[test]
    fn open_equals_high_is_high_first_with_a_flat_first_leg() {
        let path = HistoricalPath::from_ohlc(11.0, 11.0, 8.0, 9.0).expect("path");
        assert_eq!(path.kind, HistoricalPathKind::OpenHighLowClose);
        assert_eq!(path.legs()[0].direction, PathLegDirection::Flat);
        assert_eq!(path.legs()[1].direction, PathLegDirection::Falling);
    }

    #[test]
    fn open_equals_low_is_low_first_with_a_flat_first_leg() {
        let path = HistoricalPath::from_ohlc(8.0, 11.0, 8.0, 9.0).expect("path");
        assert_eq!(path.kind, HistoricalPathKind::OpenLowHighClose);
        assert_eq!(path.legs()[0].direction, PathLegDirection::Flat);
        assert_eq!(path.legs()[1].direction, PathLegDirection::Rising);
    }

    #[test]
    fn four_price_doji_keeps_four_points_and_three_flat_legs() {
        let path = HistoricalPath::from_ohlc(5.0, 5.0, 5.0, 5.0).expect("path");
        assert_eq!(path.kind, HistoricalPathKind::OpenLowHighClose);
        assert_eq!(path.points.len(), 4);
        assert!(
            path.legs()
                .iter()
                .all(|leg| leg.direction == PathLegDirection::Flat)
        );
        assert_eq!(
            path.legs()[0].cmp_crossing_prices(5.0, 5.0),
            Ordering::Equal
        );
    }

    #[test]
    fn negative_and_fractional_prices_are_accepted_when_finite() {
        let path = HistoricalPath::from_ohlc(-1.25, -1.0, -2.0, -1.5).expect("path");
        assert_eq!(path.kind, HistoricalPathKind::OpenHighLowClose);
        assert_eq!(prices(&path), [-1.25, -1.0, -2.0, -1.5]);
        let from_bar = HistoricalPath::from_bar(&Bar {
            time: 0,
            open: -1.25,
            high: -1.0,
            low: -2.0,
            close: -1.5,
            volume: 1.0,
        })
        .expect("bar path");
        assert_eq!(from_bar, path);
    }

    #[test]
    fn rising_falling_and_flat_legs_order_crossings() {
        let high_first = HistoricalPath::from_ohlc(10.0, 12.0, 7.0, 9.0).expect("path");
        let [rising, falling, close_leg] = high_first.legs();
        assert_eq!(rising.direction, PathLegDirection::Rising);
        assert_eq!(rising.cmp_crossing_prices(10.5, 11.5), Ordering::Less);
        assert_eq!(rising.cmp_crossing_prices(11.5, 10.5), Ordering::Greater);

        assert_eq!(falling.direction, PathLegDirection::Falling);
        assert_eq!(falling.cmp_crossing_prices(11.0, 9.0), Ordering::Less);
        assert_eq!(falling.cmp_crossing_prices(9.0, 11.0), Ordering::Greater);

        let doji = HistoricalPath::from_ohlc(3.0, 3.0, 3.0, 3.0).expect("path");
        let flat = doji.legs()[0];
        assert_eq!(flat.direction, PathLegDirection::Flat);
        assert_eq!(flat.cmp_crossing_prices(3.0, 3.0), Ordering::Equal);
        assert_eq!(close_leg.direction, PathLegDirection::Rising);
        assert!(rising.contains_unconsumed(10.0, 10.5));
        assert!(!rising.contains_unconsumed(11.0, 10.5));
        assert!(falling.contains_unconsumed(12.0, 11.0));
        assert!(!falling.contains_unconsumed(9.0, 11.0));
    }

    #[test]
    fn non_finite_or_inconsistent_ohlc_is_rejected() {
        assert!(HistoricalPath::from_ohlc(f64::NAN, 2.0, 1.0, 1.5).is_none());
        assert!(HistoricalPath::from_ohlc(1.0, f64::INFINITY, 0.0, 1.0).is_none());
        assert!(HistoricalPath::from_ohlc(1.0, 0.5, 0.0, 1.0).is_none());
        assert!(HistoricalPath::from_ohlc(1.0, 2.0, 1.5, 1.0).is_none());
    }

    #[test]
    fn validated_bar_clamps_inconsistent_extremes() {
        let bar = Bar {
            time: 0,
            open: 3.5,
            high: 4.0,
            low: 3.6,
            close: 3.8,
            volume: 1.0,
        };
        assert!(HistoricalPath::from_bar(&bar).is_none());
        let path = HistoricalPath::from_validated_bar(&bar).expect("clamped path");
        assert_eq!(path.points.map(|point| point.price), [3.5, 3.5, 4.0, 3.8]);

        let close_outside = Bar {
            time: 0,
            open: 96.0,
            high: 96.0,
            low: 96.0,
            close: 40.0,
            volume: 1.0,
        };
        assert!(HistoricalPath::from_bar(&close_outside).is_none());
        assert!(HistoricalPath::from_validated_bar(&close_outside).is_none());
    }
}
