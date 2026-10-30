//! CSS length / calculation values.
//!
//! `UnitValue` models both simple `(number, unit)` lengths and the four
//! CSS math functions: `calc()`, `min()`, `max()`, `clamp()`. Resolution
//! context (`parent_value`, `root_font_size`, `current_font_size`,
//! viewport dimensions) is stored once on the outer `UnitValue` and used
//! when the tree is evaluated with `to_px()`.

use crate::stylesheet::types::Unit;
use euclid::Length as EuclidLength;
use freya::prelude::Size;

const DEFAULT_ROOT_FONT_SIZE: f32 = 16.0;

/// CSS reference pixel: `1in == 96px`.
const PX_PER_INCH: f32 = 96.0;

/// Rough font-metric approximations for `ex` / `ch`.
const EX_RATIO: f32 = 0.5;
const CH_RATIO: f32 = 0.5;

// ============================================================================
// AST
// ============================================================================

/// Resolution context for a `UnitValue`. Cheap to `Copy`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolveContext {
    pub parent_value: f32,
    pub root_font_size: f32,
    pub current_font_size: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
}

impl ResolveContext {
    pub const DEFAULT: Self = Self {
        parent_value: 0.0,
        root_font_size: DEFAULT_ROOT_FONT_SIZE,
        current_font_size: DEFAULT_ROOT_FONT_SIZE,
        viewport_width: 0.0,
        viewport_height: 0.0,
    };
}

impl Default for ResolveContext {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// A top-level value: either a simple `(value, unit)`, a `calc()` tree, or
/// one of the `min` / `max` / `clamp` functions.
#[derive(Debug, Clone, PartialEq)]
pub enum UnitExpr {
    /// `"16px"`, `"1.5rem"`, `"50%"` — a bare number + unit.
    Simple(f32, Unit),
    /// `calc(<expr>)`.
    Calc(CalcExpr),
    /// `min(a, b, ...)`.
    Min(Vec<UnitExpr>),
    /// `max(a, b, ...)`.
    Max(Vec<UnitExpr>),
    /// `clamp(min, value, max)`.
    Clamp(Box<UnitExpr>, Box<UnitExpr>, Box<UnitExpr>),
}

/// The AST for a single `calc()` expression.
#[derive(Debug, Clone, PartialEq)]
pub enum CalcExpr {
    /// `10px`, `2rem`, `50%` — number + unit.
    Term(f32, Unit),
    /// A bare number (only meaningful inside `*` / `/`).
    Number(f32),
    /// A `min()` / `max()` / `clamp()` result embedded in a larger expression.
    Nested(Box<UnitExpr>),
    Add(Box<CalcExpr>, Box<CalcExpr>),
    Sub(Box<CalcExpr>, Box<CalcExpr>),
    Mul(Box<CalcExpr>, Box<CalcExpr>),
    Div(Box<CalcExpr>, Box<CalcExpr>),
    Paren(Box<CalcExpr>),
}

// ============================================================================
// The value
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct UnitValue {
    pub expr: UnitExpr,
    pub ctx: ResolveContext,
}

impl Default for UnitValue {
    fn default() -> Self {
        Self::zero()
    }
}

// ============================================================================
// Construction
// ============================================================================

impl UnitValue {
    /// Parse a CSS length or math expression.
    ///
    /// Accepts: `"16px"`, `"1.5rem"`, `"50%"`, `"100vw"`,
    /// `"calc(100% - 20px)"`, `"min(100%, 800px)"`,
    /// `"clamp(1rem, 2.5vw, 2rem)"`, etc.
    ///
    /// On parse failure returns `0px`. Surrounding whitespace is trimmed.
    pub fn new(s: impl AsRef<str>) -> Self {
        let s = s.as_ref().trim();
        let expr = Parser::new(s)
            .parse_full()
            .unwrap_or(UnitExpr::Simple(0.0, Unit::Px));
        Self {
            expr,
            ctx: ResolveContext::DEFAULT,
        }
    }

    pub const fn zero() -> Self {
        Self {
            expr: UnitExpr::Simple(0.0, Unit::Px),
            ctx: ResolveContext::DEFAULT,
        }
    }

    pub const fn px(v: f32) -> Self {
        Self::from_parts(v, Unit::Px)
    }

