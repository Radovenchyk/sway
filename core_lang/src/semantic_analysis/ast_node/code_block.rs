use super::*;
use crate::semantic_analysis::{ast_node::Mode, TypeCheckArguments};
use crate::CodeBlock;

#[derive(Clone, Debug)]
pub(crate) struct TypedCodeBlock<'sc> {
    pub(crate) contents: Vec<TypedAstNode<'sc>>,
    pub(crate) whole_block_span: Span<'sc>,
}

#[allow(clippy::too_many_arguments)]
impl<'sc> TypedCodeBlock<'sc> {
    pub(crate) fn type_check<'n>(
        arguments: TypeCheckArguments<'n, 'sc, CodeBlock<'sc>>,
    ) -> CompileResult<'sc, (Self, TypeId)> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let TypeCheckArguments {
            checkee: other,
            namespace,
            crate_namespace,
            return_type_annotation: type_annotation,
            help_text,
            self_type,
            build_config,
            dead_code_graph,
            dependency_graph,
            opts,
            ..
        } = arguments;

        // Mutable clone, because the interior of a code block must not change the surrounding
        // namespace.
        let mut local_namespace = namespace.clone();
        let evaluated_contents = other
            .contents
            .iter()
            .filter_map(|node| {
                TypedAstNode::type_check(TypeCheckArguments {
                    checkee: node.clone(),
                    namespace: &mut local_namespace,
                    crate_namespace,
                    return_type_annotation: type_annotation,
                    help_text: help_text,
                    self_type,
                    build_config,
                    dead_code_graph,
                    dependency_graph,
                    mode: Mode::NonAbi,
                    opts,
                })
                .ok(&mut warnings, &mut errors)
            })
            .collect::<Vec<TypedAstNode<'sc>>>();

        let implicit_return_span = other
            .contents
            .iter()
            .find_map(|x| match &x.content {
                AstNodeContent::ImplicitReturnExpression(expr) => Some(Some(expr.span())),
                _ => None,
            })
            .flatten();

        // find the implicit return, if any, and use it as the code block's return type.
        // The fact that there is at most one implicit return is an invariant held by the parser.
        let return_type = evaluated_contents.iter().find_map(|x| match x {
            TypedAstNode {
                content:
                    TypedAstNodeContent::ImplicitReturnExpression(TypedExpression {
                        ref return_type,
                        ..
                    }),
                ..
            } => Some(*return_type),
            _ => None,
        });

        if let Some(return_type) = return_type {
            match crate::type_engine::unify_with_self(
                return_type,
                type_annotation,
                self_type,
                &implicit_return_span
                    .clone()
                    .unwrap_or_else(|| other.whole_block_span.clone()),
            ) {
                Ok(mut ws) => {
                    warnings.append(&mut ws);
                }
                Err(e) => {
                    errors.push(CompileError::TypeError(e));
                }
            };
            // The annotation will result in a cast, so set the return type accordingly.
        }

        ok(
            (
                TypedCodeBlock {
                    contents: evaluated_contents,
                    whole_block_span: other.whole_block_span,
                },
                return_type.unwrap_or_else(|| crate::type_engine::insert_type(TypeInfo::Unit)),
            ),
            warnings,
            errors,
        )
    }

    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.contents
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
    }
}
