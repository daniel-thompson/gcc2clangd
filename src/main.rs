use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};

#[derive(Debug, Deserialize, Serialize)]
struct BuildCommand {
    command: String,
    directory: String,
    file: String,
}

impl BuildCommand {
    fn set_target(&mut self, target: &str) {
        // Some builds of clangd default to x86-64 rather than to the host
        // architecture. For that reason we still need to introduce a --target
        // argument when running on not-x86-64 architectures.
        if self.command.starts_with("gcc ") {
            self.command
                .replace_range(..4, &format!("clang --target={target} "));
            return;
        }
        let re = Regex::new(r"^([-a-zA-Z0-9]+)-gcc ").unwrap();
        let Some(caps) = re.captures(&self.command) else {
            return;
        };
        self.command = re
            .replace(&self.command, &format!("clang --target={} ", &caps[1]))
            .to_string();
    }

    /// Remove gcc-specific arguments that confuse clangd
    fn hide_unknown_arguments(&mut self) {
        for arg in [
            " -fno-allow-store-data-races ",
            " -fconserve-stack ",
            " -femit-struct-debug-baseonly ",
        ] {
            let Some(range) = self.command.find(arg) else {
                continue;
            };
            self.command.replace_range(range..range + arg.len(), " ");
        }
    }

    /// Remove -mabi=lp64 from the compiler arguments.
    ///
    /// This was introduced after chasing a really odd clangd (19.1.6) bug when
    /// building Linux/arm64 on an arm64 system. Basically clangd (silently)
    /// declines to index files when told to adopt the lp64 ABI. Given that is
    /// the default ABI that seems pretty difficult to forgive... so we'll
    /// simply make sure we get rid of it.
    fn censor_mabi_lp64(&mut self) {
        let needle = " -mabi=lp64";
        let Some(range) = self.command.find(needle) else {
            return;
        };
        self.command.replace_range(range..range + needle.len(), " ");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for fname in std::env::args().skip(1) {
        let file = File::open(&fname)?;
        let reader = BufReader::new(file);

        // Read the JSON contents of the file as an instance of `User`.
        let mut comp_db: Vec<BuildCommand> = serde_json::from_reader(reader)?;

        for bc in comp_db.iter_mut() {
            // TODO: I have an arm64 laptop so this "works for me" but will
            //       cause problems when *not* cross-compiling on x86-64
            //       systems (it should be find for cross-compiled kernel builds
            //       though).
            bc.set_target("aarch64-linux-gnu");
            bc.hide_unknown_arguments();
            bc.censor_mabi_lp64();
        }

        let file = File::create(&fname)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &comp_db)?;
    }

    Ok(())
}
