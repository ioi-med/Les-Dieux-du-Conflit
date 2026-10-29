#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod audio;
mod maze;
mod story;
use audio::{Audio, Music, Sfx};
use story::{Place, CHAPTERS};
use macroquad::prelude::*;
use std::path::PathBuf;

const W:f32=960.; const H:f32=640.;
const LEFT:f32=30.; const RIGHT:f32=930.; const TOP:f32=180.; const BOTTOM:f32=550.;
const SECTORS:usize=4; const COMMUNIST:usize=22; const CHOICE:usize=23; const CAPITALIST:usize=24;
const VERSION:&str="3.5.0";
/// Plafond de vie : 100 au départ, puis 5 points par palier de 10 ennemis vaincus.
const MAX_LIFE:f32=999.;
/// Second cycle : « Dieu de la justice ».
/// Les ennemis ordinaires ne peuvent plus blesser le joueur ; seuls les
/// demi-dieux et les dieux-zombies en sont capables.
const ZOMBIE_POWER:f32=2.;
/// Nombre de victoires pour gagner un palier de puissance.
const KILLS_PER_STEP:u32=10;
/// Paliers maximaux (100 + 100 × 5 = 600 points de vie au dernier).
const MAX_HUNTER:u32=100;
/// Dégâts de base du coup d'épée, avant les paliers de puissance.
const BASE_MELEE:f32=27.;
fn life_for(hunter:u32)->f32{(100.+5.*(hunter.min(MAX_HUNTER)/KILLS_PER_STEP) as f32).min(MAX_LIFE)}
fn power_for(hunter:u32)->f32{1.+0.04*hunter.min(MAX_HUNTER) as f32}
/// Paliers atteints, pour l'affichage.
fn display_rect(width:f32,height:f32)->Rect {let w=width.max(1.);let h=height.max(1.);let scale=(w/W).min(h/H);let vw=w/scale;let vh=h/scale;Rect::new((W-vw)/2.,(H-vh)/2.,vw,vh)}
fn window_conf()->Conf {Conf{window_title:"Les Dieux du Conflit".into(),window_width:960,window_height:640,high_dpi:false,..Default::default()}}
fn c(v:u32)->Color {Color::from_rgba((v>>16) as u8,(v>>8) as u8,v as u8,255)}
fn text(s:&str,x:f32,y:f32,size:u16,col:Color,font:&Font){draw_text_ex(s,x,y,TextParams{font:Some(font),font_size:size,color:col,..Default::default()});}
fn center(s:&str,y:f32,size:u16,col:Color,font:&Font){let x=(W-measure_text(s,Some(font),size,1.).width)/2.;text(s,x,y,size,col,font)}
fn norm(v:Vec2)->Vec2 {if v.length_squared()>0.001 {v.normalize()} else {vec2(1.,0.)}}
fn clamp_enemy(v:Vec2)->Vec2{vec2(v.x.clamp(LEFT+18.,RIGHT-18.),v.y.clamp(TOP+18.,BOTTOM-18.))}
fn clamp_player(v:Vec2)->Vec2{vec2(v.x.clamp(LEFT+1.,RIGHT-1.),v.y.clamp(TOP+1.,BOTTOM-1.))}
fn in_rift(x:f32)->bool{(350. ..410.).contains(&x)||(590. ..650.).contains(&x)}
fn rift_stage(stage:usize)->bool{stage==4||stage==10}
fn defense_stage(stage:usize,sector:usize)->bool{sector==2&&[6,13,17,20].contains(&stage)}
fn defense_time(stage:usize,sector:usize,elapsed:f32)->f32{if defense_stage(stage,sector){(22.-elapsed).max(0.)}else{0.}}
fn load_texture(bytes:&[u8])->Texture2D {let t=Texture2D::from_file_with_format(bytes,Some(ImageFormat::Png));t.set_filter(FilterMode::Nearest);t}
struct Assets {player:Texture2D,player_idle:Texture2D,slime:Texture2D,slime_idle:Texture2D,skeleton:Texture2D,skeleton_idle:Texture2D,capitalist:Texture2D,capitalist_idle:Texture2D,communist:Texture2D,communist_idle:Texture2D,fire:Texture2D,blood:Texture2D,font:Font}
impl Assets {fn new()->Self {Self{
 player:load_texture(include_bytes!("../assets/joueur_mouvement.png")),player_idle:load_texture(include_bytes!("../assets/joueur_no_mouvement.png")),
 slime:load_texture(include_bytes!("../assets/slime_mouvement.png")),slime_idle:load_texture(include_bytes!("../assets/slime_no_mouvement.png")),
 skeleton:load_texture(include_bytes!("../assets/squelette_mouvement.png")),skeleton_idle:load_texture(include_bytes!("../assets/squelette_no_mouvement.png")),
 capitalist:load_texture(include_bytes!("../assets/dieux_du_capitaliste_battle.png")),capitalist_idle:load_texture(include_bytes!("../assets/dieux_du_capitaliste_speak.png")),
 communist:load_texture(include_bytes!("../assets/dieux_du_communisme_mouvement.png")),communist_idle:load_texture(include_bytes!("../assets/dieux_du_communisme_no_mouvement.png")),
 fire:load_texture(include_bytes!("../assets/sort_de_flamme_vert.png")),blood:load_texture(include_bytes!("../assets/vague_de_sang.png")),font:load_ttf_font_from_bytes(include_bytes!("../assets/DejaVuSans.ttf")).unwrap()}}}
