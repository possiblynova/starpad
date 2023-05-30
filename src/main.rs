use {
    crate::{
        bindings::{Bind, Binds},
        interface::{prompt, Led, Pad, PadInterface},
    },
    midir::{MidiInput, MidiOutput},
    std::{
        collections::HashMap,
        fs::File,
        io::{stdin, Write},
    },
};

mod bindings;
mod dsl;
mod interface;

fn main() {
    let mut binds_path = std::env::current_dir().expect("cannot obtain current directory");
    binds_path.push("starpad.yaml");

    if !binds_path.exists() {
        writeln!(
            File::create(&binds_path).expect("cannot create new binds file"),
            "{}",
            include_str!("template.yaml")
        )
        .expect("cannot write new binds file");

        println!("A binds file has been created for you, instructions are contained within it.");
        return;
    }

    let binds_file = File::open(&binds_path).expect("cannot open binds file");
    let binds: HashMap<Pad, Bind> =
        serde_yaml::from_reader(binds_file).expect("invalid binds file");

    let input = prompt(
        "Input Port",
        MidiInput::new("StarPad Port Input").expect("cannot create input client"),
    );
    let mut led = Led::new(prompt(
        "Output Port",
        MidiOutput::new("StarPad Port Output").expect("cannot create output client"),
    ));

    led.clear();

    for (pad, bind) in binds.iter() {
        led.set(*pad, bind.inactive);
    }

    Binds::new(binds, led).run(input);

    let mut lines = stdin().lines();

    while let Some(Ok(line)) = lines.next() {
        if line.trim().eq_ignore_ascii_case("quit") {
            break;
        }
    }
}
