// This module provides color-coding functions for different types of data
// Colors help users quickly understand if values are in safe, warning, or danger ranges

use iced::Color;

/// Returns a color based on temperature value for visual feedback
/// Uses a gradient from blue (cool) to red (hot) to indicate temperature ranges
/// - Blue: Very cool temperatures
/// - Cyan: Cool temperatures
/// - Yellow: Warm temperatures
/// - Orange: Hot temperatures
/// - Red: Very hot/dangerous temperatures
pub fn temperature_color(temp: f32) -> Color {
    // Define color points: (temperature, red, green, blue)
    // These create a smooth gradient between temperature ranges
    let points = [
        (10.0, 255, 255, 230), // Very cool - light blue
        (24.0, 255, 255, 0),   // Cool - cyan
        (38.0, 255, 191, 0),   // Warm - yellow
        (52.0, 255, 128, 0),   // Hot - orange
        (66.0, 255, 64, 0),    // Very hot - red-orange
        (80.0, 255, 0, 0),     // Extremely hot - red
    ];

    // Handle temperatures below the lowest point
    if temp <= points[0].0 {
        return Color::from_rgb(
            points[0].1 as f32 / 255.0,
            points[0].2 as f32 / 255.0,
            points[0].3 as f32 / 255.0,
        );
    }

    // Handle temperatures above the highest point
    if temp >= points.last().unwrap().0 {
        let last = points.last().unwrap();
        return Color::from_rgb(
            last.1 as f32 / 255.0,
            last.2 as f32 / 255.0,
            last.3 as f32 / 255.0,
        );
    }

    // Interpolate between the appropriate color points
    for i in 0..points.len() - 1 {
        let (t1, r1, g1, b1) = points[i];
        let (t2, r2, g2, b2) = points[i + 1];
        if temp >= t1 && temp <= t2 {
            // Calculate how far we are between the two points (0.0 to 1.0)
            let ratio = (temp - t1) / (t2 - t1);
            // Interpolate each color component
            let r = r1 as f32 + (r2 as f32 - r1 as f32) * ratio;
            let g = g1 as f32 + (g2 as f32 - g1 as f32) * ratio;
            let b = b1 as f32 + (b2 as f32 - b1 as f32) * ratio;
            return Color::from_rgb(r / 255.0, g / 255.0, b / 255.0);
        }
    }

    // Fallback color (should never reach here with proper data)
    Color::from_rgb(1.0, 1.0, 1.0)
}

/// Returns a color based on power consumption value
/// Maps power usage to temperature colors for consistency
/// Low power = cool colors, high power = hot colors
pub fn power_color(power: f32) -> Color {
    // Clamp power values to reasonable ranges
    if power <= 10.0 {
        return temperature_color(10.0); // Very low power
    }
    if power >= 200.0 {
        return temperature_color(80.0); // Very high power
    }

    // Map power range (10-200W) to temperature range (10-80°C)
    let temp_equiv = 10.0 + (power - 10.0) * (80.0 - 10.0) / (200.0 - 10.0);
    temperature_color(temp_equiv)
}

pub fn voltage_color(voltage: f32) -> Color {
    let points = [(0.5, 255, 255, 255),
        (1.0, 128, 191, 255),
        (1.5, 0, 128, 255)];

    if voltage <= points[0].0 {
        return Color::from_rgb(
            points[0].1 as f32 / 255.0,
            points[0].2 as f32 / 255.0,
            points[0].3 as f32 / 255.0,
        );
    }

    if voltage >= points.last().unwrap().0 {
        let last = points.last().unwrap();
        return Color::from_rgb(
            last.1 as f32 / 255.0,
            last.2 as f32 / 255.0,
            last.3 as f32 / 255.0,
        );
    }

    for i in 0..points.len() - 1 {
        let (v1, r1, g1, b1) = points[i];
        let (v2, r2, g2, b2) = points[i + 1];
        if voltage >= v1 && voltage <= v2 {
            let ratio = (voltage - v1) / (v2 - v1);
            let r = r1 as f32 + (r2 as f32 - r1 as f32) * ratio;
            let g = g1 as f32 + (g2 as f32 - g1 as f32) * ratio;
            let b = b1 as f32 + (b2 as f32 - b1 as f32) * ratio;
            return Color::from_rgb(r / 255.0, g / 255.0, b / 255.0);
        }
    }

    // Fallback
    Color::from_rgb(1.0, 1.0, 1.0)
}

pub fn memory_color(usage: f32) -> Color {
    if usage <= 2.0 {
        return voltage_color(0.5);
    }
    if usage >= 98.0 {
        return voltage_color(1.5);
    }
    let volt_equiv = 0.5 + (usage - 2.0) * (1.5 - 0.5) / (98.0 - 2.0);
    voltage_color(volt_equiv)
}
