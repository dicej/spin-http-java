//! This is a simple utility to post-process the Wasm output of [TeaVM](https://github.com/konsoletyper/teavm) to
//! make it suitable for use as a [Spin](https://github.com/fermyon/spin) HTTP app.
//!
//! It currently does the following:
//!
//! - Replace the "teavm" and "teavmHeapTrace" imports with stub functions
//! - Rename the "start" export to "_initialize"
//! - Replace the `start` module item with a dummy function to prevent `wasmtime` from calling it
//! - Modify the `memory` module item to request a minimum number of memory pages which exceeds `org.teavm.backend.wasm.WasmTarget.maxHeapSize`

use {
    anyhow::{bail, Result},
    std::{
        io::{self, Read, Write},
        mem,
    },
    wast::{
        core::{
            Export, Expression, Func, FuncKind, FunctionType, Import, InlineExport, Instruction,
            ItemKind, ItemSig, Limits, Memory, MemoryKind, MemoryType, ModuleField, ModuleKind,
            TypeUse,
        },
        parser::{self, ParseBuffer},
        token::Index,
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

    let unreachable = |span, ty, instrs| {
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
                *field = unreachable(
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
            ModuleField::Export(Export {
                name: name @ "start",
                ..
            }) => {
                *name = "_initialize";
            }
            ModuleField::Start(Index::Num(_, span)) => {
                *field = unreachable(
                    *span,
                    TypeUse {
                        index: None,
                        inline: Some(FunctionType {
                            params: Box::new([]),
                            results: Box::new([]),
                        }),
                    },
                    Box::new([Instruction::Unreachable]),
                );
            }
            ModuleField::Memory(Memory {
                kind:
                    MemoryKind::Normal(MemoryType::B32 {
                        limits: Limits { min, max },
                        ..
                    }),
                ..
            }) => {
                *min = 4096;
                *max = None;
            }
            _ => (),
        }
    }

    io::stdout().write_all(&module.encode()?)?;

    Ok(())
}