    pub const fn em(v: f32) -> Self {
        Self::from_parts(v, Unit::Em)
    }

    pub const fn rem(v: f32) -> Self {
        Self::from_parts(v, Unit::Rem)
    }

    pub const fn percent(v: f32) -> Self {
        Self::from_parts(v, Unit::Percent)
    }

    pub const fn vw(v: f32) -> Self {
        Self::from_parts(v, Unit::Vw)
    }

    pub const fn vh(v: f32) -> Self {
        Self::from_parts(v, Unit::Vh)
    }

    pub const fn vmin(v: f32) -> Self {
        Self::from_parts(v, Unit::Vmin)
    }

    pub const fn vmax(v: f32) -> Self {
        Self::from_parts(v, Unit::Vmax)
    }

    const fn from_parts(value: f32, unit: Unit) -> Self {
        Self {
            expr: UnitExpr::Simple(value, unit),
            ctx: ResolveContext::DEFAULT,
        }
    }
}

// ============================================================================
// Builders
// ============================================================================

impl UnitValue {
    /// Set parent value (for `%`), root font size (`rem`), and current font
    /// size (`em` / `ex` / `ch`).
    pub fn with_context(
        mut self,
        parent_value: f32,
        root_font_size: f32,
        current_font_size: f32,
    ) -> Self {
        self.ctx.parent_value = parent_value;
        self.ctx.root_font_size = root_font_size;
        self.ctx.current_font_size = current_font_size;
        self
    }

    /// Set viewport dimensions (for `vw` / `vh` / `vmin` / `vmax`).
    pub fn with_viewport(mut self, width: f32, height: f32) -> Self {
        self.ctx.viewport_width = width;
        self.ctx.viewport_height = height;
        self
    }

    pub fn with_full_context(
        self,
        parent_value: f32,
        root_font_size: f32,
        current_font_size: f32,
        viewport_width: f32,
        viewport_height: f32,
    ) -> Self {
        self.with_context(parent_value, root_font_size, current_font_size)
            .with_viewport(viewport_width, viewport_height)
    }
}

// ============================================================================
// Inspection
// ============================================================================

impl UnitValue {
    /// Returns `(value, unit)` if this is a plain `(number, unit)` value,
    /// `None` for calc / min / max / clamp.
    pub fn simple(&self) -> Option<(f32, Unit)> {
        match &self.expr {
            UnitExpr::Simple(v, u) => Some((*v, *u)),
            _ => None,
        }
    }

    pub fn is_px(&self) -> bool {
        matches!(&self.expr, UnitExpr::Simple(_, Unit::Px))
    }

    pub fn is_percent(&self) -> bool {
        matches!(&self.expr, UnitExpr::Simple(_, Unit::Percent))
    }

    /// True if every term in the tree uses an absolute unit
    /// (`px`, `in`, `cm`, `mm`, `pt`, `pc`) — i.e. resolvable without any
    /// context.
    pub fn is_absolute(&self) -> bool {
        self.all_terms(|_, u| {
            matches!(
                u,
                Unit::Px | Unit::In | Unit::Cm | Unit::Mm | Unit::Pt | Unit::Pc
            )
        })
    }

    /// True if every term uses a font- or parent-relative unit.
    pub fn is_relative(&self) -> bool {
        self.all_terms(|_, u| {
            matches!(
                u,
                Unit::Em | Unit::Rem | Unit::Ex | Unit::Ch | Unit::Percent
            )
        })
    }

    /// True if every term uses a viewport unit.
    pub fn is_viewport(&self) -> bool {
        self.all_terms(|_, u| matches!(u, Unit::Vw | Unit::Vh | Unit::Vmin | Unit::Vmax))
    }

    /// True iff `to_px()` will return `Some(_)` given the current context.
    pub fn is_resolvable(&self) -> bool {
        self.to_px().is_some()
    }

