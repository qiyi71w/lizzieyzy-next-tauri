use app_model::WindowGeometryDto;

pub enum WindowSample {
    Normal(WindowGeometryDto),
    Maximized,
    Minimized,
}

/// Minimized/transition bounds never replace the last usable normal rectangle.
pub fn record_sample(previous: Option<WindowGeometryDto>, sample: WindowSample) -> Option<WindowGeometryDto> {
    match sample {
        WindowSample::Minimized => previous,
        WindowSample::Maximized => previous.map(|geometry| WindowGeometryDto { maximized: true, ..geometry }),
        WindowSample::Normal(geometry) => {
            if !geometry.x.is_finite() || !geometry.y.is_finite()
                || geometry.x == MINIMIZED_SENTINEL || geometry.y == MINIMIZED_SENTINEL
                || !geometry.width.is_finite() || geometry.width <= 0.0
                || !geometry.height.is_finite() || geometry.height <= 0.0
                || !geometry.scale_factor.is_finite() || geometry.scale_factor <= 0.0 {
                previous
            } else {
                Some(WindowGeometryDto { maximized: false, ..geometry })
            }
        }
    }
}

/// Query failure is not an invalid saved rectangle: leave recovery to an explicit retry.
pub fn restore_geometry(
    saved: Option<WindowGeometryDto>,
    areas: Result<&[WorkArea], String>,
    frame: FrameInsets,
    primary: impl FnOnce() -> Result<WorkArea, String>,
) -> Result<WindowGeometryDto, String> {
    let areas = areas?;
    if !areas.iter().any(|area| usable(area, frame)) {
        return Err("Cannot enumerate usable monitor work areas; saved geometry was retained.".into());
    }
    if let Some(saved) = saved.filter(|saved| is_reachable(saved, areas, frame)) {
        return Ok(saved);
    }
    let primary = primary()?;
    if !usable(&primary, frame) {
        return Err("Primary work area is unusable; saved geometry was retained.".into());
    }
    Ok(fit_default(&primary, frame))
}

#[cfg(test)]
mod sampling_tests {
    use super::*;

    #[test]
    fn monitor_failure_is_not_an_invalid_record_recovery() {
        let frame = FrameInsets { width: 16.0, height: 39.0 };
        let saved = WindowGeometryDto { x: 40.0, y: 50.0, width: 1100.0, height: 720.0, scale_factor: 1.0, maximized: true };
        assert_eq!(restore_geometry(Some(saved), Err("monitor query failed".into()), frame, || panic!("must not reset")), Err("monitor query failed".into()));
        assert!(restore_geometry(Some(saved), Ok(&[]), frame, || panic!("must not reset")).is_err());
        let area = WorkArea { x: 0.0, y: 0.0, width: 1920.0, height: 1040.0, scale_factor: 1.0 };
        assert_eq!(restore_geometry(Some(saved), Ok(&[area]), frame, || panic!("valid secondary record needs no primary query")), Ok(saved));
    }

    #[test]
    fn normal_maximized_minimized_preserve_the_normal_rectangle() {
        let normal = WindowGeometryDto { x: -1500.0, y: 40.0, width: 1100.0, height: 720.0, scale_factor: 1.25, maximized: false };
        let initial = record_sample(None, WindowSample::Normal(normal));
        assert_eq!(initial, Some(normal));
        let maximized = record_sample(initial, WindowSample::Maximized);
        assert_eq!(maximized, Some(WindowGeometryDto { maximized: true, ..normal }));
        assert_eq!(record_sample(maximized, WindowSample::Minimized), maximized);
        assert_eq!(record_sample(maximized, WindowSample::Normal(normal)), initial);
        for invalid in [WindowGeometryDto { x: -32000.0, ..normal }, WindowGeometryDto { width: 0.0, ..normal }, WindowGeometryDto { scale_factor: f64::NAN, ..normal }] {
            assert_eq!(record_sample(maximized, WindowSample::Normal(invalid)), maximized);
        }
    }
}

/// A monitor's usable work area, excluding taskbars, in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorkArea {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub scale_factor: f64,
}

/// Aggregate nonclient width and height in logical pixels, supplied by the native adapter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameInsets {
    pub width: f64,
    pub height: f64,
}

const DEFAULT_CLIENT_WIDTH: f64 = 1440.0;
const DEFAULT_CLIENT_HEIGHT: f64 = 900.0;
const TITLE_GRAB_WIDTH: f64 = 100.0;
const TITLE_GRAB_HEIGHT: f64 = 32.0;
const MINIMIZED_SENTINEL: f64 = -32000.0;

