use rustc_hir::{Block, BlockCheckMode, Expr, ExprKind, UnsafeSource, def};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::ty::TyCtxt;
use rustc_middle::ty::inherent::Safety;
use rustc_session::declare_lint_pass;
use rustc_span::def_id::DefId;

use clippy_utils::diagnostics::span_lint;

declare_clippy_lint! {
    /// ### What it does
    /// Detects `unsafe`-blocks that cover more code than necessary,
    /// obscuring which operations are actually unsafe.
    ///
    /// ### Why is this bad?
    /// As stated in [The Rust Reference](https://doc.rust-lang.org/reference/unsafe-keyword.html#unsafe-blocks-unsafe-),
    /// when programmers use `unsafe`-blocks they are stating that all safety requirements are met.
    /// However, it can be completely unclear which parts of the code need extra care.
    /// Combined with [clippy::undocumented_unsafe_blocks](https://rust-lang.github.io/rust-clippy/master/index.html#undocumented_unsafe_blocks),
    /// this also obscures the safety argument.
    ///
    /// ### Known Problems
    /// Although the lint tries to enforce "minimal" `unsafe`-blocks, this is not yet guaranteed.
    /// The lint is conservative in the sense that false positives are bugs,
    /// with the drawback of having an unknown amount of false negatives.
    ///
    /// ### Example
    /// ```no_run
    /// unsafe {
    ///     let x = Some(true);
    ///     let y = x.unwrap_unchecked();
    /// }
    /// ```
    /// Use instead:
    /// ```no_run
    /// let x = Some(true);
    /// let y = unsafe { x.unwrap_unchecked() };
    /// ```
    #[clippy::version = "1.85.0"]
    pub MINIMAL_UNSAFE_BLOCK,
    restriction,
    "`unsafe` blocks that cover more code than necessary"
}

declare_lint_pass!(MinimalUnsafeBlock => [MINIMAL_UNSAFE_BLOCK]);

impl<'tcx> LateLintPass<'tcx> for MinimalUnsafeBlock {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>) {
        if let ExprKind::Block(block, _) = expr.kind {
            if let BlockCheckMode::UnsafeBlock(source) = block.rules {
                match source {
                    UnsafeSource::CompilerGenerated => {},
                    UnsafeSource::UserProvided => check_user_provided_unsafe_block(cx, block),
                }
            }
        }
    }
}

fn check_user_provided_unsafe_block(cx: &LateContext<'_>, block: &Block<'_>) {
    debug_assert_eq!(block.rules, BlockCheckMode::UnsafeBlock(UnsafeSource::UserProvided));

    if !block.stmts.is_empty() {
        span_lint(
            cx,
            MINIMAL_UNSAFE_BLOCK,
            block.span,
            "this `unsafe` block is not minimal as it covers statements",
        );
    }

    if block.expr.is_some() {
        if let Some(msg) = check_user_provided_unsafe_block_expr(cx, block) {
            span_lint(
                cx,
                MINIMAL_UNSAFE_BLOCK,
                block.span,
                format!("this `unsafe` block is not minimal as {msg}"),
            );
        }
    }
}

fn check_user_provided_unsafe_block_expr(cx: &LateContext<'_>, block: &Block<'_>) -> Option<&'static str> {
    let expr = block.expr?;

    match expr.kind {
        ExprKind::Array(_) => Some("it covers unnecessarily an array"),
        ExprKind::Block(_, _) => Some("it covers unnecessarily a block"),
        ExprKind::Closure(_) => Some("it covers unnecessarily a closure"),
        ExprKind::If(_, _, _) => Some("it covers unnecessarily an `if` block"),
        ExprKind::Loop(_, _, _, _) => Some("it covers unnecessarily a `loop` block"),
        ExprKind::Tup(_) => Some("it covers unnecessarily a tuple"),
        ExprKind::Call(call, _) => {
            if is_call_safe(cx, call) {
                Some("it covers unnecessarily a safe call")
            } else {
                None
            }
        },
        ExprKind::MethodCall(_, _, _, _) => {
            let typeck = cx.typeck_results();
            if let Some(def_id) = typeck.type_dependent_def_id(expr.hir_id) {
                if is_fn_safe(cx.tcx, def_id) {
                    Some("it covers unnecessarily a safe method call")
                } else {
                    None
                }
            } else {
                None
            }
        },
        _ => {
            eprintln!("unknown: {expr:#?}");
            None
        },
        // ExprKind::ConstBlock(const_block) => todo!(),
        // ExprKind::Binary(spanned, _, _) => todo!(),
        // ExprKind::Unary(un_op, _) => todo!(),
        // ExprKind::Lit(_) => todo!(),
        // ExprKind::Cast(_, _) => todo!(),
        // ExprKind::Type(_, _) => todo!(),
        // ExprKind::DropTemps(_) => todo!(),
        // ExprKind::Let(_) => todo!(),
        // ExprKind::Match(_, _, match_source) => todo!(),
        // ExprKind::Assign(_, _, span) => todo!(),
        // ExprKind::AssignOp(spanned, _, _) => todo!(),
        // ExprKind::Field(_, ident) => todo!(),
        // ExprKind::Index(_, _, span) => todo!(),
        // ExprKind::Path(qpath) => todo!(),
        // ExprKind::AddrOf(borrow_kind, mutability, _) => todo!(),
        // ExprKind::Break(destination, _) => todo!(),
        // ExprKind::Continue(destination) => todo!(),
        // ExprKind::Ret(_) => todo!(),
        // ExprKind::Become(_) => todo!(),
        // ExprKind::InlineAsm(_) => todo!(),
        // ExprKind::OffsetOf(_, _) => todo!(),
        // ExprKind::Struct(_, _, struct_tail_expr) => todo!(),
        // ExprKind::Repeat(_, _) => todo!(),
        // ExprKind::Yield(_, yield_source) => todo!(),
        // ExprKind::UnsafeBinderCast(unsafe_binder_cast_kind, _, _) => todo!(),
        // ExprKind::Err(error_guaranteed) => todo!(),
    }
}

fn is_call_safe(cx: &LateContext<'_>, call: &Expr<'_>) -> bool {
    let typeck = cx.typeck_results();
    match call.kind {
        #[warn(clippy::single_match_else)]
        ExprKind::Path(qpath) => match typeck.qpath_res(&qpath, call.hir_id) {
            def::Res::Def(def::DefKind::Fn, def_id) => is_fn_safe(cx.tcx, def_id),
            _ => {
                eprintln!("call path unknown: {call:#?}");
                false
            },
        },
        ExprKind::Closure(_) => true,
        _ => {
            eprintln!("call unknown: {call:#?}");
            false
        },
    }
}

fn is_fn_safe(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    //TODO understand this statement (copied from other lint)
    let fn_sig = tcx.fn_sig(def_id).instantiate_identity().skip_binder();
    fn_sig.safety.is_safe()
}
