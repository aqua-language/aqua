use crate::ast;
use crate::ast::Place;
use crate::mir;
use crate::mir::BasicBlock;
use crate::mir::Constant;
use crate::mir::Function;
use crate::mir::Operand;
use crate::mir::Operation;
use crate::mir::Rvalue;
use crate::mir::Stmt;
use crate::mir::Terminator;
use crate::print::Print;

struct Printer<'a, 'b> {
    formatter: &'a mut std::fmt::Formatter<'b>,
    indent: usize,
    verbose: bool,
}

impl<'a, 'b> Print<'b> for Printer<'a, 'b> {
    fn formatter_mut(&mut self) -> &mut std::fmt::Formatter<'b> {
        self.formatter
    }

    fn indent_mut(&mut self) -> &mut usize {
        &mut self.indent
    }
}

impl<'a, 'b> Printer<'a, 'b> {
    fn new(f: &'a mut std::fmt::Formatter<'b>) -> Printer<'a, 'b> {
        Printer {
            formatter: f,
            indent: 0,
            verbose: false,
        }
    }

    fn locals(&mut self, locals: &[ast::Local]) -> std::fmt::Result {
        for (i, l) in locals.iter().enumerate() {
            if i > 0 {
                self.lit(",")?;
                self.space()?;
            }
            self.local(l)?;
        }
        Ok(())
    }

    fn ast_printer(&mut self) -> ast::display::Printer<'_, 'b> {
        let mut printer = ast::display::Printer::new(self.formatter);
        printer.indent = self.indent;
        printer
    }

    fn local(&mut self, l: &ast::Local) -> std::fmt::Result {
        self.ast_printer().local(l)
    }

    fn mir_function(&mut self, f: &Function) -> std::fmt::Result {
        self.lit("fn")?;
        self.space()?;
        self.lit(&f.name)?;
        self.lit("(")?;
        self.locals(&f.params)?;
        self.lit(")")?;
        self.space()?;
        self.lit("->")?;
        self.space()?;
        self.ast_printer().ty(&f.ty)?;
        self.space()?;
        self.lit("{")?;
        self.indent += 1;
        for l in &f.locals {
            self.newline()?;
            self.lit("let")?;
            self.space()?;
            self.local(l)?;
            self.lit(";")?;
        }
        self.newline()?;
        for (i, block) in f.blocks.iter().enumerate() {
            if i > 0 {
                self.newline()?;
            }
            self.mir_block(block)?;
        }
        self.indent -= 1;
        self.newline()?;
        self.lit("}")
    }

