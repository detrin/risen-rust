
use synthrs::synthesizer::{make_samples, quantize_samples};
use synthrs::wave::{sine_wave, organ};
use synthrs::writer::write_wav_file;
use core::time::Duration;
use rodio::{OutputStream, source::Source};
use clap::{arg, command};

fn complexwave(frequency: f64) -> impl Fn(f64) -> f64 {
    move |t| {
        let mut amp = sine_wave(frequency)(t);
        for i in 2..10 {
            amp += sine_wave(2_i32.pow(i) as f64 * frequency)(t) / i as f64 * 1.0 / 1.3_f64.powf(i as f64);
        }
        amp
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
}

#[derive(Clone, Copy)]
enum Tone {
    Simple(Note, f64),
    Faded(Note, f64),
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

    let tone_length = 60.0 / tempo as f64 ;
    melody_tones = melody_tones.iter().map(|(note, duration)| (*note, *duration * tone_length)).collect();

    let melody = Melody {
        melody: melody_tones.iter().map(|(note, duration)| Tone::Faded(*note, *duration)).collect(),
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
