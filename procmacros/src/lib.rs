extern crate proc_macro;
use proc_macro::TokenStream;

use syn::parse::{Parser};
use syn::{ Expr, ItemConst, Token, punctuated};

use std::fs::{OpenOptions, create_dir_all, remove_file};
use std::io::prelude::*;
use std::path::Path;

#[derive(Debug)]
struct Command {
    name: String,
    desc: String,
    args: Vec<Arg>,
}

#[derive(Debug)]
struct Arg {
    name: String,
    desc: String,
    arg_type: String,
    data_type: String,
    kind: String,
    optional: bool,
}

#[proc_macro_attribute]
pub fn rediscmd_doc(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut output = String::new();

    let parsed: ItemConst = syn::parse(item.clone()).unwrap();
    match *parsed.expr {
        Expr::Macro(mac) => {
            for token in mac.mac.tokens.into_iter() {
                match token {
                    proc_macro2::TokenTree::Group(g) => {
                        let cmd = parse_command(g.stream());
                        output = stringify_command(cmd);
                    },
                    _ => ()
                }
            }
        },
        _ => ()
    }

    let filepath = Path::new("doc").join("COMMAND_REFERENCE_GEN.md");

    // delete file if first attr is passed
    if &attr.to_string() == "clean" {
        match remove_file(filepath.clone()) {
            Ok(_) => (),
            Err(e) => println!("Could not delete {:?}: {}", filepath.to_str(), e) 
        }
    
    }
    
    // write out markdown
    create_dir_all("doc").unwrap();
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(filepath.to_str().unwrap())
        .unwrap();
    if let Err(e) = writeln!(file, "{}", output) {
        eprintln!("Couldn't write to file: {}", e);
    };

    item
}

fn parse_command(tokens: proc_macro2::TokenStream) -> Command {
    let mut name = String::new();
    let mut desc = String::new();
    let mut args: Vec<Arg> = Vec::new();

    let mut pos: usize = 0;
    for tt in tokens.into_iter() {
        match tt {
            proc_macro2::TokenTree::Literal(l) => {
                if pos == 0 {
                    name = l.to_string();
                }
                if pos == 1 {
                    desc = l.to_string();
                }
                pos += 1
            },
            proc_macro2::TokenTree::Group(g) => {
                args = parse_args(g.stream());
            },
            _ => ()
        }
    }

    Command {
        name,
        desc,
        args,
    }
}

fn parse_args(tokens: proc_macro2::TokenStream) -> Vec<Arg> {
    let mut args: Vec<Arg> = Vec::new();

    for tt in tokens.into_iter() {
        match tt {
            proc_macro2::TokenTree::Group(g) => {
                let parser = punctuated::Punctuated::<Expr, Token![,]>::parse_terminated;
                let parsed = parser.parse2(g.stream()).unwrap();

                let mut name = String::new();
                let mut desc = String::new();
                let mut arg_type = String::new();
                let mut data_type = String::new();
                let mut kind = String::new();
                let mut optional = true;

                let mut cursor= parsed.iter();

                let name_expr = cursor.next().unwrap();
                match name_expr {
                    Expr::Lit(l) => {
                        match &l.lit {
                            syn::Lit::Str(s) => name = s.value(),
                            _ => ()
                        }
                    }
                    _ => ()
                }

                let desc_expr = cursor.next().unwrap();
                match desc_expr {
                    Expr::Lit(l) => {
                        match &l.lit {
                            syn::Lit::Str(s) => desc = s.value(),
                            _ => ()
                        }
                    }
                    _ => ()
                }

                let at_expr = cursor.next().unwrap();
                match at_expr {
                    Expr::Path(p) => {
                        let mut segs = p.path.segments.iter();
                        match segs.next() {
                            Some(seg) => {
                                if &seg.ident.to_string() == "ArgType" {
                                    if let Some(t) = segs.next() {
                                        arg_type = t.ident.to_string();
                                    }
                                }
                            }
                            None => ()
                        }
                    }
                    _ => ()
                }

                let dt_expr = cursor.next().unwrap();
                match dt_expr {
                    Expr::Path(p) => {
                        let mut segs = p.path.segments.iter();
                        if let Some(seg) = segs.next() {
                            data_type = seg.ident.to_string();
                        }
                    }
                    _ => ()
                }

                let kind_expr = cursor.next().unwrap();
                match kind_expr {
                    Expr::Path(p) => {
                        let mut segs = p.path.segments.iter();
                        match segs.next() {
                            Some(seg) => {
                                if &seg.ident.to_string() == "Collection" {
                                    if let Some(t) = segs.next() {
                                        kind = t.ident.to_string();
                                    }
                                }
                            }
                            None => ()
                        }
                    }
                    _ => ()
                }

                let default_expr = cursor.next().unwrap();
                match default_expr {
                    Expr::Path(p) => {
                        let mut segs = p.path.segments.iter();
                        match segs.next() {
                            Some(seg) => {
                                if &seg.ident.to_string() == "None" {
                                    optional = false;
                                }
                            }
                            None => ()
                        }
                    }
                    _ => ()
                }

                args.push(Arg {
                    name,
                    desc,
                    arg_type,
                    data_type,
                    kind,
                    optional,
                })
            },
            _ => ()
        }
    }

    args
}

fn stringify_command(cmd: Command) -> String {
    let name = cmd.name.to_uppercase().replace("\"", "");
    let desc = cmd.desc.replace("\"", "");
    let args = stringify_args(cmd.args);

    let output = format!("
### {name}
#### Format
```
placeholder
```
#### Description
{desc}
#### Example
```
placeholder
```
#### Parameters
{args}
",
        name=name,
        desc=desc,
        args=args
    );
    
    output
}

fn stringify_args(args: Vec<Arg>) -> String {
    let mut output = String::new();
    for arg in args {
        let name = arg.name.to_uppercase();
        let desc = arg.desc;
        let optional = if arg.optional { "Optional" } else { "Required" };

        let arg_out = format!("
* **{name}**: {optional}. {desc}
",
            name=name,
            desc=desc,
            optional=optional
        );

        output.push_str(&arg_out);
    }

    output
}
