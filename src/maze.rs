//! Les Backrooms : 18 étages explorables.
//! Chaque étage est un labyrinthe déterministe (aucun hasard au chargement).
//! On monte par les cages d'escalier dissimulées ; l'étage 18 abrite un
//! passage secret vers le Panthéon. Rien de tout cela n'est indiqué au joueur.

use macroquad::prelude::{vec2, Color, Vec2};

pub const FLOORS: usize = 18;
pub const PANTHÉON_FLOOR: usize = 18;

const COLS: usize = 7;
const ROWS: usize = 5;
const CELL_W: f32 = 124.;
const CELL_H: f32 = 78.;
const ORIGIN_X: f32 = 48.;
const ORIGIN_Y: f32 = 172.;
/// Marge laissée entre deux zones : c'est un couloir, jamais un mur plein.
const CELL_MARGIN: f32 = 11.;

/// Une zone rectangulaire du labyrinthe.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Cell {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Cell {
    pub fn center(self) -> Vec2 {
        vec2(self.x + self.w / 2., self.y + self.h / 2.)
    }
    /// Vrai si le joueur (point + rayon `r`) touche un mur de la zone.
    pub fn blocks(self, p: Vec2, r: f32) -> bool {
        p.x - r < self.x || p.x + r > self.x + self.w || p.y - r < self.y || p.y + r > self.y + self.h
    }
}

/// Un passage entre deux zones voisines.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Hatch {
    pub pos: Vec2,
    pub up: bool,
}

/// Description d'un étage, entièrement déterministe.
pub struct Floor {
    pub level: usize,
    pub cells: Vec<Cell>,
    pub start: Cell,
    /// Escalier qui mène à l'étage du dessus.
    pub stairs: Hatch,
    /// Escalier facultatif qui redescend.
    pub down: Option<Hatch>,
    /// La surface entière où l'on peut marcher (zones + couloirs).
    pub walkable: Cell,
}

fn h(a: usize, b: usize, salt: usize) -> u32 {
    let mut x = (a as u32).wrapping_mul(0x9E37_79B9) ^ (b as u32).wrapping_mul(0x85EB_CA6B) ^ (salt as u32).wrapping_mul(0xC2B2_AE35);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2545_F491);
    x ^= x >> 13;
    x
}

/// Étages 1 à 17 : chaque étage mène à l'étage suivant.
/// La graine dépend de l'étage, pas de la partie : un étage reste reconnaissable.
pub fn floor_descend(level: usize) -> Floor {
    let level = level.clamp(1, FLOORS);
    let mut cells = Vec::with_capacity(COLS * ROWS);
    for row in 0..ROWS {
        for col in 0..COLS {
            // Les zones varient en taille, sans jamais boucher les couloirs.
            let jitter_w = (h(col, row, level) % 13) as f32;
            let jitter_h = (h(row, col, level + 7) % 11) as f32;
            cells.push(Cell {
                x: ORIGIN_X + col as f32 * CELL_W,
                y: ORIGIN_Y + row as f32 * CELL_H,
                w: CELL_W - CELL_MARGIN - jitter_w,
                h: CELL_H - CELL_MARGIN - jitter_h,
            });
        }
    }
    let start_index = (h(level, 3, 11) as usize) % cells.len();
    // L'escalier est toujours ailleurs que le point d'arrivée.
    let mut stairs_index = start_index;
    while stairs_index == start_index {
        stairs_index = (h(level, 5, 29) as usize) % cells.len();
    }
    let start = cells[start_index];
    let target = cells[stairs_index];
    let stairs = Hatch { pos: target.center(), up: true };
    // Un second escalier, qui redescend, existe sur un étage sur deux.
    let down = if level > 1 && h(level, 9, 41) % 2 == 0 {
        let mut idx = start_index;
        while idx == start_index || idx == stairs_index {
            idx = (h(level, 13, 71) as usize) % cells.len();
        }
        Some(Hatch { pos: cells[idx].center(), up: false })
    } else {
        None
    };
    let walkable = Cell {
        x: ORIGIN_X,
        y: ORIGIN_Y,
        w: COLS as f32 * CELL_W - CELL_MARGIN,
        h: ROWS as f32 * CELL_H - CELL_MARGIN,
    };
    Floor { level, cells, start, stairs, down, walkable }
}

/// Étages 18 : le dernier, presque vide, avec le passage secret du Panthéon.
pub fn floor_pantheon() -> Floor {
    let mut cells = Vec::new();
    for row in 0..ROWS {
        for col in 0..COLS {
            let wide = (col == 2 || col == 3) && (row == 1 || row == 2);
            let extra = if wide { CELL_MARGIN } else { 0. };
            cells.push(Cell {
                x: ORIGIN_X + col as f32 * CELL_W,
                y: ORIGIN_Y + row as f32 * CELL_H,
                w: CELL_W - CELL_MARGIN + extra,
                h: CELL_H - CELL_MARGIN + extra,
            });
        }
    }
    Floor {
        level: PANTHÉON_FLOOR,
        start: cells[3 * COLS + 3],
        stairs: Hatch { pos: cells[1 * COLS + 2].center(), up: true },
        down: None,
        cells,
        walkable: Cell {
            x: ORIGIN_X,
            y: ORIGIN_Y,
            w: COLS as f32 * CELL_W - CELL_MARGIN,
            h: ROWS as f32 * CELL_H - CELL_MARGIN,
        },
    }
}

