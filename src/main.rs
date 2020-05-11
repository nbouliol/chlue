use clap::{App, Arg};
// use huelib::resource::scene::Modifier;

use huelib::resource::{light, scene, Alert, Modifier, ModifierType, Scene};
use huelib::{bridge, Bridge};
use std::io::{self, stdin, stdout, Read, Write};
use termion::{
  event::Key,
  input::TermRead,
  raw::IntoRawMode,
  {clear, color, cursor, style},
};

fn main() {
  // Discover bridges in the local network and save the first IP address as `bridge_ip`.
  let bridge_ip = bridge::discover().unwrap().pop().unwrap();
  let mut stdin = io::stdin().bytes();
  // Register a new user.
  // let user = bridge::register_user(bridge_ip, "chlue", false).unwrap();
  // todo : store user locally
  let user = bridge::User {
    name: String::from("ikOhNVHbOpQHWOjkig2yPjI5E83hZheKEHn3dlQS"),
    clientkey: None,
  };
  let bridge = Bridge::new(bridge_ip, &user.name);

  let scenes = bridge.get_all_scenes().unwrap();

  let group_scenes = get_group_scene_for_user(&bridge, &scenes);

  let matches = App::new("My Super Program")
    .version("1.0")
    .author("Kevin K. <kbknapp@gmail.com>")
    .about("Does awesome things")
    .arg(
      Arg::with_name("list")
        .short("l")
        .long("list")
        .help("List all rooms and scenes"),
    )
    .arg(
      Arg::with_name("turn")
        .short("t")
        .help("Turn your scenes on/off"),
    )
    .get_matches();
  if matches.is_present("list") {
    list_group_scenes(&group_scenes);
  }
  let mut input = String::new();
  if matches.is_present("turn") {
    let mut lines: Vec<String> = group_scenes.iter().map(|x| x.group.name.clone()).collect();
    lines.sort();
    let selected_group = select_group("Choose room", &group_scenes);
    println!("SELECTED : {} ", selected_group.group.name);

    let select_scene = select_scene(
      &format!("Choose scene in {}", selected_group.group.name),
      &selected_group.scenes.as_ref().unwrap(),
    );
    println!(
      "selected scene : {} -> {}",
      select_scene.name, select_scene.id
    );
  }
}

fn get_group_scene_for_user<'a>(
  bridge: &Bridge,
  scenes: &'a Vec<huelib::resource::scene::Scene>,
) -> Vec<GroupScene<'a>> {
  let mut group_scenes: Vec<GroupScene> = Vec::new();
  let groups = bridge.get_all_groups().unwrap();

  for group in groups {
    let mut group_scene = GroupScene {
      group: group,
      scenes: None,
    };

    group_scene = group_scene.add_scenes(&scenes);
    group_scenes.push(group_scene);
  }

  group_scenes
}

fn list_group_scenes(group_scenes: &Vec<GroupScene<'_>>) {
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

// macro ?!
fn select_group<'a>(prompt: &str, lines: &'a Vec<GroupScene<'a>>) -> &'a GroupScene<'a> {
  let stdin = stdin();
  let mut stdout = stdout().into_raw_mode().unwrap();
  write!(
    stdout,
    "{}{}[?] {}{}\n",
    cursor::Hide,
    color::Fg(color::Green),
    style::Reset,
    prompt
  )
  .unwrap();

  for _ in 0..lines.len() {
    write!(stdout, "\n").unwrap();
  }

  let mut cur: usize = 0;

  let mut input = stdin.keys();

  loop {
    print!("{}", cursor::Up(lines.len() as u16));

    for (i, s) in lines.iter().enumerate() {
      write!(stdout, "\n\r{}", clear::CurrentLine).unwrap();

      if cur == i {
        write!(
          stdout,
          "{}  > {}{}",
          style::Bold,
          s.group.name,
          style::Reset
        )
        .unwrap();
      } else {
        write!(stdout, "    {}", s.group.name).unwrap();
      }
    }

    stdout.lock().flush().unwrap();

    let next = input.next().ok_or_else(|| 0).unwrap();

    match next.unwrap() {
      Key::Char('\n') => {
        // Enter
        break;
      }
      Key::Up if cur != 0 => {
        cur -= 1;
      }
      Key::Down if cur != lines.len() - 1 => {
        cur += 1;
      }
      Key::Ctrl('c') => {
        write!(stdout, "\n\r{}", cursor::Show).unwrap();
        // return Err(Error::UserAborted);
      }
      _ => {
        // pass
      }
    }
  }
  write!(stdout, "\n\r{}", cursor::Show).unwrap();

  lines.get(cur).unwrap()
}

// macro ?!
fn select_scene<'a>(prompt: &str, lines: &'a Vec<&'a Scene>) -> &'a Scene {
  let stdin = stdin();
  let mut stdout = stdout().into_raw_mode().unwrap();
  write!(
    stdout,
    "{}{}[?] {}{}\n",
    cursor::Hide,
    color::Fg(color::Green),
    style::Reset,
    prompt
  )
  .unwrap();

  for _ in 0..lines.len() {
    write!(stdout, "\n").unwrap();
  }

  let mut cur: usize = 0;

  let mut input = stdin.keys();

  loop {
    print!("{}", cursor::Up(lines.len() as u16));

    for (i, s) in lines.iter().enumerate() {
      write!(stdout, "\n\r{}", clear::CurrentLine).unwrap();

      if cur == i {
        write!(stdout, "{}  > {}{}", style::Bold, s.name, style::Reset).unwrap();
      } else {
        write!(stdout, "    {}", s.name).unwrap();
      }
    }

    stdout.lock().flush().unwrap();

    let next = input.next().ok_or_else(|| 0).unwrap();

    match next.unwrap() {
      Key::Char('\n') => {
        // Enter
        break;
      }
      Key::Up if cur != 0 => {
        cur -= 1;
      }
      Key::Down if cur != lines.len() - 1 => {
        cur += 1;
      }
      Key::Ctrl('c') => {
        write!(stdout, "\n\r{}", cursor::Show).unwrap();
        // return Err(Error::UserAborted);
      }
      _ => {
        // pass
      }
    }
  }
  write!(stdout, "\n\r{}", cursor::Show).unwrap();

  lines.get(cur).unwrap()
}

#[derive(Debug, Clone)]
pub struct GroupScene<'a> {
  group: huelib::resource::group::Group,
  scenes: Option<Vec<&'a huelib::resource::scene::Scene>>,
}

impl<'a> GroupScene<'a> {
  pub fn add_scenes(self, scenes: &'a Vec<huelib::resource::scene::Scene>) -> Self {
    let s: Vec<_> = scenes
      .iter()
      .filter(|x| match &x.group {
        Some(id) => id.clone() == self.group.id,
        None => false,
      })
      .collect();

    GroupScene {
      scenes: if s.len() != 0 { Some(s) } else { None },
      ..self
    }
  }
}
