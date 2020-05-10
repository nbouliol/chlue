use clap::{App, Arg};
// use huelib::resource::scene::Modifier;

use huelib::resource::{light, scene, Alert, Modifier, ModifierType};
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
  // if let Some(_) = matches.value_of("list") {
  if matches.is_present("list") {
    list_group_scenes(&group_scenes);
  }
  let mut input = String::new();
  if matches.is_present("turn") {
    for gs in &group_scenes {
      println!("\r {}) {}", gs.group.id, gs.group.name);
    }
    println!("Pick a room");
    let c = char::from(stdin.next().unwrap().unwrap());

    if let Some(group) = group_scenes.iter().find(|&x| x.group.id == c.to_string()) {
      // todo : create a print_scenes_for_group function
      println!("{}", &group.group.name);

      println!("{:?}", group.group.lights);
      if let Some(scenes) = &group.scenes {
        for i in 0..scenes.len() {
          println!("\r > {}) {}", scenes[i].id, scenes[i].name);
        }
        println!("Pick a scene");
        let mut buffer = String::new();
        let stdin = io::stdin();
        let mut handle = stdin.lock();

        // handle.read_to_string(&mut buffer).unwrap();
        // println!("buffer : {}", buffer);

        buffer = "ZHEoPT0jtxxOTaK".to_string(); // todo : remove
        if let Some(scene) = group
          .scenes
          .as_ref()
          .unwrap()
          .iter()
          .find(|&&x| x.id == buffer)
        {
          // bridge.set_scene(scene.id, Modifier {});
          // let modififer = huelib::resource::light::StateModifier::new().on(true);
          // let modififer = huelib::resource::scene::LightStateModifier::new().on(true);
          // ZHEoPT0jtxxOTaK

          // let light_modifier = light::StateModifier::new()
          //   .on(true)
          //   .saturation(ModifierType::Override, 10)
          //   .alert(Alert::Select)
          //   .brightness(ModifierType::Decrement, 40);

          // // Modify the attributes declared in `light_modifier` on the light with the id 1.
          // let response = bridge.set_light_state("1", &light_modifier).unwrap();
          // println!("{:?}", response);

          // let mut modifier = scene::Modifier::new(); //.on(true);
          //                                            // modifier.
          // println!("{:?}", modifier);
          // bridge
          //   .set_scene(scene.id.clone(), &modifier)
          //   .expect("failed to change state");
        } else {
          println!("No scend found with id : {}", buffer);
        }
      } else {
        println!("\r > No scene detected for this room");
      }
    } else {
      println!("Unknown room");
    }
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

fn select(lines: Vec<String>) {
  let stdin = stdin();
  let mut stdout = stdout().into_raw_mode().unwrap();
  write!(
    stdout,
    "{}{}[?] {}{}\n",
    cursor::Hide,
    color::Fg(color::Green),
    style::Reset,
    "Choose 1"
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
        write!(stdout, "{}  > {}{}", style::Bold, s, style::Reset).unwrap();
      } else {
        write!(stdout, "    {}", s).unwrap();
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
  write!(stdout, "\n\r{}", cursor::Show);
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
