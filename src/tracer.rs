use crate::span::Span;
use rhai::{BinaryExpr, Engine, Expr, FlowControl, FnCallExpr, Position, Stmt, StmtBlock};
use std::error::Error;

/// [`SpanTracer`] extracts spans from Rhai scripts, providing
/// byte offsets, line, and column information for each statement or expression.
///
/// # Example
///
/// ```rust
/// use rhai_trace::{SpanTracer, Span};
///
/// let code = r#"
///     let a = 1;
///     let b = a + 2;
/// "#;
///
/// let tracer = SpanTracer::new();
/// let spans = tracer.extract_from(code).unwrap();
///
/// for span in spans {
///     println!("Span: {}..{} (line {}, column {})", 
///              span.start(), span.end(), span.line(), span.column());
/// }
/// ```

pub struct SpanTracer {
    engine: Engine,
}

impl SpanTracer {
    pub fn new() -> Self {
        Self {
            engine: Engine::new(),
        }
    }

    /// Extracts all spans (start/end byte offsets, line, column) from a Rhai script.
    /// Returns a `Vec<Span>` on success or an error if the script cannot be compiled.
    pub fn extract_from<S: AsRef<str>>(&self, script: S) -> Result<Vec<Span>, Box<dyn Error>> {
        let script_ref = script.as_ref();
        let ast = self.engine.compile(script_ref)?;
        let mut spans = Vec::new();

        for stmt in ast.statements() {
            Self::walk_stmt(stmt, script_ref, &mut spans)?;
        }

        Ok(spans)
    }

    fn walk_stmt(stmt: &Stmt, script: &str, spans: &mut Vec<Span>) -> Result<(), Box<dyn Error>> {
        match stmt {
            Stmt::Noop(pos) => spans.push(Span::from_pos(script, pos)),
            Stmt::If(flow, pos) | Stmt::While(flow, pos) | Stmt::Do(flow, _, pos) => {
                spans.push(Span::from_pos(script, pos));
                Self::walk_flow_control(flow, script, spans)?;
            }
            Stmt::For(boxed, pos) => {
                spans.push(Span::from_pos(script, pos));
                let (_, _, flow) = &**boxed;
                Self::walk_flow_control(flow, script, spans)?;
            }
            Stmt::Var(boxed, _, pos) => {
                spans.push(Span::from_pos(script, pos));
                let (_, expr, _) = &**boxed;
                Self::walk_expr(expr, script, spans)?;
            }
            Stmt::Assignment(boxed) => {
                let (_, expr) = &**boxed;
                Self::walk_binary_expr(expr, script, spans)?;
            }
            Stmt::FnCall(boxed, pos) => {
                spans.push(Span::from_pos(script, pos));
                Self::walk_fn_call(boxed, script, spans)?;
            }
            Stmt::Block(block) => {
                spans.push(Span::from_rhai_span(
                    script,
                    block.span(),
                    &block.position(),
                ));
                Self::walk_block(block, script, spans)?;
            }
            Stmt::TryCatch(flow, pos) => {
                spans.push(Span::from_pos(script, pos));
                Self::walk_flow_control(flow, script, spans)?;
            }
            Stmt::Expr(expr) => Self::walk_expr(expr, script, spans)?,
            Stmt::BreakLoop(opt_expr, _, pos) | Stmt::Return(opt_expr, _, pos) => {
                spans.push(Span::from_pos(script, pos));
                if let Some(expr) = opt_expr {
                    Self::walk_expr(expr, script, spans)?;
                }
            }
            Stmt::Import(boxed, pos) => {
                spans.push(Span::from_pos(script, pos));
                let (expr, _) = &**boxed;
                Self::walk_expr(expr, script, spans)?;
            }
            Stmt::Export(..) | Stmt::Share(..) => {}
            &_ => {}
        }
        Ok(())
    }

    fn walk_binary_expr(
        bin: &BinaryExpr,
        script: &str,
        spans: &mut Vec<Span>,
    ) -> Result<(), Box<dyn Error>> {
        Self::walk_expr(&bin.lhs, script, spans)?;
        Self::walk_expr(&bin.rhs, script, spans)?;
        Ok(())
    }

