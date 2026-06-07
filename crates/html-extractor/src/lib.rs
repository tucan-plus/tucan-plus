use itertools::Itertools;
use proc_macro2::{Delimiter, Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned};
use syn::{
    Block, Expr, ExprClosure, Ident, LitStr, Token, braced,
    ext::IdentExt,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    token::Brace,
};

#[derive(Debug)]
struct HtmlCommands {
    commands: Vec<HtmlCommand>,
}

impl HtmlCommands {
    fn span(&self) -> Option<Span> {
        self.commands
            .iter()
            .map(HtmlCommand::span)
            .reduce(|a, b| a.join(b).unwrap_or(a))
    }
}

impl Parse for HtmlCommands {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut commands = Vec::new();
        while !input.is_empty() {
            commands.push(input.parse()?);
        }
        Ok(Self { commands })
    }
}

#[derive(Debug)]
enum HtmlCommand {
    ElementOpen(HtmlElement),
    Whitespace(HtmlWhitespace),
    ElementClose(HtmlElementClose),
    Comment(HtmlComment),
    Text(StringLiteralOrVariable),
    Use(HtmlUse),
    Extern(HtmlExtern),
    Let(Box<HtmlLet>),
}

impl HtmlCommand {
    pub fn span(&self) -> Span {
        match self {
            Self::ElementOpen(html_element) => html_element.span(),
            Self::Whitespace(html_whitespace) => html_whitespace.span(),
            Self::ElementClose(html_element_close) => html_element_close.span(),
            Self::Comment(html_comment) => html_comment.span(),
            Self::Text(string_literal_or_variable) => string_literal_or_variable.span(),
            Self::Use(expr) => expr.span(),
            Self::Extern(block) => block.span(),
            Self::Let(html_let) => html_let.span(),
        }
    }
}

impl Parse for HtmlCommand {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![let]) {
            input.parse().map(Self::Let)
        } else if lookahead.peek(Token![use]) {
            input.parse().map(Self::Use)
        } else if lookahead.peek(Token![extern]) {
            input.parse().map(Self::Extern)
        } else if lookahead.peek(Brace) || lookahead.peek(LitStr) {
            input.parse().map(Self::Text)
        } else if lookahead.peek(Token![_]) {
            input.parse().map(Self::Whitespace)
        } else if lookahead.peek(Ident::peek_any) {
            input.parse().map(Self::Text)
        } else if lookahead.peek(Token![<]) {
            if input.peek2(Token![/]) {
                input.parse().map(Self::ElementClose)
            } else if input.peek2(Token![!]) {
                input.parse().map(Self::Comment)
            } else {
                input.parse().map(Self::ElementOpen)
            }
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug)]
struct HtmlExtern {
    extern_: Token![extern],
    block: Block,
}

impl HtmlExtern {
    pub fn span(&self) -> Span {
        self.extern_
            .span()
            .join(self.block.span())
            .unwrap_or_else(|| self.extern_.span())
    }
}

impl Parse for HtmlExtern {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let extern_ = input.parse::<Token![extern]>()?;
        let block = input.parse()?;
        Ok(Self { extern_, block })
    }
}

#[derive(Debug)]
struct HtmlUse {
    use_: Token![use],
    expr: Expr,
    semi: Token![;],
}

impl HtmlUse {
    pub fn span(&self) -> Span {
        self.use_
            .span()
            .join(self.expr.span())
            .and_then(|v| v.join(self.semi.span()))
            .unwrap_or_else(|| self.use_.span())
    }
}

impl Parse for HtmlUse {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let use_ = input.parse::<Token![use]>()?;
        let expr = input.parse()?;
        let semi = input.parse::<Token![;]>()?;
        Ok(Self { use_, expr, semi })
    }
}

#[derive(Debug)]
struct HtmlLet {
    let_: Token![let],
    variable: Ident,
    eq: Token![=],
    inner: HtmlLetInner,
    semi: Token![;],
}

