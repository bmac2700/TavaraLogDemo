//=============================================================================//
//
// Tarkoitus: Sisältää funktion, jolla saa helposti tehtyä piippauksia
//
//
//
//=============================================================================//

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

//1250hz ja 200ms on kassakoneen tyylinen ääni
pub fn beep(hertz: f32, time: std::time::Duration) {
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();

    let config = device.default_output_config().unwrap();

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), hertz, time).unwrap(),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), hertz, time).unwrap(),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), hertz, time).unwrap(),
    };
}

#[allow(dead_code)]
pub fn run<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    hertz: f32,
    time: std::time::Duration,
) -> Result<(), ()>
where
    T: cpal::Sample,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    let mut sample_clock = 0f32;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        (sample_clock * hertz * 2.0 * std::f32::consts::PI / sample_rate).sin()
    };

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                write_data(data, channels, &mut next_value)
            },
            err_fn,
        )
        .unwrap();
    stream.play().unwrap();

    std::thread::sleep(time);

    Ok(())
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        let value: T = cpal::Sample::from::<f32>(&next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