fn usable(area: &WorkArea, frame: FrameInsets) -> bool {
    area.x.is_finite()
        && area.y.is_finite()
        && area.width.is_finite()
        && area.height.is_finite()
        && area.width > 0.0
        && area.height > 0.0
        && area.scale_factor.is_finite()
        && area.scale_factor > 0.0
        && (area.x + area.width).is_finite()
        && (area.y + area.height).is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width >= 0.0
        && frame.height >= 0.0
        && area.width / area.scale_factor > frame.width
        && area.height / area.scale_factor > frame.height
}

/// Whether the saved normal bounds are operable on one of the current work areas.
///
/// The outer origin and work areas are physical pixels; client size and aggregate
/// nonclient insets are logical pixels. A saved client's logical size does not change
/// with its recorded scale: each candidate monitor's *current* scale converts it to
/// physical outer size. Since aggregate insets do not identify the top/left border,
/// the title grab is approximated as a 100×32 logical rectangle starting at the
/// outer origin. This conservative approximation never combines title fragments
/// across monitors. Maximization is independent of the saved normal bounds.
/// An empty work-area slice means no reachable geometry; monitor query failures
/// must be handled by the caller, not represented by an empty slice.
pub fn is_reachable(geometry: &WindowGeometryDto, areas: &[WorkArea], frame: FrameInsets) -> bool {
    if !geometry.x.is_finite()
        || !geometry.y.is_finite()
        || geometry.x == MINIMIZED_SENTINEL
        || geometry.y == MINIMIZED_SENTINEL
        || !geometry.width.is_finite()
        || !geometry.height.is_finite()
        || geometry.width <= 0.0
        || geometry.height <= 0.0
        || !geometry.scale_factor.is_finite()
        || geometry.scale_factor <= 0.0
    {
        return false;
    }

    areas.iter().any(|area| {
        if !usable(area, frame) {
            return false;
        }
        let outer_width = (geometry.width + frame.width) * area.scale_factor;
        let outer_height = (geometry.height + frame.height) * area.scale_factor;
        let title_width = TITLE_GRAB_WIDTH * area.scale_factor;
        let title_height = TITLE_GRAB_HEIGHT * area.scale_factor;
        if !outer_width.is_finite()
            || !outer_height.is_finite()
            || !title_width.is_finite()
            || !title_height.is_finite()
            || outer_width > area.width
            || outer_height > area.height
            || outer_width < title_width
            || outer_height < title_height
        {
            return false;
        }

        let right = geometry.x + outer_width;
        let bottom = geometry.y + title_height;
        right.is_finite()
            && bottom.is_finite()
            && (right.min(area.x + area.width) - geometry.x.max(area.x)) >= title_width
            && (bottom.min(area.y + area.height) - geometry.y.max(area.y)) >= title_height
    })
}