impl HtmlLet {
    pub fn span(&self) -> Span {
        self.let_
            .span()
            .join(self.variable.span())
            .and_then(|v| v.join(self.eq.span()))
            .and_then(|v| v.join(self.inner.span()))
            .and_then(|v| v.join(self.semi.span()))
            .unwrap_or_else(|| self.let_.span())
    }
}

impl Parse for HtmlLet {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let let_ = input.parse::<Token![let]>()?;
        let variable = input.parse()?;
        let eq = input.parse::<Token![=]>()?;
        let inner = input.parse()?;
        let semi = input.parse::<Token![;]>()?;
        Ok(Self {
            let_,
            variable,
            eq,
            inner,
            semi,
        })
    }
}

#[derive(Debug)]
enum HtmlLetInner {
    Expr(Expr),
    If(HtmlIf),
    While(HtmlWhile),
}

impl HtmlLetInner {
    pub fn span(&self) -> Span {
        match self {
            Self::Expr(expr) => expr.span(),
            Self::If(html_if) => html_if.span(),
            Self::While(html_while) => html_while.span(),
        }
    }
}

impl Parse for HtmlLetInner {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![if]) {
            input.parse().map(Self::If)
        } else if input.peek(Token![while]) {
            input.parse().map(Self::While)
        } else {
            input.parse().map(Self::Expr)
        }
    }
}

#[derive(Debug)]
struct HtmlWhitespace {
    underscore: Token![_],
}

impl HtmlWhitespace {
    pub fn span(&self) -> Span {
        self.underscore.span()
    }
}

impl Parse for HtmlWhitespace {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            underscore: input.parse()?,
        })
    }
}

#[derive(Debug)]
enum DashOrColon {
    Dash,
    Colon,
}

impl Parse for DashOrColon {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![-]) {
            input.parse().map(|_: Token![-]| Self::Dash)
        } else if lookahead.peek(Token![:]) {
            input.parse().map(|_: Token![:]| Self::Colon)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug)]
enum StringLiteralOrVariable {
    Literal(LitStr),
    Variable(Ident),
    Clousure(Brace, Box<ExprClosure>),
}

impl StringLiteralOrVariable {
    pub fn span(&self) -> Span {
        match self {
            Self::Literal(lit_str) => lit_str.span(),
            Self::Variable(ident) => ident.span(),
            Self::Clousure(brace, _expr) => brace.span.join(),
        }
    }
}

impl Parse for StringLiteralOrVariable {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Brace) {
            let content;
            let brace_token = braced!(content in input);
            content.parse().map(|b| Self::Clousure(brace_token, b))
        } else if lookahead.peek(LitStr) {
            input.parse().map(Self::Literal)
        } else if lookahead.peek(Ident::peek_any) {
            input.call(Ident::parse_any).map(Self::Variable)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug)]
struct HtmlAttribute {
    ident: Punctuated<Ident, DashOrColon>,
    eq: Token![=],
    value: StringLiteralOrVariable,
}

impl HtmlAttribute {
    fn span(&self) -> Span {
        let span = self
            .ident
            .iter()
            .map(proc_macro2::Ident::span)
            .reduce(|a, b| a.join(b).unwrap_or(a))
            .unwrap();
        span.join(self.eq.span())
            .and_then(|a| a.join(self.value.span()))
            .unwrap_or(span)
    }
}

impl Parse for HtmlAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut ident: Punctuated<Ident, DashOrColon> = Punctuated::new();
        ident.push_value(input.call(Ident::parse_any)?);
        while input.peek(Token![-]) || input.peek(Token![:]) {
            ident.push_punct(input.parse()?);
            ident.push_value(input.parse()?);
        }
        let eq = input.parse::<Token![=]>()?;
        let value = input.parse()?;
        Ok(Self { ident, eq, value })
    }
}

#[derive(Debug)]
struct HtmlElement {
    open_start: Token![<],
    element: Ident,
    attributes: Vec<HtmlAttribute>,
    open_end: Token![>],
}

