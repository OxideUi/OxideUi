//! Procedural macros for OxideUI framework

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

/// Derive macro for Widget trait
#[proc_macro_derive(Widget, attributes(widget))]
pub fn derive_widget(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    let expanded = quote! {
        impl oxide_widgets::widget::Widget for #name {
            fn id(&self) -> oxide_widgets::widget::WidgetId {
                self.id
            }

            fn layout(&mut self, constraints: oxide_core::layout::Constraints) -> oxide_core::layout::Size {
                self.base.calculate_size(constraints)
            }

            fn render(&self, batch: &mut oxide_renderer::batch::RenderBatch, layout: oxide_core::layout::Layout) {
                // Default implementation
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            fn clone_widget(&self) -> Box<dyn oxide_widgets::widget::Widget> {
                Box::new(self.clone())
            }
        }
    };

    TokenStream::from(expanded)
}

/// Macro for creating widget trees declaratively
#[proc_macro]
pub fn ui(_input: TokenStream) -> TokenStream {
    // Parse the UI tree and generate widget construction code
    // This is a simplified version - a real implementation would parse a DSL
    
    let expanded = quote! {
        {
            use oxide_widgets::prelude::*;
            Container::new()
        }
    };

    TokenStream::from(expanded)
}

/// State management macro
#[proc_macro]
pub fn state(input: TokenStream) -> TokenStream {
    let input = input.to_string();
    let parts: Vec<&str> = input.split('=').collect();
    
    if parts.len() != 2 {
        panic!("state! macro expects format: state!(name = initial_value)");
    }

    let name = parts[0].trim();
    let initial = parts[1].trim();
    
    let name_ident = syn::parse_str::<Ident>(name).expect("Invalid identifier");
    
    let expanded = quote! {
        {
            use oxide_core::state::Signal;
            let #name_ident = Signal::new(#initial);
            #name_ident
        }
    };

    TokenStream::from(expanded)
}

/// Component macro for defining reusable components
#[proc_macro]
pub fn component(_input: TokenStream) -> TokenStream {
    // Parse component definition
    // This is a simplified placeholder
    
    let expanded = quote! {
        pub struct Component {
            id: oxide_widgets::widget::WidgetId,
            children: Vec<Box<dyn oxide_widgets::widget::Widget>>,
        }

        impl Component {
            pub fn new() -> Self {
                Self {
                    id: oxide_widgets::widget::generate_id(),
                    children: Vec::new(),
                }
            }
        }
    };

    TokenStream::from(expanded)
}

/// Style macro for defining widget styles
#[proc_macro]
pub fn style(_input: TokenStream) -> TokenStream {
    // Parse style definitions
    // This would parse CSS-like syntax
    
    let expanded = quote! {
        {
            use oxide_widgets::theme::*;
            StyleBuilder::new()
                .build()
        }
    };

    TokenStream::from(expanded)
}

/// Event handler macro
#[proc_macro]
pub fn on(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    let parts: Vec<&str> = input_str.split("=>").collect();
    
    if parts.len() != 2 {
        panic!("on! macro expects format: on!(event => handler)");
    }

    let _event = parts[0].trim();
    let handler = parts[1].trim();
    
    let expanded = quote! {
        {
            Box::new(move |e| {
                #handler
            })
        }
    };

    TokenStream::from(expanded)
}

/// Computed value macro
#[proc_macro]
pub fn computed(input: TokenStream) -> TokenStream {
    // Convert to proc_macro2::TokenStream so it implements ToTokens
    let input2: proc_macro2::TokenStream = input.into();
    let expanded = quote! {
        {
            use oxide_core::reactive::Computed;
            Computed::new(move || {
                #input2
            })
        }
    };

    TokenStream::from(expanded)
}

/// Effect macro for side effects
#[proc_macro]
pub fn effect(input: TokenStream) -> TokenStream {
    // Convert to proc_macro2::TokenStream so it implements ToTokens
    let input2: proc_macro2::TokenStream = input.into();
    let expanded = quote! {
        {
            use oxide_core::reactive::Effect;
            Effect::new(move || {
                #input2
            })
        }
    };

    TokenStream::from(expanded)
}

/// Watch macro for watching value changes
#[proc_macro]
pub fn watch(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    let parts: Vec<&str> = input_str.split("=>").collect();
    
    if parts.len() != 2 {
        panic!("watch! macro expects format: watch!(value => handler)");
    }

    let value = parts[0].trim();
    let handler = parts[1].trim();
    
    let expanded = quote! {
        {
            #value.on_change(move |val| {
                #handler
            })
        }
    };

    TokenStream::from(expanded)
}
