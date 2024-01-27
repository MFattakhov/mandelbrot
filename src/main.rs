use mandelbrot::{render, Complex};

use std::sync::{Arc, Mutex};

use gtk::gdk::ffi::GDK_BUTTON_PRIMARY;
use gtk::gdk::Texture;
use gtk::gdk_pixbuf::{Colorspace, Pixbuf};
use gtk::gio::ActionEntry;
use gtk::glib::Bytes;
use gtk::{glib, Application, ApplicationWindow, GestureClick};
use gtk::{prelude::*, Image};
use lazy_static::lazy_static;

const APP_ID: &str = "org.gtk_rs.Mandelbrot";

const APP_WIDTH: usize = 1000;
const APP_HEIGHT: usize = 1000;

lazy_static! {
    static ref UL: Arc<Mutex<Complex>> = Arc::new(Mutex::new(Complex {
        re: -0.75,
        im: 0.75,
    }));
    static ref LR: Arc<Mutex<Complex>> = Arc::new(Mutex::new(Complex { re: 0., im: 0. }));
}

fn lock_ul() -> std::sync::MutexGuard<'static, Complex> {
    UL.lock().expect("Failed to lock UL")
}

fn lock_lr() -> std::sync::MutexGuard<'static, Complex> {
    LR.lock().expect("Failed to lock LR")
}

fn main() -> glib::ExitCode {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Set keyboard accelerator to trigger "win.close".
    let close_accels = if cfg!(target_os = "macos") {
        ["<Meta>W"]
    } else {
        ["<Ctrl>W"]
    };
    app.set_accels_for_action("win.close", &close_accels);

    // Run the application
    app.run()
}

fn gen_pixbuf(bounds: (usize, usize)) -> Pixbuf {
    let mut pixels = [0u8; APP_HEIGHT * APP_WIDTH * 3];
    render(&mut pixels, bounds, *lock_ul(), *lock_lr());
    Pixbuf::from_bytes(
        &Bytes::from(&pixels),
        Colorspace::Rgb,
        false,
        8,
        APP_WIDTH as i32,
        APP_HEIGHT as i32,
        3 * APP_WIDTH as i32,
    )
}

fn recalculate_coordinates(x: f64, y: f64) {
    let px = x / APP_WIDTH as f64;
    let py = y / APP_HEIGHT as f64;

    let ul: &mut Complex = &mut lock_ul();
    let lr: &mut Complex = &mut lock_lr();

    let c = (*ul + *lr) / 2.;
    let m = Complex {
        re: (1. - px) * ul.re + px * lr.re,
        im: (1. - py) * ul.im + py * lr.im,
    };

    *ul += m - c;
    *lr += m - c;

    let padding = (c - *ul) / 8.;
    *ul += padding;
    *lr -= padding;
}

fn build_ui(app: &Application) {
    let image = Image::from_paintable(Some(&Texture::for_pixbuf(&gen_pixbuf((
        APP_WIDTH, APP_HEIGHT,
    )))));

    let gesture = GestureClick::builder()
        .button(GDK_BUTTON_PRIMARY as u32)
        .build();
    gesture.connect_pressed({
        let image = image.clone();
        move |gesture, _, x, y| {
            recalculate_coordinates(x, y);
            gesture.set_state(gtk::EventSequenceState::Claimed);
            image.set_from_paintable(Some(&Texture::for_pixbuf(&gen_pixbuf((
                APP_WIDTH, APP_HEIGHT,
            )))))
        }
    });

    image.add_controller(gesture);

    // Create a window and set the title
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Mandelbrot")
        .child(&image)
        .default_height(APP_HEIGHT as i32)
        .default_width(APP_WIDTH as i32)
        .resizable(false)
        .build();

    let action_close = ActionEntry::builder("close")
        .activate(|window: &ApplicationWindow, _, _| {
            window.close();
        })
        .build();
    window.add_action_entries([action_close]);

    // Present window
    window.present();
}
