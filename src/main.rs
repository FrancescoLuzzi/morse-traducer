use clap::Parser;
use morse_traducer::parser::MorseArgs;
use morse_traducer::translator::{MorseTranslator, TranslatorBuilder};
use morse_traducer::utils::{get_reader, get_writer};
use std::cell::RefCell;
use std::io::BufRead;
use std::rc::Rc;

fn main() {
    let args = MorseArgs::parse();
    let input_stream: Vec<String> = get_reader(&args.in_file)
        .lines()
        .map_while(Result::ok)
        .collect();
    let output_stream = Rc::new(RefCell::new(get_writer(&args.out_file).unwrap()));

    let mut translator = TranslatorBuilder::new()
        .input_stream(input_stream)
        .output_stream(output_stream)
        .traduction_type(args.traduction_type)
        .build_streamed()
        .unwrap();
    translator.translate(args.morse_command).unwrap();
}

#[test]
fn test_main() {
    use morse_traducer::parser::MorseCommand;
    use morse_traducer::translator::TranslatorBuilder;
    use std::cell::RefCell;
    use std::io::{Cursor, Seek, SeekFrom};
    use std::rc::Rc;
    use std::str::from_utf8;

    let output: Rc<RefCell<Cursor<Vec<u8>>>> = Rc::new(RefCell::new(Default::default()));
    let input = vec!["Hello World".into()];
    let mut translator = TranslatorBuilder::new()
        .input_stream(input)
        .output_stream(output.clone())
        .build_streamed()
        .unwrap();
    translator.translate(MorseCommand::Encode).unwrap();
    //launch with cargo test -- --nocapture
    output
        .as_ref()
        .borrow_mut()
        .seek(SeekFrom::Start(0))
        .unwrap();
    let binding = output.as_ref().borrow().clone().into_inner();
    let result = from_utf8(&binding);
    print!("{:?}", result);
}
