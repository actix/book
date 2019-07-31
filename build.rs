extern crate skeptic;

use std::{env, fs};


fn main() {
    let f = env::var("OUT_DIR").unwrap() + "/skeptic-tests.rs";
    let _ = fs::remove_file(f);
    // generates doc tests.
    skeptic::generate_doc_tests(
        &["actix/src/sec-0-quick-start.md",
          "actix/src/sec-1-getting-started.md",
          "actix/src/sec-2-actor.md",
          "actix/src/sec-3-address.md",
          "actix/src/sec-4-context.md",
          "actix/src/sec-5-arbiter.md",
          "actix/src/sec-6-sync-arbiter.md",
          "actix/src/sec-7-stream.md",
          "actix/src/sec-8-io-helpers.md",
          "actix/src/sec-9-supervisor.md",
          "actix/src/sec-10-registry.md",
          "actix/src/sec-11-helper-actors.md",
          "actix/src/sec-12-coming-from-akka.md",

        ]);
}