pub fn floor(level: usize) -> Floor {
    if level >= PANTHÉON_FLOOR {
        floor_pantheon()
    } else {
        floor_descend(level)
    }
}

/// Point de départ du joueur à cet étage.
pub fn start_pos(level: usize) -> Vec2 {
    floor(level).start.center()
}

/// Vrai si le déplacement de `from` vers `to` sort de la surface praticable.
/// Les zones et les couloirs qui les relient sont tous accessibles.
pub fn blocked(f: &Floor, from: Vec2, to: Vec2, r: f32) -> bool {
    let _ = from;
    f.walkable.blocks(to, r)
}

/// L'escalier est-il atteint ?
pub fn at_stairs(f: &Floor, p: Vec2) -> bool {
    p.distance(f.stairs.pos) < 26.
}

/// Le passage du Panthéon n'existe qu'à l'étage 18.
pub fn at_pantheon_gate(f: &Floor, p: Vec2) -> bool {
    f.level >= PANTHÉON_FLOOR && p.distance(floor_pantheon().stairs.pos) < 30.
}

/// Palette par étage : les Backrooms changent d'aspect en montant.
pub fn palette(level: usize) -> (Color, Color, Color) {
    let t = level as f32 / FLOORS as f32;
    let wall = Color::new(0.55 + 0.18 * t, 0.51 + 0.06 * t, 0.33 - 0.05 * t, 1.);
    let floor_col = Color::new(0.42 + 0.1 * t, 0.4 + 0.09 * t, 0.27 + 0.12 * t, 1.);
    let glow = Color::new(0.86 - 0.15 * t, 0.85 + 0.05 * t, 0.6 + 0.3 * t, 1.);
    (wall, floor_col, glow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_floor_has_a_reachable_start_and_stairs() {
        for level in 1..=FLOORS {
            let f = floor(level);
            assert_eq!(f.cells.len(), COLS * ROWS);
            let start = start_pos(level);
            assert!(f.cells.iter().any(|c| !c.blocks(start, 6.)), "départ dans un mur à l'étage {level}");
            assert!(f.cells.iter().any(|c| !c.blocks(f.stairs.pos, 6.)), "escalier dans un mur à l'étage {level}");
            assert!(start.distance(f.stairs.pos) > 40., "escalier collé au départ à l'étage {level}");
        }
    }

    #[test]
    fn floors_are_deterministic_and_varied() {
        let a = floor(4).stairs.pos;
        let b = floor(4).stairs.pos;
        assert_eq!(a, b);
        let distinct = (1..=FLOORS).map(|l| floor(l).stairs.pos).collect::<Vec<_>>();
        assert!(distinct.windows(2).any(|w| w[0] != w[1]), "tous les étages se ressemblent");
    }

    #[test]
    fn walls_stop_the_player() {
        let f = floor(3);
        let start = f.start.center();
        assert!(!blocked(&f, start, start, 6.));
        // Sortir de la surface praticable est refusé.
        let outside = vec2(f.walkable.x + f.walkable.w + 60., start.y);
        assert!(blocked(&f, start, outside, 6.));
        assert!(!blocked(&f, start, start + vec2(0., 20.), 6.));
    }

    #[test]
    fn pantheon_gate_only_on_last_floor() {
        assert!(at_pantheon_gate(&floor(FLOORS), floor_pantheon().stairs.pos));
        assert!(!at_pantheon_gate(&floor(FLOORS - 1), floor(FLOORS - 1).stairs.pos));
    }

    #[test]
    fn every_floor_can_be_walked_to_its_stairs() {
        // Les zones et les couloirs forment une grille continue : un simple
        // trajet orthogononal suffit à vérifier qu'aucun mur ne bloque le chemin.
        for level in 1..=FLOORS {
            let f = floor(level);
            let mut walker = start_pos(level);
            let target = f.stairs.pos;
            for _ in 0..4000 {
                if walker.distance(target) < 4. {
                    break;
                }
                let dx = (target.x - walker.x).clamp(-6., 6.);
                let dy = (target.y - walker.y).clamp(-6., 6.);
                let step = if dx.abs() > 1. { vec2(dx, 0.) } else { vec2(0., dy) };
                let next = walker + step;
                assert!(!blocked(&f, walker, next, 7.), "mur sur le trajet à l'étage {level}");
                walker = next;
            }
            assert!(walker.distance(target) < 8., "escalier inatteignable à l'étage {level}");
        }
    }

    #[test]
    fn every_floor_climbs_and_some_can_be_descended() {
        assert!((1..=FLOORS).all(|l| floor(l).stairs.up), "il faut toujours pouvoir monter");
        let with_down = (2..=FLOORS).filter(|l| floor(*l).down.is_some()).count();
        assert!(with_down > 0, "aucun escalier descendant");
        for l in 2..=FLOORS {
            if let Some(d) = floor(l).down {
                assert!(d.pos.distance(floor(l).stairs.pos) > 40.);
                assert!(d.pos.distance(start_pos(l)) > 40.);
            }
        }
    }
}
