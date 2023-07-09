
use synthrs::synthesizer::{make_samples, quantize_samples};
use synthrs::wave::{sine_wave, organ};
use synthrs::writer::write_wav_file;
use core::time::Duration;
use rodio::{OutputStream, source::Source};
use clap::{arg, command};

// generated with generate_templeos_pattern()
static TEMPLEOS_FREQS: [(f64, f64); 40] = [(0.0, 0.7914783671016566), (-0.147540984, 0.7914783671016566), (0.147540984, 0.7914783671016566), (-0.27868852499999996, 0.7914783671016566), (0.27868852499999996, 0.7914783671016566), (-0.409836066, 0.7914783671016566), (0.409836066, 0.7914783671016566), (-0.540983607, 0.7914783671016566), (0.540983607, 0.7914783671016566), (-0.6721311480000001, 0.7914783671016566), (0.6721311480000001, 0.7914783671016566), (-0.8032786890000001, 0.7914783671016566), (0.8032786890000001, 0.7914783671016566), (-0.9344262300000001, 0.7914783671016566), (0.9344262300000001, 0.7914783671016566), (2.0, 0.5800207659791673), (1.852459016, 0.5800207659791673), (2.147540984, 0.5800207659791673), (1.721311475, 0.5800207659791673), (2.2786885249999997, 0.5800207659791673), (1.590163934, 0.5800207659791673), (2.409836066, 0.5800207659791673), (1.459016393, 0.5800207659791673), (2.5409836070000003, 0.5800207659791673), (1.327868852, 0.5800207659791673), (2.672131148, 0.5800207659791673), (1.196721311, 0.5800207659791673), (2.803278689, 0.5800207659791673), (1.0655737699999999, 0.5800207659791673), (2.93442623, 0.5800207659791673), (4.0, 0.4758686316592648), (3.852459016, 0.4758686316592648), (4.147540984, 0.4758686316592648), (3.7213114750000003, 0.4758686316592648), (4.278688525, 0.4758686316592648), (4.409836066, 0.4758686316592648), (4.540983607, 0.4758686316592648), (4.672131148, 0.4758686316592648), (4.803278689, 0.4758686316592648), (4.93442623, 0.4758686316592648)];

#[allow(dead_code)]
fn generate_templeos_pattern() -> Vec<(f64, f64)> {
    let decay_constant = 0.19999992;
    let c0 = 0.43407478;
    let c1 = 0.45959631;
    let mut freqs = Vec::new();
    let mut base_freq = 0.0;
    for i in 1..=3 {
        let mut add_freq = 0.147540984;
        let max_amplitude = 1.0 - decay_constant * (i - 1) as f64;
        let mut amplitudes = Vec::new();
        for _ in 0..=7 {
            amplitudes.push((-c1 * i as f64).exp() * (max_amplitude - c0) + c0);
        }
        freqs.push((base_freq, amplitudes[0]));
        for j in 1..=7 {
            for sign in [-1.0, 1.0] {
                if sign == -1.0 && j >2 && i == 3 {
                    continue;
                }
                let freq = base_freq + sign * add_freq;
                let amplitude = amplitudes[j];

                freqs.push((freq, amplitude));
            }
            add_freq += 0.131147541
        }
        base_freq += 2.0;
    }
    freqs
}

fn complexwave(frequency: f64) -> impl Fn(f64) -> f64 {
    let freqs = TEMPLEOS_FREQS;
    move |t: f64| -> f64 {
        let mut sum = 0.0;
        for (freq, amplitude) in freqs.iter() {
            sum += amplitude * sine_wave(*freq * frequency)(t);
        }
        sum
    }
}

#[derive(Clone, Copy)]
struct Note {
    letter: char,
    octave: i32,
}

