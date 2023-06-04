use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};
use std::str::{self, FromStr};

use clap::{self, Parser};

/// tuple struct with two string slices with static lifetime (aka: as long as the program runs)
#[derive(Debug)]
struct Letter<'a>(&'a str, &'a str);

impl<'a> Letter<'a> {
    const A: Self = Self("a", ".-");
    const B: Self = Self("b", "-...");
    const C: Self = Self("c", "-.-.");
    const D: Self = Self("d", "-..");
    const E: Self = Self("e", ".");
    const F: Self = Self("f", "..-.");
    const G: Self = Self("g", "--.");
    const H: Self = Self("h", "....");
    const I: Self = Self("i", "..");
    const J: Self = Self("j", ".---");
    const K: Self = Self("k", "-.-");
    const L: Self = Self("l", ".-..");
    const M: Self = Self("m", "--");
    const N: Self = Self("n", "-.");
    const O: Self = Self("o", "---");
    const P: Self = Self("p", ".--.");
    const Q: Self = Self("q", "--.-");
    const R: Self = Self("r", ".-.");
    const S: Self = Self("s", "...");
    const T: Self = Self("t", "-");
    const U: Self = Self("u", "..-");
    const V: Self = Self("v", "...-");
    const W: Self = Self("w", ".--");
    const X: Self = Self("x", "-..-");
    const Y: Self = Self("y", "-.--");
    const Z: Self = Self("z", "--..");
    const ONE: Self = Self("1", ".----");
    const TWO: Self = Self("2", "..---");
    const THREE: Self = Self("3", "...--");
    const FOUR: Self = Self("4", "....-");
    const FIVE: Self = Self("5", ".....");
    const SIX: Self = Self("6", "-....");
    const SEVEN: Self = Self("7", "--...");
    const EIGHT: Self = Self("8", "---..");
    const NINE: Self = Self("9", "----.");
    const ZERO: Self = Self("0", "-----");
    const SPACE: Self = Self(" ", "/");

    pub fn concat_morse(args: Vec<Letter<'_>>) -> String {
        let mut output = String::from("");
        if let Some(letter) = args.first() {
            let Letter(_, morse) = letter;
            output = String::from(*morse);
        }
        for letter in args.iter().skip(1) {
            let Letter(_, morse) = letter;
            output = output + " " + morse;
        }
        output
    }
    pub fn concat_text(args: Vec<Letter<'_>>) -> String {
        let mut output = String::from("");
        if let Some(letter) = args.first() {
            let Letter(text, _) = letter;
            output = String::from(*text);
        }
        for letter in args.iter().skip(1) {
            let Letter(text, _) = letter;
            output = output + text;
        }
        output
    }
}

impl FromStr for Letter<'_> {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "a" | ".-" => Ok(Letter::A),
            "b" | "-..." => Ok(Letter::B),
            "c" | "-.-." => Ok(Letter::C),
            "d" | "-.." => Ok(Letter::D),
            "e" | "." => Ok(Letter::E),
            "f" | "..-." => Ok(Letter::F),
            "g" | "--." => Ok(Letter::G),
            "h" | "...." => Ok(Letter::H),
            "i" | ".." => Ok(Letter::I),
            "j" | ".---" => Ok(Letter::J),
            "k" | "-.-" => Ok(Letter::K),
            "l" | ".-.." => Ok(Letter::L),
            "m" | "--" => Ok(Letter::M),
            "n" | "-." => Ok(Letter::N),
            "o" | "---" => Ok(Letter::O),
            "p" | ".--." => Ok(Letter::P),
            "q" | "--.-" => Ok(Letter::Q),
            "r" | ".-." => Ok(Letter::R),
            "s" | "..." => Ok(Letter::S),
            "t" | "-" => Ok(Letter::T),
            "u" | "..-" => Ok(Letter::U),
            "v" | "...-" => Ok(Letter::V),
            "w" | ".--" => Ok(Letter::W),
            "x" | "-..-" => Ok(Letter::X),
            "y" | "-.--" => Ok(Letter::Y),
            "z" | "--.." => Ok(Letter::Z),
            "1" | ".----" => Ok(Letter::ONE),
            "2" | "..---" => Ok(Letter::TWO),
            "3" | "...--" => Ok(Letter::THREE),
            "4" | "....-" => Ok(Letter::FOUR),
            "5" | "....." => Ok(Letter::FIVE),
            "6" | "-...." => Ok(Letter::SIX),
            "7" | "--..." => Ok(Letter::SEVEN),
            "8" | "---.." => Ok(Letter::EIGHT),
            "9" | "----." => Ok(Letter::NINE),
            "0" | "-----" => Ok(Letter::ZERO),
            " " | "/" => Ok(Letter::SPACE),
            _ => Err(format!(
                "No representation found for the string: {}",
                s.to_string()
            )),
        }
    }
}

