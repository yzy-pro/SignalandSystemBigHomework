use plotters::prelude::*;
use crate::filter_response;

const PLOT_WIDTH: u32 = 1200;
const PLOT_HEIGHT: u32 = 600;

/// Plot magnitude response (linear scale)
pub fn plot_magnitude_response(
    frequencies: &[f64],
    magnitude: &[f64],
    output_path: &str,
    title: &str,
    max_freq: Option<f64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(output_path, (PLOT_WIDTH, PLOT_HEIGHT)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_freq_val = max_freq.unwrap_or(*frequencies.last().unwrap_or(&10000.0));
    let max_mag = magnitude.iter()
        .zip(frequencies.iter())
        .filter(|(_, &f)| f <= max_freq_val)
        .map(|(&m, _)| m)
        .fold(0.0, f64::max)
        .max(1.1);

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 30).into_font())
        .margin(15)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..max_freq_val, 0.0..max_mag)?;

    chart.configure_mesh()
        .x_desc("Frequency (Hz)")
        .y_desc("Magnitude")
        .draw()?;

    chart.draw_series(LineSeries::new(
        frequencies.iter()
            .zip(magnitude.iter())
            .filter(|(&f, _)| f <= max_freq_val)
            .map(|(&f, &m)| (f, m)),
        &BLUE,
    ))?;

    root.present()?;
    Ok(())
}

/// Plot magnitude response in dB scale
pub fn plot_magnitude_response_db(
    frequencies: &[f64],
    magnitude: &[f64],
    output_path: &str,
    title: &str,
    max_freq: Option<f64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(output_path, (PLOT_WIDTH, PLOT_HEIGHT)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_freq_val = max_freq.unwrap_or(*frequencies.last().unwrap_or(&10000.0));
    
    let magnitude_db: Vec<f64> = magnitude.iter()
        .map(|&m| filter_response::magnitude_to_db(m))
        .collect();

    let min_db = magnitude_db.iter()
        .zip(frequencies.iter())
        .filter(|(_, &f)| f <= max_freq_val)
        .map(|(&db, _)| db)
        .fold(f64::INFINITY, f64::min)
        .max(-80.0);
    
    let max_db = 10.0;

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 30).into_font())
        .margin(15)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..max_freq_val, min_db..max_db)?;

    chart.configure_mesh()
        .x_desc("Frequency (Hz)")
        .y_desc("Magnitude (dB)")
        .draw()?;

    chart.draw_series(LineSeries::new(
        frequencies.iter()
            .zip(magnitude_db.iter())
            .filter(|(&f, _)| f <= max_freq_val)
            .map(|(&f, &db)| (f, db)),
        &BLUE,
    ))?;

    root.present()?;
    Ok(())
}

/// Plot phase response
pub fn plot_phase_response(
    frequencies: &[f64],
    phase: &[f64],
    output_path: &str,
    title: &str,
    max_freq: Option<f64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(output_path, (PLOT_WIDTH, PLOT_HEIGHT)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_freq_val = max_freq.unwrap_or(*frequencies.last().unwrap_or(&10000.0));

    let phase_deg: Vec<f64> = phase.iter()
        .map(|&p| filter_response::phase_to_degrees(p))
        .collect();

    let min_phase = phase_deg.iter()
        .zip(frequencies.iter())
        .filter(|(_, &f)| f <= max_freq_val)
        .map(|(&p, _)| p)
        .fold(f64::INFINITY, f64::min) - 10.0;
    
    let max_phase = phase_deg.iter()
        .zip(frequencies.iter())
        .filter(|(_, &f)| f <= max_freq_val)
        .map(|(&p, _)| p)
        .fold(f64::NEG_INFINITY, f64::max) + 10.0;

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 30).into_font())
        .margin(15)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..max_freq_val, min_phase..max_phase)?;

    chart.configure_mesh()
        .x_desc("Frequency (Hz)")
        .y_desc("Phase (degrees)")
        .draw()?;

    chart.draw_series(LineSeries::new(
        frequencies.iter()
            .zip(phase_deg.iter())
            .filter(|(&f, _)| f <= max_freq_val)
            .map(|(&f, &p)| (f, p)),
        &RED,
    ))?;

    root.present()?;
    Ok(())
}

/// Plot combined magnitude responses of high-pass and low-pass filters
pub fn plot_combined_magnitude(
    frequencies: &[f64],
    hp_magnitude: &[f64],
    lp_magnitude: &[f64],
    output_path: &str,
    title: &str,
    max_freq: Option<f64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(output_path, (PLOT_WIDTH, PLOT_HEIGHT)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_freq_val = max_freq.unwrap_or(*frequencies.last().unwrap_or(&10000.0));
    
    let max_mag = hp_magnitude.iter()
        .chain(lp_magnitude.iter())
        .zip(frequencies.iter().cycle())
        .filter(|(_, &f)| f <= max_freq_val)
        .map(|(&m, _)| m)
        .fold(0.0, f64::max)
        .max(1.1);

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 30).into_font())
        .margin(15)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..max_freq_val, 0.0..max_mag)?;

    chart.configure_mesh()
        .x_desc("Frequency (Hz)")
        .y_desc("Magnitude")
        .draw()?;

    // Draw high-pass filter
    chart.draw_series(LineSeries::new(
        frequencies.iter()
            .zip(hp_magnitude.iter())
            .filter(|(&f, _)| f <= max_freq_val)
            .map(|(&f, &m)| (f, m)),
        &BLUE,
    ))?.label("High-pass")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    // Draw low-pass filter
    chart.draw_series(LineSeries::new(
        frequencies.iter()
            .zip(lp_magnitude.iter())
            .filter(|(&f, _)| f <= max_freq_val)
            .map(|(&f, &m)| (f, m)),
        &RED,
    ))?.label("Low-pass")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;
    Ok(())
}