    fn mir_stmt(&mut self, stmt: &Stmt) -> std::fmt::Result {
        // self.newline()?;
        // self.lit("//")?;
        // self.space()?;
        // self.lit(" live_in =")?;
        // self.space()?;
        // self.lit("[")?;
        // self.places(&stmt.live_in.as_slice())?;
        // self.lit("]")?;
        // self.newline()?;
        match &stmt.op {
            Operation::Assign(place, rvalue) => {
                self.ast_printer().place(place)?;
                self.space()?;
                self.lit("=")?;
                self.space()?;
                self.rvalue(rvalue)?;
            }
            Operation::Live(l) => {
                self.lit("StorageLive")?;
                self.lit("(")?;
                self.lit(&l.name)?;
                self.lit(")")?;
            }
            Operation::Dead(l) => {
                self.lit("StorageDead")?;
                self.lit("(")?;
                self.lit(&l.name)?;
                self.lit(")")?;
            }
            Operation::Call { dest, func, args } => {
                self.ast_printer().place(dest)?;
                self.space()?;
                self.lit("=")?;
                self.space()?;
                self.operand(func)?;
                self.lit("(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.lit(",")?;
                        self.space()?;
                    }
                    self.operand(arg)?;
                }
                self.lit(")")?;
            }
            Operation::Noop => {}
        }
        self.lit(";")?;
        if self.verbose {
            self.newline()?;
            self.lit("//")?;
            self.space()?;
            self.lit("live_out =")?;
            self.space()?;
            self.lit("[")?;
            self.places(&stmt.live_out.as_slice())?;
            self.lit("]")?;
            self.newline()?;
        }
        Ok(())
    }

    fn dom(&mut self, block: &BasicBlock) -> std::fmt::Result {
        if self.verbose {
            self.lit("//")?;
            self.space()?;
            self.lit("dom")?;
            self.lit("(")?;
            for (i, dom) in block.dom.iter().enumerate() {
                if i > 0 {
                    self.lit(",")?;
                    self.space()?;
                }
                self.lit(&dom)?;
            }
            self.lit(")")?;
            self.newline()?;
        }
        Ok(())
    }

    fn mir_block(&mut self, block: &BasicBlock) -> std::fmt::Result {
        self.dom(block)?;
        self.punct("'")?;
        self.lit(&block.id)?;
        self.lit(":")?;
        self.space()?;
        self.lit("{")?;
        self.indent += 1;
        for stmt in &block.stmts {
            self.newline()?;
            self.mir_stmt(stmt)?;
        }
        if let Some(ref terminator) = block.terminator {
            self.newline()?;
            self.terminator(terminator)?;
            self.lit(";")?;
        }
        self.indent -= 1;
        self.newline()?;
        self.lit("}")
    }

    fn places(&mut self, places: &[Place]) -> std::fmt::Result {
        for (i, place) in places.iter().enumerate() {
            if i > 0 {
                self.lit(",")?;
                self.space()?;
            }
            self.ast_printer().place(place)?;
        }
        Ok(())
    }

    fn terminator(&mut self, terminator: &Terminator) -> std::fmt::Result {
        match terminator {
            Terminator::Return => self.lit("return"),
            Terminator::Goto(block_id) => {
                self.lit("goto")?;
                self.space()?;
                self.punct("'")?;
                self.lit(&block_id)
            }
            Terminator::IfElse(cond, block_id1, block_id2) => {
                self.lit("if")?;
                self.space()?;
                self.operand(cond)?;
                self.space()?;
                self.lit("goto")?;
                self.space()?;
                self.punct("'")?;
                self.lit(&block_id1)?;
                self.space()?;
                self.lit("else")?;
                self.space()?;
                self.lit("goto")?;
                self.space()?;
                self.punct("'")?;
                self.lit(&block_id2)
            }
        }
    }

    fn rvalue(&mut self, rvalue: &Rvalue) -> std::fmt::Result {
        match rvalue {
            Rvalue::Use(operand) => self.operand(operand),
            Rvalue::Ref { mutable, place } => {
                self.lit("&")?;
                if *mutable {
                    self.lit("mut")?;
                    self.space()?;
                }
                self.ast_printer().place(place)
            }
        }
    }

    fn operand(&mut self, operand: &Operand) -> std::fmt::Result {
        match operand {
            Operand::Constant(c) => {
                self.lit("const")?;
                self.space()?;
                self.constant(c)
            }
            Operand::Copy(place) => {
                self.lit("copy")?;
                self.space()?;
                self.ast_printer().place(place)
            }
            Operand::Move(place) => {
                self.lit("move")?;
                self.space()?;
                self.ast_printer().place(place)
            }
            Operand::Function(name, ts) => {
                self.lit(&name)?;
                self.ast_printer().type_args(ts)
            }
        }
    }

    fn constant(&mut self, c: &Constant) -> std::fmt::Result {
        match c {
            Constant::Int(i) => {
                self.lit(i)?;
            }
            Constant::Bool(b) => {
                self.lit(b)?;
            }
            Constant::String(s) => {
                self.lit("\"")?;
                self.lit(&s)?;
                self.lit("\"")?;
            }
            Constant::Unit => {
                self.lit("()")?;
            }
            Constant::Float(s) => {
                self.lit(s)?;
            }
            Constant::Char(c) => {
                self.lit("'")?;
                self.lit(c)?;
                self.lit("'")?;
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for mir::Function {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Printer::new(f).mir_function(self)
    }
}

impl<'a> std::fmt::Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Printer::new(f).mir_stmt(self)
    }
}

pub struct Verbose<T>(T);

impl Function {
    pub fn verbose(&self) -> Verbose<&mir::Function> {
        Verbose(&self)
    }
}

impl mir::Stmt {
    pub fn verbose(&self) -> Verbose<&Stmt> {
        Verbose(&self)
    }
}

impl<'a> std::fmt::Display for Verbose<&'a Function> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut printer = Printer::new(f);
        printer.verbose = true;
        printer.mir_function(&self.0)
    }
}

impl std::fmt::Display for Verbose<&Stmt> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut printer = Printer::new(f);
        printer.verbose = true;
        printer.mir_stmt(&self.0)
    }
}