fn sprite(t:&Texture2D,p:Vec2,frame:usize,cell:f32,scale:f32,flip:bool,tint:Color){let size=cell*scale;draw_texture_ex(t,p.x-size/2.,p.y-size/2.,tint,DrawTextureParams{dest_size:Some(vec2(size,size)),source:Some(Rect::new(frame as f32*cell,0.,cell,cell)),flip_x:flip,..Default::default()})}
#[derive(Clone,Copy,PartialEq,Debug)] enum Kind {Slime,Skeleton,Wisp,Guard,Rival,Communist,Capitalist}
impl Kind {fn hp(self)->f32{match self{Self::Slime=>90.,Self::Skeleton=>125.,Self::Wisp=>80.,Self::Guard=>170.,Self::Rival=>260.,Self::Communist=>850.,Self::Capitalist=>950.}} fn speed(self)->f32{match self{Self::Slime=>60.,Self::Skeleton=>78.,Self::Wisp=>105.,Self::Guard=>55.,Self::Rival=>118.,_=>52.}} fn boss(self)->bool{matches!(self,Self::Communist|Self::Capitalist)}}
struct Enemy{id:u32,kind:Kind,pos:Vec2,hp:f32,cooldown:f32,hurt:f32,frozen:f32,demigod:bool,zombie:bool,judged:bool}
impl Enemy{
 fn new(id:u32,kind:Kind,x:f32,y:f32)->Self{Self{id,kind,pos:vec2(x,y),hp:kind.hp(),cooldown:1.1+(id%5) as f32*0.17,hurt:0.,frozen:0.,demigod:false,zombie:false,judged:false}}
 /// Promotion en demi-dieu : plus de vie, plus de vitesse, et le droit de
 /// blesser un dieu de la justice.
 fn promoted(mut self)->Self{self.demigod=true;self.hp=kind_hp(&self.kind)*1.6;self.cooldown*=0.75;self}
 /// Dieu revenu en zombie : deux fois plus de vie, de vitesse et de dégâts.
 fn zombified(mut self)->Self{self.zombie=true;self.hp=kind_hp(&self.kind)*ZOMBIE_POWER;self.cooldown/=ZOMBIE_POWER;self}
 /// Multiplicateur de dégâts au contact : les dieux-zombies frappent deux fois plus fort.
 /// Un vivant jugé voit ses capacités doublées : vitesse, cadence et frappe.
 fn judged_boost(&self)->f32{if self.judged{2.}else{1.}}
 fn touch_multiplier(&self)->f32{(if self.zombie{ZOMBIE_POWER}else if self.demigod{1.5}else{1.})*self.judged_boost()}
 /// Une âme morte-vivante : squelette, garde ou dieu revenu en zombie.
 fn undead(&self)->bool{matches!(self.kind,Kind::Skeleton|Kind::Guard)||self.zombie}
 fn hp_max(&self)->f32{if self.zombie{kind_hp(&self.kind)*ZOMBIE_POWER}else if self.demigod{kind_hp(&self.kind)*1.6}else{kind_hp(&self.kind)}}
 /// Un ennemi peut-il blesser un joueur devenu dieu de la justice ?
 fn can_hurt_justice(&self)->bool{self.demigod||self.kind.boss()||self.zombie}
}
fn kind_hp(k:&Kind)->f32{k.hp()}
#[derive(Clone,Copy,PartialEq,Debug)] enum Projectile{Flame,Wave,Blood,Coin,Building,Bone}
struct Shot{pos:Vec2,vel:Vec2,kind:Projectile,friendly:bool,life:f32,hit_ids:Vec<u32>,damage:f32,divine:bool}
impl Shot{
 fn new(pos:Vec2,vel:Vec2,kind:Projectile,friendly:bool)->Self{Self{pos,vel,kind,friendly,life:3.5,hit_ids:Vec::new(),damage:10.,divine:false}}
 fn hit(pos:Vec2,vel:Vec2,kind:Projectile,friendly:bool,damage:f32,divine:bool)->Self{Self{damage,divine,..Self::new(pos,vel,kind,friendly)}}
}
#[derive(Clone,Copy,PartialEq,Debug)] enum Ice{Unknown,Ready,Spent}
/// Réserve de Glace éternelle : un seul usage normal, ou 999 avec le code GLACE.
#[derive(Clone,Copy,PartialEq,Debug)] struct IceStock{charges:u32}
impl IceStock{
 fn empty()->Self{Self{charges:0}}
 fn available(&self)->bool{self.charges>0}
 fn take(&mut self)->bool{if self.charges>0{self.charges-=1;true}else{false}}
}
#[derive(Clone,Copy,PartialEq,Debug)] enum IceOutcome{Frozen,NoTarget,Forbidden,Unavailable}
#[derive(Clone,Copy,PartialEq,Debug)] enum JudgeOutcome{Judged(bool),NoTarget,Forbidden,Used}
#[derive(Clone,Copy,PartialEq,Debug)] enum Mode{Title,Story,Playing,Backrooms,Pantheon,Paused,Choice,Dead,End}
#[derive(Clone,Copy,Debug)] struct Progress{stage:usize,sector:usize,charges:u32,duel_two:bool,demigod:bool,divine_freeze:bool,ice_granted:bool,floor:usize,hunter:u32,max_hp:f32,justice:bool}
impl Progress{
 fn encode(self)->String{format!("8 {} {} {} {} {} {} {} {} {} {} {}\n",self.stage,self.sector,self.charges,self.duel_two as u8,self.demigod as u8,self.divine_freeze as u8,self.ice_granted as u8,self.floor,self.hunter,self.max_hp.round() as u32,self.justice as u8)}
 fn decode(s:&str)->Option<Self>{let mut a=s.split_whitespace();if a.next()? != "8"{return None;}let stage:usize=a.next()?.parse().ok()?;let sector:usize=a.next()?.parse().ok()?;let charges:u32=a.next()?.parse().ok()?;let duel_two=match a.next()?{"0"=>false,"1"=>true,_=>return None};let demigod=match a.next()?{"0"=>false,"1"=>true,_=>return None};let divine_freeze=match a.next()?{"0"=>false,"1"=>true,_=>return None};let ice_granted=match a.next()?{"0"=>false,"1"=>true,_=>return None};let floor:usize=a.next()?.parse().ok()?;let hunter:u32=a.next()?.parse().ok()?;let max_hp:f32=a.next()?.parse::<u32>().ok()? as f32;let justice=match a.next()?{"0"=>false,"1"=>true,_=>return None};if stage>=CHAPTERS.len()||sector>=SECTORS||charges>999||!(1..=maze::FLOORS).contains(&floor)||hunter>MAX_HUNTER||max_hp<100.||max_hp>MAX_LIFE||a.next().is_some(){return None}Some(Self{stage,sector,charges,duel_two,demigod,divine_freeze,ice_granted,floor,hunter,max_hp,justice})}
}
fn save_path()->Option<PathBuf>{if let Some(p)=std::env::var_os("APPDATA"){Some(PathBuf::from(p).join("LesDieuxDuConflit").join("aventure-v3.txt"))} else {std::env::var_os("HOME").map(|p|PathBuf::from(p).join(".local/share/LesDieuxDuConflit/aventure-v3.txt"))}}
fn load_progress()->Option<Progress>{let path=save_path()?;Progress::decode(&std::fs::read_to_string(path).ok()?)}
fn write_progress(p:Progress)->std::io::Result<()>{let path=save_path().ok_or_else(||std::io::Error::new(std::io::ErrorKind::NotFound,"Dossier de sauvegarde introuvable"))?;std::fs::create_dir_all(path.parent().unwrap())?;std::fs::write(path,p.encode())}
struct Game {mode:Mode,stage:usize,sector:usize,player:Vec2,face:Vec2,hp:f32,max_hp:f32,hunter:u32,attack:f32,swing:f32,flame_cd:f32,wave_cd:f32,jump:f32,immune:f32,elapsed:f32,enemies:Vec<Enemy>,shots:Vec<Shot>,heals:Vec<Vec2>,clues:Vec<Vec2>,duel_two:bool,ice:Ice,stock:IceStock,divine_freeze:bool,ice_granted:bool,return_pos:Vec2,return_face:Vec2,backs:usize,won_betrayal:bool,justice:bool,judgement:bool,message:String,message_time:f32,kills:u32,defense_spawned:u8,statues:Vec<(Vec2,Kind)>,persist:bool,save_failed:bool,demigod:bool,divinity:u8,back_floor:usize,last_hit:f32,hurt_flash:f32,sfx_queue:Vec<Sfx>,music_wish:Music}
impl Game{
 fn new(persist:bool)->Self {Self{mode:Mode::Title,stage:0,sector:0,player:vec2(100.,360.),face:vec2(1.,0.),hp:100.,max_hp:100.,hunter:0,attack:0.,swing:0.,flame_cd:0.,wave_cd:0.,jump:0.,immune:0.,elapsed:0.,enemies:vec![],shots:vec![],heals:vec![],clues:vec![],duel_two:false,ice:Ice::Unknown,stock:IceStock::empty(),divine_freeze:false,ice_granted:false,return_pos:vec2(100.,360.),return_face:vec2(1.,0.),backs:0,justice:false,judgement:false,won_betrayal:false,message:String::new(),message_time:0.,kills:0,defense_spawned:0,statues:Vec::new(),persist,save_failed:false,demigod:false,divinity:0,back_floor:1,last_hit:0.,hurt_flash:0.,sfx_queue:Vec::new(),music_wish:Music::None}}
 fn progress(&self)->Progress{Progress{stage:self.stage,sector:self.sector,charges:self.stock.charges,duel_two:self.duel_two,demigod:self.demigod,divine_freeze:self.divine_freeze,ice_granted:self.ice_granted,floor:self.back_floor.clamp(1,maze::FLOORS),hunter:self.hunter,max_hp:self.max_hp,justice:self.justice}}
 fn reset_ice(&mut self){self.justice=false;self.judgement=false;self.ice=Ice::Unknown;self.stock=IceStock::empty();self.divine_freeze=false;self.ice_granted=false;self.demigod=false;self.divinity=0;self.hunter=0;self.max_hp=100.;self.hp=100.;}
 fn give_ice(&mut self,charges:u32){self.stock=IceStock{charges:charges.min(999)};self.ice=Ice::Ready;self.ice_granted=true;}
 fn refresh_ice(&mut self){self.ice=if self.stock.available(){Ice::Ready}else{Ice::Spent};}
 /// Puissance du coup d'épée selon les ennemis déjà vaincus.
 /// Le code INV porte la vie maximale à 999 : les coups passent alors à 999.
 fn overpower(&self)->bool{self.max_hp>=MAX_LIFE}
 fn melee_damage(&self)->f32{if self.overpower(){999.}else{BASE_MELEE*power_for(self.hunter)}}
 /// Enregistre des victoires : tous les dix ennemis, +5 points de vie maximum
 /// et des coups plus forts. Les points gagnés sont rendus immédiatement.
 fn register_kills(&mut self,count:u32){
  if count==0{return;}
  for _ in 0..count{
   if self.hunter>=MAX_HUNTER{break;}
   self.hunter+=1;
   if self.hunter%KILLS_PER_STEP==0{
    let before=self.max_hp;self.max_hp=life_for(self.hunter);
    self.hp=(self.hp+(self.max_hp-before)).min(self.max_hp);
   }
  }
 }
 fn divine(&self)->bool{self.demigod||self.divine_freeze}
 fn cue(&mut self,s:Sfx){self.sfx_queue.push(s)}
 fn request_music(&mut self,m:Music){self.music_wish=m}
 fn save(&mut self){if self.persist {self.save_failed=write_progress(self.progress()).is_err();}}
 fn notify(&mut self,s:&str){self.message=s.to_owned();self.message_time=3.;}
 fn setup(&mut self,stage:usize,sector:usize,full_health:bool,story:bool){assert!(stage<CHAPTERS.len()&&sector<SECTORS);self.stage=stage;self.sector=sector;self.max_hp=life_for(self.hunter).max(self.max_hp).min(MAX_LIFE);if full_health {self.hp=self.max_hp;}self.player=vec2(105.,360.);self.face=vec2(1.,0.);self.attack=0.;self.swing=0.;self.flame_cd=0.;self.wave_cd=0.;self.jump=0.;self.immune=0.;self.elapsed=0.;self.shots.clear();self.heals.clear();self.clues.clear();self.kills=0;self.defense_spawned=0;self.judgement=self.justice;self.statues.clear();self.message_time=0.;self.enemies=spawn_enemies_for(stage,sector,self.justice);
  if sector==0 || sector==2 {self.heals.push(vec2(165.,245.));}
  if sector==1||sector==2 {self.clues=vec![vec2(300.,240.),vec2(500.,475.),vec2(825.,245.)];}
  if self.enemies.iter().any(|e|e.kind.boss()){self.cue(Sfx::Roar);self.request_music(Music::Boss);}else{self.request_music(match (self.stage+self.sector)%3{0=>Music::Surface1,1=>Music::Surface2,_=>Music::Surface3});}
  self.mode=if story{Mode::Story}else{Mode::Playing};self.save();
 }
 fn advance_sector(&mut self){self.cue(Sfx::Door);if self.sector+1<SECTORS {self.setup(self.stage,self.sector+1,false,false);}else if self.stage==CHOICE {self.mode=Mode::Choice;self.save();}else if self.stage==CAPITALIST {self.won_betrayal=true;self.mode=Mode::End;}else{self.setup(self.stage+1,0,true,true);}}
 fn heal(&mut self,amount:f32){self.hp=(self.hp+amount).min(self.max_hp);}
 /// Dégâts infligés par une source identifiée (corps à corps, projectile).
 fn damage_from(&mut self,amount:f32,divine:bool){
  if self.justice&&!divine{return;} // un dieu de la justice ignore les simples mortels
  self.damage(amount);
 }
 fn damage(&mut self,amount:f32){if self.immune<=0.&&self.jump<=0.{self.hp=(self.hp-amount).max(0.);self.immune=0.8;self.hurt_flash=0.3;self.cue(Sfx::Hurt);if self.hp<=0.{self.cue(Sfx::Explosion);}}}
 fn melee(&mut self){if self.attack>0.{return;}self.attack=0.28;self.swing=0.15;self.cue(Sfx::Swing);let hit=self.melee_damage();let mut touched=false;for e in &mut self.enemies{let delta=e.pos-self.player;if e.frozen<=0.&&delta.length()<88.&&(delta.length()<25.||norm(delta).dot(self.face)>-0.1){e.hp-=hit;e.hurt=0.16;touched=true;e.pos=clamp_enemy(e.pos+self.face*13.);}}if touched{self.cue(Sfx::Hit);}}
 fn freeze_one(&mut self)->IceOutcome {
  if !self.stock.available(){return IceOutcome::Unavailable;}
  let target=self.enemies.iter().enumerate().filter(|(_,e)|e.hp>0.&&e.frozen<=0.).min_by(|(_,a),(_,b)|a.pos.distance_squared(self.player).total_cmp(&b.pos.distance_squared(self.player))).map(|(i,_)|i);
  let Some(i)=target else{return IceOutcome::NoTarget;};
  if self.enemies[i].kind.boss()&&!self.divine(){return IceOutcome::Forbidden;}
  self.enemies[i].frozen=1.5;self.stock.take();self.refresh_ice();self.save();self.cue(Sfx::Ice);if self.stock.available(){self.notify(&format!("GLACE ÉTERNELLE : adversaire emprisonné. {} usages restants.",self.stock.charges));}else{self.notify("GLACE ÉTERNELLE : adversaire emprisonné. Dernier usage.");}IceOutcome::Frozen
 }
 fn enter_backrooms(&mut self){if self.demigod{self.player=clamp_player(self.player);return;}self.return_pos=clamp_player(self.player);self.return_face=self.face;self.back_floor=self.back_floor.clamp(1,maze::FLOORS);self.player=maze::start_pos(self.back_floor);self.face=vec2(0.,-1.);self.mode=Mode::Backrooms;self.backs+=1;self.cue(Sfx::Portal);self.request_music(Music::Backrooms);}
 fn backroom_move(&mut self,d:Vec2,dt:f32){let f=maze::floor(self.back_floor);let speed=if self.hp<30.{110.}else{150.};let step=norm(d)*speed*dt;let target=self.player+step;let mut ok=target;if maze::blocked(&f,self.player,target,7.){let x_only=vec2(target.x,self.player.y);let y_only=vec2(self.player.x,target.y);ok=if !maze::blocked(&f,self.player,x_only,7.){x_only}else if !maze::blocked(&f,self.player,y_only,7.){y_only}else{self.player};}self.player=ok;self.face=norm(d);}
 /// Monte ou descend d'un étage. Renvoie vrai si le joueur a changé d'étage.
 fn climb(&mut self,up:bool)->bool{let f=maze::floor(self.back_floor);
  let here=if up{maze::at_stairs(&f,self.player)}else{f.down.map(|d|self.player.distance(d.pos)<26.).unwrap_or(false)};
  if !here{return false;}if up&&self.back_floor>=maze::FLOORS{return false;}if !up&&self.back_floor<=1{return false;}self.back_floor=if up{self.back_floor+1}else{self.back_floor-1};self.player=maze::start_pos(self.back_floor);self.face=vec2(0.,-1.);self.hp=(self.hp+14.).min(self.max_hp);self.cue(Sfx::Portal);self.save();self.notify("Un escalier. L'air change de goût.");true}
 fn backroom_update(&mut self,dt:f32){self.elapsed+=dt;self.hp=(self.hp-dt*0.22).max(0.);self.hurt_flash=(self.hurt_flash-dt).max(0.);let d=movement();if d.length_squared()>0.{self.backroom_move(norm(d),dt);}if is_key_pressed(KeyCode::E){if !self.climb(true){self.climb(false);}}
  if maze::at_pantheon_gate(&maze::floor(self.back_floor),self.player){self.enter_pantheon();return;}
  if self.hp<=0.{self.mode=Mode::Dead;return;}
  if ctrl_g(){self.leave_backrooms();}}
 fn leave_backrooms(&mut self){self.mode=Mode::Playing;self.cue(Sfx::Portal);self.player=clamp_player(self.return_pos);self.player.x=self.player.x.clamp(LEFT+40.,RIGHT-40.);self.player.y=self.player.y.clamp(TOP+40.,BOTTOM-40.);self.face=self.return_face;self.immune=1.;self.hp=(self.hp+18.).min(self.max_hp);
  if !self.ice_granted{self.give_ice(1);self.notify("Vous rapportez quelque chose de froid dans la main.");}else{self.notify("De retour à la surface.");}
  self.save();}
 fn enter_pantheon(&mut self){self.mode=Mode::Pantheon;self.divinity=self.divinity.saturating_add(1).min(3);self.cue(Sfx::Demigod);self.request_music(Music::Pantheon);}
 /// Second cycle : le joueur devient dieu de la justice et tout recommence.
 fn start_justice(&mut self){
  self.justice=true;
  self.hunter=0;self.max_hp=100.;self.hp=100.;
  self.kills=0;self.defense_spawned=0;
  self.setup(0,0,true,true);
  self.notify("DIEU DE LA JUSTICE : les mortels ne peuvent plus vous atteindre. Les dieux reviennent en zombies.");
  self.save();
 }
 /// Jugement des âmes : un seul usage par quartier, réservé au dieu de la justice.
 fn judge_nearest(&mut self)->JudgeOutcome{
  if !self.justice{return JudgeOutcome::Forbidden;}
  if !self.judgement{return JudgeOutcome::Used;}
  let target=self.enemies.iter().enumerate().filter(|(_,e)|e.hp>0.&&e.frozen<=0.).min_by(|(_,a),(_,b)|a.pos.distance_squared(self.player).total_cmp(&b.pos.distance_squared(self.player))).map(|(i,_)|i);
  let Some(i)=target else{return JudgeOutcome::NoTarget;};
  self.judgement=false;
  let undead=self.enemies[i].undead();
  if undead{self.enemies[i].hp=(self.enemies[i].hp/2.).max(1.);}
  else{self.enemies[i].judged=true;}
  self.cue(Sfx::Demigod);
  self.notify(if undead{"JUGEMENT : âme morte-vivante, elle tombe à la moitié de sa vie."}else{"JUGEMENT : âme vivante, ses capacités sont doublées."});
  JudgeOutcome::Judged(undead)
 }
 /// Code secret du menu pause : mène au dernier chapitre, pouvoirs débloqués.
 fn apply_password(&mut self,code:&str)->bool{
  match code.trim().to_ascii_uppercase().as_str(){
   "ML"=>{
    self.refresh_ice();
    self.setup(COMMUNIST,0,true,true);
    self.notify("CODE ACCEPTÉ : la route du dernier dieu s'ouvre. Flamme [E] et vague [T] sont à vous.");
    self.save();
    true
   }
   "GLACE"=>{
    self.give_ice(999);
    self.notify("CODE ACCEPTÉ : la Glace éternelle déborde. 999 usages.");
    self.save();
    true
   }
   "INV"=>{
    self.max_hp=MAX_LIFE;self.hp=MAX_LIFE;
    // Au-delà du dernier palier normal, les coups deviennent dévastateurs.
    self.hunter=MAX_HUNTER;
    self.notify("CODE ACCEPTÉ : vie maximale 999 et coups surpuissants.");
    self.save();
    true
   }
   _=>false,
  }
 }
 /// Dégâts affichés au joueur, pour information.
 fn shown_damage(&self)->u32{self.melee_damage().round() as u32}
 fn accept_demigod(&mut self){self.demigod=true;self.divine_freeze=true;self.divinity=self.divinity.max(1);self.hp=self.max_hp;if !self.ice_granted{self.give_ice(1);}else{self.refresh_ice();}self.mode=Mode::Playing;self.player=clamp_player(self.return_pos);self.face=self.return_face;self.immune=2.;self.notify("Votre sang a changé. Les dieux ne vous arrêteront plus.");self.cue(Sfx::Demigod);self.save();}
 fn decline_demigod(&mut self){self.hp=(self.hp+30.).min(self.max_hp);self.notify("Le Panthéon referme ses portes. Vous restez mortel.");}
 /// Un pas de déplacement. Renvoie vrai si le joueur vient d'entrer dans les Backrooms.
 fn walk(&mut self,d:Vec2,dt:f32)->bool{self.face=d;let old=self.player;let next=self.player+d*205.*dt;
  if next.x<LEFT||next.x>RIGHT||next.y<TOP||next.y>BOTTOM {
   if self.demigod {self.player=vec2(next.x.clamp(LEFT,RIGHT),next.y.clamp(TOP,BOTTOM));}
   else {self.enter_backrooms();return true;}
  } else {self.player=clamp_player(next);}
  if rift_stage(self.stage)&&self.jump<=0.&&in_rift(self.player.x){self.player=old;}
  false}
 fn update(&mut self,dt:f32){
  self.elapsed+=dt;self.last_hit+=dt;self.hurt_flash=(self.hurt_flash-dt).max(0.);self.attack=(self.attack-dt).max(0.);self.swing=(self.swing-dt).max(0.);self.flame_cd=(self.flame_cd-dt).max(0.);self.wave_cd=(self.wave_cd-dt).max(0.);self.jump=(self.jump-dt).max(0.);self.immune=(self.immune-dt).max(0.);self.message_time=(self.message_time-dt).max(0.);
  let d=movement();if is_key_pressed(KeyCode::Space)&&self.jump<=0.{self.jump=0.68;}
  if d.length_squared()>0.&&self.walk(norm(d),dt){return;}
  if is_key_down(KeyCode::R){self.melee();}
  if self.stage>=3&&is_key_pressed(KeyCode::E)&&self.flame_cd<=0.{self.flame_cd=0.8;self.cue(Sfx::Flame);self.shots.push(Shot::new(self.player+self.face*32.,self.face*410.,Projectile::Flame,true));}
  if self.stage>=12&&is_key_pressed(KeyCode::T)&&self.wave_cd<=0.{self.wave_cd=2.4;self.cue(Sfx::Wave);self.shots.push(Shot::new(self.player+self.face*32.,self.face*300.,Projectile::Wave,true));}
  if is_key_pressed(KeyCode::Y){match self.judge_nearest(){
   JudgeOutcome::Forbidden=>{self.notify("Le Jugement des âmes appartient au dieu de la justice.");}
   JudgeOutcome::Used=>{self.notify("Un seul jugement par quartier. Le suivant vous attend plus loin.");}
   JudgeOutcome::NoTarget=>{self.notify("Aucune âme à juger ici.");}
   JudgeOutcome::Judged(_)=>{}}
  }
  if is_key_pressed(KeyCode::X){match self.freeze_one(){IceOutcome::Forbidden=>{self.cue(Sfx::IceBlock);self.notify("La Glace se brise : un dieu ne peut pas être gelé par un mortel.");}IceOutcome::Unavailable=>self.notify(if self.ice==Ice::Unknown{"Rien ne se passe. Votre main est vide."}else{"La Glace éternelle a déjà été utilisée."}),_=>{}}}
  if defense_stage(self.stage,self.sector)&&self.defense_spawned<2&&self.elapsed>7.*(self.defense_spawned as f32+1.) {
   let n=self.defense_spawned;self.defense_spawned+=1;let base=100+n as u32*2;
   self.enemies.push(Enemy::new(base,Kind::Guard,750.,225.+n as f32*190.));
   self.enemies.push(Enemy::new(base+1,Kind::Wisp,800.,460.-n as f32*190.));
   self.notify("Renforts ! Résistez jusqu'à leur départ !");
  }
  let mut damage=0.;let mut add=Vec::new();let mut froze=false;let mut divine_touch=false;
  for e in &mut self.enemies {if e.hp<=0.{continue;}e.hurt=(e.hurt-dt).max(0.);if e.frozen>0.{e.frozen-=dt;if e.frozen<=0.{e.frozen=-1.;e.hp=0.;froze=true;}continue;}e.cooldown-=dt;let delta=self.player-e.pos;let dist=delta.length();let dir=norm(delta);
   if e.kind==Kind::Rival&&self.duel_two {let mut v=Vec2::ZERO;if is_key_down(KeyCode::Left){v.x-=1.}if is_key_down(KeyCode::Right){v.x+=1.}if is_key_down(KeyCode::Up){v.y-=1.}if is_key_down(KeyCode::Down){v.y+=1.}if v.length_squared()>0.{e.pos=clamp_enemy(e.pos+norm(v)*175.*dt);}if (is_key_pressed(KeyCode::RightShift)||is_key_pressed(KeyCode::K))&&e.cooldown<=0.{e.cooldown=0.42;if dist<75.{damage=17.;}}}
   else {let target=if e.kind.boss(){165.}else{35.};if dist>target {e.pos=clamp_enemy(e.pos+dir*e.kind.speed()*e.judged_boost()*dt);}if e.kind==Kind::Wisp{e.pos.y=(e.pos.y+(self.elapsed*4.).sin()*20.*dt).clamp(TOP+20.,BOTTOM-20.);}if e.cooldown<=0.{match e.kind{
    Kind::Communist=>{e.cooldown=(if e.zombie{1.35/ZOMBIE_POWER}else{1.35})/e.judged_boost();let dmg=if e.zombie{20.*ZOMBIE_POWER}else{10.};for ang in [-0.42_f32,0.,0.42]{let v=vec2(dir.x*ang.cos()-dir.y*ang.sin(),dir.x*ang.sin()+dir.y*ang.cos());add.push(Shot::hit(e.pos,v*190.,Projectile::Blood,false,dmg,e.can_hurt_justice()));}},
    Kind::Capitalist=>{e.cooldown=(if e.zombie{1./ZOMBIE_POWER}else{1.})/e.judged_boost();let kind=if (self.elapsed as i32)%3==0 {Projectile::Building}else{Projectile::Coin};let dmg=if kind==Projectile::Building{19.}else{10.};add.push(Shot::hit(e.pos,dir*220.,kind,false,if e.zombie{dmg*ZOMBIE_POWER}else{dmg},e.can_hurt_justice()));},
    Kind::Skeleton=>{e.cooldown=2.2/e.judged_boost();if dist<500.{let dmg=if e.zombie{10.*ZOMBIE_POWER}else if e.demigod{15.}else{10.};add.push(Shot::hit(e.pos,dir*185.,Projectile::Bone,false,dmg,e.can_hurt_justice()));}},
    _=>if dist<48.{let base=if e.kind==Kind::Guard{15.}else{10.};damage=base*e.touch_multiplier();if e.can_hurt_justice(){divine_touch=true;}e.cooldown=1.3/e.judged_boost();},}}}
   if e.kind.boss()&&dist<62.{damage=15.*e.touch_multiplier();divine_touch=true;}
  }
  self.shots.extend(add);if damage>0.{self.damage_from(damage,divine_touch);}damage=0.;
  let mut boom=false;let over=self.overpower();let spell_power=power_for(self.hunter);
  for s in &mut self.shots{s.pos+=s.vel*dt;s.life-=dt;if s.friendly{for e in &mut self.enemies{let radius=if e.kind.boss(){52.}else{28.};if e.frozen<=0.&&e.hp>0.&&s.pos.distance(e.pos)<radius&&!s.hit_ids.contains(&e.id){e.hp-=if over{999.}else if s.kind==Projectile::Wave{42.*spell_power}else{38.*spell_power};e.hurt=0.15;s.hit_ids.push(e.id);if s.kind==Projectile::Flame{s.life=0.;break;}}}}else if s.pos.distance(self.player)<if s.kind==Projectile::Building{32.}else{21.}{s.life=0.;damage=s.damage;divine_touch=s.divine;if s.kind==Projectile::Building{boom=true;}}}
  if damage>0.{self.damage_from(damage,divine_touch);}self.shots.retain(|s|s.life>0.&&s.pos.x>0.&&s.pos.x<W&&s.pos.y>150.&&s.pos.y<H);
  let mut deaths=0u32;for e in &self.enemies{if e.hp<=0.{deaths+=1;if e.frozen<0.{self.statues.push((e.pos,e.kind));}self.kills+=1;if self.kills%3==0{self.heals.push(e.pos);}}}self.enemies.retain(|e|e.hp>0.);
  for _ in 0..deaths.min(3){self.cue(Sfx::Die);}if froze{self.cue(Sfx::IceBlock);}if boom{self.cue(Sfx::Explosion);}
  let before_step=self.hunter/KILLS_PER_STEP;self.register_kills(deaths);if self.hunter/KILLS_PER_STEP>before_step{self.cue(Sfx::Demigod);self.notify(&format!("PALIER DE PUISSANCE {} : vie maximale {} · coups renforcés",self.hunter/KILLS_PER_STEP,self.max_hp.ceil() as i32));self.save();}
  let old=self.heals.len();self.heals.retain(|p|p.distance(self.player)>26.);let count=old-self.heals.len();if count>0{self.heal(25.*count as f32);self.cue(Sfx::Heal);self.notify("Soin récupéré : +25 points de vie");}
  let old=self.clues.len();self.clues.retain(|p|p.distance(self.player)>28.);if old>self.clues.len(){self.cue(Sfx::Coin);self.notify("Document récupéré !");}
  if self.hp<=0.{self.mode=Mode::Dead;return;}
  if self.enemies.is_empty()&&self.clues.is_empty()&&defense_time(self.stage,self.sector,self.elapsed)<=0.&&self.player.distance(vec2(885.,365.))<48.{self.advance_sector();}
 }
}
fn movement()->Vec2{let mut d=Vec2::ZERO;if is_key_down(KeyCode::A)||is_key_down(KeyCode::Q){d.x-=1.}if is_key_down(KeyCode::D){d.x+=1.}if is_key_down(KeyCode::Z)||is_key_down(KeyCode::W){d.y-=1.}if is_key_down(KeyCode::S){d.y+=1.}d}
fn ctrl_g()->bool{(is_key_down(KeyCode::LeftControl)||is_key_down(KeyCode::RightControl))&&is_key_pressed(KeyCode::G)}
fn spawn_enemies_for(stage:usize,sector:usize,justice:bool)->Vec<Enemy>{let mut v=spawn_enemies(stage,sector);if justice{
 // Second cycle : certains ennemis sont promus demi-dieux, un sur trois.
 if stage==COMMUNIST&&sector==3{v.retain(|e|!e.kind.boss());v.push(Enemy::new(900,Kind::Communist,750.,360.).zombified());}
 if stage==CAPITALIST&&sector==3{v.retain(|e|!e.kind.boss());v.push(Enemy::new(900,Kind::Capitalist,750.,360.).zombified());}
 let mut rank=0u32;
 for e in v.iter_mut(){if e.kind.boss(){continue;}if e.demigod{continue;}rank+=1;if rank%3==0{let pos=e.pos;let id=e.id;let kind=e.kind;*e=Enemy::new(id,kind,pos.x,pos.y).promoted();}}
 }v}
