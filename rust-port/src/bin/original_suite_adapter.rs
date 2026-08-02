use std::env;
use std::process::ExitCode;

use rust_port::AlgorithmFlags;

fn emit_flags() {
    let flags = [
        ("FLOAT", AlgorithmFlags::FLOAT.bits()),
        ("SIGNED", AlgorithmFlags::SIGNED.bits()),
        ("NOEXP", AlgorithmFlags::NOEXP.bits()),
        ("PATH", AlgorithmFlags::PATH.bits()),
        ("LOCALEALPHA", AlgorithmFlags::LOCALEALPHA.bits()),
        ("LOCALENUM", AlgorithmFlags::LOCALENUM.bits()),
        ("IGNORECASE", AlgorithmFlags::IGNORECASE.bits()),
        ("LOWERCASEFIRST", AlgorithmFlags::LOWERCASEFIRST.bits()),
        ("GROUPLETTERS", AlgorithmFlags::GROUPLETTERS.bits()),
        ("UNGROUPLETTERS", AlgorithmFlags::UNGROUPLETTERS.bits()),
        ("NANLAST", AlgorithmFlags::NANLAST.bits()),
        (
            "COMPATIBILITYNORMALIZE",
            AlgorithmFlags::COMPATIBILITYNORMALIZE.bits(),
        ),
        ("NUMAFTER", AlgorithmFlags::NUMAFTER.bits()),
        ("PRESORT", AlgorithmFlags::PRESORT.bits()),
        ("DEFAULT", AlgorithmFlags::DEFAULT.bits()),
        ("INT", AlgorithmFlags::INT.bits()),
        ("UNSIGNED", AlgorithmFlags::UNSIGNED.bits()),
        ("REAL", AlgorithmFlags::REAL.bits()),
        ("LOCALE", AlgorithmFlags::LOCALE.bits()),
        ("I", AlgorithmFlags::INT.bits()),
        ("U", AlgorithmFlags::UNSIGNED.bits()),
        ("F", AlgorithmFlags::FLOAT.bits()),
        ("S", AlgorithmFlags::SIGNED.bits()),
        ("R", AlgorithmFlags::REAL.bits()),
        ("N", AlgorithmFlags::NOEXP.bits()),
        ("P", AlgorithmFlags::PATH.bits()),
        ("LA", AlgorithmFlags::LOCALEALPHA.bits()),
        ("LN", AlgorithmFlags::LOCALENUM.bits()),
        ("L", AlgorithmFlags::LOCALE.bits()),
        ("IC", AlgorithmFlags::IGNORECASE.bits()),
        ("LF", AlgorithmFlags::LOWERCASEFIRST.bits()),
        ("G", AlgorithmFlags::GROUPLETTERS.bits()),
        ("UG", AlgorithmFlags::UNGROUPLETTERS.bits()),
        ("C", AlgorithmFlags::UNGROUPLETTERS.bits()),
        ("CAPITALFIRST", AlgorithmFlags::UNGROUPLETTERS.bits()),
        ("NL", AlgorithmFlags::NANLAST.bits()),
        ("CN", AlgorithmFlags::COMPATIBILITYNORMALIZE.bits()),
        ("NA", AlgorithmFlags::NUMAFTER.bits()),
        ("PS", AlgorithmFlags::PRESORT.bits()),
    ];

    for (name, value) in flags {
        println!("{name}\t{value}");
    }
}

fn main() -> ExitCode {
    match env::args().nth(1).as_deref() {
        Some("flags") => {
            emit_flags();
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("usage: original-suite-adapter flags");
            ExitCode::from(2)
        }
    }
}
