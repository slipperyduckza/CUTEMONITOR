use iced::Color;

/// Color-coding for temperature values (10°C to 80°C range)
/// Maps temperature to intuitive gradient from cool blues to hot reds
pub fn temperature_color(temp: f32) -> Color {
    let points = [
        (10.0, 255, 255, 230),   // Very cool - light blue
        (24.0, 255, 255, 0),     // Cool - cyan  
        (38.0, 255, 191, 0),     // Warm - yellow
        (52.0, 255, 128, 0),     // Hot - orange
        (66.0, 255, 64, 0),      // Very hot - red-orange
        (80.0, 255, 0, 0),       // Extremely hot - red
    ];

    // Clamp temperature to range
    let clamped_temp = temp.clamp(points[0].0, points.last().unwrap().0);

    // Interpolate between color points
    for i in 0..points.len() - 1 {
        let (t1, r1, g1, b1) = points[i];
        let (t2, r2, g2, b2) = points[i + 1];
        if clamped_temp >= t1 && clamped_temp <= t2 {
            let ratio = (clamped_temp - t1) / (t2 - t1);
            let r = r1 as f32 + (r2 as f32 - r1 as f32) * ratio;
            let g = g1 as f32 + (g2 as f32 - g1 as f32) * ratio;
            let b = b1 as f32 + (b2 as f32 - b1 as f32) * ratio;
            return Color::from_rgb(r / 255.0, g / 255.0, b / 255.0);
        }
    }
    
    // Fallback to last point color
    let last = points.last().unwrap();
    Color::from_rgb(last.1 as f32 / 255.0, last.2 as f32 / 255.0, last.3 as f32 / 255.0)
}

/// Color-coding for utilization percentages with exponential curve
/// Uses power 3.8 for smoother transitions at low-mid usage
pub fn utilization_color(utilization: f32) -> Color {
    let clamped_util = utilization.clamp(0.0, 100.0);
    let ratio = clamped_util / 100.0;
    
    // Apply exponential curve for smoother low-mid range transitions
    let exp_ratio = ratio.powf(3.8);
    
    // Color points: white (0%) -> light red (50%) -> red (100%)
    if exp_ratio <= 0.5 {
        // White to light red
        let local_ratio = exp_ratio * 2.0;
        let r = 1.0;
        let g = 1.0 - local_ratio * 0.5; // 1.0 -> 0.5
        let b = 1.0 - local_ratio * 0.5; // 1.0 -> 0.5
        Color::from_rgb(r, g, b)
    } else {
        // Light red to red
        let local_ratio = (exp_ratio - 0.5) * 2.0;
        let r = 1.0;
        let g = 0.5 - local_ratio * 0.5; // 0.5 -> 0.0
        let b = 0.5 - local_ratio * 0.5; // 0.5 -> 0.0
        Color::from_rgb(r, g, b)
    }
}

/// Maps power consumption to temperature-equivalent colors
/// Range: 10W to 200W, mapped to 10°C to 80°C temperature colors
#[allow(dead_code)]
pub fn power_color(power: f32) -> Color {
    let clamped_power = power.clamp(10.0, 200.0);
    
    // Map power range to temperature range (10°C to 80°C)
    // Formula: temp_equiv = 10.0 + (power - 10.0) * 70.0 / 190.0
    let temp_equiv = 10.0 + (clamped_power - 10.0) * 70.0 / 190.0;
    
    // Use temperature color mapping
    temperature_color(temp_equiv)
}

/// Color-coding for voltage levels
/// Range: 0.5V to 1.5V, mapped from white to blue
pub fn voltage_color(voltage: f32) -> Color {
    let clamped_voltage = voltage.clamp(0.5, 1.5);
    let ratio = (clamped_voltage - 0.5) / 1.0; // Normalize to 0.0-1.0
    
    // Color points: white (0.5V) -> light blue (1.0V) -> blue (1.5V)
    if ratio <= 0.5 {
        // White to light blue
        let local_ratio = ratio * 2.0;
        let r = 1.0 - local_ratio * 0.5; // 1.0 -> 0.5
        let g = 1.0 - local_ratio * 0.25; // 1.0 -> 0.75
        let b = 1.0; // 1.0 -> 1.0
        Color::from_rgb(r, g, b)
    } else {
        // Light blue to blue
        let local_ratio = (ratio - 0.5) * 2.0;
        let r = 0.5 - local_ratio * 0.5; // 0.5 -> 0.0
        let g = 0.75 - local_ratio * (0.75 - 0.5); // 0.75 -> 0.5
        let b = 1.0; // 1.0 -> 1.0
        Color::from_rgb(r, g, b)
    }
}

/// Maps memory usage percentage to voltage-equivalent colors
/// Range: 2% to 98%, mapped to voltage range (0.5V-1.5V)
pub fn memory_color(memory_usage: f32) -> Color {
    let clamped_usage = memory_usage.clamp(2.0, 98.0);
    
    // Map memory usage to voltage range (0.5V to 1.5V)
    // Formula: voltage_equiv = 0.5 + (usage - 2.0) * 1.0 / 96.0
    let voltage_equiv = 0.5 + (clamped_usage - 2.0) * 1.0 / 96.0;
    
    // Use voltage color mapping
    voltage_color(voltage_equiv)
}