fn spawn_enemies(stage:usize,sector:usize)->Vec<Enemy>{let mut result=Vec::new();if stage==1&&sector==3{result.push(Enemy::new(0,Kind::Rival,740.,350.));return result;}if sector==3&&(stage==COMMUNIST||stage==CAPITALIST){let boss=if stage==COMMUNIST{Kind::Communist}else{Kind::Capitalist};result.push(Enemy::new(0,boss,750.,360.));for i in 0..3{result.push(Enemy::new(i+1,if i%2==0{Kind::Guard}else{Kind::Skeleton},450.+i as f32*120.,245.+(i%2) as f32*225.));}return result;}
 let count=if stage==0{4+sector}else{6+(stage+sector)%4};for i in 0..count{let options=[Kind::Slime,Kind::Skeleton,Kind::Wisp,Kind::Guard,Kind::Skeleton,Kind::Slime];let kind=options[(stage*3+sector+i)%options.len()];let x=430.+((i*127+sector*41)%440) as f32;let y=215.+((i*137+stage*31)%285) as f32;result.push(Enemy::new(i as u32,kind,x,y));}result}
fn panel(x:f32,y:f32,w:f32,h:f32){draw_rectangle(x+6.,y+7.,w,h,Color::new(0.,0.,0.,0.45));draw_rectangle(x,y,w,h,c(0x171c2c));draw_rectangle_lines(x,y,w,h,2.,c(0x9a8058));}
fn fit(s:&str,x:f32,y:f32,size:u16,width:f32,col:Color,font:&Font){let measured=measure_text(s,Some(font),size,1.).width;let scale=if measured>width{width/measured}else{1.};draw_text_ex(s,x,y,TextParams{font:Some(font),font_size:size,font_scale:scale,color:col,..Default::default()});}
fn environment(stage:usize,sector:usize){let place=CHAPTERS[stage].place;let (sky,floor,trim)=match place{Place::City=>(0x2b3143,0x34404a,0x657275),Place::Arena=>(0x49343e,0x645248,0xb39671),Place::Sewers=>(0x1f3736,0x304844,0x7aa590),Place::Factory=>(0x3b3338,0x514747,0xb99473),Place::Archives=>(0x393047,0x4a4050,0xc6a37d),Place::Gardens=>(0x283b37,0x425642,0xa3bc7e),Place::Palace=>(0x393047,0x4c404c,0xc9a46e),Place::Red=>(0x3e2730,0x55363b,0xba8176),Place::Void=>(0x29213b,0x3c303d,0xccad72)};
 clear_background(c(0x101521));draw_rectangle(0.,0.,W,145.,c(0x171d2d));draw_rectangle(0.,145.,W,38.,c(sky));draw_rectangle(0.,183.,W,427.,c(floor));
 // A continuous skyline and windows give each setting a distinct silhouette.
 for i in 0..18 {let x=i as f32*56.;let h=21.+((i*11+stage*3)%30) as f32;draw_rectangle(x,180.-h,48.,h,c(sky));draw_rectangle_lines(x,180.-h,48.,h,1.,c(trim));for j in 0..3{draw_rectangle(x+8.+j as f32*12.,185.-h,5.,7.,if (i+j+sector)%4==0{c(0xe3bc72)}else{c(0x586371)});}}
 for x in (0..960).step_by(48){for y in (186..610).step_by(48){let alpha=if (x/48+y/48)%2==0{0.035}else{0.015};draw_rectangle(x as f32,y as f32,48.,48.,Color::new(1.,1.,1.,alpha));draw_rectangle_lines(x as f32,y as f32,48.,48.,1.,Color::new(0.,0.,0.,0.10));}}
 // Decorative props, deliberately non-colliding so the arena remains fair.
 for i in 0..8{let x=70.+i as f32*115.;let y=if i%2==0{197.}else{532.};match place{Place::Gardens=>{draw_circle(x,y,12.,c(0x65865e));draw_circle(x-9.,y+4.,8.,c(0x739763));draw_rectangle(x-2.,y+7.,4.,7.,c(0x7b6044));},Place::Archives=>{draw_rectangle(x-12.,y-13.,24.,25.,c(0x795b47));for j in 0..4{draw_rectangle(x-9.+j as f32*5.,y-9.,3.,17.,c(0xb08c69));}},Place::Sewers=>{draw_ellipse(x,y,18.,7.,0.,c(0x203333));draw_ellipse_lines(x,y,18.,7.,0.,2.,c(0x84a891));},Place::Palace|Place::Void=>{draw_rectangle(x-8.,y-12.,16.,24.,c(0x947758));draw_circle(x,y-12.,5.,c(0xf6cb71));},_=>{draw_rectangle(x-9.,y-9.,18.,18.,c(0x695a51));draw_line(x-9.,y-9.,x+9.,y+9.,2.,c(0xa18c6e));draw_line(x+9.,y-9.,x-9.,y+9.,2.,c(0xa18c6e));}}}
 draw_rectangle_lines(LEFT,TOP,RIGHT-LEFT,BOTTOM-TOP,2.,c(trim));draw_rectangle(0.,610.,W,30.,c(0x111824));}
