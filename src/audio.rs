use std::ffi::CString;
use std::path::Path;

use raylib::audio::RaylibAudio;
use raylib::ffi;

pub const BACKGROUND_MUSIC_PATH: &str = "src/assets/music/Aphex-Twin-Green-Calx.mp3";

#[derive(Default)]
pub struct AudioManager {
    _audio_device: Option<RaylibAudio>,
    music: Option<ffi::Music>,
}

impl AudioManager {
    pub fn new() -> Self {
        let audio_device = match RaylibAudio::init_audio_device() {
            Ok(device) => Some(device),
            Err(err) => {
                eprintln!("Aviso: No se pudo inicializar dispositivo de audio: {err}");
                None
            }
        };

        let mut manager = Self {
            _audio_device: audio_device,
            music: None,
        };

        if manager._audio_device.is_some() {
            manager.load_music(BACKGROUND_MUSIC_PATH);
        }

        manager
    }

    pub fn load_music(&mut self, path: &str) {
        if !Path::new(path).exists() {
            return;
        }

        let Ok(c_path) = CString::new(path) else {
            return;
        };

        unsafe {
            if ffi::IsAudioDeviceReady() {
                let music = ffi::LoadMusicStream(c_path.as_ptr());
                if !music.stream.buffer.is_null() {
                    ffi::SetMusicVolume(music, 0.60);
                    ffi::PlayMusicStream(music);
                    self.music = Some(music);
                }
            }
        }
    }

    pub fn update_music(&mut self) {
        if let Some(music) = self.music {
            unsafe {
                if ffi::IsAudioDeviceReady() {
                    ffi::UpdateMusicStream(music);
                    if !ffi::IsMusicStreamPlaying(music) {
                        ffi::PlayMusicStream(music);
                    }
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn set_music_volume(&mut self, volume: f32) {
        if let Some(music) = self.music {
            unsafe {
                if ffi::IsAudioDeviceReady() {
                    ffi::SetMusicVolume(music, volume.clamp(0.0, 1.0));
                }
            }
        }
    }

    pub fn play_shot(&mut self) {}

    pub fn play_hit_painting(&mut self) {}

    pub fn play_hit_guard(&mut self) {}

    pub fn play_alert(&mut self) {}

    pub fn play_damage(&mut self) {}

    pub fn play_success(&mut self) {}

    pub fn play_game_over(&mut self) {}

    pub fn play_ui(&mut self) {}
}

impl Drop for AudioManager {
    fn drop(&mut self) {
        if let Some(music) = self.music.take() {
            unsafe {
                if ffi::IsAudioDeviceReady() {
                    ffi::UnloadMusicStream(music);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_manager_handles_lifecycle_safely() {
        let mut audio = AudioManager::default();
        audio.update_music();
        audio.play_shot();
        audio.play_hit_painting();
        audio.play_hit_guard();
        audio.play_alert();
        audio.play_damage();
        audio.play_success();
        audio.play_game_over();
        audio.play_ui();
        audio.set_music_volume(0.5);
    }
}
