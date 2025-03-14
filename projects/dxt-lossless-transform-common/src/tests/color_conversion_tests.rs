use crate::color_565::Color565;
use crate::color_8888::Color8888;

#[test]
fn can_convert_color_8888_to_565() {
    // Test with pure colors
    let red = Color8888::new(255, 0, 0, 255);
    let green = Color8888::new(0, 255, 0, 255);
    let blue = Color8888::new(0, 0, 255, 255);

    let red_565 = red.to_color_565();
    let green_565 = green.to_color_565();
    let blue_565 = blue.to_color_565();

    // Test that the converted colors have the correct components
    // The implementation in Color565 expands values to use all bits
    assert_eq!(red_565.red(), 255);
    assert_eq!(red_565.green(), 0);
    assert_eq!(red_565.blue(), 0);

    assert_eq!(green_565.red(), 0);
    assert_eq!(green_565.green(), 255);
    assert_eq!(green_565.blue(), 0);

    assert_eq!(blue_565.red(), 0);
    assert_eq!(blue_565.green(), 0);
    assert_eq!(blue_565.blue(), 255);
}

#[test]
fn can_convert_color_565_to_8888() {
    // Test with pure colors
    let red_565 = Color565::from_rgb(255, 0, 0);
    let green_565 = Color565::from_rgb(0, 255, 0);
    let blue_565 = Color565::from_rgb(0, 0, 255);

    let red_8888 = red_565.to_color_8888();
    let green_8888 = green_565.to_color_8888();
    let blue_8888 = blue_565.to_color_8888();

    // Test that the converted colors have the correct components
    assert_eq!(red_8888.r, 255);
    assert_eq!(red_8888.g, 0);
    assert_eq!(red_8888.b, 0);
    assert_eq!(red_8888.a, 255);

    assert_eq!(green_8888.r, 0);
    assert_eq!(green_8888.g, 255);
    assert_eq!(green_8888.b, 0);
    assert_eq!(green_8888.a, 255);

    assert_eq!(blue_8888.r, 0);
    assert_eq!(blue_8888.g, 0);
    assert_eq!(blue_8888.b, 255);
    assert_eq!(blue_8888.a, 255);
}

#[test]
fn can_round_trip_8888_to_565() {
    // Test various colors for round trip conversion
    let test_colors = [
        Color8888::new(255, 0, 0, 255),     // Red
        Color8888::new(0, 255, 0, 255),     // Green
        Color8888::new(0, 0, 255, 255),     // Blue
        Color8888::new(255, 255, 0, 255),   // Yellow
        Color8888::new(255, 0, 255, 255),   // Magenta
        Color8888::new(0, 255, 255, 255),   // Cyan
        Color8888::new(128, 128, 128, 255), // Gray
        Color8888::new(255, 128, 64, 255),  // Orange
        Color8888::new(64, 128, 255, 255),  // Light Blue
    ];

    for original in &test_colors {
        // Convert to RGB565 and back
        let color_565 = original.to_color_565();
        let round_trip = color_565.to_color_8888();

        // Due to precision loss in RGB565 format, we can't expect exact equality
        // Instead, check that the difference is within acceptable bounds (≤ 8 for each channel)
        let r_diff = if original.r >= round_trip.r {
            original.r - round_trip.r
        } else {
            round_trip.r - original.r
        };
        let g_diff = if original.g >= round_trip.g {
            original.g - round_trip.g
        } else {
            round_trip.g - original.g
        };
        let b_diff = if original.b >= round_trip.b {
            original.b - round_trip.b
        } else {
            round_trip.b - original.b
        };

        assert!(
            r_diff <= 8,
            "Red channel difference too large: {} vs {}",
            original.r,
            round_trip.r
        );
        assert!(
            g_diff <= 4,
            "Green channel difference too large: {} vs {}",
            original.g,
            round_trip.g
        );
        assert!(
            b_diff <= 8,
            "Blue channel difference too large: {} vs {}",
            original.b,
            round_trip.b
        );

        // Alpha should always be preserved as 255
        assert_eq!(round_trip.a, 255);
    }
}