fn heart(p:Vec2){draw_circle(p.x,p.y,13.,c(0x244842));draw_rectangle(p.x-3.,p.y-9.,6.,18.,c(0xa9f2b2));draw_rectangle(p.x-9.,p.y-3.,18.,6.,c(0xa9f2b2));}
fn draw_world(g:&Game,a:&Assets){let f=&a.font;environment(g.stage,g.sector);text(&format!("{:02}  {}",g.stage+1,CHAPTERS[g.stage].title),25.,32.,22,c(0xf1d296),f);text(&format!("QUARTIER {} / {}   •   ENNEMIS {}   •   DOCUMENTS {}",g.sector+1,SECTORS,g.enemies.len(),g.clues.len()),26.,56.,14,c(0xc1cbd2),f);
 draw_rectangle(27.,75.,204.,16.,c(0x10151b));draw_rectangle(30.,78.,198.*(g.hp/g.max_hp).clamp(0.,1.),10.,c(0xdc6265));text(&format!("VIE {}/{}",g.hp.ceil() as i32,g.max_hp.ceil() as i32),242.,89.,15,WHITE,f);
 let flame=if g.stage<3{"FLAMME : —"}else if g.flame_cd>0.{"FLAMME : …"}else{"[E] FLAMME"};let wave=if g.stage<12{"VAGUE : —"}else if g.wave_cd>0.{"VAGUE : …"}else{"[T] VAGUE"};
 text(flame,355.,89.,14,if g.stage>=3{c(0xb7e994)}else{c(0x83909f)},f);text(wave,525.,89.,14,if g.stage>=12{c(0xffa7a2)}else{c(0x83909f)},f);
 if g.stock.charges>0{text(&format!("[X] GLACE : {}",g.stock.charges),685.,89.,14,if g.stock.available(){c(0xa4ebf8)}else{c(0x83909f)},f);}else if g.ice==Ice::Spent{text("GLACE : 0",685.,89.,14,c(0x83909f),f);}
 if g.justice{text("DIEU DE LA JUSTICE · seuls les demi-dieux et les zombies peuvent vous blesser",26.,140.,12,c(0xa9d7f5),f);}
 if defense_time(g.stage,g.sector,g.elapsed)>0.{text(&format!("RÉSISTER AUX RENFORTS : {} s",defense_time(g.stage,g.sector,g.elapsed).ceil() as i32),26.,122.,13,c(0xffd494),f);}else{text(&format!("{} ENNEMIS VAINCUS · COUPS {}{}",g.hunter,g.shown_damage(),if g.hunter>=MAX_HUNTER{" · PALIER MAXIMUM".to_owned()}else{format!(" · PROCHAIN PALIER DANS {} VICTOIRES",KILLS_PER_STEP-(g.hunter%KILLS_PER_STEP))}),26.,122.,12,c(0x97a7af),f);}if g.save_failed{text("SAUVEGARDE IMPOSSIBLE",700.,122.,12,c(0xff9797),f);}
 if rift_stage(g.stage){for x in [350.,590.]{draw_rectangle(x,TOP,60.,BOTTOM-TOP,c(0x121922));draw_rectangle_lines(x,TOP,60.,BOTTOM-TOP,2.,c(0x8ab4a8));for y in (195..550).step_by(52){draw_line(x+7.,y as f32,x+53.,y as f32+22.,2.,c(0x283b42));}}}
 // Door is always visible but only opens when every objective is met.
 let open=g.enemies.is_empty()&&g.clues.is_empty()&&defense_time(g.stage,g.sector,g.elapsed)<=0.;draw_rectangle(863.,326.,48.,78.,if open{c(0x3c7259)}else{c(0x4a4243)});draw_rectangle_lines(863.,326.,48.,78.,3.,if open{c(0xa5f0ad)}else{c(0xa78272)});draw_circle(901.,364.,3.,c(0xe7d493));text(if open{"SORTIE"}else{"FERMÉ"},861.,316.,12,if open{c(0xb0efa6)}else{c(0xd3b0a2)},f);
 for p in &g.clues {draw_rectangle(p.x-9.,p.y-12.,18.,24.,c(0xe9d49e));for i in 0..3{draw_line(p.x-5.,p.y-6.+i as f32*5.,p.x+5.,p.y-6.+i as f32*5.,1.,c(0x735d48));}draw_circle_lines(p.x,p.y,18.,2.,c(0xf9e29e));}
 for p in &g.heals{heart(*p);}
 for (pos,kind) in &g.statues {let (tex,cell,scale)=match kind{Kind::Communist=>(&a.communist_idle,64.,2.),Kind::Capitalist=>(&a.capitalist_idle,64.,2.),Kind::Skeleton|Kind::Guard=>(&a.skeleton_idle,32.,2.3),Kind::Slime=>(&a.slime_idle,32.,2.),_=>(&a.player_idle,32.,2.1)};sprite(tex,*pos,0,cell,scale,true,c(0xa3e8f4));let size=if kind.boss(){105.}else{60.};draw_rectangle(pos.x-size/2.,pos.y-size/2.,size,size,Color::new(0.4,0.88,1.,0.42));draw_rectangle_lines(pos.x-size/2.,pos.y-size/2.,size,size,4.,c(0xb9f9ff));draw_line(pos.x-size/3.,pos.y+size/3.,pos.x+size/3.,pos.y-size/3.,2.,c(0xf0ffff));}
 for e in &g.enemies {draw_ellipse(e.pos.x,e.pos.y+20.,25.,7.,0.,Color::new(0.,0.,0.,0.35));let frame=(g.elapsed*6.) as usize%4;let tint=if e.frozen>0.{c(0xa5edff)}else if e.hurt>0.{c(0xff9b98)}else if e.zombie{c(0x9fe07a)}else if e.demigod{c(0xffe9a8)}else{WHITE};match e.kind{
 Kind::Slime=>if e.cooldown<0.35{sprite(&a.slime_idle,e.pos,0,32.,2.,false,tint)}else{sprite(&a.slime,e.pos,frame,32.,2.,false,tint)},Kind::Skeleton=>if e.cooldown<0.35{sprite(&a.skeleton_idle,e.pos,0,32.,2.,false,tint)}else{sprite(&a.skeleton,e.pos,frame,32.,2.,false,tint)},
 Kind::Wisp=>{draw_circle(e.pos.x,e.pos.y,17.,c(0x75d6ce));draw_circle(e.pos.x-5.,e.pos.y-3.,4.,c(0x213947));draw_circle(e.pos.x+6.,e.pos.y-3.,4.,c(0x213947));draw_circle_lines(e.pos.x,e.pos.y,22.,2.,c(0xb0f5e6));},
 Kind::Guard=>{sprite(&a.skeleton,e.pos,frame,32.,2.3,false,c(0xe3bf82));draw_rectangle(e.pos.x-20.,e.pos.y+3.,40.,7.,c(0x967148));},Kind::Rival=>sprite(&a.player,e.pos,frame,32.,2.2,true,c(0xff9c9c)),Kind::Communist=>sprite(&a.communist,e.pos,frame,64.,2.,true,tint),Kind::Capitalist=>sprite(&a.capitalist,e.pos,frame,64.,2.,true,tint)}
 if e.frozen>0.{let size=if e.kind.boss(){105.}else{60.};draw_rectangle(e.pos.x-size/2.,e.pos.y-size/2.,size,size,Color::new(0.4,0.88,1.,0.42));draw_rectangle_lines(e.pos.x-size/2.,e.pos.y-size/2.,size,size,4.,c(0xb9f9ff));draw_line(e.pos.x-size/3.,e.pos.y+size/3.,e.pos.x+size/3.,e.pos.y-size/3.,2.,c(0xf0ffff));}
 let bw=if e.kind.boss(){120.}else{42.};let y=e.pos.y-if e.kind.boss(){65.}else{37.};draw_rectangle(e.pos.x-bw/2.,y,bw,5.,c(0x151923));draw_rectangle(e.pos.x-bw/2.,y,bw*(e.hp/e.hp_max()).clamp(0.,1.),5.,c(0xe67368));
  if e.demigod&&!e.zombie{text("DEMI-DIEU",e.pos.x-30.,y-8.,11,c(0xffe9a8),f);draw_circle_lines(e.pos.x,e.pos.y,26.,2.,c(0xffe9a8));}
  if e.judged{text("JUGÉ ×2",e.pos.x-22.,y-8.,12,c(0xc9b3ff),f);draw_circle_lines(e.pos.x,e.pos.y,24.,2.,c(0xc9b3ff));}
  if e.zombie{text("ZOMBIE",e.pos.x-24.,y-8.,12,c(0x9fe07a),f);draw_circle_lines(e.pos.x,e.pos.y,if e.kind.boss(){62.}else{28.},2.,c(0x9fe07a));}}
 for s in &g.shots {match s.kind{Projectile::Flame=>sprite(&a.fire,s.pos,(g.elapsed*8.) as usize%2,32.,1.5,s.vel.x<0.,WHITE),Projectile::Wave|Projectile::Blood=>sprite(&a.blood,s.pos,(g.elapsed*8.) as usize%2,32.,if s.kind==Projectile::Wave{2.3}else{1.8},s.vel.x<0.,WHITE),Projectile::Coin=>{draw_circle(s.pos.x,s.pos.y,10.,c(0xf5cb63));draw_circle_lines(s.pos.x,s.pos.y,7.,2.,c(0x9b6a31));},Projectile::Building=>{draw_rectangle(s.pos.x-22.,s.pos.y-27.,44.,54.,c(0xa59d88));for i in 0..3{for j in 0..3{draw_rectangle(s.pos.x-16.+i as f32*12.,s.pos.y-20.+j as f32*14.,7.,7.,c(0x47485b));}}},Projectile::Bone=>{draw_line(s.pos.x-8.,s.pos.y-8.,s.pos.x+8.,s.pos.y+8.,5.,c(0xe8e2cc));}}}
 draw_ellipse(g.player.x,g.player.y+19.,20.,7.,0.,Color::new(0.,0.,0.,0.4));let height=if g.jump>0.{(std::f32::consts::PI*(g.jump/0.68)).sin()*34.}else{0.};if g.immune<=0.||(g.elapsed*12.) as i32%2==0{let moving=movement().length_squared()>0.;if moving{sprite(&a.player,vec2(g.player.x,g.player.y-height),(g.elapsed*7.) as usize%4,32.,2.2,g.face.x<0.,WHITE);}else{sprite(&a.player_idle,vec2(g.player.x,g.player.y-height),0,32.,2.2,g.face.x<0.,WHITE);}}if g.swing>0.{let p=g.player+g.face*42.-vec2(0.,height);draw_circle_lines(p.x,p.y,25.,5.,c(0xf9e1a0));}
 if g.justice{text(if g.judgement{"[Y] JUGEMENT"}else{"JUGEMENT : 0"},800.,89.,14,if g.judgement{c(0xc9b3ff)}else{c(0x83909f)},f);}
 if g.message_time>0.{fit(&g.message,285.,158.,15,440.,c(0xffe0a2),f);}let hint=if g.stage==1&&g.sector==3&&g.duel_two{"J2 : FLÈCHES + MAJ DROITE OU K"}else if rift_stage(g.stage){"ESPACE : SAUTER LES FAILLES · R : FRAPPER · SORTIE APRÈS OBJECTIFS"}else{"ZQSD/WASD : BOUGER   R : FRAPPER   ESPACE : SAUTER   ÉCHAP : PAUSE"};text(hint,27.,632.,13,c(0xc3ced2),f);
 if g.justice{text("Y : JUGER UNE ÂME (1 FOIS PAR QUARTIER)",660.,632.,12,c(0xc9b3ff),f);}
}
fn draw_backrooms(g:&Game,a:&Assets){let f=&a.font;let floor=maze::floor(g.back_floor);let (wall,plane,glow)=maze::palette(g.back_floor);
 clear_background(Color::new(0.075,0.07,0.05,1.));
 // Néons du plafond : lumière qui bourdonne et clignote par instants.
 for i in 0..6{let x=82.+i as f32*159.;let pulse=0.55+0.45*(g.elapsed*2.1+i as f32).sin().abs();draw_rectangle(x-30.,150.,60.,10.,Color::new(glow.r,glow.g,glow.b,0.35*pulse));draw_rectangle(x-24.,160.,48.,82.,Color::new(glow.r,glow.g,glow.b,0.035*pulse));}
 // Zones : murs, sol, revêtement usé.
 let area=floor.walkable;draw_rectangle(area.x-8.,area.y-8.,area.w+16.,area.h+16.,Color::new(0.16,0.145,0.1,1.));
 for cell in &floor.cells {draw_rectangle(cell.x-5.,cell.y-5.,cell.w+10.,cell.h+10.,Color::new(wall.r*0.55,wall.g*0.55,wall.b*0.5,1.));draw_rectangle(cell.x,cell.y,cell.w,cell.h,plane);draw_rectangle_lines(cell.x,cell.y,cell.w,cell.h,3.,wall);
  let seed=((cell.x*7.3+cell.y*3.1) as usize)%4;
  match seed{0=>{draw_rectangle(cell.x+14.,cell.y+cell.h-30.,26.,20.,Color::new(wall.r*0.85,wall.g*0.8,wall.b*0.8,0.7));},1=>{draw_circle(cell.x+cell.w-22.,cell.y+24.,9.,Color::new(wall.r*0.75,wall.g*0.72,wall.b*0.7,0.45));},
   2=>{for k in 0..3{draw_line(cell.x+12.,cell.y+16.+k as f32*13.,cell.x+cell.w-12.,cell.y+16.+k as f32*13.,1.,Color::new(0.,0.,0.,0.1));}},_=>{draw_rectangle(cell.x+cell.w*0.5-12.,cell.y+cell.h*0.5-10.,24.,20.,Color::new(0.,0.,0.,0.08));}}}
 // Moquette humide : quelques taches déterministes.
 for i in 0..14{let x=60.+((i*137+g.back_floor*53)%880) as f32;let y=182.+((i*91+g.back_floor*37)%430) as f32;draw_circle(x,y,6.+((i%3) as f32)*4.,Color::new(0.2,0.17,0.1,0.22));}
 // Escalier (discret) : marches, avec une flèche selon le sens.
 let st=floor.stairs;let col=if st.up{c(0xcfe7ff)}else{c(0xffd9a8)};
 draw_rectangle(st.pos.x-20.,st.pos.y-14.,40.,28.,Color::new(0.06,0.06,0.05,0.65));for k in 0..3{draw_line(st.pos.x-14.+k as f32*4.,st.pos.y+10.-k as f32*8.,st.pos.x+14.,st.pos.y+10.-k as f32*8.,2.,col);}
 text("↑",st.pos.x-5.,st.pos.y+5.,16,col,f);
 if let Some(d)=floor.down{draw_rectangle(d.pos.x-20.,d.pos.y-14.,40.,28.,Color::new(0.06,0.06,0.05,0.5));for k in 0..3{draw_line(d.pos.x-14.+k as f32*4.,d.pos.y-10.+k as f32*8.,d.pos.x+14.,d.pos.y-10.+k as f32*8.,2.,c(0xffd9a8));}text("↓",d.pos.x-5.,d.pos.y+5.,16,c(0xffd9a8),f);}
 // Niveau exploré : le joueur n'a aucune consigne, seulement le compteur.
 text("LES BACKROOMS",30.,44.,26,c(0xf2ecc9),f);
 let pips:usize=g.back_floor.min(18);for i in 0..18{let x=32.+i as f32*13.;let on=i<pips;draw_rectangle(x,58.,8.,8.,if on{c(0xe9dfa2)}else{Color::new(0.9,0.86,0.6,0.18)});}
 text(&format!("NIVEAU {}",g.back_floor),296.,68.,15,c(0xd8d2a8),f);
 if g.back_floor>=maze::FLOORS{text("La moquette s'arrête. Une lumière différente vient d'en haut.",30.,96.,15,c(0xf2e7b2),f);}else{text("ZQSD/WASD : MARCHER    E : INTERAGIR",30.,96.,14,c(0xcdc7a3),f);}
 sprite(&a.player,g.player,(g.elapsed*7.) as usize%4,32.,2.2,g.face.x<0.,if g.hurt_flash>0.{c(0xffb0a8)}else{WHITE});
 // Vie en jeu : les Backrooms usent le joueur lentement.
 draw_rectangle(27.,601.,204.,14.,c(0x10151b));draw_rectangle(30.,604.,198.*(g.hp/g.max_hp).clamp(0.,1.),8.,c(0xdc6265));text(&format!("VIE {}/{}",g.hp.ceil() as i32,g.max_hp.ceil() as i32),242.,614.,13,WHITE,f);
 text("ÉCHAP : PAUSE",790.,614.,12,c(0xb9b394),f);
 if g.message_time>0.{fit(&g.message,270.,150.,15,420.,c(0xf6e8b8),f);}}

