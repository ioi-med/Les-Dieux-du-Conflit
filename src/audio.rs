//! Ambiances et effets sonores intégrés au jeu.
//! Sons libres CC0 ; sources et licences : `assets/audio/CREDITS-AUDIO.md`.
//! La musique de fond est jouée en boucle, un seul morceau à la fois.
//! Si aucune carte son n'est disponible, les chargements échouent proprement
//! et le jeu continue sans son.

use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};

/// Effets sonores du jeu.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sfx {
    Swing,     // coup d'épée
    Hit,       // touche un ennemi
    Hurt,      // le joueur est blessé
    Die,       // un ennemi meurt
    Coin,      // document ramassé
    Heal,      // soin
    Door,      // une issue s'ouvre, un quartier change
    Ice,       // Glace éternelle lancée
    IceBlock,  // statue de glace qui se brise
    Portal,    // passage entre les étages
    Roar,      // un dieu s'éveille
    Explosion, // éclats d'un projectile
    Ui,        // petit clic d'interface
    Ui2,       // validation
    Flame,     // flamme verte
    Wave,      // vague de sang
    Demigod,   // le Panthéon reconnaît le joueur
}

impl Sfx {
    fn index(self) -> usize {
        match self {
            Sfx::Swing => 0,
            Sfx::Hit => 1,
            Sfx::Hurt => 2,
            Sfx::Die => 3,
            Sfx::Coin => 4,
            Sfx::Heal => 5,
            Sfx::Door => 6,
            Sfx::Ice => 7,
            Sfx::IceBlock => 8,
            Sfx::Portal => 9,
            Sfx::Roar => 10,
            Sfx::Explosion => 11,
            Sfx::Ui => 12,
            Sfx::Ui2 => 13,
            Sfx::Flame => 14,
            Sfx::Wave => 15,
            Sfx::Demigod => 16,
        }
    }
}

const SFX_COUNT: usize = 17;
const SFX_BYTES: [&[u8]; SFX_COUNT] = [
    include_bytes!("../assets/audio/swing.ogg"),
    include_bytes!("../assets/audio/hit.ogg"),
    include_bytes!("../assets/audio/hurt.ogg"),
    include_bytes!("../assets/audio/die.ogg"),
    include_bytes!("../assets/audio/coin.ogg"),
    include_bytes!("../assets/audio/heal.ogg"),
    include_bytes!("../assets/audio/door.ogg"),
    include_bytes!("../assets/audio/ice.ogg"),
    include_bytes!("../assets/audio/iceblock.ogg"),
    include_bytes!("../assets/audio/portal.ogg"),
    include_bytes!("../assets/audio/roar.ogg"),
    include_bytes!("../assets/audio/explosion.ogg"),
    include_bytes!("../assets/audio/ui.ogg"),
    include_bytes!("../assets/audio/ui2.ogg"),
    include_bytes!("../assets/audio/flame.ogg"),
    include_bytes!("../assets/audio/wave.ogg"),
    include_bytes!("../assets/audio/demigod.ogg"),
];

/// Musique d'ambiance, une piste à la fois.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Music {
    None,
    Menu,
    Surface1,
    Surface2,
    Surface3,
    Boss,
    Backrooms,
    Pantheon,
    Victory,
}

impl Music {
    fn index(self) -> usize {
        match self {
            Music::None => 0,
            Music::Menu => 0,
            Music::Surface1 => 1,
            Music::Surface2 => 2,
            Music::Surface3 => 3,
            Music::Boss => 4,
            Music::Backrooms => 5,
            Music::Pantheon => 6,
            Music::Victory => 7,
        }
    }
    fn volume(self) -> f32 {
        match self {
            Music::None => 0.,
            Music::Menu => 0.55,
            Music::Surface1 | Music::Surface2 | Music::Surface3 => 0.48,
            Music::Boss => 0.58,
            Music::Backrooms => 0.62,
            Music::Pantheon => 0.52,
            Music::Victory => 0.6,
        }
    }
}

const MUSIC_COUNT: usize = 8;
const MUSIC_BYTES: [&[u8]; MUSIC_COUNT] = [
    include_bytes!("../assets/audio/menu.ogg"),
    include_bytes!("../assets/audio/surface1.ogg"),
    include_bytes!("../assets/audio/surface2.ogg"),
    include_bytes!("../assets/audio/surface3.ogg"),
    include_bytes!("../assets/audio/boss.ogg"),
    include_bytes!("../assets/audio/backrooms.ogg"),
    include_bytes!("../assets/audio/pantheon.ogg"),
    include_bytes!("../assets/audio/victory.ogg"),
];