    fn all_terms<F>(&self, f: F) -> bool
    where
        F: Fn(f32, Unit) -> bool + Copy,
    {
        unit_expr_all_terms(&self.expr, f)
    }
}

/// Free-function tree-walker for `UnitExpr`.
///
/// (`UnitValue::all_terms` can't recurse directly into `&UnitExpr` or
/// `&Box<UnitExpr>`, so the recursive logic lives here.)
fn unit_expr_all_terms<F>(expr: &UnitExpr, f: F) -> bool
where
    F: Fn(f32, Unit) -> bool + Copy,
{
    match expr {
        UnitExpr::Simple(v, u) => f(*v, *u),
        UnitExpr::Calc(e) => calc_all_terms(e, f),
        UnitExpr::Min(args) | UnitExpr::Max(args) => args.iter().all(|a| unit_expr_all_terms(a, f)),
        UnitExpr::Clamp(a, b, c) => {
            unit_expr_all_terms(a, f) && unit_expr_all_terms(b, f) && unit_expr_all_terms(c, f)
        }
    }
}

fn calc_all_terms<F>(e: &CalcExpr, f: F) -> bool
where
    F: Fn(f32, Unit) -> bool + Copy,
{
    match e {
        CalcExpr::Term(v, u) => f(*v, *u),
        // Bare numbers don't carry a unit, so they don't affect the check.
        CalcExpr::Number(_) => true,
        CalcExpr::Nested(u) => unit_expr_all_terms(u, f),
        CalcExpr::Add(a, b) | CalcExpr::Sub(a, b) | CalcExpr::Mul(a, b) | CalcExpr::Div(a, b) => {
            calc_all_terms(a, f) && calc_all_terms(b, f)
        }
        CalcExpr::Paren(inner) => calc_all_terms(inner, f),
    }
}

// ============================================================================
// Conversion
// ============================================================================

impl UnitValue {
    /// Resolve to pixels.
    ///
    /// Returns `None` if any subterm can't be resolved with the current
    /// context (e.g. `%` with `parent_value <= 0`, `vw` with no viewport,
    /// division by zero).
    pub fn to_px(&self) -> Option<f32> {
        eval_unit_expr(&self.expr, &self.ctx)
    }

    /// Returns the numeric value if this is a plain percentage, else `None`.
    /// Calc expressions that *contain* a percentage do not count.
    pub fn to_percent(&self) -> Option<f32> {
        match &self.expr {
            UnitExpr::Simple(v, Unit::Percent) => Some(*v),
            _ => None,
        }
    }

    /// Convert to a Freya `Size`.
    ///
    /// - plain `%` → `Size::Percentage(value)`
    /// - anything resolvable → `Size::Pixels(px)`
    /// - unresolvable → `Size::Pixels(0.0)`
    pub fn to_freya_size(&self) -> Size {
        if let Some(percent) = self.to_percent() {
            return Size::Percentage(EuclidLength::new(percent));
        }
        if let Some(px) = self.to_px() {
            return Size::Pixels(EuclidLength::new(px));
        }
        Size::Pixels(EuclidLength::new(0.0))
    }

