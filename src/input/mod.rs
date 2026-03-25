use xcap::{Monitor, XCapResult};

pub fn primary_monitor() -> XCapResult<Monitor> {
    Monitor::all().map(|v| {
        v.into_iter()
            .find(|m| m.is_primary().unwrap())
            .expect("how do you not have a primary monitor")
    })
}

fn foo() {
    let monitor = primary_monitor().unwrap();
    monitor.capture_image()
}
