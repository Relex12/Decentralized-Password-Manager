#[macro_export]
macro_rules! afaire(
    ($arg:expr) => { {
        println!(
            "\x1b[43m/!\\ A FAIRE /!\\ Fonction {}\x1b[0m",
            $arg
        );
    } }
);