/// Fit the default 1440×900 logical client within a usable work area and center
/// its physical outer bounds. The caller must supply a finite work area and
/// nonnegative aggregate frame insets leaving positive client space; there is no
/// usable window to return if that precondition fails. Tiny valid work areas are
/// fitted without imposing a 1100×720 minimum or expanding beyond the work area.
pub fn fit_default(area: &WorkArea, frame: FrameInsets) -> WindowGeometryDto {
    assert!(usable(area, frame), "window work area must allow positive client dimensions");
    let width = DEFAULT_CLIENT_WIDTH.min(area.width / area.scale_factor - frame.width);
    let height = DEFAULT_CLIENT_HEIGHT.min(area.height / area.scale_factor - frame.height);
    let outer_width = (width + frame.width) * area.scale_factor;
    let outer_height = (height + frame.height) * area.scale_factor;
    WindowGeometryDto {
        x: area.x + (area.width - outer_width) / 2.0,
        y: area.y + (area.height - outer_height) / 2.0,
        width,
        height,
        scale_factor: area.scale_factor,
        maximized: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME: FrameInsets = FrameInsets {
        width: 16.0,
        height: 40.0,
    };
    const PRIMARY: WorkArea = WorkArea {
        x: 0.0,
        y: 0.0,
        width: 1920.0,
        height: 1040.0,
        scale_factor: 1.0,
    };

    fn geometry(x: f64, y: f64) -> WindowGeometryDto {
        WindowGeometryDto {
            x,
            y,
            width: 1100.0,
            height: 720.0,
            scale_factor: 1.0,
            maximized: false,
        }
    }

    #[test]
    fn retains_noncentered_and_negative_screen_positions() {
        let secondary = WorkArea {
            x: -2560.0,
            y: -100.0,
            width: 2560.0,
            height: 1400.0,
            scale_factor: 1.0,
        };
        assert!(is_reachable(&geometry(1300.0, 40.0), &[PRIMARY], FRAME));
        assert!(is_reachable(&geometry(-2520.0, -80.0), &[PRIMARY, secondary], FRAME));
        assert!(!is_reachable(&geometry(-2520.0, -80.0), &[PRIMARY], FRAME));
    }

    #[test]
    fn rejects_invalid_coordinates_dimensions_and_scale_even_when_maximized() {
        let baseline = geometry(50.0, 50.0);
        for invalid in [
            WindowGeometryDto { x: f64::NAN, ..baseline },
            WindowGeometryDto { y: f64::INFINITY, ..baseline },
            WindowGeometryDto { x: MINIMIZED_SENTINEL, ..baseline },
            WindowGeometryDto { y: MINIMIZED_SENTINEL, ..baseline },
            WindowGeometryDto { width: 0.0, ..baseline },
            WindowGeometryDto { height: -1.0, ..baseline },
            WindowGeometryDto { width: f64::INFINITY, ..baseline },
            WindowGeometryDto { height: f64::NAN, ..baseline },
            WindowGeometryDto { scale_factor: 0.0, ..baseline },
            WindowGeometryDto { scale_factor: f64::INFINITY, ..baseline },
        ] {
            assert!(!is_reachable(&invalid, &[PRIMARY], FRAME));
        }
        assert!(is_reachable(
            &WindowGeometryDto { maximized: true, ..baseline },
            &[PRIMARY],
            FRAME
        ));
    }

    #[test]
    fn shrunken_area_rejects_oversized_client_without_resizing_a_valid_record() {
        let saved = geometry(500.0, 200.0);
        let smaller = WorkArea { width: 1000.0, ..PRIMARY };
        assert!(is_reachable(&saved, &[PRIMARY], FRAME));
        assert!(!is_reachable(&saved, &[smaller], FRAME));
        assert!(!is_reachable(&saved, &[], FRAME));
    }

    #[test]
    fn converts_logical_client_using_target_not_recorded_dpi() {
        let high_dpi = WorkArea {
            x: 1920.0,
            y: 0.0,
            width: 3000.0,
            height: 1800.0,
            scale_factor: 2.0,
        };
        let saved = WindowGeometryDto {
            x: 1950.0,
            y: 30.0,
            width: 1440.0,
            height: 800.0,
            scale_factor: 1.25,
            maximized: false,
        };
        assert!(is_reachable(&saved, &[high_dpi], FRAME));
        let narrower = WorkArea { width: 2900.0, ..high_dpi };
        assert!(!is_reachable(&saved, &[narrower], FRAME));
        assert_eq!(fit_default(&high_dpi, FRAME).scale_factor, 2.0);
    }

    #[test]
    fn title_requires_one_monitor_intersection_of_100_by_32_logical() {
        let saved = geometry(0.0, 0.0);
        let title_only = WorkArea { width: 100.0, height: 32.0, ..PRIMARY };
        let right_edge = geometry(PRIMARY.width - 100.0, 0.0);
        assert!(is_reachable(&right_edge, &[PRIMARY], FRAME));
        assert!(!is_reachable(&geometry(PRIMARY.width - 99.0, 0.0), &[PRIMARY], FRAME));
        assert!(is_reachable(&geometry(0.0, PRIMARY.height - 32.0), &[PRIMARY], FRAME));
        assert!(!is_reachable(&geometry(0.0, PRIMARY.height - 31.0), &[PRIMARY], FRAME));
        assert!(!is_reachable(&saved, &[title_only], FRAME));
    }

    #[test]
    fn fits_default_client_and_small_workarea_without_a_size_floor() {
        let fitted = fit_default(&PRIMARY, FRAME);
        assert_eq!((fitted.x, fitted.y, fitted.width, fitted.height), (232.0, 50.0, 1440.0, 900.0));
        assert!(!fitted.maximized);
        assert!(is_reachable(&fitted, &[PRIMARY], FRAME));

        let small = WorkArea {
            x: -1000.0,
            y: 120.0,
            width: 800.0,
            height: 600.0,
            scale_factor: 1.25,
        };
        let fitted = fit_default(&small, FRAME);
        assert_eq!((fitted.x, fitted.y, fitted.width, fitted.height), (-1000.0, 120.0, 624.0, 440.0));
        assert_eq!(fitted.scale_factor, 1.25);
        assert!(is_reachable(&fitted, &[small], FRAME));
    }

    #[test]
    fn rejects_unusable_workarea_without_trusting_its_intersection() {
        let saved = geometry(0.0, 0.0);
        assert!(!is_reachable(&saved, &[WorkArea { scale_factor: 0.0, ..PRIMARY }], FRAME));
        assert!(!is_reachable(&saved, &[PRIMARY], FrameInsets { width: f64::NAN, ..FRAME }));
        assert!(!is_reachable(&saved, &[WorkArea { width: 8.0, ..PRIMARY }], FRAME));
    }
}
