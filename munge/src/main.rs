//! This is a simple utility to post-process the Wasm output of [TeaVM](https://github.com/konsoletyper/teavm) to
//! make it suitable for use as a [Spin](https://github.com/fermyon/spin) HTTP app.
//!
//! It replaces the "teavm" and "teavmHeapTrace" imports with stub functions since Spin won't know how to deal with
//! them at runtime.

use {
    anyhow::{bail, Result},
    std::{
        io::{self, Read, Write},
        mem,
    },
    wast::{
        core::{
            Expression, Func, FuncKind, Import, InlineExport, Instruction, ItemKind, ItemSig,
            ModuleField, ModuleKind, TypeUse,
        },
        parser::{self, ParseBuffer},
        Wat,
    },
};

fn main() -> Result<()> {
    let mut wasm = Vec::new();
    io::stdin().read_to_end(&mut wasm)?;
    let wat = wasmprinter::print_bytes(&wasm)?;
    let buffer = ParseBuffer::new(&wat)?;
    let wat = parser::parse::<Wat>(&buffer)?;
    let mut module = match wat {
        Wat::Module(module) => module,
        Wat::Component(_) => bail!("components not yet supported"),
    };

    let fields = match &mut module.kind {
        ModuleKind::Text(fields) => fields,
        ModuleKind::Binary(_) => bail!("binary modules not yet supported"),
    };

    let stub = |span, ty, instrs| {
        ModuleField::Func(Func {
            span,
            id: None,
            name: None,
            exports: InlineExport { names: Vec::new() },
            kind: FuncKind::Inline {
                locals: Vec::new(),
                expression: Expression { instrs },
            },
            ty,
        })
    };

    for field in fields {
        match field {
            ModuleField::Import(Import {
                span,
                module: module @ ("teavm" | "teavmHeapTrace"),
                item:
                    ItemSig {
                        kind: ItemKind::Func(ty),
                        ..
                    },
                ..
            }) => {
                *field = stub(
                    *span,
                    mem::replace(
                        ty,
                        TypeUse {
                            index: None,
                            inline: None,
                        },
                    ),
                    Box::new([if *module == "teavmHeapTrace" {
                        Instruction::Return
                    } else {
                        Instruction::Unreachable
                    }]),
                );
            }
            _ => (),
        }
    }

    io::stdout().write_all(&module.encode()?)?;

    Ok(())
}