impl Note {
    fn to_frequency(&self) -> f64 {
        let mut semitones = match self.letter {
            'c' => -9,
            'd' => -7,
            'e' => -5,
            'f' => -4,
            'g' => -2,
            'a' => 0,
            'b' => 2,
            _ => panic!("Invalid note"),
        };
        semitones += (self.octave - 4) * 12;
        440.0 * 2.0_f64.powf(semitones as f64 / 12.0)
    }

    fn create_simple_tone(&self, duration: f64, sample_rate: u32) -> Vec<f64> {
        let frequency = self.to_frequency();
        make_samples(duration, sample_rate as usize, |t: f64| -> f64 {
            0.5 * sine_wave(frequency)(t)
        })
    }

    fn create_faded_tone(&self, duration: f64, sample_rate: u32, tone_length: f64) -> Vec<f64> {
        let frequency = self.to_frequency();
        let pause_length = 1.0 - tone_length;
        make_samples(duration, sample_rate as usize, |t: f64| -> f64 {
            let amp = organ(frequency)(t);
            if t < 2.0 * pause_length * duration {
                0.5 * amp * (t / (2.0 * pause_length * duration))
            } else if t < (1.0 - 2.0 * pause_length) * duration {
                0.5 * amp
            } else {
                0.5 * amp * ((duration - t) / (2.0 * pause_length * duration))
            }
        })
    }    

    fn create_templeos_faded_tone(&self, duration: f64, sample_rate: u32, tone_length: f64) -> Vec<f64> {
        let frequency = self.to_frequency();
        let pause_length = 1.0 - tone_length;
        make_samples(duration, sample_rate as usize, |t: f64| -> f64 {
            let amp = complexwave(frequency)(t);
            if t < 2.0 * pause_length * duration {
                0.5 * amp * (t / (2.0 * pause_length * duration))
            } else if t < (1.0 - 2.0 * pause_length) * duration {
                0.5 * amp
            } else {
                0.5 * amp * ((duration - t) / (2.0 * pause_length * duration))
            }
        })
    }  
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum Tone {
    Simple(Note, f64),
    Faded(Note, f64),
    TempleOS(Note, f64),
}

struct Melody {
    melody: Vec<Tone>,
    sample_rate: u32,
    tone_length: f64,
}

impl Melody {
    fn create_melody(&self) -> Vec<f64> {
        let mut samples = Vec::new();
        for tone in &self.melody {
            match tone {
                Tone::Simple(note, duration) => {
                    let mut note_samples = note.create_simple_tone(*duration, self.sample_rate);
                    samples.append(&mut note_samples);
                },
                Tone::Faded(note, duration) => {
                    let mut note_samples = note.create_faded_tone(*duration, self.sample_rate, self.tone_length);
                    samples.append(&mut note_samples);
                },
                Tone::TempleOS(note, duration) => {
                    let mut note_samples = note.create_templeos_faded_tone(*duration, self.sample_rate, self.tone_length);
                    samples.append(&mut note_samples);
                },
            }
        }
        samples
    }
}

#[derive(Clone)]
struct MelodyAudio {
    melody: Vec<f64>,
    sample_rate: u32,
    position: usize,
}

impl MelodyAudio {
    fn from_melody(melody: Melody) -> MelodyAudio {
        MelodyAudio {
            melody: melody.create_melody(),
            sample_rate: melody.sample_rate,
            position: 0,
        }
    }
}

impl Source for MelodyAudio {
    fn channels(&self) -> u16 {
        return 1;
    }

    fn sample_rate(&self) -> u32 {
        return self.sample_rate;
    }   

    fn current_frame_len(&self) -> Option<usize> {
        return None;
    }

    fn total_duration(&self) -> Option<Duration> {
        return None;
    }
}

impl Iterator for MelodyAudio {
    type Item = f32;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= self.melody.len() {
            return None;
        }
        let sample = self.melody[self.position] as f32;
        self.position += 1;
        Some(sample)
    }
}

