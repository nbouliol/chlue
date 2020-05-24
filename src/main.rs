use huelib::resource::{group, light, Modifier};
use huelib::{bridge, Bridge};
use std::env;
use std::fmt::Display;
use std::io::{stdin, stdout, Write};
use structopt::StructOpt;
use termion::{
  event::Key,
  input::TermRead,
  raw::IntoRawMode,
  {clear, color, cursor, style},
};
use thiserror::Error;

type Result<T> = std::result::Result<T, ChlueError>;

fn main() -> Result<()> {
  let bridge_ip = bridge::discover()?.pop().unwrap();

  let opt = Opt::from_args();

  let user: bridge::User;

  if let Some(username) = opt.username {
    user = bridge::User {
      name: username,
      clientkey: None,
    };
  } else {
    user = bridge::register_user(bridge_ip, "chlue", false)?;
    println!("User has been created. username : {}", user.name);
  }

  let bridge = Bridge::new(bridge_ip, &user.name);

  let mut scenes = bridge.get_all_scenes()?;
  scenes.sort_by(|a, b| a.name.cmp(&b.name));

  let group_scenes = get_group_scene_for_user(&bridge, &scenes)?;

  if opt.list_scenes {
    list_group_scenes(&group_scenes);
    return Ok(());
  }

  if opt.scene {
    let mut lines: Vec<String> = group_scenes.iter().map(|x| x.group.name.clone()).collect();
    lines.sort();
    let selected_group = select(
      "Choose room",
      &group_scenes,
      |x| x.group.name.to_owned(),
      Select::Vertical,
    )?;
    println!("SELECTED : {} ", selected_group.group.name);

    let selected_scene = select(
      &format!("Choose scene in {}", selected_group.group.name),
      &selected_group.scenes.as_ref().unwrap(),
      |x| x.name.to_string(),
      Select::Vertical,
    )?;

    let choices = vec!["on", "off"];
    let on = select("Set the scene", &choices, |x| x.clone(), Select::Horizontal)?;

    if *on == "on" {
      let modifier = group::StateModifier::new()
        .scene(&selected_scene.id)
        .on(true);
      bridge.set_group_state(&selected_group.group.id, &modifier)?;
    } else if let Some(lights) = &selected_scene.lights {
      for light in lights {
        let modifier = light::StateModifier::new().on(false);

        bridge.set_light_state(light, &modifier)?;
      }
    }
  }

  if opt.light {
    let mut lights = bridge.get_all_lights()?;
    lights.sort_by(|a, b| a.name.cmp(&b.name));
    let selected_light = select(
      "Pick a light",
      &lights,
      |x| x.name.clone(),
      Select::Vertical,
    )?;

    let choices = vec!["on", "off"];
    let on = select("Set the light", &choices, |x| x.clone(), Select::Horizontal)?;

    let modifier = light::StateModifier::new().on(*on == "on");

    bridge.set_light_state(&selected_light.id, &modifier)?;
  }

  Ok(())
}

fn get_group_scene_for_user<'a>(
  bridge: &Bridge,
  scenes: &'a [huelib::resource::scene::Scene],
) -> Result<Vec<GroupScene<'a>>> {
  let mut group_scenes: Vec<GroupScene> = Vec::new();
  let mut groups = bridge.get_all_groups()?;
  groups.sort_by(|a, b| a.name.cmp(&b.name));

  for group in groups {
    let mut group_scene = GroupScene {
      group,
      scenes: None,
    };

    group_scene = group_scene.add_scenes(scenes);
    group_scenes.push(group_scene);
  }

  Ok(group_scenes)
}