impl PartialEq for Letter<'_> {
    fn eq(&self, other: &Self) -> bool {
        let Self(human1, morse1) = self;
        let Self(human2, morse2) = other;
        human1 == human2 && morse1 == morse2
    }
}

#[derive(Debug)]
enum MorseTraductionType {
    Text,
    Audio,
}

impl FromStr for MorseTraductionType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "text" => Ok(MorseTraductionType::Text),
            "audio" => Ok(MorseTraductionType::Audio),
            _ => Err(format!("Type of output not found: {}", s.to_string())),
        }
    }
}

#[derive(Debug)]
enum MorseCommand {
    Encode,
    Decode,
}

impl FromStr for MorseCommand {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "e" | "encode" => Ok(MorseCommand::Encode),
            "d" | "decode" => Ok(MorseCommand::Decode),
            _ => Err(format!("Morse command not found: {}", s.to_string())),
        }
    }
}

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct MorseArgs {
    /// Morse command:
    /// -encode
    /// -decode
    pub morse_command: MorseCommand,

    /// Type of traduction from human readable text to morse:
    /// -text
    /// -audio
    pub traduction_type: MorseTraductionType,

    /// Name of the file to read, if the value is "-" read from stdin
    #[clap(short, long)]
    in_file: String,

    /// Name of the file to read, if the value is "-" write to stdout
    #[clap(short, long, default_value = "-")]
    out_file: String,
}

trait MorseTranslator {
    fn traduce(&mut self, command: MorseCommand) -> io::Result<()>;
}

trait MorseEncoder<T> {
    fn encode(raw_data: T) -> T;
}

trait MorseDecoder<T> {
    fn decode(raw_data: T) -> T;
}

fn get_reader(arg: &str) -> Box<dyn BufRead> {
    match arg {
        "-" => Box::new(io::stdin().lock()),
        x if x.is_empty() => Box::new(io::stdin().lock()),
        file_name => Box::new(BufReader::new(
            OpenOptions::new().read(true).open(file_name).unwrap(),
        )),
    }
}

fn get_writer(arg: &str) -> Box<dyn Write> {
    match arg {
        "-" => Box::new(io::stdout().lock()),
        x if x.is_empty() => Box::new(io::stdout().lock()),
        file_name => Box::new(
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(file_name)
                .unwrap(),
        ),
    }
}

struct TextMorseTranslator {
    // idea, create struct AudioMorseTranslation for audio implementation
    // create struct OptionMorseTranslation with functions:
    // - in_file(Option<&str>)  -> using get_reader
    // - out_file(Option<&str>) -> using get_writer
    // - traduction_type(MorseTraductionType)
    // - traduction_options(MorseCommand)
    // this patter will create and use a TextMorseTranslator
    // or an AudioMorseTranslation trasparently
    input_filename: String,
    output_filename: String,
}

impl MorseTranslator for TextMorseTranslator {
    fn traduce(&mut self, command: MorseCommand) -> io::Result<()> {
        let traduce_cmd = match command {
            MorseCommand::Encode => Self::encode,
            MorseCommand::Decode => Self::decode,
        };

        let traduced_lines = get_reader(&self.input_filename)
            .lines()
            .map(|line| traduce_cmd(line.unwrap()));

        let mut output = get_writer(&self.output_filename);
        for line in traduced_lines {
            output.write((line + "\n").as_bytes())?;
        }
        output.flush()
    }
}

impl TextMorseTranslator {
    fn new() -> Self {
        TextMorseTranslator {
            input_filename: "".to_string(),
            output_filename: "".to_string(),
        }
    }

    fn in_file(&mut self, input_filename: &str) -> &mut Self {
        self.input_filename = input_filename.to_owned();
        self
    }

    fn out_file(&mut self, output_filename: &str) -> &mut Self {
        self.output_filename = output_filename.to_owned();
        self
    }
}

impl MorseEncoder<String> for TextMorseTranslator {
    fn encode(line: String) -> String {
        Letter::concat_morse(
            line.bytes()
                .map(
                    |byte| match Letter::from_str(str::from_utf8(&[byte]).unwrap()) {
                        Ok(letter) => letter,
                        Err(err) => panic!("Character not supported {:?}", err),
                    },
                )
                .collect::<Vec<Letter<'_>>>(),
        )
    }
}

impl MorseDecoder<String> for TextMorseTranslator {
    fn decode(line: String) -> String {
        Letter::concat_text(
            line.split_whitespace()
                .map(|morse_letter| match Letter::from_str(morse_letter) {
                    Ok(letter) => letter,
                    Err(err) => panic!("Character not supported {:?}", err),
                })
                .collect::<Vec<Letter<'_>>>(),
        )
    }
}

fn main() {
    let args = MorseArgs::parse();
    TextMorseTranslator::new()
        .out_file(&args.out_file)
        .in_file(&args.in_file)
        .traduce(args.morse_command)
        .unwrap();
}
