//! Transaction engine binary implemented for parsing a single CSV file input

use std::io::{Read, Write};

use csv::{Reader, ReaderBuilder, Writer};
use transaction_engine::{Action, SingleThreadedEngine, SyncEngine};

/// Behaviour on deserialization error
///
/// I wasn't sure which would be best here, but we'll assume well structured
/// input and ignore if we can't deserialize. But you can change the behaviour
/// here andthe other variants should work (though log doesn't send the output
/// anywhere. Proabably another csv file, but that would include more config)
const ERROR_BEHAVIOUR: ErrorBehaviour = ErrorBehaviour::Ignore;

#[allow(dead_code)]
enum ErrorBehaviour {
    Ignore,
    Log, // TODO: configure out file?
    Crash,
}

fn main() {
    // Clap is nice, but who needs options
    let input = std::env::args().nth(1).expect("no input file given");

    // Create a new reader. `csv`'s default is to assume there is a header
    let reader = ReaderBuilder::default()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_path(input)
        .expect("failed to read file as csv");

    // Write to stdout
    let mut writer = Writer::from_writer(std::io::stdout());

    process(reader, &mut writer);
}

fn process<R: Read, W: Write>(reader: Reader<R>, writer: &mut Writer<W>) {
    let reader = reader.into_deserialize::<Action>();
    let mut engine = SingleThreadedEngine::new();
    let mut errors = Vec::new();
    match ERROR_BEHAVIOUR {
        ErrorBehaviour::Ignore => engine.process_all(reader.filter_map(Result::ok)),
        ErrorBehaviour::Log => engine.process_all(reader.filter_map(|res| match res {
            Ok(action) => Some(action),
            Err(e) => {
                errors.push(e);
                None
            }
        })),
        ErrorBehaviour::Crash => {
            engine.process_all(reader.map(|res| res.expect("failed to deserialize record: {}")))
        }
    }
    .expect("failed to process");

    engine
        .state()
        .accounts()
        .for_each(|data| writer.serialize(data).expect("failed to write to stdout"));
}

// TODO: fix tests with static output though hashmap will produce random client orders
// #[cfg(test)]
// mod tests {
//     use super::*;

//     const EXPECT: &str = include_str!("../test_data/output.csv");

//     const DENSE: &str = include_str!("../test_data/dense.csv");
//     const PRETTY: &str = include_str!("../test_data/pretty.csv");

//     #[test]
//     fn test_dense() {
//         let reader = ReaderBuilder::default()
//             .has_headers(true)
//             .trim(csv::Trim::All)
//             .from_reader(DENSE.as_bytes());

//         let mut writer = Writer::from_writer(Vec::new());
//         process(reader, &mut writer);

//         let result =
//             String::from_utf8(writer.into_inner().expect("Failed to get result bytes")).unwrap();

//         assert_eq!(result.as_str(), EXPECT);
//     }

//     #[test]
//     fn test_pretty() {
//         let reader = ReaderBuilder::default()
//             .has_headers(true)
//             .trim(csv::Trim::All)
//             .from_reader(PRETTY.as_bytes());

//         let mut writer = Writer::from_writer(Vec::new());
//         process(reader, &mut writer);

//         let result =
//             String::from_utf8(writer.into_inner().expect("Failed to get result bytes")).unwrap();

//         assert_eq!(result.as_str(), EXPECT);
//     }
// }