/// Gestionnaire de sons : effets ponctuels et musique de fond en boucle.
pub struct Audio {
    sfx: Vec<Option<Sound>>,
    music: Vec<Option<Sound>>,
    current: Music,
    muted: bool,
    /// Volume général, de 0 à 1, réglable en jeu.
    master: f32,
}

impl Audio {
    /// Charge les sons intégrés. Un échec laisse simplement le jeu silencieux.
    /// Doit être appelé dans la boucle graphique (les chargements sont asynchrones).
    pub async fn new() -> Self {
        let mut sfx = Vec::with_capacity(SFX_COUNT);
        for bytes in SFX_BYTES {
            sfx.push(load_sound_from_bytes(bytes).await.ok());
        }
        let mut music = Vec::with_capacity(MUSIC_COUNT);
        for bytes in MUSIC_BYTES {
            music.push(load_sound_from_bytes(bytes).await.ok());
        }
        Self { sfx, music, current: Music::None, muted: false, master: 1. }
    }

    /// Vrai si au moins une piste a pu être chargée (périphérique audio présent).
    pub fn available(&self) -> bool {
        self.music.iter().any(|m| m.is_some())
    }

    /// Nombre d'effets et de musiques réellement chargés.
    pub fn counts(&self) -> (usize, usize) {
        (
            self.sfx.iter().filter(|s| s.is_some()).count(),
            self.music.iter().filter(|m| m.is_some()).count(),
        )
    }

    pub fn muted(&self) -> bool {
        self.muted
    }

    /// Volume général actuel, en pourcentage arrondi.
    pub fn master_percent(&self) -> u32 {
        (self.master * 100.).round() as u32
    }

    /// Baisse ou hausse le volume général de 10 %, entre 0 et 110 %.
    pub fn change_master(&mut self, delta: f32) {
        self.master = (self.master + delta).clamp(0., 1.1);
        let keep = self.current;
        self.stop_current();
        if keep != Music::None {
            self.current = Music::None;
            self.play(keep);
        }
    }

    /// Coupe ou rétablit tout le son. Renvoie le nouvel état.
    pub fn toggle_mute(&mut self) -> bool {
        self.muted = !self.muted;
        let keep = self.current;
        self.stop_current();
        if !self.muted {
            self.current = Music::None;
            self.play(keep);
        }
        self.muted
    }

    fn stop_current(&mut self) {
        if let Some(s) = self.music.get(self.current.index()).and_then(|m| m.as_ref()) {
            stop_sound(s);
        }
        self.current = Music::None;
    }

    /// Joue une musique en boucle, ou ne fait rien si elle joue déjà.
    pub fn play(&mut self, music: Music) {
        if self.muted || self.current == music {
            return;
        }
        self.stop_current();
        if music == Music::None {
            return;
        }
        if let Some(s) = self.music.get(music.index()).and_then(|m| m.as_ref()) {
            play_sound(s, PlaySoundParams { looped: true, volume: music.volume() * self.master });
            self.current = music;
        }
    }

    /// Joue un effet sonore, sans interrompre la musique.
    pub fn sfx(&self, sfx: Sfx) {
        if self.muted {
            return;
        }
        if let Some(s) = self.sfx.get(sfx.index()).and_then(|s| s.as_ref()) {
            play_sound(s, PlaySoundParams { looped: false, volume: 0.9 * self.master });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sound_and_track_has_its_slot() {
        // Les indices doivent couvrir exactement les tableaux embarqués.
        let mut seen = [false; SFX_COUNT];
        for s in [
            Sfx::Swing, Sfx::Hit, Sfx::Hurt, Sfx::Die, Sfx::Coin, Sfx::Heal, Sfx::Door,
            Sfx::Ice, Sfx::IceBlock, Sfx::Portal, Sfx::Roar, Sfx::Explosion, Sfx::Ui,
            Sfx::Ui2, Sfx::Flame, Sfx::Wave, Sfx::Demigod,
        ] {
            assert!(!seen[s.index()], "indice d'effet en double");
            seen[s.index()] = true;
        }
        assert!(seen.iter().all(|x| *x));
        assert_eq!(Music::Menu.index(), 0);
        assert_eq!(Music::Victory.index(), MUSIC_COUNT - 1);
    }

    #[test]
    fn music_volumes_stay_reasonable() {
        for m in [Music::Menu, Music::Boss, Music::Backrooms, Music::Pantheon] {
            assert!(m.volume() > 0. && m.volume() <= 0.7);
        }
        assert_eq!(Music::None.volume(), 0.);
    }
}