fn draw_pantheon(g:&Game,a:&Assets){let f=&a.font;
 clear_background(c(0x1a1425));for i in 0..9{let x=40.+i as f32*110.;draw_rectangle(x,180.,34.,430.,c(0x2c2440));draw_rectangle(x-6.,168.,46.,14.,c(0x40355c));draw_rectangle(x-6.,598.,46.,14.,c(0x40355c));}
 for i in 0..60{let x=((i*173)%930) as f32;let y=200.+((i*97)%380) as f32;draw_circle(x,y,1.6+((i%3) as f32),c(0xffe9ab));}
 draw_rectangle(0.,0.,W,150.,Color::new(0.1,0.08,0.16,0.9));
 text("LE PANTHÉON",30.,46.,27,c(0xffe8ab),f);text("Les dieux du Conflit déposent ici leurs couronnes et leurs comptes.",30.,80.,16,c(0xe7d9b8),f);
 let s=g.elapsed;draw_circle(480.,300.,46.,Color::new(0.99,0.86,0.5,0.16+0.06*(s*1.6).sin()));draw_circle_lines(480.,300.,46.,2.,c(0xffe1a0));
 sprite(&a.capitalist_idle,vec2(330.,360.),0,64.,2.4,false,c(0xfff0c4));sprite(&a.communist_idle,vec2(630.,360.),0,64.,2.4,true,c(0xfff0c4));sprite(&a.player,vec2(480.,556.),(g.elapsed*6.) as usize%4,32.,2.3,false,WHITE);
 panel(120.,418.,720.,196.);
 center("Une voix sans visage : « Mortel, tu as gravi nos étages. »",452.,17,c(0xfff1c8),f);
 center("« Jure, et ton nom entrera dans nos registres. »",480.,17,c(0xfff1c8),f);
 center("[E] PRÊTER SERMENT · DEVENIR DEMI-DIEU",536.,19,c(0xa8f3c1),f);
 center("[R] REFUSER · RESTER MORTEL",570.,19,c(0xf7b7a5),f);
 if g.message_time>0.{fit(&g.message,240.,606.,13,480.,c(0xd9c9a8),f);}}

