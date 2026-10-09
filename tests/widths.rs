use plugin::{render_status_line, render_stock_status_line, visible_width, StatusState};

#[test]
fn nano_status_bar_renders_every_width_1_to_100() {
    let state = StatusState::default();
    for cols in 1..=100 {
        let line = render_status_line(cols, &state);
        assert!(
            visible_width(&line) > 0 || cols < 4,
            "empty nano line at width {cols}"
        );
        assert!(
            visible_width(&line) <= cols,
            "nano width {} exceeds {cols}",
            visible_width(&line)
        );
    }
}

#[test]
fn stock_status_bar_renders_every_width_1_to_100() {
    let state = StatusState::default();
    for cols in 1..=100 {
        let line = render_stock_status_line(cols, &state);
        assert!(
            visible_width(&line) <= cols,
            "stock width {} exceeds {cols}",
            visible_width(&line)
        );
    }
}

#[test]
fn stock_and_nano_differ_at_80() {
    let state = StatusState::default();
    let stock = render_stock_status_line(80, &state);
    let nano = render_status_line(80, &state);
    assert_ne!(stock, nano);
    assert!(
        stock.contains('\u{e0b0}'),
        "stock line should use a chevron"
    );
    assert!(
        !nano.contains('\u{e0b0}'),
        "nano line should not use a chevron"
    );
}
