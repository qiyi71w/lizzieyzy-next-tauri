use std::{ffi::c_void, sync::mpsc, time::Duration};

use app_preferences::window_geometry::FrameInsets;
use gtk::{
    glib::translate::{FromGlibPtrNone, ToGlibPtr},
    prelude::*,
    HeaderBar, Widget,
};
use tauri::WebviewWindow;

const UI_TIMEOUT: Duration = Duration::from_secs(2);

// GTK calls in this module run on the UI thread. Unlike check_resize(),
// realizing a hidden GtkWindow computes its requested content size and allocates
// the header before GTK creates its (still unmapped) GdkWindow.
pub(super) fn configure(window: &WebviewWindow) -> Result<(), String> {
    let native = window.gtk_window().map_err(|e| e.to_string())?;
    if native.is_visible() || native.is_mapped() {
        return Err("GTK titlebar must be configured before the window is shown".into());
    }

    if let Some(existing) = native.titlebar() {
        // Tao already installs an EventBox containing a HeaderBar on Wayland.
        // Do not replace it: Tao owns its resizing/decoration-layout handler.
        let header = header_bar(&existing)?;
        // Tao's event surface sits above its children by default, intercepting
        // native caption buttons. Keep the surface below the controls, as on X11.
        if let Some(event_box) = existing.downcast_ref::<gtk::EventBox>() {
            event_box.set_above_child(false);
        }
        let weak = header.downgrade();
        native.connect_title_notify(move |window| {
            if let Some(header) = weak.upgrade() {
                header.set_title(window.title().as_deref());
            }
        });
        header.set_title(native.title().as_deref());
        existing.show_all();
    } else {
        if native.is_realized() {
            return Err("Cannot install GTK native HeaderBar: Tauri already realized its X11 window".into());
        }
        let title = native.title().map(|title| title.to_string()).unwrap_or_default();
        let header = HeaderBar::builder()
            .show_close_button(true)
            .decoration_layout("menu:minimize,maximize,close")
            .title(title)
            .build();
        // Supply a native caption event surface below its controls: placing it
        // above children would intercept minimize/maximize/close button clicks.
        let titlebar = gtk::EventBox::new();
        titlebar.set_above_child(false);
        titlebar.set_can_focus(false);
        titlebar.add(&header);
        native.set_titlebar(Some(&titlebar));
        let weak = header.downgrade();
        native.connect_resizable_notify(move |window| {
            if let Some(header) = weak.upgrade() {
                header.set_decoration_layout(Some(if window.is_resizable() {
                    "menu:minimize,maximize,close"
                } else {
                    "menu:minimize,close"
                }));
            }
        });
        titlebar.show_all();
    }

    // show_all() on *children* is safe while the toplevel stays hidden. GTK
    // needs visible children for its real pre-map requisition and allocation.
    if let Ok(content) = window.default_vbox() {
        content.show_all();
    }
    if !native.is_realized() {
        native.realize();
    }
    if !native.is_realized() || native.is_mapped() {
        return Err("GTK did not realize an unmapped window for title measurement".into());
    }
    let titlebar = native
        .titlebar()
        .ok_or("GTK did not retain its native titlebar")?;
    let header = header_bar(&titlebar)?;
    header.realize();
    if header.allocated_width() <= 0 || header.allocated_height() <= 0 {
        return Err("GTK did not allocate the hidden native titlebar".into());
    }
    Ok(())
}

fn header_bar(titlebar: &Widget) -> Result<HeaderBar, String> {
    if let Ok(header) = titlebar.clone().downcast::<HeaderBar>() {
        return Ok(header);
    }
    if let Ok(box_) = titlebar.clone().downcast::<gtk::EventBox>() {
        if let Some(child) = box_.child() {
            return child
                .downcast::<HeaderBar>()
                .map_err(|_| "Tao's titlebar does not contain a GTK HeaderBar".into());
        }
    }
    Err("Native GTK titlebar is not a HeaderBar".into())
}

struct ControlBounds<'a> {
    header: &'a HeaderBar,
    left: i32,
    right: i32,
    buttons: usize,
    error: Option<String>,
}

