#![feature(box_patterns)]

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

fn spec_to_command(spec: &command::Spec, rng: &mut ThreadRng) -> Vec<String> {
    match *spec {
        command::Spec::Int { min, max } => {
            vec![format!("{}",
                         bounded_i32(rng.gen(), min.unwrap_or(i32::MIN), max.unwrap_or(i32::MAX)))]
        }
        command::Spec::Token(ref token) => vec![token.to_owned()],
        command::Spec::Enum(ref values) => vec![rng.choose(values).unwrap().to_owned()],
        command::Spec::OneOf(ref options) => spec_to_command(rng.choose(options).unwrap(), rng),
        command::Spec::Chain(ref chain) => {
            chain
                .iter()
                .flat_map(|c| spec_to_command(c, rng))
                .collect()
        }
        command::Spec::Opt(box ref spec) => {
            if rng.gen() {
                spec_to_command(spec, rng)
            } else {
                vec![]
            }
        }
        command::Spec::Many {
            box ref spec,
            min,
            max,
            ref delim,
        } => {
            let min = min.unwrap_or(0) as i32;
            let max = max.unwrap_or(3) as i32;
            let n = bounded_i32(rng.gen(), min, max);
            let mut parts: Vec<String> = vec![];
            for i in 0..n {
                if i != 0 {
                    parts.push(delim.to_owned());
                }
                parts.extend(spec_to_command(spec, rng));
            }
            parts
        }
        command::Spec::Doc { box ref spec, .. } => spec_to_command(spec, rng),
    }
}

fn commands(command_spec: &command::Spec) -> Vec<String> {
    let mut rng = rand::thread_rng();
    vec![spec_to_command(command_spec, &mut rng).join(" ")]
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
                command_spec: &command::Spec)
                -> Vec<String> {
        commands(command_spec)
    }
}
