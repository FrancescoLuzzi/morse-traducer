use crate::parser::{MorseCommand, MorseTraductionType};
use crate::polyphonia::SAMPLE_RATE;
use crate::wav::wav_writer::{WavBuilder, WavOutBuffer};
use crate::Letter;
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use std::str::{self, FromStr};

pub trait MorseTranslator<T, W, R> {
    fn translate(&mut self, command: MorseCommand) -> Result<R, Box<dyn Error>>;

    fn translate_to_text(&mut self, command: MorseCommand) -> Result<R, Box<dyn Error>>;

    fn translate_to_audio(&mut self, command: MorseCommand) -> Result<R, Box<dyn Error>>;

    fn encode(raw_data: T) -> W;

    fn decode(raw_data: T) -> W;
}

pub struct StreamedMorseTranslator<T: WavOutBuffer> {
    // idea, create struct AudioMorseTranslation for audio implementation
    // create struct MorseTranslation with functions:
    // - in_file(&str)  -> using get_reader
    // - out_file(&str) -> using get_writer
    // - traduction_type(MorseTraductionType)
    // - traduction_options(MorseCommand)
    // this patter will create and use a StreamedMorseTranslator
    // or an AudioMorseTranslation trasparently
    input_stream: Vec<String>,
    pub output_stream: Rc<RefCell<T>>,
    pub traduction_type: MorseTraductionType,
}

impl<'l, T: WavOutBuffer> MorseTranslator<&str, Vec<Letter<'l>>, ()>
    for StreamedMorseTranslator<T>
{
    fn translate(&mut self, command: MorseCommand) -> Result<(), Box<dyn Error>> {
        match self.traduction_type {
            MorseTraductionType::Text => self.translate_to_text(command),
            MorseTraductionType::Audio => self.translate_to_audio(command),
        }
    }

    fn translate_to_audio(&mut self, command: MorseCommand) -> Result<(), Box<dyn Error>> {
        let read_cmd = match command {
            MorseCommand::Encode => Self::encode,
            MorseCommand::Decode => Self::decode,
        };

        let translated_lines = self.input_stream.iter().flat_map(|line| read_cmd(line));
        let mut output = self.output_stream.as_ref().borrow_mut();
        let wav = WavBuilder::new()
            .sample_rate(SAMPLE_RATE)
            .set_output(&mut *output);
        let mut wav = wav.init()?;
        wav.write_half_words(&Letter::concat_audio(translated_lines))?;
        // Letter::concat_audio(translated_lines),
        wav.close()?;
        Ok(())
    }

    fn translate_to_text(&mut self, command: MorseCommand) -> Result<(), Box<dyn Error>> {
        let read_cmd = match command {
            MorseCommand::Encode => Self::encode,
            MorseCommand::Decode => Self::decode,
        };

        let translate_cmd = match command {
            MorseCommand::Encode => Letter::concat_morse,
            MorseCommand::Decode => Letter::concat_text,
        };

        let translated_lines = self.input_stream.iter().map(|line| read_cmd(line));

        let mut output = self.output_stream.as_ref().borrow_mut();
        let last_index = translated_lines.len() - 1;
        for (i, line) in translated_lines.map(translate_cmd).enumerate() {
            output.write_all(&line)?;
            if i != last_index {
                output.write_all(b"\n")?;
            }
        }
        output.flush()?;
        Ok(())
    }

    fn encode(line: &str) -> Vec<Letter<'l>> {
        line.bytes()
            .map(
                |byte| match Letter::from_str(str::from_utf8(&[byte]).unwrap()) {
                    Ok(letter) => letter,
                    Err(err) => panic!("Character not supported {:?}", err),
                },
            )
            .collect::<Vec<Letter<'_>>>()
    }

    fn decode(line: &str) -> Vec<Letter<'l>> {
        line.split_whitespace()
            .map(|morse_letter| match Letter::from_str(morse_letter) {
                Ok(letter) => letter,
                Err(err) => panic!("Character not supported {:?}", err),
            })
            .collect::<Vec<Letter<'_>>>()
    }
}

pub struct TranslatorBuilder<T: WavOutBuffer> {
    traduction_type: MorseTraductionType,
    input_stream: Option<Vec<String>>,
    output_stream: Option<Rc<RefCell<T>>>,
}

impl<T: WavOutBuffer> TranslatorBuilder<T> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn input_stream(&mut self, input_stream: Vec<String>) -> &mut Self {
        self.input_stream = Some(input_stream);
        self
    }

    pub fn output_stream(&mut self, out_stream: Rc<RefCell<T>>) -> &mut Self {
        self.output_stream = Some(out_stream);
        self
    }

    pub fn traduction_type(&mut self, traduction_type: MorseTraductionType) -> &mut Self {
        self.traduction_type = traduction_type;
        self
    }

    pub fn build_streamed(&self) -> Result<StreamedMorseTranslator<T>, String> {
        Ok(StreamedMorseTranslator {
            input_stream: self
                .input_stream
                .as_ref()
                .expect("input_stream not set")
                .clone(),
            output_stream: self
                .output_stream
                .as_ref()
                .expect("output_stream not set")
                .clone(),
            traduction_type: self.traduction_type.clone(),
        })
    }
}

impl<T: WavOutBuffer> Default for TranslatorBuilder<T> {
    fn default() -> Self {
        TranslatorBuilder {
            input_stream: None,
            output_stream: None,
            traduction_type: MorseTraductionType::Text,
        }
    }
}