fn main() {
    let matches = command!() 
        .arg(arg!([output] "The output select from ['file', 'speaker']"))
        .arg(arg!([tempo] "The tempo of the song"))
        .get_matches();

    let output = matches.get_one::<String>("output").unwrap();
    let mut tempo: u64 = 90;
    if let Some(tempo_str) = matches.get_one::<String>("tempo") {
        tempo = tempo_str.parse::<u64>().unwrap();
    }

    let melody_first = [
        (Note { letter: 'd', octave: 5 }, 0.5),
        (Note { letter: 'e', octave: 5 }, 0.5),
        (Note { letter: 'f', octave: 5 }, 1.0),
        (Note { letter: 'f', octave: 5 }, 1.0),
        (Note { letter: 'e', octave: 5 }, 0.25),
        (Note { letter: 'e', octave: 5 }, 0.25),
        (Note { letter: 'f', octave: 5 }, 0.5),

        (Note { letter: 'd', octave: 5 }, 1.0),
        (Note { letter: 'c', octave: 5 }, 0.5),
        (Note { letter: 'd', octave: 5 }, 0.5),
        (Note { letter: 'd', octave: 5 }, 0.5),
        (Note { letter: 'e', octave: 5 }, 0.5),
        (Note { letter: 'c', octave: 5 }, 0.5),
        (Note { letter: 'g', octave: 5 }, 0.25),
        (Note { letter: 'f', octave: 5 }, 0.25),
    ];

    let melody_second = [
        (Note { letter: 'd', octave: 5 }, 0.5),
        (Note { letter: 'c', octave: 5 }, 0.5),
        (Note { letter: 'd', octave: 5 }, 1.0),
        (Note { letter: 'e', octave: 5 }, 1.0),
        (Note { letter: 'a', octave: 4 }, 0.5),
        (Note { letter: 'a', octave: 4 }, 0.5),

        (Note { letter: 'e', octave: 5 }, 0.25),
        (Note { letter: 'e', octave: 5 }, 0.25),
        (Note { letter: 'f', octave: 5 }, 0.5),
        (Note { letter: 'e', octave: 5 }, 0.5),
        (Note { letter: 'd', octave: 5 }, 0.25),
        (Note { letter: 'g', octave: 5 }, 0.25),
        (Note { letter: 'b', octave: 4 }, 0.5),
        (Note { letter: 'd', octave: 5 }, 0.25),
        (Note { letter: 'c', octave: 5 }, 0.25),
        (Note { letter: 'f', octave: 5 }, 1.0),
    ];

    let mut melody_tones = Vec::with_capacity(melody_first.len() * 2 + melody_second.len() * 2);
    melody_tones.extend_from_slice(&melody_first);
    melody_tones.extend_from_slice(&melody_first);
    melody_tones.extend_from_slice(&melody_second);
    melody_tones.extend_from_slice(&melody_second);
    //  add one to octave
    // melody_tones = melody_tones.iter().map(|(note, duration)| (Note { letter: note.letter, octave: note.octave + 1 }, *duration)).collect();

    let tone_length = 60.0 / tempo as f64 ;
    melody_tones = melody_tones.iter().map(|(note, duration)| (*note, *duration * tone_length)).collect();

    let melody = Melody {
        melody: melody_tones.iter().map(|(note, duration)| Tone::TempleOS(*note, *duration)).collect(),
        sample_rate: 44_100,
        tone_length: 0.98,
    };

    let melody_audio = MelodyAudio::from_melody(melody);
    let melody_len = melody_audio.melody.len();

    if output == "speaker" {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();

        let _result = stream_handle.play_raw(melody_audio);
        let length_in_secs = melody_len as f64 / 44_100.0;

        std::thread::sleep(std::time::Duration::from_secs(length_in_secs as u64 + 1));
        return;
    } else if output == "file" {
        write_wav_file(
            "out/risen.wav",
            44_100,
            &quantize_samples::<i16>(&melody_audio.melody),
        ).expect("failed");
    }

}