    /// Resolve as a font size in pixels.
    ///
    /// `%` is interpreted against `parent_value` (the parent's font size).
    pub fn to_font_size(&self) -> f32 {
        if let Some(pct) = self.to_percent() {
            return (pct / 100.0) * self.ctx.parent_value;
        }
        self.to_px().unwrap_or(0.0)
    }
}

fn eval_unit_expr(expr: &UnitExpr, ctx: &ResolveContext) -> Option<f32> {
    match expr {
        UnitExpr::Simple(v, u) => convert(*v, *u, ctx),
        UnitExpr::Calc(e) => eval_calc_expr(e, ctx),
        UnitExpr::Min(args) => {
            let mut best: Option<f32> = None;
            for a in args {
                let v = eval_unit_expr(a, ctx)?;
                best = Some(best.map_or(v, |x| x.min(v)));
            }
            best
        }
        UnitExpr::Max(args) => {
            let mut best: Option<f32> = None;
            for a in args {
                let v = eval_unit_expr(a, ctx)?;
                best = Some(best.map_or(v, |x| x.max(v)));
            }
            best
        }
        UnitExpr::Clamp(lo, val, hi) => {
            let lo = eval_unit_expr(lo, ctx)?;
            let val = eval_unit_expr(val, ctx)?;
            let hi = eval_unit_expr(hi, ctx)?;
            Some(lo.max(val.min(hi)))
        }
    }
}

fn eval_calc_expr(expr: &CalcExpr, ctx: &ResolveContext) -> Option<f32> {
    match expr {
        CalcExpr::Term(v, u) => convert(*v, *u, ctx),
        CalcExpr::Number(n) => Some(*n),
        CalcExpr::Nested(u) => eval_unit_expr(u, ctx),
        CalcExpr::Paren(inner) => eval_calc_expr(inner, ctx),
        CalcExpr::Add(a, b) => Some(eval_calc_expr(a, ctx)? + eval_calc_expr(b, ctx)?),
        CalcExpr::Sub(a, b) => Some(eval_calc_expr(a, ctx)? - eval_calc_expr(b, ctx)?),
        CalcExpr::Mul(a, b) => Some(eval_calc_expr(a, ctx)? * eval_calc_expr(b, ctx)?),
        CalcExpr::Div(a, b) => {
            let d = eval_calc_expr(b, ctx)?;
            if d == 0.0 {
                None
            } else {
                Some(eval_calc_expr(a, ctx)? / d)
            }
        }
    }
}

fn convert(v: f32, u: Unit, ctx: &ResolveContext) -> Option<f32> {
    match u {
        Unit::Px => Some(v),
        Unit::Percent => {
            if ctx.parent_value > 0.0 {
                Some(v / 100.0 * ctx.parent_value)
            } else {
                None
            }
        }
        Unit::Em => Some(v * ctx.current_font_size),
        Unit::Rem => Some(v * ctx.root_font_size),
        Unit::In => Some(v * PX_PER_INCH),
        Unit::Cm => Some(v * PX_PER_INCH / 2.54),
        Unit::Mm => Some(v * PX_PER_INCH / 25.4),
        Unit::Pt => Some(v * PX_PER_INCH / 72.0),
        Unit::Pc => Some(v * PX_PER_INCH / 6.0),
        Unit::Ex => Some(v * EX_RATIO * ctx.current_font_size),
        Unit::Ch => Some(v * CH_RATIO * ctx.current_font_size),
        Unit::Vw => {
            if ctx.viewport_width > 0.0 {
                Some(v / 100.0 * ctx.viewport_width)
            } else {
                None
            }
        }
        Unit::Vh => {
            if ctx.viewport_height > 0.0 {
                Some(v / 100.0 * ctx.viewport_height)
            } else {
                None
            }
        }
        Unit::Vmin => {
            if ctx.viewport_width > 0.0 && ctx.viewport_height > 0.0 {
                Some(v / 100.0 * ctx.viewport_width.min(ctx.viewport_height))
            } else {
                None
            }
        }
        Unit::Vmax => {
            if ctx.viewport_width > 0.0 && ctx.viewport_height > 0.0 {
                Some(v / 100.0 * ctx.viewport_width.max(ctx.viewport_height))
            } else {
                None
            }
        }
    }
}

// ============================================================================
// Parser
// ============================================================================

#[derive(Debug, PartialEq)]
enum ParseError {
    ExpectedNumber,
    ExpectedCloseParen,
    UnknownUnit(String),
    UnknownFunction(String),
    ClampArity(usize),
    TrailingInput,
}

struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Self {
        Self {
            input: s.as_bytes(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn skip_ws(&mut self) {
        while matches!(
            self.peek(),
            Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r')
        ) {
            self.advance();
        }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn parse_full(&mut self) -> Result<UnitExpr, ParseError> {
        let expr = self.parse_unit_expr()?;
        self.skip_ws();
        if !self.at_end() {
            return Err(ParseError::TrailingInput);
        }
        Ok(expr)
    }

    fn parse_unit_expr(&mut self) -> Result<UnitExpr, ParseError> {
        self.skip_ws();
        let start = self.pos;

        // Read an identifier; if followed by '(', it's a function call.
        while self
            .peek()
            .map_or(false, |c| c.is_ascii_alphabetic() || c == b'-')
        {
            self.advance();
        }
        let ident_end = self.pos;

        if ident_end > start && self.peek() == Some(b'(') {
            let name = std::str::from_utf8(&self.input[start..ident_end]).unwrap();
            self.advance(); // consume '('
            let result = if name.eq_ignore_ascii_case("calc") {
                UnitExpr::Calc(self.parse_calc_expr()?)
            } else if name.eq_ignore_ascii_case("min") {
                UnitExpr::Min(self.parse_args_until_close()?)
            } else if name.eq_ignore_ascii_case("max") {
                UnitExpr::Max(self.parse_args_until_close()?)
            } else if name.eq_ignore_ascii_case("clamp") {
                let args = self.parse_args_until_close()?;
                if args.len() != 3 {
                    return Err(ParseError::ClampArity(args.len()));
                }
                let mut it = args.into_iter();
                UnitExpr::Clamp(
                    Box::new(it.next().unwrap()),
                    Box::new(it.next().unwrap()),
                    Box::new(it.next().unwrap()),
                )
            } else {
                return Err(ParseError::UnknownFunction(name.to_string()));
            };
            self.skip_ws();
            if self.peek() != Some(b')') {
                return Err(ParseError::ExpectedCloseParen);
            }
            self.advance();
            Ok(result)
        } else {
            // Simple term — reset and parse via the dedicated path.
            self.pos = start;
            let (v, u) = self.parse_simple_term()?;
            Ok(UnitExpr::Simple(v, u))
        }
    }

    fn parse_simple_term(&mut self) -> Result<(f32, Unit), ParseError> {
        // Capture the absolute start of the number (including sign) so we
        // slice the correct range even if `self.pos` is not 0.
        let number_start = self.pos;

        if self.peek() == Some(b'-') {
            self.advance();
        }
        let digit_start = self.pos;
        while self.peek().map_or(false, |c| c.is_ascii_digit()) {
            self.advance();
        }
        if self.peek() == Some(b'.') {
            self.advance();
            while self.peek().map_or(false, |c| c.is_ascii_digit()) {
                self.advance();
            }
        }
        if self.pos == digit_start {
            return Err(ParseError::ExpectedNumber);
        }
        let num_str = std::str::from_utf8(&self.input[number_start..self.pos])
            .ok()
            .and_then(|s| s.parse::<f32>().ok())
            .ok_or(ParseError::ExpectedNumber)?;

        let unit_start = self.pos;
        while self
            .peek()
            .map_or(false, |c| c.is_ascii_alphabetic() || c == b'%')
        {
            self.advance();
        }
        let unit_str = std::str::from_utf8(&self.input[unit_start..self.pos]).unwrap();
        let unit = if unit_str.is_empty() {
            Unit::Px
        } else {
            unit_str
                .parse::<Unit>()
                .map_err(|_| ParseError::UnknownUnit(unit_str.to_string()))?
        };
        Ok((num_str, unit))
    }

    fn parse_calc_expr(&mut self) -> Result<CalcExpr, ParseError> {
        let mut lhs = self.parse_calc_term()?;
        loop {
            self.skip_ws();
            match self.peek() {
                Some(b'+') => {
                    self.advance();
                    let rhs = self.parse_calc_term()?;
                    lhs = CalcExpr::Add(Box::new(lhs), Box::new(rhs));
                }
                Some(b'-') => {
                    self.advance();
                    let rhs = self.parse_calc_term()?;
                    lhs = CalcExpr::Sub(Box::new(lhs), Box::new(rhs));
                }
                _ => break,
            }
        }
        Ok(lhs)
    }

    fn parse_calc_term(&mut self) -> Result<CalcExpr, ParseError> {
        let mut lhs = self.parse_calc_atom()?;
        loop {
            self.skip_ws();
            match self.peek() {
                Some(b'*') => {
                    self.advance();
                    let rhs = self.parse_calc_atom()?;
                    lhs = CalcExpr::Mul(Box::new(lhs), Box::new(rhs));
                }
                Some(b'/') => {
                    self.advance();
                    let rhs = self.parse_calc_atom()?;
                    lhs = CalcExpr::Div(Box::new(lhs), Box::new(rhs));
                }
                _ => break,
            }
        }
        Ok(lhs)
    }

    fn parse_calc_atom(&mut self) -> Result<CalcExpr, ParseError> {
        self.skip_ws();

        if self.looks_like_function() {
            let inner = self.parse_unit_expr()?;
            return Ok(match inner {
                UnitExpr::Calc(e) => e,
                UnitExpr::Simple(v, u) => CalcExpr::Term(v, u),
                other => CalcExpr::Nested(Box::new(other)),
            });
        }

        if self.peek() == Some(b'(') {
            self.advance();
            let inner = self.parse_calc_expr()?;
            self.skip_ws();
            if self.peek() != Some(b')') {
                return Err(ParseError::ExpectedCloseParen);
            }
            self.advance();
            return Ok(CalcExpr::Paren(Box::new(inner)));
        }

        if self.peek() == Some(b'-') {
            self.advance();
            let inner = self.parse_calc_atom()?;
            return Ok(CalcExpr::Mul(
                Box::new(CalcExpr::Number(-1.0)),
                Box::new(inner),
            ));
        }

        let num_start = self.pos;
        while self.peek().map_or(false, |c| c.is_ascii_digit()) {
            self.advance();
        }
        if self.peek() == Some(b'.') {
            self.advance();
            while self.peek().map_or(false, |c| c.is_ascii_digit()) {
                self.advance();
            }
        }
        if num_start == self.pos {
            return Err(ParseError::ExpectedNumber);
        }
        let value: f32 = std::str::from_utf8(&self.input[num_start..self.pos])
            .unwrap()
            .parse()
            .map_err(|_| ParseError::ExpectedNumber)?;

        let unit_start = self.pos;
        while self
            .peek()
            .map_or(false, |c| c.is_ascii_alphabetic() || c == b'%')
        {
            self.advance();
        }
        let unit_str = std::str::from_utf8(&self.input[unit_start..self.pos]).unwrap();
        if unit_str.is_empty() {
            Ok(CalcExpr::Number(value))
        } else {
            let unit = unit_str
                .parse::<Unit>()
                .map_err(|_| ParseError::UnknownUnit(unit_str.to_string()))?;
            Ok(CalcExpr::Term(value, unit))
        }
    }

    fn looks_like_function(&self) -> bool {
        let mut p = self.pos;
        while p < self.input.len() && (self.input[p].is_ascii_alphabetic() || self.input[p] == b'-')
        {
            p += 1;
        }
        p > self.pos && p < self.input.len() && self.input[p] == b'('
    }

    fn parse_args_until_close(&mut self) -> Result<Vec<UnitExpr>, ParseError> {
        let mut args = Vec::new();
        loop {
            self.skip_ws();
            args.push(self.parse_arg()?);
            self.skip_ws();
            if self.peek() == Some(b',') {
                self.advance();
            } else {
                break;
            }
        }
        Ok(args)
    }

    fn parse_arg(&mut self) -> Result<UnitExpr, ParseError> {
        self.skip_ws();
        if self.looks_like_function() {
            return self.parse_unit_expr();
        }
        let expr = self.parse_calc_expr()?;
        // Collapse single-term expressions to `Simple` for cleaner trees.
        Ok(match expr {
            CalcExpr::Term(v, u) => UnitExpr::Simple(v, u),
            other => UnitExpr::Calc(other),
        })
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-3, "expected {b}, got {a}");
    }

    fn assert_parsed(input: &str, expected_value: f32, expected_unit: Unit) {
        let l = UnitValue::new(input);
        assert_eq!(
            l.simple(),
            Some((expected_value, expected_unit)),
            "failed to parse {input:?}: got {:?}",
            l.simple(),
        );
    }

    #[test]
    fn parses_simple_lengths() {
        assert_parsed("50px", 50.0, Unit::Px);
        assert_parsed("-10px", -10.0, Unit::Px);
        assert_parsed("1.5rem", 1.5, Unit::Rem);
        assert_parsed("2em", 2.0, Unit::Em);
        assert_parsed("50%", 50.0, Unit::Percent);
        assert_parsed("100vw", 100.0, Unit::Vw);
        assert_parsed("1in", 1.0, Unit::In);
        assert_parsed("72pt", 72.0, Unit::Pt);
    }

    #[test]
    fn bare_number_defaults_to_px() {
        assert_parsed("0", 0.0, Unit::Px);
        assert_parsed("42", 42.0, Unit::Px);
        assert_parsed("-3", -3.0, Unit::Px);
    }

    #[test]
    fn whitespace_is_trimmed() {
        assert_parsed(" 50px", 50.0, Unit::Px);
        assert_parsed("50px ", 50.0, Unit::Px);
        assert_parsed("\t2rem\n", 2.0, Unit::Rem);
    }

    #[test]
    fn invalid_input_falls_back_to_zero() {
        for input in ["", "garbage", "50 px", "50.5.5px", "50foo"] {
            let l = UnitValue::new(input);
            assert_eq!(
                l.simple(),
                Some((0.0, Unit::Px)),
                "invalid input {input:?} should fall back to 0px, got {:?}",
                l.simple(),
            );
        }
    }

    // ------------------------------------------------------------------
    // calc()
    // ------------------------------------------------------------------

    #[test]
    fn calc_subtraction() {
        let l = UnitValue::new("calc(100% - 20px)").with_context(800.0, 16.0, 16.0);
        approx(l.to_px().unwrap(), 780.0);
    }

    #[test]
    fn calc_addition_mixed_units() {
        let l = UnitValue::new("calc(1rem + 5px)").with_context(0.0, 10.0, 16.0);
        approx(l.to_px().unwrap(), 15.0);
    }

    #[test]
    fn calc_multiplication() {
        approx(UnitValue::new("calc(2 * 10px)").to_px().unwrap(), 20.0);
        approx(UnitValue::new("calc(10px * 3)").to_px().unwrap(), 30.0);
    }

    #[test]
    fn calc_division() {
        approx(UnitValue::new("calc(100px / 4)").to_px().unwrap(), 25.0);
    }

    #[test]
    fn calc_nested_parens() {
        let l = UnitValue::new("calc((100% - 20px) / 2)").with_context(800.0, 16.0, 16.0);
        approx(l.to_px().unwrap(), 390.0);
    }

    #[test]
    fn calc_unresolvable_without_context() {
        assert_eq!(UnitValue::new("calc(100% - 20px)").to_px(), None);
    }

    #[test]
    fn calc_negative_term() {
        approx(UnitValue::new("calc(-10px)").to_px().unwrap(), -10.0);
        approx(UnitValue::new("calc(1px - -2px)").to_px().unwrap(), 3.0);
    }

    #[test]
    fn calc_division_by_zero_is_none() {
        assert_eq!(UnitValue::new("calc(10px / 0)").to_px(), None);
    }

    // ------------------------------------------------------------------
    // min / max / clamp
    // ------------------------------------------------------------------

    #[test]
    fn min_function() {
        let l = UnitValue::new("min(100%, 800px)").with_context(1000.0, 16.0, 16.0);
        approx(l.to_px().unwrap(), 800.0);
        let l = UnitValue::new("min(100%, 800px)").with_context(500.0, 16.0, 16.0);
        approx(l.to_px().unwrap(), 500.0);
    }

    #[test]
    fn max_function() {
        let l = UnitValue::new("max(200px, 50%)").with_context(800.0, 16.0, 16.0);
        approx(l.to_px().unwrap(), 400.0);
        let l = UnitValue::new("max(200px, 50%)").with_context(200.0, 16.0, 16.0);
        approx(l.to_px().unwrap(), 200.0);
    }

    #[test]
    fn clamp_function_fluid_typography() {
        let make = |vw: f32| UnitValue::new("clamp(1rem, 2.5vw, 2rem)").with_viewport(vw, 600.0);
        approx(make(1000.0).to_px().unwrap(), 25.0);
        approx(make(100.0).to_px().unwrap(), 16.0);
        approx(make(2000.0).to_px().unwrap(), 32.0);
    }

    #[test]
    fn min_max_unresolvable_without_context() {
        assert_eq!(UnitValue::new("min(100%, 800px)").to_px(), None);
        assert_eq!(UnitValue::new("max(50%, 200px)").to_px(), None);
    }

    // ------------------------------------------------------------------
    // to_freya_size
    // ------------------------------------------------------------------

    #[test]
    fn to_size_percent_yields_percentage_variant() {
        match UnitValue::new("50%").to_freya_size() {
            Size::Percentage(p) => approx(p.get(), 50.0),
            _ => panic!("expected Size::Percentage for `50%`"),
        }
    }

    #[test]
    fn to_size_px_yields_pixels_variant() {
        match UnitValue::new("12px").to_freya_size() {
            Size::Pixels(p) => approx(p.get(), 12.0),
            _ => panic!("expected Size::Pixels for `12px`"),
        }
    }

    #[test]
    fn to_size_calc_resolves_to_pixels() {
        match UnitValue::new("calc(100% - 20px)")
            .with_context(800.0, 16.0, 16.0)
            .to_freya_size()
        {
            Size::Pixels(p) => approx(p.get(), 780.0),
            _ => panic!("expected Size::Pixels for resolved calc"),
        }
    }

    #[test]
    fn to_size_viewport_without_context_falls_back_to_zero() {
        match UnitValue::new("100vw").to_freya_size() {
            Size::Pixels(p) => approx(p.get(), 0.0),
            _ => panic!("expected Size::Pixels(0) fallback"),
        }
    }

    // ------------------------------------------------------------------
    // to_font_size
    // ------------------------------------------------------------------

    #[test]
    fn to_font_size_px() {
        approx(UnitValue::new("18px").to_font_size(), 18.0);
    }

    #[test]
    fn to_font_size_percent_is_relative_to_parent() {
        approx(
            UnitValue::new("50%")
                .with_context(32.0, 16.0, 16.0)
                .to_font_size(),
            16.0,
        );
    }

    #[test]
    fn to_font_size_clamp_fluid() {
        approx(
            UnitValue::new("clamp(1rem, 2.5vw, 2rem)")
                .with_viewport(1000.0, 600.0)
                .to_font_size(),
            25.0,
        );
    }

    // ------------------------------------------------------------------
    // Const constructors & builders
    // ------------------------------------------------------------------

    #[test]
    fn const_constructors() {
        assert_eq!(UnitValue::zero(), UnitValue::new("0px"));
        assert_eq!(UnitValue::px(12.0), UnitValue::new("12px"));
        assert_eq!(UnitValue::em(1.5), UnitValue::new("1.5em"));
        assert_eq!(UnitValue::rem(2.0), UnitValue::new("2rem"));
        assert_eq!(UnitValue::percent(50.0), UnitValue::new("50%"));
    }

    #[test]
    fn builders_are_order_independent() {
        let a = UnitValue::new("2rem")
            .with_context(0.0, 20.0, 16.0)
            .with_viewport(800.0, 600.0);
        let b = UnitValue::new("2rem")
            .with_viewport(800.0, 600.0)
            .with_context(0.0, 20.0, 16.0);
        assert_eq!(a, b);
    }

    // ------------------------------------------------------------------
    // Predicates
    // ------------------------------------------------------------------

    #[test]
    fn unit_predicates() {
        assert!(UnitValue::new("10px").is_absolute());
        assert!(!UnitValue::new("50%").is_absolute());
        assert!(UnitValue::new("50%").is_relative());
        assert!(UnitValue::new("10vw").is_viewport());
        assert!(UnitValue::new("10px").is_px());
        assert!(UnitValue::new("50%").is_percent());

        // Mixed-unit calc: neither purely absolute nor purely relative.
        assert!(!UnitValue::new("calc(100% - 20px)").is_absolute());
        assert!(!UnitValue::new("calc(100% - 20px)").is_relative());

        assert!(!UnitValue::new("calc(100% - 20px)").is_resolvable());
        assert!(
            UnitValue::new("calc(100% - 20px)")
                .with_context(800.0, 16.0, 16.0)
                .is_resolvable()
        );
    }

    // ------------------------------------------------------------------
    // Parser robustness
    // ------------------------------------------------------------------

    #[test]
    fn parser_rejects_malformed_input_gracefully() {
        for input in [
            "calc(",
            "calc(10px",
            "min()",
            "clamp(1px, 2px)",
            "unknown(1px)",
            "calc(10px",
            ")",
        ] {
            let l = UnitValue::new(input);
            assert_eq!(
                l.simple(),
                Some((0.0, Unit::Px)),
                "input {input:?} should fall back to 0px",
            );
        }
    }

    #[test]
    fn parser_is_case_insensitive_for_functions() {
        approx(UnitValue::new("CALC(2 * 5px)").to_px().unwrap(), 10.0);
        approx(UnitValue::new("Min(10px, 20px)").to_px().unwrap(), 10.0);
    }

    #[test]
    fn nested_calc_collapses() {
        approx(
            UnitValue::new("calc(calc(1px + 2px) * 2)").to_px().unwrap(),
            6.0,
        );
    }
}
