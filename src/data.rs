use raylib::prelude::*;
use std::time::{Duration, Instant};

pub mod history;

pub fn draw_graph_slice<D, I>(
    d: &mut D,
    src: I,
    start_time: Instant,
    over_last: Duration,
    dx: std::ops::RangeInclusive<i32>,
    dy: std::ops::RangeInclusive<i32>,
    color: Color,
) -> Option<()>
where
    D: RaylibDraw,
    I: Iterator<Item = (Instant, i32)> + Clone,
{
    let earliest = Instant::now() - over_last;
    let last = src.clone().last().map(|(_, y)| (Instant::now(), y));
    draw_graph(
        d,
        src.chain(last)
            .skip_while(|(timestamp, _)| timestamp < &earliest)
            .map(|(timestamp, y)| ((timestamp - start_time).as_millis() as i32, y)),
        dx,
        dy,
        color,
    )
}

pub fn draw_graph<D, I>(
    d: &mut D,
    src: I,
    dx: std::ops::RangeInclusive<i32>,
    dy: std::ops::RangeInclusive<i32>,
    color: Color,
) -> Option<()>
where
    D: RaylibDraw,
    I: Iterator<Item = (i32, i32)> + Clone,
{
    use raylib::prelude::*;
    use std::num::NonZeroI32;

    let xrange = (dx.end() - dx.start()) as f32;
    let yrange = (dy.end() - dy.start()) as f32;

    let xmin = *dx.start() as f32;
    let ymax = *dy.end() as f32;

    let x_it = src.clone().map(|(x, _)| x);
    let first = x_it.clone().min()?;
    let last = x_it.max()?;
    let domain = (last.checked_sub(first)).and_then(NonZeroI32::new)?.get() as f32;

    let y_it = src.clone().map(|(_, y)| y);
    let lowest = y_it.clone().min()?;
    let highest = y_it.max()?;
    let range = (highest.checked_sub(lowest))
        .and_then(NonZeroI32::new)?
        .get() as f32;

    let remap = |(x, y): (i32, i32)| -> Vector2 {
        Vector2::new(
            xmin + ((x - first) as f32 * xrange) / domain,
            ymax - ((y - lowest) as f32 * yrange) / range,
        )
    };

    for [p1, p2] in src.map_windows::<_, _, 2>(|&item| item) {
        let p1 = remap(p1);
        let p2 = remap(p2);
        d.draw_line_strip(&[p1, Vector2::new(p1.x, p2.y), p2], color);
    }

    Some(())
}
