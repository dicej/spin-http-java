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
        token::Float64,
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

    let ignore = |span, ty, instrs| {
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
        if let ModuleField::Import(Import {
            span,
            module: "teavm" | "teavmHeapTrace",
            field: name,
            item:
                ItemSig {
                    kind: ItemKind::Func(ty),
                    ..
                },
            ..
        }) = field
        {
            *field = ignore(
                *span,
                mem::replace(
                    ty,
                    TypeUse {
                        index: None,
                        inline: None,
                    },
                ),
                if *name == "currentTimeMillis" {
                    Box::new([
                        Instruction::F64Const(Float64 {
                            bits: 0_f64.to_bits(),
                        }),
                        Instruction::Return,
                    ])
                } else {
                    Box::new([Instruction::Return])
                },
            );
        }
    }

    io::stdout().write_all(&module.encode()?)?;

    Ok(())
}