// gtk_container_get_children omits internal GTK HeaderBar decoration buttons.
// forall includes them, so measure their actual allocated hit regions.
fn visit_children(container: &gtk::Container, bounds: &mut ControlBounds<'_>) {
    unsafe extern "C" fn visit(widget: *mut gtk::ffi::GtkWidget, data: *mut c_void) {
        let bounds = &mut *(data as *mut ControlBounds<'_>);
        if bounds.error.is_some() {
            return;
        }
        let widget: Widget = Widget::from_glib_none(widget);
        // The toplevel is deliberately hidden during restoration; inspect the
        // child's own visibility, not visibility through all its ancestors.
        if !widget.get_visible() {
            return;
        }
        widget.realize();
        if widget.is::<gtk::Button>() {
            let width = widget.allocated_width();
            let Some((x, _)) = widget.translate_coordinates(bounds.header, 0, 0) else {
                bounds.error = Some("Cannot locate GTK title button within the native HeaderBar".into());
                return;
            };
            if width <= 0 {
                bounds.error = Some("GTK title button is not allocated".into());
                return;
            }
            let middle = bounds.header.allocated_width() / 2;
            if x < middle && x + width > middle {
                bounds.error = Some("GTK title button overlaps the draggable title center".into());
                return;
            }
            if x + width <= middle {
                bounds.left = bounds.left.max(x + width);
            } else {
                bounds.right = bounds.right.min(x);
            }
            bounds.buttons += 1;
        } else if let Ok(container) = widget.downcast::<gtk::Container>() {
            visit_children(&container, bounds);
        }
    }
    unsafe {
        gtk::ffi::gtk_container_forall(
            container.to_glib_none().0,
            Some(visit),
            bounds as *mut ControlBounds<'_> as *mut c_void,
        );
    }
}

fn measure(native: &gtk::ApplicationWindow) -> Result<(f64, f64, f64, f64), String> {
    if !native.is_realized() {
        return Err("GTK window is not realized; native title allocation is unavailable".into());
    }
    let titlebar = native.titlebar().ok_or("GTK window has no native titlebar")?;
    let header = header_bar(&titlebar)?;
    header.realize();
    let (width, height) = (header.allocated_width(), header.allocated_height());
    if width <= 0 || height <= 0 {
        return Err("GTK native titlebar has no measured allocation".into());
    }
    let (header_x, header_y) = header
        .translate_coordinates(native, 0, 0)
        .ok_or("Cannot locate GTK HeaderBar in the native window")?;
    let gdk_window = native.window().ok_or("GTK native window has no GdkWindow")?;
    let (_, origin_x, origin_y) = gdk_window.origin();
    let frame = gdk_window.frame_extents();
    if frame.width() <= 0 || frame.height() <= 0 {
        return Err("GTK window has no native outer frame allocation".into());
    }

    let mut bounds = ControlBounds {
        header: &header,
        left: 0,
        right: width,
        buttons: 0,
        error: None,
    };
    visit_children(header.upcast_ref::<gtk::Container>(), &mut bounds);
    if let Some(error) = bounds.error {
        return Err(error);
    }
    if bounds.buttons == 0 {
        return Err("GTK native title buttons are not allocated".into());
    }
    // Header padding is draggable too; the surrounding window allocation owns
    // resize borders. Exclude controls, not CSS padding inside the titlebar.
    let left = origin_x - frame.x() + header_x + bounds.left;
    let right = origin_x - frame.x() + header_x + bounds.right;
    let top = origin_y - frame.y() + header_y;
    let bottom = top + height;
    if left < 0
        || right > frame.width()
        || top < 0
        || bottom > frame.height()
        || right <= left
        || bottom <= top
    {
        return Err("GTK native title has no valid draggable region within the outer frame".into());
    }
    Ok((
        f64::from(left),
        f64::from(frame.width() - right),
        f64::from(top),
        f64::from(bottom - top),
    ))
}

// GDK frame extents and GTK allocations are logical; title insets are not
// scaled a second time.

pub(super) fn frame(window: &WebviewWindow) -> Result<FrameInsets, String> {
    on_ui(window, |native| {
        let (title_left, title_right, title_top, title_height) = measure(native)?;
        let outer = native
            .window()
            .ok_or("GTK native window has no GdkWindow")?
            .frame_extents();
        let (client_width, client_height) = native.size();
        if client_width <= 0
            || client_height <= 0
            || outer.width() < client_width
            || outer.height() < client_height
        {
            return Err("GTK native frame or client size is not allocated".into());
        }
        Ok(FrameInsets {
            width: f64::from(outer.width() - client_width),
            height: f64::from(outer.height() - client_height),
            title_left,
            title_right,
            title_top,
            title_height,
        })
    })
}

pub(super) fn position_is_managed(window: &WebviewWindow) -> Result<bool, String> {
    on_ui(window, |native| {
        Ok(native.display().type_().name() == "GdkWaylandDisplay")
    })
}

pub(super) fn normal_bounds(window: &WebviewWindow) -> Result<app_model::WindowGeometryDto, String> {
    on_ui(window, |native| {
        if !native.is_realized() {
            return Err("GTK window is not realized; native client size is unavailable".into());
        }
        let (width, height) = native.size();
        if width <= 0 || height <= 0 {
            return Err("GTK native client size is not allocated".into());
        }
        let managed = native.display().type_().name() == "GdkWaylandDisplay";
        let outer = native
            .window()
            .ok_or("GTK native window has no GdkWindow")?
            .frame_extents();
        let scale = f64::from(native.scale_factor());
        Ok(app_model::WindowGeometryDto {
            x: (!managed).then(|| f64::from(outer.x()) * scale),
            y: (!managed).then(|| f64::from(outer.y()) * scale),
            width: f64::from(width),
            height: f64::from(height),
            scale_factor: scale,
            maximized: false,
        })
    })
}

pub(super) fn managed_reset_bounds(window: &WebviewWindow) -> Result<app_model::WindowGeometryDto, String> {
    on_ui(window, |native| {
        let gdk_window = native.window().ok_or("GTK native window has no GdkWindow")?;
        let monitor = native
            .display()
            .monitor_at_window(&gdk_window)
            .ok_or("Cannot query the current Wayland output for window reset.")?;
        let area = monitor.workarea();
        let outer = gdk_window.frame_extents();
        let (width, height) = native.size();
        let available_width = area.width() - (outer.width() - width).max(0);
        let available_height = area.height() - (outer.height() - height).max(0);
        if available_width <= 0 || available_height <= 0 {
            return Err("Current Wayland output has no usable client area.".into());
        }
        Ok(app_model::WindowGeometryDto {
            x: None,
            y: None,
            width: f64::from(available_width).min(1440.0),
            height: f64::from(available_height).min(900.0),
            scale_factor: f64::from(native.scale_factor()),
            maximized: false,
        })
    })
}

pub(super) fn set_position(window: &WebviewWindow, x: f64, y: f64) -> Result<(), String> {
    on_ui(window, move |native| {
        let outer = native
            .window()
            .ok_or("GTK native window has no GdkWindow")?
            .frame_extents();
        let (gravity_x, gravity_y) = native.position();
        let scale = f64::from(native.scale_factor());
        // A restored user position may intentionally leave the body off-screen.
        // Do not let the WM treat it as an automatic application placement.
        native.set_geometry_hints(None::<&Widget>, None, gtk::gdk::WindowHints::USER_POS);
        // GtkWindow::move_ accepts the gravity reference, not the outer CSD
        // origin. Preserve GTK's measured shadow/frame offset across restarts.
        native.move_(
            (x / scale).round() as i32 + gravity_x - outer.x(),
            (y / scale).round() as i32 + gravity_y - outer.y(),
        );
        Ok(())
    })
}

fn on_ui<T: Send + 'static>(
    window: &WebviewWindow,
    query: impl FnOnce(&gtk::ApplicationWindow) -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    let (tx, rx) = mpsc::sync_channel(1);
    let handle = window.clone();
    window
        .run_on_main_thread(move || {
            let result = handle
                .gtk_window()
                .map_err(|error| error.to_string())
                .and_then(|native| query(&native));
            let _ = tx.send(result);
        })
        .map_err(|error| error.to_string())?;
    rx.recv_timeout(UI_TIMEOUT)
        .map_err(|_| "Timed out querying native GTK geometry on the UI thread".to_string())?
}