impl HtmlElement {
    pub fn span(&self) -> Span {
        let attrspan = self
            .attributes
            .iter()
            .map(HtmlAttribute::span)
            .reduce(|a, b| a.join(b).unwrap_or(a));
        self.open_start
            .span()
            .join(self.element.span())
            .map(|s| attrspan.and_then(|attrspan| s.join(attrspan)).unwrap_or(s))
            .and_then(|s| s.join(self.open_end.span()))
            .unwrap_or_else(|| self.open_start.span())
    }
}

impl Parse for HtmlElement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let open_start = input.parse::<Token![<]>()?;
        let element = input.call(Ident::parse_any)?;
        let mut attributes = Vec::new();
        while !input.peek(Token!(>)) {
            attributes.push(input.parse()?);
        }
        let open_end = input.parse()?;
        Ok(Self {
            open_start,
            element,
            attributes,
            open_end,
        })
    }
}

#[derive(Debug)]
struct HtmlElementClose {
    close_start: Token![<],
    close_slash: Token![/],
    element: Ident,
    close_end: Token![>],
}

impl HtmlElementClose {
    pub fn span(&self) -> Span {
        self.close_start
            .span()
            .join(self.close_slash.span())
            .and_then(|s| s.join(self.element.span()))
            .and_then(|s| s.join(self.close_end.span()))
            .unwrap_or_else(|| self.close_start.span())
    }
}

impl Parse for HtmlElementClose {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let close_start = input.parse::<Token![<]>()?;
        let close_slash = input.parse::<Token![/]>()?;
        let element = input.call(Ident::parse_any)?;
        let close_end = input.parse::<Token![>]>()?;
        Ok(Self {
            close_start,
            close_slash,
            element,
            close_end,
        })
    }
}

#[derive(Debug)]
struct HtmlComment {
    token1: Token![<],
    token2: Token![!],
    token3: Token![-],
    token4: Token![-],
    comment: LitStr,
    token5: Token![-],
    token6: Token![-],
    token7: Token![>],
}

impl HtmlComment {
    pub fn span(&self) -> Span {
        [
            self.token1.span(),
            self.token2.span(),
            self.token3.span(),
            self.token4.span(),
            self.comment.span(),
            self.token5.span(),
            self.token6.span(),
            self.token7.span(),
        ]
        .into_iter()
        .reduce(|a, b| a.join(b).unwrap_or(a))
        .unwrap()
    }
}

impl Parse for HtmlComment {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let token1 = input.parse::<Token![<]>()?;
        let token2 = input.parse::<Token![!]>()?;
        let token3 = input.parse::<Token![-]>()?;
        let token4 = input.parse::<Token![-]>()?;
        let comment: LitStr = input.parse()?;
        let token5 = input.parse::<Token![-]>()?;
        let token6 = input.parse::<Token![-]>()?;
        let token7 = input.parse::<Token![>]>()?;
        Ok(Self {
            token1,
            token2,
            token3,
            token4,
            comment,
            token5,
            token6,
            token7,
        })
    }
}

#[derive(Debug)]
struct HtmlIf {
    if_: Token![if],
    conditional: TokenStream,
    brace_token: Brace,
    body: HtmlCommands,
    eq: Token![=],
    gt: Token![>],
    result_expr: TokenStream,
    else_: Option<HtmlElse>,
}

#[derive(Debug)]
struct HtmlElse {
    else_: Token![else],
    brace_token: Brace,
    body: HtmlCommands,
    eq: Token![=],
    gt: Token![>],
    result_expr: TokenStream,
}

impl HtmlElse {
    pub fn span(&self) -> Span {
        [
            Some(self.else_.span()),
            Some(self.brace_token.span.span()),
            self.body.span(),
            Some(self.eq.span()),
            Some(self.gt.span()),
            Some(self.result_expr.span()),
        ]
        .into_iter()
        .flatten()
        .reduce(|a, b| a.join(b).unwrap_or(a))
        .unwrap()
    }
}

impl HtmlIf {
    pub fn span(&self) -> Span {
        [
            Some(self.if_.span()),
            Some(self.conditional.span()),
            Some(self.brace_token.span.span()),
            self.body.span(),
            Some(self.eq.span()),
            Some(self.eq.span()),
            Some(self.gt.span()),
            Some(self.result_expr.span()),
            self.else_.as_ref().map(HtmlElse::span),
        ]
        .into_iter()
        .flatten()
        .reduce(|a, b| a.join(b).unwrap_or(a))
        .unwrap()
    }
}

