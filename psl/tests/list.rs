extern crate psl;
extern crate psl_lexer;
extern crate rspec;
extern crate idna;

use std::{env, mem};
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

use psl::{Psl, List};
use psl_lexer::request;
use rspec::report::ExampleReport;

#[test]
fn list_behaviour() {
    let list = List::new();

    rspec::run(&rspec::describe("the official test", (), |ctx| {
        let tests = "https://raw.githubusercontent.com/publicsuffix/list/master/tests/tests.txt";
        let body = request(tests).unwrap_or_else(|_| {
            let root = env::var("CARGO_MANIFEST_DIR").unwrap();
            let path = Path::new(&root).join("tests").join("tests.txt");
            let mut file = File::open(path).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            contents
        });

        let mut parse = false;

        for (i, line) in body.lines().enumerate() {
            match line {
                line if line.trim().is_empty() => { parse = true; continue; }
                line if line.starts_with("//") => { continue; }
                line => {
                    if !parse { continue; }
                    let mut test = line.split_whitespace().peekable();
                    if test.peek().is_none() {
                        continue;
                    }
                    let input = match test.next() {
                        Some("null") => "",
                        Some(res) => res,
                        None => { panic!(format!("line {} of the test file doesn't seem to be valid", i)); },
                    };
                    let var = if let Ok(var) = env::var("PSL_TLD") { var } else { String::new() };
                    if !var.trim().is_empty() && !input.trim().trim_right_matches('.').ends_with(&var) {
                        continue;
                    }
                    let (expected_root, expected_suffix) = match test.next() {
                        Some("null") => (None, None),
                        Some(root) => {
                            let suffix = {
                                let parts: Vec<&str> = root.split('.').rev().collect();
                                (&parts[..parts.len()-1]).iter().rev()
                                    .map(|part| *part)
                                    .collect::<Vec<_>>()
                                    .join(".")
                            };
                            (Some(root.to_string()), Some(suffix.to_string()))
                        },
                        None => { panic!(format!("line {} of the test file doesn't seem to be valid", i)); },
                    };
                    let is_punycode = input.contains("xn--");
                    let domain = if is_punycode { idna::domain_to_unicode(input).0 } else { input.to_owned() };
                    let (mut found_root, mut found_suffix) = if domain.starts_with(".") || domain.contains("..") {
                        (None, None)
                    } else {
                        list.domain(&domain.to_lowercase())
                        .map(|d| {
                            let domain = d.to_string();
                            let suffix = d.suffix().to_string();
                            (Some(domain), Some(suffix))
                        })
                        .unwrap_or((None, None))
                    };
                    if is_punycode {
                        found_root = found_root.map(|p| idna::domain_to_ascii(&p).unwrap());
                        found_suffix = found_suffix.map(|p| idna::domain_to_ascii(&p).unwrap());
                    }
                    ctx.when(msg(format!("input is `{}`", input)), |ctx| {
                        let full_domain = expected_root.is_some();

                        ctx.it(msg(format!("means the root domain {}", val(&expected_root))), move |_| {
                            if expected_root == found_root {
                                ExampleReport::Success
                            } else {
                                let msg = format!("expected `{:?}` but found `{:?}` on line {} of `test_psl.txt`", expected_root, found_root, i+1);
                                ExampleReport::Failure(Some(msg))
                            }
                        });

                        if full_domain {
                            ctx.it(msg(format!("also means the suffix {}", val(&expected_suffix))), move |_| {
                                if expected_suffix == found_suffix {
                                    ExampleReport::Success
                                } else {
                                    let msg = format!("expected `{:?}` but found `{:?}` on line {} of `test_psl.txt`", expected_suffix, found_suffix, i+1);
                                    ExampleReport::Failure(Some(msg))
                                }
                            });
                        }
                    });
                }
            }
        }
    }));
}

// Converts a String to &'static str
//
// This will leak memory but that's OK for our testing purposes
fn msg(s: String) -> &'static str {
    unsafe {
        let ret = mem::transmute(&s as &str);
        mem::forget(s);
        ret
    }
}

fn val(s: &Option<String>) -> String {
    match *s {
        Some(ref v) => format!("should be `{}`", v),
        None => format!("is invalid"),
    }
}