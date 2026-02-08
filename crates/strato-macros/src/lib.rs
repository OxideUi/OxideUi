use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, Ident, Lit, Token,
};

// --- Parsed Structures ---

struct View {
    root: WidgetNode,
}

struct WidgetNode {
    name: Ident,
    builder_arg: Option<Expr>,
    props: Vec<Prop>,
    children: Option<Vec<Child>>,
}

struct Prop {
    name: Ident,
    value: PropValue,
}

enum PropValue {
    Node(WidgetNode),
    Expr(Expr),
}

enum Child {
    Node(WidgetNode),
    Expr(Expr),
}

// --- Parsing Logic ---

impl Parse for View {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let root = input.parse()?;
        Ok(View { root })
    }
}

impl Parse for WidgetNode {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let content;
        braced!(content in input);

        let mut builder_arg = None;
        let mut props = Vec::new();
        let mut children = None;

        if !content.is_empty() {
            let is_key_value = if content.peek(Ident) {
                content.peek2(Token![:])
            } else {
                false
            };

            if !is_key_value {
                let arg: Expr = content.parse()?;
                builder_arg = Some(arg);

                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                }
            }
        }

        while !content.is_empty() {
            if content.peek(Ident) && content.peek2(Token![:]) {
                let key: Ident = content.parse()?;
                content.parse::<Token![:]>()?;

                if key == "children" {
                    let children_content;
                    bracketed!(children_content in content);

                    let parsed_children: Punctuated<Child, Token![,]> =
                        children_content.parse_terminated(Child::parse, Token![,])?;
                    children = Some(parsed_children.into_iter().collect());
                } else {
                    // Parse value: could be WidgetNode (DSL) or Expr
                    let value = if content.peek(Ident) && content.peek2(syn::token::Brace) {
                        let node: WidgetNode = content.parse()?;
                        PropValue::Node(node)
                    } else {
                        let expr: Expr = content.parse()?;
                        PropValue::Expr(expr)
                    };
                    props.push(Prop { name: key, value });
                }

                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                }
            } else {
                return Err(content.error("Expected property or children"));
            }
        }

        Ok(WidgetNode {
            name,
            builder_arg,
            props,
            children,
        })
    }
}

impl Parse for Child {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Ident) && input.peek2(syn::token::Brace) {
            let node: WidgetNode = input.parse()?;
            Ok(Child::Node(node))
        } else {
            let expr: Expr = input.parse()?;
            Ok(Child::Expr(expr))
        }
    }
}

// --- Code Generation ---

impl ToTokens for View {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // Return the root UiNode
        self.root.to_tokens(tokens);
    }
}

impl ToTokens for PropValue {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            PropValue::Node(_) => {
                tokens.extend(
                    quote! { compile_error!("Unexpected nested widget in PropValue generation") },
                );
            }
            PropValue::Expr(expr) => {
                tokens.extend(quote! { strato_core::ui_node::PropValue::from(#expr) });
            }
        }
    }
}

impl ToTokens for WidgetNode {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name_str = self.name.to_string();
        let props = &self.props;

        // Handle props
        let mut prop_tokens = Vec::new();

        // 1. Add constructor arg as "value" or specific prop
        if let Some(arg) = &self.builder_arg {
            // Heuristic: Text/Button -> "text", others -> "value"
            let prop_name = match name_str.as_str() {
                "Text" | "Button" | "Label" => "text",
                "Image" => "source",
                _ => "value",
            };

            prop_tokens.push(quote! {
                (#prop_name.to_string(), strato_core::ui_node::PropValue::from(#arg))
            });
        }

        // 2. Add standard props
        for prop in props {
            let key = prop.name.to_string();
            let value = &prop.value;

            match value {
                PropValue::Node(_node) => {
                    if key == "child" {
                        // Handled in children section
                    } else {
                        // ERROR: Nested widgets in props (other than child) are FORBIDDEN in this pure AST.
                        // We could panic here or emit a compile error.
                        // For now, emit a compile error via quote if possible, or just ignore.
                        // panic!("Nested widgets in properties (except 'child') are not supported in Semantic AST. Found widget in '{}'", key);

                        // Better: prevent compilation
                        let err_msg = format!(
                            "Property '{}' contains a Widget. Widgets can only be children.",
                            key
                        );
                        prop_tokens.push(quote! { compile_error!(#err_msg) });
                    }
                }
                PropValue::Expr(expr) => {
                    prop_tokens.push(quote! {
                        (#key.to_string(), strato_core::ui_node::PropValue::from(#expr))
                    });
                }
            }
        }

        let mut children_tokens = Vec::new();
        // 1. Explicit children from `children: [...]`
        if let Some(children) = &self.children {
            for child in children {
                match child {
                    Child::Node(node) => {
                        children_tokens.push(quote! { #node });
                    }
                    Child::Expr(expr) => {
                        // Heuristic: string literal -> Text node
                        if let Expr::Lit(syn::ExprLit {
                            lit: Lit::Str(_), ..
                        }) = expr
                        {
                            children_tokens.push(
                                quote! { strato_core::ui_node::UiNode::Text(#expr.to_string()) },
                            );
                        } else {
                            // Dynamic expression? We can't easily turn it into UiNode unless it IS a UiNode.
                            // Assuming expression evaluates to UiNode.
                            children_tokens.push(quote! { #expr });
                        }
                    }
                }
            }
        }

        // 2. "child" prop moved to children
        for prop in props {
            if prop.name == "child" {
                if let PropValue::Node(node) = &prop.value {
                    children_tokens.push(quote! { #node });
                }
            }
        }

        tokens.extend(quote! {
            strato_core::ui_node::UiNode::Widget(strato_core::ui_node::WidgetNode {
                name: #name_str.to_string(),
                props: vec![ #(#prop_tokens),* ],
                children: vec![ #(#children_tokens),* ],
            })
        });
    }
}

// --- Macro Entry Point ---

/// Declarative UI definition macro
///
/// ```rust,ignore
/// use strato_macros::view;
///
/// view! {
///     Column {
///         spacing: 10.0,
///         children: [
///             Text { "Hello" },
///             Button { child: Text { "Click" } }
///         ]
///     }
/// }
/// ```
#[proc_macro]
pub fn view(input: TokenStream) -> TokenStream {
    let view_def = parse_macro_input!(input as View);
    quote! {
        {
            use strato_widgets::prelude::*;
            #view_def
        }
    }
    .into()
}

/// Derive macro for Widget trait (Placeholder)
#[proc_macro_derive(Widget, attributes(widget))]
pub fn derive_widget(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
