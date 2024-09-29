#![feature(rustc_private)]
#![feature(let_chains)]
#![warn(unused_extern_crates)]

extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

use clippy_utils::{diagnostics::span_lint_hir_and_then, is_def_id_trait_method};
use rustc_hir::{
    def::DefKind,
    intravisit::{walk_expr, walk_fn, FnKind, Visitor},
    Body, Expr, ExprKind, FnDecl, Node, YieldSource,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::hir::nested_filter;
use rustc_session::impl_lint_pass;
use rustc_span::{
    def_id::{LocalDefId, LocalDefIdSet},
    Span,
};

dylint_linting::dylint_library!();

#[no_mangle]
pub fn register_lints(sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
    dylint_linting::init_config(sess);
    lint_store.register_lints(&[UNUSED_ASYNC]);
    lint_store.register_late_pass(|_| Box::new(UnusedAsync::default()));
}

rustc_session::declare_lint!(pub UNUSED_ASYNC, Warn, "finds async functions with no await statements");

#[derive(Default)]
pub struct UnusedAsync {
    /// Keeps track of async functions used as values (i.e. path expressions to async functions that
    /// are not immediately called)
    async_fns_as_value: LocalDefIdSet,
    /// Functions with unused `async`, linted post-crate after we've found all uses of local async
    /// functions
    unused_async_fns: Vec<UnusedAsyncFn>,
}

#[derive(Debug, Copy, Clone)]
struct UnusedAsyncFn {
    def_id: LocalDefId,
    fn_span: Span,
    await_in_async_block: Option<Span>,
}

impl_lint_pass!(UnusedAsync => [UNUSED_ASYNC]);

struct AsyncFnVisitor<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    /// Also keep track of `await`s in nested async blocks so we can mention
    /// it in a note
    await_in_async_block: Option<Span>,
    async_block_await_stack: Vec<bool>,
    is_valid: bool,
}

impl<'a, 'tcx> Visitor<'tcx> for AsyncFnVisitor<'a, 'tcx> {
    type NestedFilter = nested_filter::OnlyBodies;

    fn visit_expr(&mut self, ex: &'tcx Expr<'tcx>) {
        if let ExprKind::Yield(_, YieldSource::Await { .. }) = ex.kind {
            if let Some(has_await) = self.async_block_await_stack.last_mut() {
                *has_await = true;
            }

            if self.async_block_await_stack.len() > 1 && self.await_in_async_block.is_none() {
                self.await_in_async_block = Some(ex.span);
            }
        }

        let is_async_block = matches!(
            ex.kind,
            ExprKind::Closure(rustc_hir::Closure {
                kind: rustc_hir::ClosureKind::Coroutine(rustc_hir::CoroutineKind::Desugared(
                    rustc_hir::CoroutineDesugaring::Async,
                    _
                )),
                ..
            })
        );

        if is_async_block {
            self.async_block_await_stack.push(false);
        }

        walk_expr(self, ex);

        if is_async_block {
            self.is_valid &= self.async_block_await_stack.pop().unwrap();
        }
    }

    fn nested_visit_map(&mut self) -> Self::Map {
        self.cx.tcx.hir()
    }
}

impl<'tcx> LateLintPass<'tcx> for UnusedAsync {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        fn_kind: FnKind<'tcx>,
        fn_decl: &'tcx FnDecl<'tcx>,
        body: &Body<'tcx>,
        span: Span,
        def_id: LocalDefId,
    ) {
        if !span.from_expansion()
            && fn_kind.asyncness().is_async()
            && !is_def_id_trait_method(cx, def_id)
        {
            let mut visitor = AsyncFnVisitor {
                cx,
                await_in_async_block: None,
                async_block_await_stack: Vec::new(),
                is_valid: true,
            };
            walk_fn(&mut visitor, fn_kind, fn_decl, body.id(), def_id);

            if !visitor.is_valid {
                // Don't lint just yet, but store the necessary information for later.
                // The actual linting happens in `check_crate_post`, once we've found all
                // uses of local async functions that do require asyncness to pass typeck
                self.unused_async_fns.push(UnusedAsyncFn {
                    await_in_async_block: visitor.await_in_async_block,
                    fn_span: span,
                    def_id,
                });
            }
        }
    }

    fn check_path(
        &mut self,
        cx: &LateContext<'tcx>,
        path: &rustc_hir::Path<'tcx>,
        hir_id: rustc_hir::HirId,
    ) {
        fn is_node_func_value(node: Node<'_>, expected_receiver: Span) -> bool {
            matches!(
                node,
                Node::Expr(Expr {
                    kind: ExprKind::Call(Expr { span, .. }, _) | ExprKind::MethodCall(_, Expr { span, .. }, ..),
                    ..
                }) if *span != expected_receiver
            )
        }

        // Find paths to local async functions that aren't immediately called.
        // E.g. `async fn f() {}; let x = f;`
        // Depending on how `x` is used, f's asyncness might be required despite not having any `await`
        // statements, so don't lint at all if there are any such paths.
        if let Some(def_id) = path.res.opt_def_id()
            && let Some(local_def_id) = def_id.as_local()
            && cx.tcx.def_kind(def_id) == DefKind::Fn
            && cx.tcx.asyncness(def_id).is_async()
            && is_node_func_value(cx.tcx.parent_hir_node(hir_id), path.span)
        {
            self.async_fns_as_value.insert(local_def_id);
        }
    }

    // After collecting all unused `async` and problematic paths to such functions,
    // lint those unused ones that didn't have any path expressions to them.
    fn check_crate_post(&mut self, cx: &LateContext<'tcx>) {
        let iter = self
            .unused_async_fns
            .iter()
            .filter(|UnusedAsyncFn { def_id, .. }| (!self.async_fns_as_value.contains(def_id)));

        for fun in iter {
            span_lint_hir_and_then(
                cx,
                UNUSED_ASYNC,
                cx.tcx.local_def_id_to_hir_id(fun.def_id),
                fun.fn_span,
                "unused `async` for function with no await statements",
                |diag| {
                    diag.help("consider removing the `async` from this function");

                    if let Some(span) = fun.await_in_async_block {
                        diag.span_note(
                            span,
                            "`await` used in an async block, which does not require \
                            the enclosing function to be `async`",
                        );
                    }
                },
            );
        }
    }
}