fn list_group_scenes(group_scenes: &[GroupScene<'_>]) {
  for gs in group_scenes {
    println!("{} :", gs.group.name);
    if let Some(scenes) = &gs.scenes {
      for i in 0..scenes.len() {
        println!("\r > {}", scenes[i].name);
      }
    } else {
      println!("\r > No scene detected for this room");
    }
  }
}

fn select<'a, T, F, D>(prompt: &str, lines: &'a [T], closur: F, direction: Select) -> Result<&'a T>
where
  F: Fn(&T) -> D,
  D: Display,
{
  let stdin = stdin();
  let mut stdout = stdout().into_raw_mode()?;
  let mut cur: usize = 0;
  let mut len: u16 = 0;

  writeln!(
    stdout,
    "{}{}[?] {}{}",
    cursor::Hide,
    color::Fg(color::Green),
    style::Reset,
    prompt
  )?;

  if direction == Select::Vertical {
    for _ in 0..lines.len() {
      writeln!(stdout)?;
    }
  }

  let mut input = stdin.keys();

  loop {
    if direction == Select::Vertical {
      len = lines.len() as u16;
    }
    print!("{}", cursor::Up(len));

    if direction == Select::Horizontal {
      write!(stdout, "\n\r{}", clear::CurrentLine)?;
    }
    for (i, s) in lines.iter().enumerate() {
      if direction == Select::Vertical {
        write!(stdout, "\n\r{}", clear::CurrentLine)?;
      }
      if cur == i {
        write!(stdout, "{}  > {}{}", style::Bold, closur(s), style::Reset)?;
      } else {
        write!(stdout, "    {}", closur(s))?;
      }
    }

    stdout.lock().flush()?;

    let next = input.next().ok_or_else(|| 0).unwrap();

    match next.unwrap() {
      Key::Char('\n') => {
        // Enter
        break;
      }
      Key::Up | Key::Left if cur != 0 => {
        cur -= 1;
      }
      Key::Down | Key::Right if cur != lines.len() - 1 => {
        cur += 1;
      }
      Key::Ctrl('c') => {
        write!(stdout, "\n\r{}", cursor::Show)?;
        return Err(ChlueError::UserAborted);
      }
      _ => {
        // pass
      }
    }
  }
  write!(stdout, "\n\r{}", cursor::Show)?;

  Ok(lines.get(cur).unwrap())
}

#[derive(Debug, Clone)]
pub struct GroupScene<'a> {
  group: huelib::resource::group::Group,
  scenes: Option<Vec<&'a huelib::resource::scene::Scene>>,
}

impl<'a> GroupScene<'a> {
  pub fn add_scenes(self, scenes: &'a [huelib::resource::scene::Scene]) -> Self {
    let s: Vec<_> = scenes
      .iter()
      .filter(|x| match &x.group {
        Some(id) => id.clone() == self.group.id,
        None => false,
      })
      .collect();

    GroupScene {
      scenes: if !s.is_empty() { Some(s) } else { None },
      ..self
    }
  }
}

#[derive(StructOpt, Debug)]
#[structopt(name = env!("CARGO_PKG_NAME"), about = env!("CARGO_PKG_DESCRIPTION"))]
struct Opt {
  #[structopt(
    long,
    conflicts_with = "scene",
    conflicts_with = "light",
    help = "prints all the scenes by group"
  )]
  list_scenes: bool,

  #[structopt(
    short,
    long,
    conflicts_with = "list_scenes",
    conflicts_with = "light",
    help = "turn a scene on / off"
  )]
  scene: bool,

  #[structopt(
    short,
    long,
    conflicts_with = "list_scenes",
    conflicts_with = "scene",
    help = "turn a light on / off"
  )]
  light: bool,

  #[structopt(
    short,
    long,
    help = "bridge username, if not supplied you have to click on the bridge and a new user will be created and outputed"
  )]
  username: Option<String>,
}

#[derive(Debug, Error)]
enum ChlueError {
  #[error(transparent)]
  IoError(#[from] std::io::Error),

  #[error(transparent)]
  HuelibError(#[from] huelib::Error),

  #[error(transparent)]
  EnvError(#[from] std::env::VarError),

  #[error("Aborted by user")]
  UserAborted,
}

#[derive(PartialEq, Debug)]
enum Select {
  Vertical,
  Horizontal,
}