impl Parse for HtmlIf {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let if_ = input.parse::<Token![if]>()?;
        let conditional = input
            .step(|cursor| {
                let mut trees = vec![];
                let mut rest = *cursor;
                while let Some((tt, next)) = rest.token_tree() {
                    match &tt {
                        TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => {
                            return Ok((trees, rest));
                        }
                        _ => {
                            trees.push(tt);
                            rest = next;
                        }
                    }
                }
                Err(cursor.error("no `;` or `else` was found after this point"))
            })?
            .into_iter()
            .collect();
        let body_parse_buffer;
        let brace_token = braced!(body_parse_buffer in input);
        let body = body_parse_buffer.parse()?;
        let eq = input.parse::<Token![=]>()?;
        let gt = input.parse::<Token![>]>()?;
        let result_expr: TokenStream = input
            .step(|cursor| {
                let mut trees = vec![];
                let mut rest = *cursor;
                while let Some((tt, next)) = rest.token_tree() {
                    match &tt {
                        TokenTree::Punct(punct) if punct.as_char() == ';' => {
                            return Ok((trees, rest));
                        }
                        TokenTree::Ident(ident) if ident == "else" => {
                            return Ok((trees, rest));
                        }
                        _ => {
                            trees.push(tt);
                            rest = next;
                        }
                    }
                }
                Err(cursor.error("no `;` or `else` was found after this point"))
            })?
            .into_iter()
            .collect();
        let else_ = if input.peek(Token![else]) {
            let else_ = input.parse::<Token![else]>()?;
            let body_parse_buffer;
            let brace_token = braced!(body_parse_buffer in input);
            let body = body_parse_buffer.parse()?;
            let eq = input.parse::<Token![=]>()?;
            let gt = input.parse::<Token![>]>()?;
            let result_expr: TokenStream = input
                .step(|cursor| {
                    let mut trees = vec![];
                    let mut rest = *cursor;
                    while let Some((tt, next)) = rest.token_tree() {
                        match &tt {
                            TokenTree::Punct(punct) if punct.as_char() == ';' => {
                                return Ok((trees, rest));
                            }
                            _ => {
                                trees.push(tt);
                                rest = next;
                            }
                        }
                    }
                    Err(cursor.error("no `;` or `else` was found after this point"))
                })?
                .into_iter()
                .collect();
            Some(HtmlElse {
                else_,
                brace_token,
                body,
                eq,
                gt,
                result_expr,
            })
        } else {
            None
        };
        Ok(Self {
            if_,
            conditional,
            brace_token,
            body,
            eq,
            gt,
            result_expr,
            else_,
        })
    }
}

#[derive(Debug)]
struct HtmlWhile {
    while_: Token![while],
    conditional: TokenStream,
    brace_token: Brace,
    body: HtmlCommands,
    eq: Token![=],
    gt: Token![>],
    result_expr: TokenStream,
}

impl HtmlWhile {
    pub fn span(&self) -> Span {
        [
            Some(self.while_.span()),
            Some(self.conditional.span()),
            Some(self.brace_token.span.span()),
            self.body.span(),
            Some(self.eq.span()),
            Some(self.gt.span()),
            Some(self.result_expr.span()),
        ]
        .into_iter()
        .flatten()
        .reduce(|a, b| a.join(b).unwrap_or(a))
        .unwrap()
    }
}