    fn walk_flow_control(
        flow: &FlowControl,
        script: &str,
        spans: &mut Vec<Span>,
    ) -> Result<(), Box<dyn Error>> {
        Self::walk_expr(&flow.expr, script, spans)?;
        Self::walk_block(&flow.body, script, spans)?;
        Self::walk_block(&flow.branch, script, spans)?;
        Ok(())
    }

    fn walk_block(
        block: &StmtBlock,
        script: &str,
        spans: &mut Vec<Span>,
    ) -> Result<(), Box<dyn Error>> {
        for stmt in block.statements() {
            Self::walk_stmt(stmt, script, spans)?;
        }
        Ok(())
    }

    fn walk_expr(expr: &Expr, script: &str, spans: &mut Vec<Span>) -> Result<(), Box<dyn Error>> {
        spans.push(Span::from_pos(script, Self::expr_position(expr)));

        match expr {
            Expr::FnCall(f, _) | Expr::MethodCall(f, _) => {
                Self::walk_fn_call(f.as_ref(), script, spans)?;
            }
            Expr::Array(arr, _) | Expr::InterpolatedString(arr, _) => {
                for elem in arr.iter() {
                    Self::walk_expr(elem, script, spans)?;
                }
            }
            Expr::Map(map_box, _) => {
                let (pairs, _) = &**map_box;
                for (_, expr) in pairs.iter() {
                    Self::walk_expr(expr, script, spans)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn walk_fn_call(
        fn_call: &FnCallExpr,
        script: &str,
        spans: &mut Vec<Span>,
    ) -> Result<(), Box<dyn Error>> {
        // Use the first argument's position as an approximation
        if let Some(arg) = fn_call.args.first() {
            spans.push(Span::from_pos(script, Self::expr_position(arg)));
        }
        for arg in &fn_call.args {
            Self::walk_expr(arg, script, spans)?;
        }
        Ok(())
    }

    fn expr_position(expr: &Expr) -> &Position {
        match expr {
            Expr::DynamicConstant(_, pos)
            | Expr::BoolConstant(_, pos)
            | Expr::IntegerConstant(_, pos)
            | Expr::FloatConstant(_, pos)
            | Expr::CharConstant(_, pos)
            | Expr::StringConstant(_, pos)
            | Expr::InterpolatedString(_, pos)
            | Expr::Array(_, pos)
            | Expr::Map(_, pos)
            | Expr::Unit(pos)
            | Expr::Variable(_, _, pos)
            | Expr::ThisPtr(pos)
            | Expr::Property(_, pos)
            | Expr::MethodCall(_, pos)
            | Expr::FnCall(_, pos)
            | Expr::Dot(_, _, pos)
            | Expr::Index(_, _, pos)
            | Expr::And(_, pos)
            | Expr::Or(_, pos)
            | Expr::Coalesce(_, pos)
            | Expr::Custom(_, pos) => pos,
            Expr::Stmt(block) => block
                .statements()
                .first()
                .map(|s| Self::stmt_position(s))
                .unwrap_or_else(|| &Position::NONE),
            &_ => &Position::NONE,
        }
    }

    fn stmt_position(stmt: &Stmt) -> &Position {
        match stmt {
            Stmt::Noop(pos)
            | Stmt::If(_, pos)
            | Stmt::While(_, pos)
            | Stmt::Do(_, _, pos)
            | Stmt::For(_, pos)
            | Stmt::Var(_, _, pos)
            | Stmt::FnCall(_, pos)
            | Stmt::TryCatch(_, pos)
            | Stmt::BreakLoop(_, _, pos)
            | Stmt::Return(_, _, pos)
            | Stmt::Import(_, pos) => pos,
            Stmt::Assignment(_) => &Position::NONE,
            Stmt::Block(block) => block
                .statements()
                .first()
                .map(Self::stmt_position)
                .unwrap_or(&Position::NONE),
            Stmt::Export(..) | Stmt::Share(..) => &Position::NONE,
            Stmt::Expr(expr) => Self::expr_position(expr),
            &_ => &Position::NONE,
        }
    }
}
