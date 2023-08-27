use std::cmp;
use xcb::{x, Connection, Event, Xid};

fn main() {
    let (conn, _) = Connection::connect(None).expect("Failed to connect to X server");
    println!("Connected to X server");

    let setup = conn.get_setup();
    let screen = setup.roots().next().expect("Failed to get root window");
    let root_window = screen.root();

    let modkey = x::ModMask::CONTROL;

    conn.send_request(&x::GrabKey {
        owner_events: true,
        grab_window: root_window,
        modifiers: modkey,
        key: 67, // F1
        pointer_mode: x::GrabMode::Async,
        keyboard_mode: x::GrabMode::Async,
    });

    conn.send_request(&x::GrabButton {
        owner_events: true,
        grab_window: root_window,
        event_mask: x::EventMask::BUTTON_PRESS
            | x::EventMask::BUTTON_RELEASE
            | x::EventMask::POINTER_MOTION,
        pointer_mode: x::GrabMode::Async,
        keyboard_mode: x::GrabMode::Async,
        confine_to: x::Window::none(),
        cursor: x::Cursor::none(),
        button: x::ButtonIndex::N1, // Left mouse button
        modifiers: modkey,
    });

    conn.send_request(&x::GrabButton {
        owner_events: true,
        grab_window: root_window,
        event_mask: x::EventMask::BUTTON_PRESS
            | x::EventMask::BUTTON_RELEASE
            | x::EventMask::POINTER_MOTION,
        pointer_mode: x::GrabMode::Async,
        keyboard_mode: x::GrabMode::Async,
        confine_to: x::Window::none(),
        cursor: x::Cursor::none(),
        button: x::ButtonIndex::N3, // Right mouse button
        modifiers: modkey,
    });

    conn.flush().unwrap();

    // store the last window Modkey + Click event to move the window around
    let mut start: Option<Box<x::ButtonPressEvent>> = None;
    let mut geom: Option<Box<x::GetGeometryReply>> = None;

    loop {
        let ev = conn.wait_for_event().unwrap();
        match ev {
            Event::X(x::Event::KeyPress(ev)) => {
                let window = ev.child();
                conn.send_request(&x::ConfigureWindow {
                    window,
                    value_list: &[x::ConfigWindow::StackMode(x::StackMode::Above)],
                });
            }

            Event::X(x::Event::ButtonPress(ev)) => {
                let window = ev.child();
                let cookie = conn.send_request(&x::GetGeometry {
                    drawable: x::Drawable::Window(window),
                });
                geom = Some(Box::new(conn.wait_for_reply(cookie).unwrap()));
                start = Some(Box::new(ev));
            }

            Event::X(x::Event::MotionNotify(ev)) => {
                let geom = geom.as_ref().unwrap();
                let mut window_x = geom.x() as i32;
                let mut window_y = geom.y() as i32;
                let mut window_w = geom.width() as u32;
                let mut window_h = geom.height() as u32;

                let start = start.as_ref().unwrap();
                let btn = start.detail();

                // pointer offset
                let pointer_x = ev.event_x() - start.root_x();
                let pointer_y = ev.event_y() - start.root_y();

                if btn == 1 {
                    // left mouse button, move window
                    window_x += pointer_x as i32;
                    window_y += pointer_y as i32;
                } else if btn == 3 {
                    // right mouse button, resize window
                    let w = cmp::max(1, window_w as i32 + pointer_x as i32);
                    let h = cmp::max(1, window_h as i32 + pointer_y as i32);
                    window_w = w as u32;
                    window_h = h as u32;
                }

                conn.send_request(&x::ConfigureWindow {
                    window: start.child(),
                    value_list: &[
                        x::ConfigWindow::X(window_x),
                        x::ConfigWindow::Y(window_y),
                        x::ConfigWindow::Width(window_w),
                        x::ConfigWindow::Height(window_h),
                    ],
                });
            }

            Event::X(x::Event::ButtonRelease(_)) => {
                start = None;
                geom = None;
            }

            _ => {}
        }

        conn.flush().unwrap();
    }
}