impl Parse for HtmlWhile {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let while_ = input.parse::<Token![while]>()?;
        let conditional = input
            .step(|cursor| {
                let mut trees = vec![];
                let mut rest = *cursor;
                while let Some((tt, next)) = rest.token_tree() {
                    match &tt {
                        TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => {
                            return Ok((trees, rest));
                        }
                        _ => {
                            trees.push(tt);
                            rest = next;
                        }
                    }
                }
                Err(cursor.error("no `;` or `else` was found after this point"))
            })?
            .into_iter()
            .collect();
        let body_parse_buffer;
        let brace_token = braced!(body_parse_buffer in input);
        let body = body_parse_buffer.parse()?;
        let eq = input.parse::<Token![=]>()?;
        let gt = input.parse::<Token![>]>()?;
        let result_expr: TokenStream = input
            .step(|cursor| {
                let mut trees = vec![];
                let mut rest = *cursor;
                while let Some((tt, next)) = rest.token_tree() {
                    match &tt {
                        TokenTree::Punct(punct) if punct.as_char() == ';' => {
                            return Ok((trees, rest));
                        }
                        _ => {
                            trees.push(tt);
                            rest = next;
                        }
                    }
                }
                Err(cursor.error("no `;` or `else` was found after this point"))
            })?
            .into_iter()
            .collect();
        Ok(Self {
            while_,
            conditional,
            brace_token,
            body,
            eq,
            gt,
            result_expr,
        })
    }
}

fn convert_commands(commands: &HtmlCommands) -> Vec<TokenStream> {
    commands.commands.iter().map(convert_command).collect()
}

