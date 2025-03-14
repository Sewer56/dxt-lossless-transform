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
    // Test colors that should perfectly round-trip from 8888 to 565 and back.
    let test_colors = [
        Color8888::new(255, 0, 0, 255),     // Red
        Color8888::new(0, 255, 0, 255),     // Green
        Color8888::new(0, 0, 255, 255),     // Blue
        Color8888::new(0, 0, 0, 255),       // Black
        Color8888::new(255, 255, 255, 255), // White
        Color8888::new(132, 130, 132, 255), // Gray (with small error)
        Color8888::new(0, 255, 255, 255),   // Cyan
        Color8888::new(255, 0, 255, 255),   // Magenta
        Color8888::new(255, 255, 0, 255),   // Yellow
    ];

    for original in &test_colors {
        // Convert to RGB565 and back
        let color_565 = original.to_color_565();
        let round_trip = color_565.to_color_8888();

        assert_eq!(
            original.r, round_trip.r,
            "Red channel should be preserved exactly for values divisible by 8: {} vs {}",
            original.r, round_trip.r
        );
        assert_eq!(
            original.g, round_trip.g,
            "Green channel should be preserved exactly for values divisible by 4: {} vs {}",
            original.g, round_trip.g
        );
        assert_eq!(
            original.b, round_trip.b,
            "Blue channel should be preserved exactly for values divisible by 8: {} vs {}",
            original.b, round_trip.b
        );

        // Alpha should always be preserved as 255
        assert_eq!(round_trip.a, 255);
    }
}
