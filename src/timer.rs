use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

pub fn run_countdown(total_seconds: i32, title: String, tx: Sender<String>) {
    thread::spawn(move || {
        for remaining in (0..=total_seconds).rev() {
            let minutes = remaining / 60;
            let seconds = remaining % 60;
            let label = format!("{:02}:{:02}", minutes, seconds);
            let _ = tx.send(label);
            thread::sleep(Duration::from_secs(1));
        }

        let _ = Command::new("notify-send")
            .args(["--app-name=Jetimer", "Time's up!", &title])
            .status();

        play_alert_sound();

        let _ = tx.send("QUIT_NOW".to_string());
    });
}

pub fn get_zenity_output(args: &[&str]) -> Option<String> {
    let output = Command::new("zenity").args(args).output().ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn play_alert_sound() {
    let sound_path = "/usr/share/sounds/freedesktop/stereo/complete.oga";

    if !std::path::Path::new(sound_path).exists() {
        play_generated_beep();
        return;
    }

    if Command::new("pw-play").arg(sound_path).status().is_ok() {
        return;
    }

    if Command::new("paplay").arg(sound_path).status().is_ok() {
        return;
    }

    if Command::new("canberra-gtk-play")
        .args(["--file", sound_path, "--description", "Timer finished"])
        .status()
        .is_ok()
    {
        return;
    }

    if Command::new("ffplay")
        .args([sound_path, "-nodisp", "-autoexit", "-loglevel", "quiet"])
        .status()
        .is_ok()
    {
        return;
    }

    let _ = Command::new("echo").arg("-e").arg("\x07").status();
}
fn play_generated_beep() {
    let sample_rate: u32 = 44100;
    let duration_secs: f32 = 0.6;
    let frequency: f32 = 880.0;
    let samples_count = (sample_rate as f32 * duration_secs) as usize;

    let mut wav_data = Vec::new();
    let data_size = (samples_count * 2) as u32; // 16-bit = 2 bytes per sample

    // RIFF Header
    wav_data.extend_from_slice(b"RIFF");
    wav_data.extend_from_slice(&(36 + data_size).to_le_bytes());
    wav_data.extend_from_slice(b"WAVE");

    // fmt sub-chunk
    wav_data.extend_from_slice(b"fmt ");
    wav_data.extend_from_slice(&16u32.to_le_bytes()); // Subchunk1Size
    wav_data.extend_from_slice(&1u16.to_le_bytes()); // AudioFormat (PCM)
    wav_data.extend_from_slice(&1u16.to_le_bytes()); // NumChannels (Mono)
    wav_data.extend_from_slice(&sample_rate.to_le_bytes());
    wav_data.extend_from_slice(&(sample_rate * 2).to_le_bytes()); // ByteRate
    wav_data.extend_from_slice(&2u16.to_le_bytes()); // BlockAlign
    wav_data.extend_from_slice(&16u16.to_le_bytes()); // BitsPerSample

    // data sub-chunk
    wav_data.extend_from_slice(b"data");
    wav_data.extend_from_slice(&data_size.to_le_bytes());

    for i in 0..samples_count {
        let t = i as f32 / sample_rate as f32;

        let decay = (1.0 - (i as f32 / samples_count as f32)).powf(2.0);

        let sample = (t * frequency * 2.0 * std::f32::consts::PI).sin();
        let amplitude = (sample * i16::MAX as f32 * 0.4 * decay) as i16;

        wav_data.extend_from_slice(&amplitude.to_le_bytes());
    }

    let players = [
        ("pw-play", &["-"] as &[&str]),
        ("paplay", &[] as &[&str]),
        ("canberra-gtk-play", &["--file", "/dev/stdin"] as &[&str]),
        (
            "ffplay",
            &["-nodisp", "-autoexit", "-loglevel", "quiet", "-i", "pipe:0"] as &[&str],
        ),
    ];

    for (cmd, args) in players {
        if let Ok(mut child) = Command::new(cmd)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(&wav_data);
            }
            if let Ok(status) = child.wait() {
                if status.success() {
                    return;
                }
            }
        }
    }

    let _ = Command::new("echo").arg("-e").arg("\x07").status();
}