#[allow(clippy::too_many_lines)]
fn convert_command(command: &HtmlCommand) -> TokenStream {
    match command {
        HtmlCommand::ElementOpen(input) => {
            let tag = input.element.to_string();

            let attributes = input.attributes.iter().map(|iter| {
                let name = iter
                    .ident
                    .pairs()
                    .map(|p| {
                        p.value().to_string()
                            + match p.punct() {
                                Some(DashOrColon::Colon) => ":",
                                Some(DashOrColon::Dash) => "-",
                                None => "",
                            }
                    })
                    .join("");
                let value = &iter.value;
                match value {
                    StringLiteralOrVariable::Literal(lit_str) => {
                        quote_spanned! {lit_str.span()=>
                            #[allow(unused_mut)]
                            let mut html_handler = html_handler.attribute(#name, #lit_str);
                        }
                    }
                    StringLiteralOrVariable::Clousure(_brace, expr) => {
                        quote_spanned! {expr.span()=>
                            #[allow(unused_mut)]
                            let (mut html_handler, tmp_internal_html_extractor_proc_macro_2) =
                                html_handler.attribute_value(#name);
                            #[allow(clippy::redundant_closure_call)]
                            (#expr)(tmp_internal_html_extractor_proc_macro_2);
                        }
                    }
                    StringLiteralOrVariable::Variable(ident) => {
                        quote_spanned! {ident.span()=>
                            #[allow(unused_mut)]
                            let (mut html_handler, #ident) =
                                html_handler.attribute_value(#name);
                        }
                    }
                }
            });

            let open = quote_spanned! {input.open_start.span()=>
                #[allow(unused_mut)]
                let mut html_handler = html_handler.next_child_tag_open_start(#tag);
            };

            let close = quote_spanned! {input.open_end.span()=>
                #[allow(unused_mut)]
                let mut html_handler = html_handler.tag_open_end();
            };

            quote! {
                #open
                #(
                    #attributes
                )*
                #close
            }
        }
        HtmlCommand::Whitespace(html_whitespace) => {
            quote_spanned! {html_whitespace.underscore.span()=>
                #[allow(unused_mut)]
                let mut html_handler = html_handler.skip_whitespace();
            }
        }
        HtmlCommand::ElementClose(html_element_close) => {
            let name = html_element_close.element.to_string();
            quote_spanned! {html_element_close.close_start.span()=>
                #[allow(unused_mut)]
                let mut html_handler = html_handler.close_element(#name);
            }
        }
        HtmlCommand::Comment(_html_comment) => {
            quote! {}
        }
        HtmlCommand::Text(html_text) => match html_text {
            StringLiteralOrVariable::Literal(lit_str) => {
                quote_spanned! {lit_str.span()=>
                    #[allow(unused_mut)]
                    let mut html_handler = html_handler.skip_text(#lit_str);
                }
            }
            StringLiteralOrVariable::Clousure(_brace, expr) => {
                quote_spanned! {expr.span()=>
                    #[allow(unused_mut)]
                    let (mut html_handler, tmp_internal_html_extractor_proc_macro_2) =
                        html_handler.text();
                    #[allow(clippy::redundant_closure_call)]
                    (#expr)(tmp_internal_html_extractor_proc_macro_2);
                }
            }
            StringLiteralOrVariable::Variable(ident) => {
                quote_spanned! {ident.span()=>
                    #[allow(unused_mut)]
                    let (mut html_handler, #ident) = html_handler.text();
                }
            }
        },
        HtmlCommand::Use(HtmlUse {
            use_: _,
            expr,
            semi,
        }) => {
            quote! {
                #[allow(unused_mut)]
                let mut html_handler = #expr #semi
            }
        }
        HtmlCommand::Extern(HtmlExtern { extern_: _, block }) => {
            let body = &block.stmts;
            quote! {
                #(#body)*
            }
        }
        HtmlCommand::Let(html_let) => {
            let variable = &html_let.variable;
            match &html_let.inner {
                HtmlLetInner::Expr(expr) => {
                    quote_spanned! {expr.span()=>
                        #[allow(unused_mut)]
                        let (mut html_handler, #variable) = #expr;
                    }
                }
                HtmlLetInner::If(HtmlIf {
                    if_,
                    conditional,
                    brace_token,
                    body,
                    eq: _,
                    gt: _,
                    result_expr,
                    else_,
                }) => {
                    let body_stmts = convert_commands(body);
                    let temp_var = Ident::new("temp_var", Span::mixed_site());
                    else_.as_ref().map_or_else(
                        || {
                            quote! {
                                #[allow(unused_mut, clippy::if_not_else)]
                                let (mut html_handler, #variable) = if #conditional {
                                    #(#body_stmts)*
                                    (html_handler, Some(#result_expr))
                                } else {
                                    (html_handler, None)
                                };
                            }
                        },
                        |HtmlElse {
                             else_,
                             brace_token: else_brace_token,
                             body: else_body,
                             eq: _,
                             gt: _,
                             result_expr: else_result_expr,
                         }| {
                            let else_body_stmts = convert_commands(else_body);
                            let if_inner = quote_spanned! {brace_token.span.span()
                                                            .join(result_expr.span())
                                                            .unwrap_or_else(|| brace_token.span.span())=>
                                {
                                    #(#body_stmts)*
                                    (html_handler, ::itertools::Either::Left(#result_expr))
                                }
                            };
                            let else_inner = quote_spanned! {else_brace_token.span.span()
                                                                .join(else_result_expr.span())
                                                                .unwrap_or_else(|| else_brace_token.span.span())=>
                                {
                                    #(#else_body_stmts)*
                                    (html_handler, ::itertools::Either::Right(#else_result_expr))
                                }
                            };
                            quote! {
                                #[allow(unused_mut,
                                        clippy::suspicious_else_formatting,
                                        clippy::branches_sharing_code)]
                                let (mut html_handler, #temp_var) = #if_ #conditional
                                    #if_inner
                                #else_
                                    #else_inner
                                ;
                                let #variable = #temp_var;
                            }
                        },
                    )
                }
                HtmlLetInner::While(html_while) => {
                    let conditional = &html_while.conditional;
                    let body = convert_commands(&html_while.body);
                    let result_expr = &html_while.result_expr;
                    let temp_vec = Ident::new("temp_vec", Span::mixed_site());
                    quote_spanned! {html_while.body.span().unwrap_or_else(|| html_while.brace_token.span.span())=>
                        let mut #temp_vec = Vec::new();
                        while #conditional {
                            html_handler = {
                                let (html_handler, tmp) = {
                                    #(#body)*

                                    (html_handler, #result_expr)
                                };
                                #temp_vec.push(tmp);
                                html_handler
                            };
                        }
                        #[allow(unused_mut)]
                        let mut #variable = #temp_vec;
                    }
                }
            }
        }
    }
}

#[proc_macro]
pub fn html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as HtmlCommands);

    let expanded = convert_commands(&input);
    let result = quote! {
        #(#expanded)*
    };

    proc_macro::TokenStream::from(result)
}
