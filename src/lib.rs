extern crate rand;
extern crate serde_json;

extern crate brdgme_game;
extern crate brdgme_cmd;

use rand::{Rng, ThreadRng};

use brdgme_game::Gamer;
use brdgme_game::bot::Botter;
use brdgme_game::command;
use brdgme_cmd::bot_cli;

use std::i32;
use std::io::{Read, Write};

pub struct RandBot;

fn bounded_i32(v: i32, min: i32, max: i32) -> i32 {
    assert!(min <= max);
    let mut v = v;
    let range_size = max - min + 1;
    if v < min {
        v += range_size * ((min - v) / range_size + 1);
    }
    min + (v - min) % range_size
}

fn spec_entry(specs: &command::Specs, spec: &command::Spec, rng: &mut ThreadRng) -> Vec<String> {
    match spec.kind {
        command::Kind::Int { min, max } => {
            vec![format!("{}",
                         bounded_i32(rng.gen(), min.unwrap_or(i32::MIN), max.unwrap_or(i32::MAX)))]
        }
        command::Kind::Token(ref token) => vec![token.to_owned()],
        command::Kind::Ref(ref cmd) => spec_entry(specs, &specs.specs[cmd], rng),
        command::Kind::Enum(ref values) => vec![rng.choose(values).unwrap().to_owned()],
        command::Kind::OneOf(ref options) => spec_entry(specs, rng.choose(options).unwrap(), rng),
        command::Kind::Chain(ref chain) => {
            chain
                .iter()
                .flat_map(|c| spec_entry(specs, c, rng))
                .collect()
        }
    }
}

fn commands(command_spec: &command::Specs) -> Vec<String> {
    let mut rng = rand::thread_rng();
    vec![spec_entry(command_spec,
                    &command_spec.specs[&command_spec.entry],
                    &mut rng)
                 .join(" ")]
}

// / Most bots just want to use `brdgme_cmd::bot_cli`, however because RandBot
// doesn't care about game / state, we implement a more simplified version of
// the CLI here. This allows the bot to be used / with arbitrary games as long
// as the command spec is generated.
pub fn cli<I, O>(input: I, output: &mut O)
    where I: Read,
          O: Write
{
    let request = serde_json::from_reader::<_, bot_cli::Request>(input).unwrap();
    writeln!(output,
             "{}",
             serde_json::to_string(&commands(&request.command_spec)).unwrap())
            .unwrap();
}

impl<T: Gamer> Botter<T> for RandBot {
    fn commands(&mut self,
                _player: usize,
                _pub_state: &T::PubState,
                _players: &[String],
                command_spec: &command::Specs)
                -> Vec<String> {
        commands(command_spec)
    }
}
