use std::path::{PathBuf, Path};
use std::fs::File;
use std::io::BufWriter;
use crate::Channels;

/// Visualizes audio as a waveform in a png file. If the data
/// is mono, it creates one file. If the data is stereo,
/// it creates two files (with left and right prefix)
pub fn visualize(samples: &[i16], channels: Channels, directory: &str, filename: &str) {
    let image_width = 1000;
    let image_height = 500;
    if channels.is_stereo() {
        assert_eq!(0, samples.len() % 2, "If stereo is provided, the length of the audio data must be even!");
        let (left, right) = channels.stereo_interleavement().to_channel_data(samples);
        visualize(&left, Channels::Mono, directory, &format!("left_{}", filename));
        visualize(&right, Channels::Mono, directory, &format!("right_{}", filename));
        return;
    }

    // needed for offset calculation; width per sample
    let width_per_sample = image_width as f64 / samples.len() as f64;
    // height in pixel per possible value of a sample; counts in that the y axis lays in the middle
    let height_per_max_amplitude = image_height as f64 / 2_f64 / i16::max_value() as f64;

    // RGB image data
    let mut image = vec![vec![(255,255,255); image_width]; image_height];
    for (sample_index, sample_value) in samples.into_iter().enumerate() {
        // x offset; from left
        let x = (sample_index as f64 * width_per_sample) as usize;
        // y offset; from top
        // image_height/2: there is our y-axis
        let y = ((image_height/2) as f64 + *sample_value as f64 * height_per_max_amplitude) as usize;

        image[y][x] = (0,0,0);
    }

    let mut path = PathBuf::new();
    path.push(directory);
    path.push(filename);
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, image_width as u32, image_height as u32); // Width is 2 pixels and height is 1.
    encoder.set_color(png::ColorType::RGB);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    // data must be RGBA sequence: RGBARGBARGBA...
    let rgb_data = image.into_iter()
        .flat_map(|row| row.into_iter())
        .flat_map(|(r, g, b)| vec![r, g, b].into_iter())
        .map(|v| v as u8)
        .collect::<Vec<u8>>();

    writer.write_image_data(&rgb_data).unwrap(); // Save
}

#[cfg(test)]
mod tests {
    use super::*;
    use minimp3::{Decoder as Mp3Decoder, Frame as Mp3Frame, Error as Mp3Error};
    use crate::test::{TEST_SAMPLES_DIR, TEST_OUT_DIR};
    use crate::ChannelInterleavement;

    #[test]
    fn test_visualize_1() {
        let mut path = PathBuf::new();
        path.push(TEST_SAMPLES_DIR);
        path.push("sample_1.mp3");
        let mut decoder = Mp3Decoder::new(File::open(path).unwrap());

        let mut lrlr_mp3_samples = vec![];
        loop {
            match decoder.next_frame() {
                Ok(Mp3Frame { data: samples_of_frame, .. }) => {
                    for sample in samples_of_frame {
                        lrlr_mp3_samples.push(sample);
                    }
                }
                Err(Mp3Error::Eof) => break,
                Err(e) => panic!("{:?}", e),
            }
        }

        visualize(
            &lrlr_mp3_samples,
            Channels::Stereo(ChannelInterleavement::LRLR),
            TEST_OUT_DIR,
            "sample_1_waveform.png"
        );
    }
}