fn overlay(){draw_rectangle(0.,0.,W,H,Color::new(0.02,0.03,0.06,0.84));}
#[macroquad::main(window_conf)]
async fn main(){
 if std::env::args().any(|x|x=="--self-test"){
  // Simulate completion with the real update loop, including convoy timers.
  // No input is injected: enemy deaths and document collection are forced.
  let mut game=Game::new(false);
  game.setup(COMMUNIST,3,true,false);game.enemies.truncate(1);game.give_ice(1);
  // Un mortel ne peut pas geler un dieu ; un demi-dieu le peut.
  assert!(matches!(game.freeze_one(),IceOutcome::Forbidden));assert_eq!(game.stock.charges,1);
  game.demigod=true;assert!(matches!(game.freeze_one(),IceOutcome::Frozen));
  for _ in 0..60 {game.update(0.033);}assert!(game.enemies.is_empty());assert_eq!(game.statues.len(),1);assert_eq!(game.stock.charges,0);
  // 18 étages parcourus par le vrai déplacement, puis le Panthéon.
  let mut walker=Game::new(false);walker.enter_backrooms();
  for level in 1..=maze::FLOORS {
   assert_eq!(walker.back_floor,level);
   let stairs=maze::floor(level).stairs.pos;
   for _ in 0..400 {if walker.player.distance(stairs)<6.{break;}walker.hp=walker.max_hp;walker.backroom_move(norm(stairs-walker.player),0.033);}
   walker.hp=walker.max_hp;
   assert!(walker.climb(true)||level==maze::FLOORS,"escalier inaccessible à l'étage {level}");
  }
  assert!(maze::at_pantheon_gate(&maze::floor(maze::FLOORS),walker.player));walker.enter_pantheon();assert_eq!(walker.mode,Mode::Pantheon);walker.accept_demigod();assert!(walker.demigod&&walker.stock.charges>0);
  game=Game::new(false);game.setup(0,0,true,false);let mut completed=0;
  for stage in 0..CHAPTERS.len(){if stage==CAPITALIST {assert_eq!(game.mode,Mode::Choice);game.setup(CAPITALIST,0,true,false);}
   for sector in 0..SECTORS {assert_eq!((game.stage,game.sector),(stage,sector));game.mode=Mode::Playing;
    let mut done=false;for _ in 0..800 {for e in &mut game.enemies{e.hp=0.;}game.clues.clear();game.shots.clear();game.player=vec2(885.,365.);game.hp=game.max_hp;game.immune=1.;game.update(0.033);
     if game.stage!=stage||game.sector!=sector||game.mode==Mode::Choice||game.mode==Mode::End {done=true;break;}}
    assert!(done,"Quartier bloqué : {stage}/{sector}");completed+=1;
   }}
  assert_eq!(game.mode,Mode::End);assert_eq!(completed,100);
  assert!(Progress::decode(&Progress{stage:12,sector:1,charges:999,duel_two:false,demigod:true,divine_freeze:true,ice_granted:true,floor:9,hunter:3,max_hp:120.,justice:false}.encode()).is_some());
  // Second cycle : dieu de la justice, ennemis promus, dieux zombies.
  let mut j=Game::new(false);j.start_justice();assert!(j.justice);
  for stage in 0..CHAPTERS.len(){if stage==CAPITALIST{assert_eq!(j.mode,Mode::Choice);j.setup(CAPITALIST,0,true,false);}
   for sector in 0..SECTORS{let before=(j.stage,j.sector);j.mode=Mode::Playing;
    for _ in 0..1200{for e in &mut j.enemies{e.hp=0.;}j.clues.clear();j.shots.clear();j.player=vec2(885.,365.);j.hp=j.max_hp;j.immune=1.;j.update(0.033);
     if (j.stage,j.sector)!=before||j.mode==Mode::Choice||j.mode==Mode::End{break;}}
    if (j.stage,j.sector)==before&&j.mode!=Mode::Choice&&j.mode!=Mode::End{panic!("Second cycle bloqué : {stage}/{sector}");}
   }}
  assert!(j.justice&&j.hunter>=100,"le second cycle doit aussi faire progresser le personnage");
  println!("SELF-TEST OK : {completed} quartiers, puis le cycle de la justice jusqu'à la fin");return;
 }
 let a=Assets::new();let mut sound=Audio::new().await;
 if std::env::args().any(|x|x=="--audio-check"){let (fx,mu)=sound.counts();println!("AUDIO : {} effets chargés / {}, {} musiques chargées / {}. Périphérique : {}",fx,17,mu,8,if sound.available(){"disponible"}else{"absent"});return;}
 let mut g=Game::new(true);let mut available=load_progress();let mut password=String::new();let mut code_focus=false;g.request_music(Music::Menu);
 // Options de démonstration, réservées aux essais : --demo=backrooms:N, pantheon, boss
 for arg in std::env::args().filter(|x|x.starts_with("--demo=")){
  let value=&arg[7..];
  if let Some(n)=value.strip_prefix("backrooms:"){g.setup(0,0,true,false);g.enter_backrooms();g.back_floor=n.parse().unwrap_or(1);g.player=maze::start_pos(g.back_floor);}
  else if value=="pantheon"{g.setup(0,0,true,false);g.enter_backrooms();g.back_floor=maze::FLOORS;g.player=maze::start_pos(maze::FLOORS);g.enter_pantheon();}
  else if value=="boss"{g.setup(CAPITALIST,3,true,false);}
  else if value=="fin"{g.setup(CAPITALIST,3,true,false);g.won_betrayal=true;g.mode=Mode::End;}
  else if value=="justice"{g.start_justice();g.mode=Mode::Playing;g.setup(COMMUNIST,3,true,false);g.mode=Mode::Playing;}
  else if value=="titre"{}
  else if value=="monde"{g.setup(3,1,true,true);g.mode=Mode::Playing;}
  else if value=="backrooms"{g.setup(0,0,true,false);g.enter_backrooms();g.player=maze::start_pos(1);}
  else if value=="flamme"{g.setup(12,1,true,false);g.shots.push(Shot::new(g.player+vec2(60.,-20.),vec2(410.,0.),Projectile::Flame,true));}
  else if value=="panneau"{g.setup(1,3,true,false);g.mode=Mode::Story;}
 }
 loop{let dt=get_frame_time().min(0.033);let mut camera=Camera2D::from_display_rect(display_rect(screen_width(),screen_height()));camera.zoom.y=-camera.zoom.y;set_camera(&camera);
 match g.mode{
 Mode::Title=>{environment(0,0);overlay();center("LES DIEUX DU CONFLIT",180.,38,c(0xf6d794),&a.font);center("LE PRIX DU SILENCE",222.,19,c(0xd7bd93),&a.font);sprite(&a.capitalist_idle,vec2(300.,330.),0,64.,1.6,false,WHITE);sprite(&a.player,vec2(480.,330.),(get_time()*5.) as usize%4,32.,3.,false,WHITE);sprite(&a.communist_idle,vec2(660.,330.),0,64.,1.6,true,WHITE);
 center("ENTRÉE : NOUVELLE PARTIE",427.,20,c(0xbfeaa0),&a.font);if let Some(p)=available{center(&format!("C : CONTINUER · CHAPITRE {} · QUARTIER {}",p.stage+1,p.sector+1),463.,17,c(0xb8d9ef),&a.font);center("Une nouvelle partie remplace la sauvegarde.",494.,13,c(0xbac1c7),&a.font);}else{center("Votre aventure est sauvegardée entre les quartiers.",469.,15,c(0xbac1c7),&a.font);}center("25 chapitres · combats · exploration · choix",552.,15,c(0xabb9c1),&a.font);
 if !sound.available(){center("SON INDISPONIBLE : aucun périphérique audio détecté",586.,13,c(0xe8a89a),&a.font);}text(VERSION,905.,620.,12,c(0x6b7684),&a.font);
 if is_key_pressed(KeyCode::Enter){sound.sfx(Sfx::Ui2);g=Game::new(true);g.reset_ice();g.setup(0,0,true,true);sound.play(Music::Surface1);}if is_key_pressed(KeyCode::C){if let Some(p)=available{sound.sfx(Sfx::Ui2);g=Game::new(true);g.duel_two=p.duel_two;g.demigod=p.demigod;g.divine_freeze=p.divine_freeze;g.stock=IceStock{charges:p.charges};g.ice_granted=p.ice_granted;g.refresh_ice();g.back_floor=p.floor;g.hunter=p.hunter;g.max_hp=p.max_hp;g.justice=p.justice;g.judgement=p.justice;g.setup(p.stage,p.sector,true,p.sector==0);}}},
 Mode::Story=>{draw_world(&g,&a);overlay();panel(75.,103.,810.,432.);text(&format!("CHAPITRE {:02} / {}",g.stage+1,CHAPTERS.len()),115.,151.,15,c(0xa9bac8),&a.font);fit(CHAPTERS[g.stage].title,115.,195.,27,720.,c(0xf6d496),&a.font);for (i,line) in CHAPTERS[g.stage].lines.iter().enumerate(){fit(line,115.,252.+i as f32*35.,18,720.,WHITE,&a.font);}
 if g.stage==1{text(if g.duel_two{"DUEL FINAL : 2 JOUEURS · touche 2 pour changer"}else{"DUEL FINAL : ORDINATEUR · touche 2 pour changer"},115.,433.,15,c(0xb7eaa4),&a.font);if is_key_pressed(KeyCode::Key2){g.duel_two=!g.duel_two;g.save();}}
 else if g.stage==3{text("NOUVEAU : FLAMME VERTE [E]",115.,433.,16,c(0xaee88e),&a.font);}else if g.stage==12{text("NOUVEAU : VAGUE DE SANG [T]",115.,433.,16,c(0xf5a39a),&a.font);}else{text("4 quartiers à traverser · soins verts à ramasser",115.,433.,15,c(0xb8c7cb),&a.font);}
 text("ENTRÉE · CONTINUER",115.,488.,18,c(0xb6e69c),&a.font);if is_key_pressed(KeyCode::Enter){sound.sfx(Sfx::Ui2);g.mode=Mode::Playing;}},
 Mode::Playing=>{if is_key_pressed(KeyCode::Escape){g.mode=Mode::Paused;}else{g.update(dt);}if g.mode==Mode::Playing{draw_world(&g,&a);}},
 Mode::Backrooms=>{g.backroom_update(dt);match g.mode{Mode::Backrooms=>draw_backrooms(&g,&a),Mode::Pantheon=>draw_pantheon(&g,&a),Mode::Dead=>{draw_backrooms(&g,&a);},_=>{draw_world(&g,&a);}}},
 Mode::Pantheon=>{g.elapsed+=dt;if is_key_pressed(KeyCode::E){sound.sfx(Sfx::Demigod);g.accept_demigod();}else if is_key_pressed(KeyCode::R){sound.sfx(Sfx::Ui);g.decline_demigod();}if g.mode==Mode::Pantheon{draw_pantheon(&g,&a);}else if g.mode==Mode::Dead{draw_pantheon(&g,&a);}else{draw_world(&g,&a);}},
 Mode::Paused=>{draw_world(&g,&a);overlay();panel(200.,140.,560.,400.);
  center("PAUSE",205.,30,c(0xf1d296),&a.font);
  center("ÉCHAP : REPRENDRE",258.,18,c(0xbfeaa0),&a.font);
  center("Q : QUITTER VERS LE MENU",292.,15,c(0xb7c3ce),&a.font);
  center(if sound.muted(){"M : RÉTABLIR LE SON"}else{"M : COUPER LE SON"},320.,15,c(0xb7c3ce),&a.font);
  center("F2 : SAISIR UN CODE",348.,15,c(0xd9c58c),&a.font);
  center(&format!("VOLUME {} %   ·   MOINS / PLUS POUR RÉGLER",sound.master_percent()),376.,13,c(0x9fb0bf),&a.font);
  if is_key_pressed(KeyCode::Minus){sound.change_master(-0.1);}
  if is_key_pressed(KeyCode::Equal){sound.change_master(0.1);}
  draw_rectangle(268.,392.,424.,36.,c(0x10151b));draw_rectangle_lines(268.,392.,424.,36.,2.,if code_focus{c(0xd9c58c)}else{c(0x3b4654)});
  text("CODE",284.,416.,14,c(0x9fb0bf),&a.font);
  let shown=if password.is_empty(){"".to_owned()}else{str::repeat("*",password.chars().count())};
  let cursor=if code_focus&&(get_time()*2.) as i32%2==0{"_"}else{""};
  text(&format!("{shown}{cursor}"),352.,417.,19,c(0xf4e3b0),&a.font);
  if code_focus{center("LETTRES ET CHIFFRES · ENTRÉE : VALIDER · ÉCHAP : QUITTER LE CHAMP",516.,11,c(0x8c99a6),&a.font);}
  else if !password.is_empty(){center(&format!("{} CARACTÈRE(S) SAISIS",password.chars().count()),516.,11,c(0x8c99a6),&a.font);}
  else{center("CE QUE VOUS CHERCHEZ N'EST PAS ÉCRIT ICI",516.,11,c(0x6d7b88),&a.font);}
  if code_focus{
   while let Some(ch)=get_char_pressed(){if ch.is_ascii_alphanumeric()&&password.chars().count()<8{password.push(ch.to_ascii_uppercase());}}
   if is_key_pressed(KeyCode::Backspace){password.pop();}
   if is_key_pressed(KeyCode::Enter){
    let jumps_to_chapter=password.trim().eq_ignore_ascii_case("ML");
    if g.apply_password(&password){password.clear();code_focus=false;sound.sfx(Sfx::Demigod);if jumps_to_chapter{sound.play(Music::Boss);g.mode=Mode::Story;}else{g.mode=Mode::Playing;}}else{password.clear();code_focus=false;sound.sfx(Sfx::Ui);}}
   if is_key_pressed(KeyCode::Escape){code_focus=false;password.clear();}
  } else {
   if is_key_pressed(KeyCode::F2){code_focus=true;password.clear();sound.sfx(Sfx::Ui);}
   if is_key_pressed(KeyCode::Escape){g.mode=Mode::Playing;}
   if is_key_pressed(KeyCode::Q){sound.sfx(Sfx::Ui);g.save();available=load_progress();g.mode=Mode::Title;g.request_music(Music::Menu);}
   if is_key_pressed(KeyCode::M){sound.toggle_mute();}
  }},
 Mode::Choice=>{draw_world(&g,&a);overlay();panel(78.,120.,804.,400.);text("LE SALAIRE DU SILENCE",118.,174.,26,c(0xf5d294),&a.font);text("Le dieu du Capitalisme réclame les preuves et votre silence.",118.,239.,18,WHITE,&a.font);text("Sélène attend votre réponse. À qui doit servir votre force ?",118.,276.,18,WHITE,&a.font);text("[A] ACCEPTER · rester au service du dieu",118.,352.,19,c(0xbce7a8),&a.font);text("[T] TRAHIR · combattre votre employeur",118.,398.,19,c(0xfab49a),&a.font);if is_key_pressed(KeyCode::A){sound.sfx(Sfx::Ui2);g.won_betrayal=false;g.mode=Mode::End;g.request_music(Music::Victory);}if is_key_pressed(KeyCode::T){sound.sfx(Sfx::Roar);g.setup(CAPITALIST,0,true,true);}},
 Mode::Dead=>{draw_world(&g,&a);overlay();panel(185.,184.,590.,270.);center("VOUS ÊTES TOMBÉ",258.,29,c(0xffaa99),&a.font);center("Le quartier recommence avec toute votre vie.",310.,17,WHITE,&a.font);center("Vos objets restent dans l'état où vous les avez laissés.",342.,15,c(0xb8ced4),&a.font);center("ENTRÉE · RÉESSAYER",398.,19,c(0xbfeaa0),&a.font);if is_key_pressed(KeyCode::Enter){g.setup(g.stage,g.sector,true,false);}},
 Mode::End=>{environment(CAPITALIST,3);overlay();panel(78.,108.,804.,440.);
  center(if g.won_betrayal{"LA CITÉ REPREND SES COMPTES"}else{"LE SILENCE A UN PRIX"},168.,24,c(0xf6d496),&a.font);
  let lines=if g.won_betrayal{["Sélène et son frère rendent les registres aux habitants.","La cité bâtit ses propres règles, sans choisir un nouveau maître.","Vous avez brisé le contrat et payé le prix de la liberté."]}else{["Vous acceptez la récompense et remettez les registres au dieu.","Sélène part avec son frère ; la vérité reste cachée.","La cité est sauvée d'un dieu, mais pas de ses dettes."]};
  for (i,line) in lines.iter().enumerate(){center(line,212.+i as f32*28.,16,WHITE,&a.font);}
  draw_rectangle(110.,300.,740.,2.,c(0x4d5668));
  center("ON VOUS PROPOSE LE TRÔNE DE DIEU DE LA JUSTICE",332.,18,c(0xa9d7f5),&a.font);
  center("Tout recommencer, mais le monde aura changé :",360.,15,c(0xcbd4dd),&a.font);
  center("· les simples mortels ne pourront plus vous blesser ;",388.,14,c(0xcbd4dd),&a.font);
  center("· ceux promus demi-dieux, si ;",410.,14,c(0xcbd4dd),&a.font);
  center("· les deux dieux reviendront en zombies, deux fois plus forts.",432.,14,c(0xcbd4dd),&a.font);
  if g.justice{center("VOUS ÊTES DÉJÀ DIEU DE LA JUSTICE · ENTREE : MENU",476.,15,c(0xa8f3c1),&a.font);}
  else{center("[J] ACCEPTER LE TRÔNE · [ENTRÉE] RETOUR AU MENU",476.,16,c(0xbfeaa0),&a.font);}
  if is_key_pressed(KeyCode::Enter){available=load_progress();g.mode=Mode::Title;g.request_music(Music::Menu);}
  if !g.justice&&is_key_pressed(KeyCode::J){g.start_justice();sound.sfx(Sfx::Demigod);sound.play(Music::Surface1);g.mode=Mode::Story;}
 }
 }
 for cue in g.sfx_queue.drain(..){sound.sfx(cue);}
 sound.play(g.music_wish);
 next_frame().await;
 }
}
#[cfg(test)]mod tests{
 use super::*;
 /// Nombre de charges réellement écrites dans la sauvegarde du jeu.
 fn saved_progress(g:&Game)->u32{g.progress().charges}
 fn saved_progress_parts(g:&Game)->Progress{g.progress()}
 #[test]fn chapters_are_populated(){assert_eq!(CHAPTERS.len(),25);for i in 0..CHAPTERS.len(){for j in 0..SECTORS{assert!(!spawn_enemies(i,j).is_empty());}}}
 #[test]fn final_bosses_and_duel_spawn(){assert_eq!(spawn_enemies(COMMUNIST,3)[0].kind,Kind::Communist);assert_eq!(spawn_enemies(CAPITALIST,3)[0].kind,Kind::Capitalist);assert_eq!(spawn_enemies(1,3)[0].kind,Kind::Rival);}
 #[test]fn sector_and_story_progression(){let mut g=Game::new(false);g.setup(0,0,true,true);for stage in 0..=CHOICE{for sector in 0..SECTORS{assert_eq!((g.stage,g.sector),(stage,sector));g.advance_sector();}}assert_eq!(g.mode,Mode::Choice);g.setup(CAPITALIST,0,true,true);for _ in 0..SECTORS{g.advance_sector();}assert_eq!(g.mode,Mode::End);assert!(g.won_betrayal);}
 #[test]fn backrooms_award_only_once(){let mut g=Game::new(false);g.setup(0,0,true,false);g.enter_backrooms();assert_eq!(g.mode,Mode::Backrooms);g.leave_backrooms();assert_eq!(g.stock.charges,1);assert_eq!(g.freeze_one(),IceOutcome::Frozen);assert_eq!(g.ice,Ice::Spent);assert!(g.enemies.iter().any(|e|e.frozen>0.));g.enter_backrooms();g.leave_backrooms();assert_eq!(g.stock.charges,0);assert_eq!(g.freeze_one(),IceOutcome::Unavailable);}
 #[test]fn freeze_without_target_does_not_consume(){let mut g=Game::new(false);g.give_ice(1);assert_eq!(g.freeze_one(),IceOutcome::NoTarget);assert_eq!(g.stock.charges,1);}
 #[test]fn heal_and_damage_are_bounded(){let mut g=Game::new(false);g.hp=85.;g.heal(25.);assert_eq!(g.hp,100.);g.damage(40.);assert_eq!(g.hp,60.);g.damage(40.);assert_eq!(g.hp,60.);}
 #[test]fn every_ten_kills_raise_life_and_power(){
  let mut g=Game::new(false);g.setup(0,0,true,false);
  assert!(g.hunter<=20,"départ déjà trop fort : {}",g.hunter);
  g.hunter=0;g.max_hp=100.;g.hp=100.;
  let base_hp=g.max_hp;let base_hit=g.melee_damage();
  // La force augmente à chaque victoire, la vie par paliers de dix.
  g.register_kills(9);assert_eq!((g.hunter,g.max_hp),(9,base_hp));assert!(g.melee_damage()>base_hit);
  g.register_kills(1);assert_eq!(g.hunter,10);assert_eq!(g.max_hp,base_hp+5.);assert!(g.melee_damage()>base_hit);
  assert_eq!(g.hp,g.max_hp,"les nouveaux points de vie sont rendus tout de suite");
  g.register_kills(1000);assert_eq!(g.max_hp,life_for(MAX_HUNTER));assert!(g.max_hp<MAX_LIFE);
  assert!(g.melee_damage()>base_hit*4.);
 }
 #[test]fn killing_enemies_in_play_grants_progress(){
  let mut g=Game::new(false);g.setup(0,0,true,false);
  g.hunter=0;g.max_hp=100.;g.hp=100.;
  let base=g.max_hp;g.register_kills(KILLS_PER_STEP);
  assert_eq!(g.hunter,KILLS_PER_STEP);assert_eq!(g.max_hp,base+5.);
  // La progression est sauvegardée et relue telle quelle.
  let p=g.progress();assert_eq!((p.hunter,p.max_hp),(KILLS_PER_STEP,base+5.));
  let q=Progress::decode(&p.encode()).unwrap();assert_eq!((q.hunter,q.max_hp),(KILLS_PER_STEP,base+5.));
 }
 #[test]fn password_inv_gives_999_life_and_huge_hits(){
  let mut g=Game::new(false);g.setup(0,0,true,false);

  assert!(g.apply_password("inv"));assert_eq!(g.max_hp,MAX_LIFE);assert_eq!(g.hp,999.);
  assert_eq!(g.melee_damage(),999.,"le code INV donne des coups à 999");assert!(g.overpower());
  g.heal(500.);assert_eq!(g.hp,999.,"les soins ne dépassent jamais la vie maximale");
  // Un ennemi ordinaire tombe en un seul coup.
  g.enemies.truncate(1);g.enemies[0].pos=g.player+vec2(40.,0.);g.face=vec2(1.,0.);g.melee();assert!(g.enemies[0].hp<=0.);
  // Les sorts aussi frappent à 999.
  assert!(g.overpower());
 }
 #[test]fn save_rejects_impossible_life(){
  let good=Progress{stage:0,sector:0,charges:0,duel_two:false,demigod:false,divine_freeze:false,ice_granted:false,floor:1,hunter:12,max_hp:160.,justice:false};
  assert!(Progress::decode(&good.encode()).is_some());
  assert!(Progress::decode("7 0 0 0 0 0 0 0 1 12 1200").is_none());
  assert!(Progress::decode("7 0 0 0 0 0 0 0 1 500 300").is_none());
  assert!(Progress::decode("7 0 0 0 0 0 0 0 1 12 160").is_none(),"l'ancien format 7 est refusé");
 }
 #[test]fn save_roundtrip_and_reject_corruption(){let p=Progress{stage:12,sector:3,charges:7,duel_two:true,demigod:true,divine_freeze:true,ice_granted:true,floor:18,hunter:40,max_hp:300.,justice:true};let q=Progress::decode(&p.encode()).unwrap();assert_eq!((q.stage,q.sector,q.charges,q.duel_two,q.demigod,q.divine_freeze,q.ice_granted,q.floor,q.hunter),(12,3,7,true,true,true,true,18,40));assert_eq!(q.max_hp,300.);assert!(Progress::decode("7 99 4 0 0 0 0 0 0 1 0 100").is_none());assert!(Progress::decode("7 1 0 1000 0 0 0 0 0 1 0 100").is_none());assert!(Progress::decode("7 1 0 1 0 0 0 0 0 19 0 100").is_none());assert!(Progress::decode("3 1 0 1 0 0").is_none());}
 #[test]fn resizes_preserve_aspect_and_center(){for(w,h)in[(960.,640.),(1568.,854.),(600.,800.),(1920.,1080.)]{let r=display_rect(w,h);assert!(r.w>=W-0.001&&r.h>=H-0.001);assert!((r.x+r.w/2.-W/2.).abs()<0.001);assert!((r.y+r.h/2.-H/2.).abs()<0.001);assert!((w/r.w-h/r.h).abs()<0.001);}}
 #[test]fn rifts_have_safe_landing_spaces(){assert!(in_rift(375.));assert!(in_rift(620.));assert!(!in_rift(325.));assert!(!in_rift(565.));assert!(!in_rift(670.));}
 #[test]fn melee_damages_nearby_target(){let mut g=Game::new(false);g.player=vec2(100.,300.);g.face=vec2(1.,0.);g.enemies.push(Enemy::new(1,Kind::Slime,145.,300.));g.melee();assert_eq!(g.enemies[0].hp,Kind::Slime.hp()-27.);}
 #[test]fn convoy_timer_is_only_on_selected_rooms(){assert!(defense_time(6,2,0.)>0.);assert_eq!(defense_time(6,2,23.),0.);assert_eq!(defense_time(6,1,0.),0.);}
 #[test]fn gods_cannot_be_frozen_by_a_mortal(){
  for (stage,kind) in [(COMMUNIST,Kind::Communist),(CAPITALIST,Kind::Capitalist)]{
   let mut g=Game::new(false);g.setup(stage,3,true,false);g.enemies.truncate(1);assert_eq!(g.enemies[0].kind,kind);
   g.give_ice(1);assert_eq!(g.freeze_one(),IceOutcome::Forbidden);assert_eq!(g.stock.charges,1);
   g.demigod=true;assert_eq!(g.freeze_one(),IceOutcome::Frozen);assert_eq!(g.stock.charges,0);
  }
 }
 #[test]fn a_mortal_can_still_freeze_minions(){let mut g=Game::new(false);g.setup(0,0,true,false);g.enemies.truncate(1);g.give_ice(1);assert_eq!(g.freeze_one(),IceOutcome::Frozen);}
 #[test]fn descending_stairs_return_one_floor(){let mut g=Game::new(false);g.enter_backrooms();let level=(2..=maze::FLOORS).find(|l|maze::floor(*l).down.is_some()).unwrap();g.back_floor=level;g.player=maze::floor(level).down.unwrap().pos;assert!(g.climb(false));assert_eq!(g.back_floor,level-1);}
 #[test]fn pantheon_is_only_reached_from_the_last_floor(){
  let mut g=Game::new(false);g.enter_backrooms();assert_eq!(g.back_floor,1);
  g.player=maze::floor(1).stairs.pos;g.climb(true);assert_eq!(g.back_floor,2);
  g.player=maze::floor(2).stairs.pos;g.climb(true);assert_eq!(g.back_floor,3);
  g.player=maze::floor(3).stairs.pos;assert!(g.climb(true));assert_eq!(g.back_floor,4);
  let low=(2..=maze::FLOORS).find(|l|maze::floor(*l).down.is_some()).unwrap();g.back_floor=low;g.player=maze::floor(low).down.unwrap().pos;assert!(g.climb(false));assert_eq!(g.back_floor,low-1);
  g.back_floor=maze::FLOORS;g.player=maze::start_pos(maze::FLOORS);g.enter_pantheon();assert_eq!(g.mode,Mode::Pantheon);
 }
 #[test]fn demigods_no_longer_fall_into_the_backrooms(){
  let mut g=Game::new(false);g.setup(0,0,true,false);
  g.demigod=true;g.player=vec2(LEFT+2.,360.);
  // Un pas vers la bordure gauche ne déclenche plus l'entrée.
  for _ in 0..40{assert!(!g.walk(vec2(-1.,0.),0.033));}assert_eq!(g.mode,Mode::Playing);assert!(g.player.x>=LEFT);
  // Un mortel, lui, y tombe toujours.
  let mut m=Game::new(false);m.setup(0,0,true,false);m.player=vec2(LEFT+2.,360.);assert!(m.walk(vec2(-1.,0.),0.05));assert_eq!(m.mode,Mode::Backrooms);
 }
 #[test]fn password_ml_reaches_the_last_god_with_spells(){
  let mut g=Game::new(false);g.setup(0,0,true,false);
  assert!(!g.apply_password("ZZ"));assert_eq!(g.stage,0);
  assert!(g.apply_password(" ml "));assert_eq!(g.stage,COMMUNIST);assert_eq!(g.mode,Mode::Story);
  assert!(g.stage>=3&&g.stage>=12);
  assert_eq!(g.enemies.iter().filter(|e|e.kind==Kind::Communist).count(),0);
  // Le chapitre se parcourt normalement, et le boss du communisme s'y trouve.
  g.mode=Mode::Playing;g.setup(COMMUNIST,3,true,false);
  assert!(g.enemies.iter().any(|e|e.kind==Kind::Communist),"boss absent du dernier quartier");
  for sector in 0..SECTORS{g.setup(COMMUNIST,sector,true,false);if sector==3{assert!(g.enemies.iter().any(|e|e.kind==Kind::Communist));}else{assert!(!g.enemies.iter().any(|e|e.kind.boss()));}}
 }
 #[test]fn password_glace_gives_999_uses_and_survives_saving(){
  let mut g=Game::new(false);g.setup(0,0,true,false);
  assert!(g.apply_password("glace"));assert_eq!(g.stock.charges,999);
  // Les charges se consomment une par une, même sur un dieu si l'on est demi-dieu.
  g.divine_freeze=true;g.enemies.truncate(1);
  assert_eq!(g.freeze_one(),IceOutcome::Frozen);assert_eq!(g.stock.charges,998);
  // La sauvegarde garde la réserve exacte : reprendre ne redonne pas 999.
  assert_eq!(saved_progress(&g),998);
  let restored=Progress::decode(&saved_progress_parts(&g).encode()).unwrap();
  assert_eq!(restored.charges,998);
  // Un code inconnu ne change rien.
  let before=g.stock.charges;assert!(!g.apply_password("GLACEE"));assert_eq!(g.stock.charges,before);
 }
 #[test]fn the_divine_freeze_is_kept_in_the_save(){
  let mut g=Game::new(false);g.setup(0,0,true,false);g.enter_pantheon();g.accept_demigod();
  assert!(g.divine_freeze);assert_eq!(saved_progress(&g),g.stock.charges);
  let p=saved_progress_parts(&g);assert!(p.divine_freeze);
 }
 #[test]fn judgement_halves_the_undead_and_empowers_the_living(){
  let mut g=Game::new(false);g.setup(0,0,true,false);
  // Réservé au dieu de la justice.
  assert_eq!(g.judge_nearest(),JudgeOutcome::Forbidden);
  g.start_justice();g.mode=Mode::Playing;assert!(g.judgement);
  g.enemies=vec![Enemy::new(1,Kind::Skeleton,150.,360.)];
  let before=g.enemies[0].hp;
  assert_eq!(g.judge_nearest(),JudgeOutcome::Judged(true));
  assert_eq!(g.enemies[0].hp,before/2.,"une âme morte-vivante tombe à la moitié de sa vie");
  // Un seul jugement par quartier.
  g.judgement=false;assert_eq!(g.judge_nearest(),JudgeOutcome::Used);
 }
 #[test]fn judging_a_living_soul_doubles_its_abilities(){
  let mut g=Game::new(false);g.start_justice();g.mode=Mode::Playing;
  g.enemies=vec![Enemy::new(1,Kind::Slime,150.,360.)];
  let before=g.enemies[0].touch_multiplier();
  assert_eq!(g.judge_nearest(),JudgeOutcome::Judged(false));
  assert!(g.enemies[0].judged);assert_eq!(g.enemies[0].touch_multiplier(),before*2.);
  assert_eq!(g.enemies[0].hp,g.enemies[0].hp_max(),"la vie d'un vivant jugé ne change pas");
 }
 #[test]fn a_zombie_god_counts_as_undead(){
  let mut g=Game::new(false);g.start_justice();g.mode=Mode::Playing;
  g.enemies=vec![Enemy::new(1,Kind::Capitalist,200.,360.).zombified()];
  let before=g.enemies[0].hp;
  assert_eq!(g.judge_nearest(),JudgeOutcome::Judged(true));
  assert_eq!(g.enemies[0].hp,before/2.);
 }
 #[test]fn one_judgement_per_room(){
  let mut g=Game::new(false);g.start_justice();g.mode=Mode::Playing;
  assert!(g.judgement);g.advance_sector();assert!(g.judgement,"le jugement revient au quartier suivant");
  g.judgement=false;g.setup(0,1,true,false);assert!(g.judgement);
 }
 #[test]fn justice_cycle_blocks_ordinary_damage(){
  let mut g=Game::new(false);g.setup(0,0,true,false);g.hp=100.;
  g.damage_from(10.,false);assert_eq!(g.hp,90.,"cycle normal : un mortel blesse");
  g.start_justice();g.immune=0.;g.hp=g.max_hp;let start=g.hp;
  g.damage_from(30.,false);assert_eq!(g.hp,start,"un mortel ne peut plus blesser un dieu de la justice");
  g.immune=0.;g.damage_from(30.,true);assert!(g.hp<start,"un demi-dieu ou un zombie blesse toujours");
 }
 #[test]fn justice_cycle_spawns_demigods_and_zombie_gods(){
  let normal=spawn_enemies_for(0,0,false);assert!(normal.iter().all(|e|!e.demigod&&!e.zombie));
  let just=spawn_enemies_for(0,0,true);assert!(just.iter().any(|e|e.demigod),"aucun demi-dieu dans le second cycle");
  assert!(just.iter().filter(|e|e.demigod).all(|e|e.can_hurt_justice()));
  for (stage,kind) in [(COMMUNIST,Kind::Communist),(CAPITALIST,Kind::Capitalist)]{
   let gods=spawn_enemies_for(stage,3,true);
   let zombie=gods.iter().find(|e|e.zombie).expect("aucun dieu-zombie");
   assert_eq!(zombie.kind,kind);
   assert_eq!(zombie.hp_max(),kind_hp(&kind)*2.,"le dieu-zombie doit avoir deux fois plus de vie");
   assert_eq!(zombie.touch_multiplier(),2.,"le dieu-zombie frappe deux fois plus fort");
   assert!(zombie.can_hurt_justice());
   assert_eq!(gods.iter().filter(|e|e.kind.boss()&&!e.zombie).count(),0,"le dieu normal ne doit pas rester");
  }
 }
 #[test]fn justice_cycle_is_saved_and_restarts_the_adventure(){
  let mut g=Game::new(false);g.setup(CAPITALIST,3,true,false);g.hunter=42;g.max_hp=300.;
  g.start_justice();
  assert!(g.justice);assert_eq!((g.stage,g.sector),(0,0));assert_eq!(g.hunter,0);assert_eq!(g.max_hp,100.);
  let p=g.progress();assert!(p.justice);assert!(Progress::decode(&p.encode()).unwrap().justice);
  assert!(Progress::decode("8 0 0 0 0 0 0 0 1 0 100 1").is_some());
  assert!(Progress::decode("8 0 0 0 0 0 0 0 1 0 100 2").is_none());
 }
 #[test]
 fn boss_health_exceeds_minions(){assert!(Kind::Capitalist.hp()>Kind::Guard.hp());assert!(Kind::Communist.hp()>Kind::Skeleton.hp());}
}
