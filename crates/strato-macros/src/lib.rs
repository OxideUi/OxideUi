use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Token, Ident, Expr, Lit,
    punctuated::Punctuated,
    bracketed, braced,
    ext::IdentExt
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

        Ok(WidgetNode { name, builder_arg, props, children })
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
        self.root.to_tokens(tokens);
    }
}

impl ToTokens for PropValue {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            PropValue::Node(node) => node.to_tokens(tokens),
            PropValue::Expr(expr) => expr.to_tokens(tokens),
        }
    }
}

impl ToTokens for WidgetNode {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        let props = &self.props;
        
        let mut builder_chain = if let Some(arg) = &self.builder_arg {
             quote! { #name::new(#arg) }
        } else {
             quote! { #name::new() }
        };
        
        for prop in props {
            let prop_name = &prop.name;
            let prop_value = &prop.value;
           
            builder_chain = quote! {
                #builder_chain.#prop_name(#prop_value)
            };
        }

        if let Some(children) = &self.children {
            let children_tokens: Vec<_> = children.iter().map(|child| {
                match child {
                    Child::Node(node) => {
                        quote! { Box::new(#node) }
                    }
                    Child::Expr(expr) => {
                        if let Expr::Lit(syn::ExprLit { lit: Lit::Str(_), .. }) = expr {
                             quote! { Box::new(strato_widgets::prelude::Text::new(#expr)) }
                        } else {
                            quote! { Box::new(#expr) }
                        }
                    }
                }
            }).collect();

            // children() usually takes Vec<Box<dyn Widget>>
            builder_chain = quote! {
                #builder_chain.children(vec![ #(#children_tokens),* ])
            };
        }

        tokens.extend(builder_chain);
    }
}

// --- Macro Entry Point ---

/// Declarative UI definition macro
///
/// Example:
/// ```rust
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
    }.into()
}

/// Derive macro for Widget trait (Placeholder)
#[proc_macro_derive(Widget, attributes(widget))]
pub fn derive_widget(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
