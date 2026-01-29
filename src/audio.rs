use cpal::{
    FromSample, SizedSample, SupportedStreamConfig, Device,
    traits::{DeviceTrait, HostTrait, StreamTrait}
};

use crossbeam_channel::{bounded, Receiver, Sender};

pub struct AudioPlayer {
    stream: cpal::Stream,
    sender: Sender<f32>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device()
            .expect("No output device available");

        let config = device.default_output_config().unwrap();

        let (sender, receiver) = bounded::<f32>(48000);

        let stream = match config.sample_format() {
            cpal::SampleFormat::I8 => Self::build_stream::<i8>(&device, &config.into(), receiver),
            cpal::SampleFormat::I16 => Self::build_stream::<i16>(&device, &config.into(), receiver),
            cpal::SampleFormat::I32 => Self::build_stream::<i32>(&device, &config.into(), receiver),
            cpal::SampleFormat::I64 => Self::build_stream::<i64>(&device, &config.into(), receiver),

            cpal::SampleFormat::U8 => Self::build_stream::<u8>(&device, &config.into(), receiver),
            cpal::SampleFormat::U16 => Self::build_stream::<u16>(&device, &config.into(), receiver),
            cpal::SampleFormat::U32 => Self::build_stream::<u32>(&device, &config.into(), receiver),
            cpal::SampleFormat::U64 => Self::build_stream::<u64>(&device, &config.into(), receiver),

            cpal::SampleFormat::F32 => Self::build_stream::<f32>(&device, &config.into(), receiver),
            cpal::SampleFormat::F64 => Self::build_stream::<f64>(&device, &config.into(), receiver),

            sample_format => panic!("Unsupported sample format '{sample_format}'"),
        };

        stream.play().unwrap();

        return Self {
            stream: stream,
            sender,
        };
    }

    fn build_stream<T>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        receiver: Receiver<f32>,
    ) -> cpal::Stream
    where
        T: cpal::Sample + SizedSample
    {
        device.build_output_stream(
            config,
            move |output: &mut [T], _| {
                for sample in output.iter_mut() {
                    let value: f32 = receiver.try_recv().unwrap_or(0.0);
 //                   *sample = T::from_sample(value);
                }
            },
            |err| eprintln!("Audio error: {:?}", err),
            None,
        )
        .unwrap()
    }
}


pub struct Output {
    device: Device,
    config: SupportedStreamConfig,
}

impl Output {
    pub fn initialise() -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host.default_output_device()
            .expect("no output device available");

        let config = device.default_output_config().unwrap();

        return Ok(Output { device, config });
    }

    pub fn play(&self, func: Box<dyn FnMut() -> f64 + Send + 'static>) {
        let device = self.device.clone();

        match self.config.sample_format() {
            // CASE: format is i8. DO => run as i8
            cpal::SampleFormat::I8 => run::<i8>(func, device, self.config.config()),
            cpal::SampleFormat::I16 => run::<i16>(func, device, self.config.config()),
            cpal::SampleFormat::I32 => run::<i32>(func, device, self.config.config()),
            cpal::SampleFormat::I64 => run::<i64>(func, device, self.config.config()),

            cpal::SampleFormat::U8 => run::<u8>(func, device, self.config.config()),
            cpal::SampleFormat::U16 => run::<u16>(func, device, self.config.config()),
            cpal::SampleFormat::U32 => run::<u32>(func, device, self.config.config()),
            cpal::SampleFormat::U64 => run::<u64>(func, device, self.config.config()),

            cpal::SampleFormat::F32 => run::<f32>(func, device, self.config.config()),
            cpal::SampleFormat::F64 => run::<f64>(func, device, self.config.config()),
            sample_format => panic!("Unsupported sample format '{sample_format}'"),
        };
    }
}

fn run<T>(
    mut func: Box<dyn FnMut() -> f64 + Send + 'static>, 
    device: cpal::Device, 
    config: cpal::StreamConfig
) where 
    T: SizedSample + FromSample<f64> + Send + 'static 
{
    println!("Runnign");
    std::thread::spawn(move || {
        println!("Running again");
        let channels = config.channels as usize;

        let mut next_value = move || func();
        let err_fn = |err| eprintln!("an error occurred on stream: {err}");

        let stream = device.build_output_stream(
            &config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                return process_frame(data, channels, &mut next_value);
            },
            err_fn,
            None,
        ).expect("failed to build stream");

        stream.play().expect("failed to play");


        loop {
            std::thread::sleep(std::time::Duration::from_millis(5000));
        }
    });
}

fn process_frame<T>(
    data: &mut [T],
    num_channels: usize,
    oscillator: &mut dyn FnMut() -> f64,
    //oscillator: &mut Oscillator,
) where
    T: SizedSample + FromSample<f64>,
{
    for frame in data.chunks_mut(num_channels) {
        let value: T = T::from_sample(oscillator());
        //let value: SampleType = SampleType::from_sample(oscillator.tick());

        // copy the same value to all channels
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
