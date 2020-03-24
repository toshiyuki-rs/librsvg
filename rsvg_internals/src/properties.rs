//! CSS properties, specified values, computed values.

use cssparser::{
    self, BasicParseErrorKind, DeclarationListParser, ParseErrorKind, Parser, ParserInput, ToCss,
};
use markup5ever::{expanded_name, local_name, namespace_url, ns, QualName};
use std::collections::HashSet;

use crate::css::{DeclParser, Declaration, Origin};
use crate::error::*;
use crate::parsers::{Parse, ParseValue};
use crate::property_bag::PropertyBag;
use crate::property_defs::*;
use crate::property_macros::Property;

/// Representation of a single CSS property value.
///
/// `Unspecified` is the `Default`; it means that the corresponding property is not present.
///
/// `Inherit` means that the property is explicitly set to inherit
/// from the parent element.  This is useful for properties which the
/// SVG or CSS specs mandate that should not be inherited by default.
///
/// `Specified` is a value given by the SVG or CSS stylesheet.  This will later be
/// resolved into part of a `ComputedValues` struct.
#[derive(Clone)]
pub enum SpecifiedValue<T>
where
    T: Property<ComputedValues> + Clone + Default,
{
    Unspecified,
    Inherit,
    Specified(T),
}

impl<T> SpecifiedValue<T>
where
    T: Property<ComputedValues> + Clone + Default,
{
    pub fn compute(&self, src: &T, src_values: &ComputedValues) -> T {
        let value: T = match *self {
            SpecifiedValue::Unspecified => {
                if <T as Property<ComputedValues>>::inherits_automatically() {
                    src.clone()
                } else {
                    Default::default()
                }
            }

            SpecifiedValue::Inherit => src.clone(),

            SpecifiedValue::Specified(ref v) => v.clone(),
        };

        value.compute(src_values)
    }
}

impl<T> Default for SpecifiedValue<T>
where
    T: Property<ComputedValues> + Clone + Default,
{
    fn default() -> SpecifiedValue<T> {
        SpecifiedValue::Unspecified
    }
}

/// Embodies "which property is this" plus the property's value
#[derive(Clone)]
pub enum ParsedProperty {
    BaselineShift(SpecifiedValue<BaselineShift>),
    ClipPath(SpecifiedValue<ClipPath>),
    ClipRule(SpecifiedValue<ClipRule>),
    Color(SpecifiedValue<Color>),
    ColorInterpolationFilters(SpecifiedValue<ColorInterpolationFilters>),
    Direction(SpecifiedValue<Direction>),
    Display(SpecifiedValue<Display>),
    EnableBackground(SpecifiedValue<EnableBackground>),
    Fill(SpecifiedValue<Fill>),
    FillOpacity(SpecifiedValue<FillOpacity>),
    FillRule(SpecifiedValue<FillRule>),
    Filter(SpecifiedValue<Filter>),
    FloodColor(SpecifiedValue<FloodColor>),
    FloodOpacity(SpecifiedValue<FloodOpacity>),
    FontFamily(SpecifiedValue<FontFamily>),
    FontSize(SpecifiedValue<FontSize>),
    FontStretch(SpecifiedValue<FontStretch>),
    FontStyle(SpecifiedValue<FontStyle>),
    FontVariant(SpecifiedValue<FontVariant>),
    FontWeight(SpecifiedValue<FontWeight>),
    LetterSpacing(SpecifiedValue<LetterSpacing>),
    LightingColor(SpecifiedValue<LightingColor>),
    Marker(SpecifiedValue<Marker>), // this is a shorthand property
    MarkerEnd(SpecifiedValue<MarkerEnd>),
    MarkerMid(SpecifiedValue<MarkerMid>),
    MarkerStart(SpecifiedValue<MarkerStart>),
    Mask(SpecifiedValue<Mask>),
    Opacity(SpecifiedValue<Opacity>),
    Overflow(SpecifiedValue<Overflow>),
    ShapeRendering(SpecifiedValue<ShapeRendering>),
    StopColor(SpecifiedValue<StopColor>),
    StopOpacity(SpecifiedValue<StopOpacity>),
    Stroke(SpecifiedValue<Stroke>),
    StrokeDasharray(SpecifiedValue<StrokeDasharray>),
    StrokeDashoffset(SpecifiedValue<StrokeDashoffset>),
    StrokeLinecap(SpecifiedValue<StrokeLinecap>),
    StrokeLinejoin(SpecifiedValue<StrokeLinejoin>),
    StrokeOpacity(SpecifiedValue<StrokeOpacity>),
    StrokeMiterlimit(SpecifiedValue<StrokeMiterlimit>),
    StrokeWidth(SpecifiedValue<StrokeWidth>),
    TextAnchor(SpecifiedValue<TextAnchor>),
    TextDecoration(SpecifiedValue<TextDecoration>),
    TextRendering(SpecifiedValue<TextRendering>),
    UnicodeBidi(SpecifiedValue<UnicodeBidi>),
    Visibility(SpecifiedValue<Visibility>),
    WritingMode(SpecifiedValue<WritingMode>),
    XmlLang(SpecifiedValue<XmlLang>), // not a property, but a non-presentation attribute
    XmlSpace(SpecifiedValue<XmlSpace>), // not a property, but a non-presentation attribute
}

/// Used to match `ParsedProperty` to their discriminant
///
/// The `PropertyId::UnsetProperty` can be used as a sentinel value, as
/// it does not match any `ParsedProperty` discriminant; it is really the
/// number of valid values in this enum.
#[repr(u8)]
#[derive(Copy, Clone, PartialEq)]
enum PropertyId {
    BaselineShift,
    ClipPath,
    ClipRule,
    Color,
    ColorInterpolationFilters,
    Direction,
    Display,
    EnableBackground,
    Fill,
    FillOpacity,
    FillRule,
    Filter,
    FloodColor,
    FloodOpacity,
    FontFamily,
    FontSize,
    FontStretch,
    FontStyle,
    FontVariant,
    FontWeight,
    LetterSpacing,
    LightingColor,
    Marker,
    MarkerEnd,
    MarkerMid,
    MarkerStart,
    Mask,
    Opacity,
    Overflow,
    ShapeRendering,
    StopColor,
    StopOpacity,
    Stroke,
    StrokeDasharray,
    StrokeDashoffset,
    StrokeLinecap,
    StrokeLinejoin,
    StrokeOpacity,
    StrokeMiterlimit,
    StrokeWidth,
    TextAnchor,
    TextDecoration,
    TextRendering,
    UnicodeBidi,
    Visibility,
    WritingMode,
    XmlLang,
    XmlSpace,
    UnsetProperty,
}

impl ParsedProperty {
    #[rustfmt::skip]
    fn get_property_id(&self) -> PropertyId {
        use ParsedProperty::*;

        match *self {
            BaselineShift(_)             => PropertyId::BaselineShift,
            ClipPath(_)                  => PropertyId::ClipPath,
            ClipRule(_)                  => PropertyId::ClipRule,
            Color(_)                     => PropertyId::Color,
            ColorInterpolationFilters(_) => PropertyId::ColorInterpolationFilters,
            Direction(_)                 => PropertyId::Direction,
            Display(_)                   => PropertyId::Display,
            EnableBackground(_)          => PropertyId::EnableBackground,
            Fill(_)                      => PropertyId::Fill,
            FillOpacity(_)               => PropertyId::FillOpacity,
            FillRule(_)                  => PropertyId::FillRule,
            Filter(_)                    => PropertyId::Filter,
            FloodColor(_)                => PropertyId::FloodColor,
            FloodOpacity(_)              => PropertyId::FloodOpacity,
            FontFamily(_)                => PropertyId::FontFamily,
            FontSize(_)                  => PropertyId::FontSize,
            FontStretch(_)               => PropertyId::FontStretch,
            FontStyle(_)                 => PropertyId::FontStyle,
            FontVariant(_)               => PropertyId::FontVariant,
            FontWeight(_)                => PropertyId::FontWeight,
            LetterSpacing(_)             => PropertyId::LetterSpacing,
            LightingColor(_)             => PropertyId::LightingColor,
            Marker(_)                    => PropertyId::Marker,
            MarkerEnd(_)                 => PropertyId::MarkerEnd,
            MarkerMid(_)                 => PropertyId::MarkerMid,
            MarkerStart(_)               => PropertyId::MarkerStart,
            Mask(_)                      => PropertyId::Mask,
            Opacity(_)                   => PropertyId::Opacity,
            Overflow(_)                  => PropertyId::Overflow,
            ShapeRendering(_)            => PropertyId::ShapeRendering,
            StopColor(_)                 => PropertyId::StopColor,
            StopOpacity(_)               => PropertyId::StopOpacity,
            Stroke(_)                    => PropertyId::Stroke,
            StrokeDasharray(_)           => PropertyId::StrokeDasharray,
            StrokeDashoffset(_)          => PropertyId::StrokeDashoffset,
            StrokeLinecap(_)             => PropertyId::StrokeLinecap,
            StrokeLinejoin(_)            => PropertyId::StrokeLinejoin,
            StrokeOpacity(_)             => PropertyId::StrokeOpacity,
            StrokeMiterlimit(_)          => PropertyId::StrokeMiterlimit,
            StrokeWidth(_)               => PropertyId::StrokeWidth,
            TextAnchor(_)                => PropertyId::TextAnchor,
            TextDecoration(_)            => PropertyId::TextDecoration,
            TextRendering(_)             => PropertyId::TextRendering,
            UnicodeBidi(_)               => PropertyId::UnicodeBidi,
            Visibility(_)                => PropertyId::Visibility,
            WritingMode(_)               => PropertyId::WritingMode,
            XmlLang(_)                   => PropertyId::XmlLang,
            XmlSpace(_)                  => PropertyId::XmlSpace,
        }
    }
}

impl PropertyId {
    fn as_u8(&self) -> u8 {
        *self as u8
    }

    fn as_usize(&self) -> usize {
        *self as usize
    }
}

/// Holds the specified CSS properties for an element
#[derive(Clone)]
pub struct SpecifiedValues {
    indices: [u8; PropertyId::UnsetProperty as usize],
    props: Vec<ParsedProperty>,
}

impl Default for SpecifiedValues {
    fn default() -> Self {
        SpecifiedValues {
            // this many elements, with the same value
            indices: [PropertyId::UnsetProperty.as_u8(); PropertyId::UnsetProperty as usize],
            props: Vec::new(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ComputedValues {
    pub baseline_shift: BaselineShift,
    pub clip_path: ClipPath,
    pub clip_rule: ClipRule,
    pub color: Color,
    pub color_interpolation_filters: ColorInterpolationFilters,
    pub direction: Direction,
    pub display: Display,
    pub enable_background: EnableBackground,
    pub fill: Fill,
    pub fill_opacity: FillOpacity,
    pub fill_rule: FillRule,
    pub filter: Filter,
    pub flood_color: FloodColor,
    pub flood_opacity: FloodOpacity,
    pub font_family: FontFamily,
    pub font_size: FontSize,
    pub font_stretch: FontStretch,
    pub font_style: FontStyle,
    pub font_variant: FontVariant,
    pub font_weight: FontWeight,
    pub letter_spacing: LetterSpacing,
    pub lighting_color: LightingColor,
    pub marker_end: MarkerEnd,
    pub marker_mid: MarkerMid,
    pub marker_start: MarkerStart,
    pub mask: Mask,
    pub opacity: Opacity,
    pub overflow: Overflow,
    pub shape_rendering: ShapeRendering,
    pub stop_color: StopColor,
    pub stop_opacity: StopOpacity,
    pub stroke: Stroke,
    pub stroke_dasharray: StrokeDasharray,
    pub stroke_dashoffset: StrokeDashoffset,
    pub stroke_line_cap: StrokeLinecap,
    pub stroke_line_join: StrokeLinejoin,
    pub stroke_opacity: StrokeOpacity,
    pub stroke_miterlimit: StrokeMiterlimit,
    pub stroke_width: StrokeWidth,
    pub text_anchor: TextAnchor,
    pub text_decoration: TextDecoration,
    pub text_rendering: TextRendering,
    pub unicode_bidi: UnicodeBidi,
    pub visibility: Visibility,
    pub writing_mode: WritingMode,
    pub xml_lang: XmlLang,   // not a property, but a non-presentation attribute
    pub xml_space: XmlSpace, // not a property, but a non-presentation attribute
}

#[rustfmt::skip]
pub fn parse_property<'i>(prop_name: &QualName, input: &mut Parser<'i, '_>, accept_shorthands: bool) -> Result<ParsedProperty, ParseError<'i>> {
    // please keep these sorted
    match prop_name.expanded() {
        expanded_name!("", "baseline-shift") =>
            Ok(ParsedProperty::BaselineShift(parse_input(input)?)),

        expanded_name!("", "clip-path") =>
            Ok(ParsedProperty::ClipPath(parse_input(input)?)),

        expanded_name!("", "clip-rule") =>
            Ok(ParsedProperty::ClipRule(parse_input(input)?)),

        expanded_name!("", "color") =>
            Ok(ParsedProperty::Color(parse_input(input)?)),

        expanded_name!("", "color-interpolation-filters") =>
            Ok(ParsedProperty::ColorInterpolationFilters(parse_input(input)?)),

        expanded_name!("", "direction") =>
            Ok(ParsedProperty::Direction(parse_input(input)?)),

        expanded_name!("", "display") =>
            Ok(ParsedProperty::Display(parse_input(input)?)),

        expanded_name!("", "enable-background") =>
            Ok(ParsedProperty::EnableBackground(parse_input(input)?)),

        expanded_name!("", "fill") =>
            Ok(ParsedProperty::Fill(parse_input(input)?)),

        expanded_name!("", "fill-opacity") =>
            Ok(ParsedProperty::FillOpacity(parse_input(input)?)),

        expanded_name!("", "fill-rule") =>
            Ok(ParsedProperty::FillRule(parse_input(input)?)),

        expanded_name!("", "filter") =>
            Ok(ParsedProperty::Filter(parse_input(input)?)),

        expanded_name!("", "flood-color") =>
            Ok(ParsedProperty::FloodColor(parse_input(input)?)),

        expanded_name!("", "flood-opacity") =>
            Ok(ParsedProperty::FloodOpacity(parse_input(input)?)),

        expanded_name!("", "font-family") =>
            Ok(ParsedProperty::FontFamily(parse_input(input)?)),

        expanded_name!("", "font-size") =>
            Ok(ParsedProperty::FontSize(parse_input(input)?)),

        expanded_name!("", "font-stretch") =>
            Ok(ParsedProperty::FontStretch(parse_input(input)?)),

        expanded_name!("", "font-style") =>
            Ok(ParsedProperty::FontStyle(parse_input(input)?)),

        expanded_name!("", "font-variant") =>
            Ok(ParsedProperty::FontVariant(parse_input(input)?)),

        expanded_name!("", "font-weight") =>
            Ok(ParsedProperty::FontWeight(parse_input(input)?)),

        expanded_name!("", "letter-spacing") =>
            Ok(ParsedProperty::LetterSpacing(parse_input(input)?)),

        expanded_name!("", "lighting-color") =>
            Ok(ParsedProperty::LightingColor(parse_input(input)?)),

        expanded_name!("", "marker") => {
            if accept_shorthands {
                Ok(ParsedProperty::Marker(parse_input(input)?))
            } else {
                let loc = input.current_source_location();
                Err(loc.new_custom_error(ValueErrorKind::UnknownProperty))
            }
        }

        expanded_name!("", "marker-end") =>
            Ok(ParsedProperty::MarkerEnd(parse_input(input)?)),

        expanded_name!("", "marker-mid") =>
            Ok(ParsedProperty::MarkerMid(parse_input(input)?)),

        expanded_name!("", "marker-start") =>
            Ok(ParsedProperty::MarkerStart(parse_input(input)?)),

        expanded_name!("", "mask") =>
            Ok(ParsedProperty::Mask(parse_input(input)?)),

        expanded_name!("", "opacity") =>
            Ok(ParsedProperty::Opacity(parse_input(input)?)),

        expanded_name!("", "overflow") =>
            Ok(ParsedProperty::Overflow(parse_input(input)?)),

        expanded_name!("", "shape-rendering") =>
            Ok(ParsedProperty::ShapeRendering(parse_input(input)?)),

        expanded_name!("", "stop-color") =>
            Ok(ParsedProperty::StopColor(parse_input(input)?)),

        expanded_name!("", "stop-opacity") =>
            Ok(ParsedProperty::StopOpacity(parse_input(input)?)),

        expanded_name!("", "stroke") =>
            Ok(ParsedProperty::Stroke(parse_input(input)?)),

        expanded_name!("", "stroke-dasharray") =>
            Ok(ParsedProperty::StrokeDasharray(parse_input(input)?)),

        expanded_name!("", "stroke-dashoffset") =>
            Ok(ParsedProperty::StrokeDashoffset(parse_input(input)?)),

        expanded_name!("", "stroke-linecap") =>
            Ok(ParsedProperty::StrokeLinecap(parse_input(input)?)),

        expanded_name!("", "stroke-linejoin") =>
            Ok(ParsedProperty::StrokeLinejoin(parse_input(input)?)),

        expanded_name!("", "stroke-miterlimit") =>
            Ok(ParsedProperty::StrokeMiterlimit(parse_input(input)?)),

        expanded_name!("", "stroke-opacity") =>
            Ok(ParsedProperty::StrokeOpacity(parse_input(input)?)),

        expanded_name!("", "stroke-width") =>
            Ok(ParsedProperty::StrokeWidth(parse_input(input)?)),

        expanded_name!("", "text-anchor") =>
            Ok(ParsedProperty::TextAnchor(parse_input(input)?)),

        expanded_name!("", "text-decoration") =>
            Ok(ParsedProperty::TextDecoration(parse_input(input)?)),

        expanded_name!("", "text-rendering") =>
            Ok(ParsedProperty::TextRendering(parse_input(input)?)),

        expanded_name!("", "unicode-bidi") =>
            Ok(ParsedProperty::UnicodeBidi(parse_input(input)?)),

        expanded_name!("", "visibility") =>
            Ok(ParsedProperty::Visibility(parse_input(input)?)),

        expanded_name!("", "writing-mode") =>
            Ok(ParsedProperty::WritingMode(parse_input(input)?)),

        _ => {
            let loc = input.current_source_location();
            Err(loc.new_custom_error(ValueErrorKind::UnknownProperty))
        }
    }
}

impl ComputedValues {
    pub fn is_overflow(&self) -> bool {
        match self.overflow {
            Overflow::Auto | Overflow::Visible => true,
            _ => false,
        }
    }

    pub fn is_visible(&self) -> bool {
        match (self.display, self.visibility) {
            (Display::None, _) => false,
            (_, Visibility::Visible) => true,
            _ => false,
        }
    }
}

impl SpecifiedValues {
    fn property_index(&self, id: PropertyId) -> Option<usize> {
        let v = self.indices[id.as_usize()];

        if v == PropertyId::UnsetProperty.as_u8() {
            None
        } else {
            Some(v as usize)
        }
    }

    fn set_property(&mut self, prop: &ParsedProperty, replace: bool) {
        let id = prop.get_property_id();

        if id == PropertyId::Marker {
            unreachable!("should have processed shorthands earlier");
        }

        if let Some(index) = self.property_index(id) {
            if replace {
                self.props[index] = prop.clone();
            }
        } else {
            self.props.push(prop.clone());
            let pos = self.props.len() - 1;
            self.indices[id.as_usize()] = pos as u8;
        }
    }

    #[rustfmt::skip]
    fn set_property_expanding_shorthands(&mut self, prop: &ParsedProperty, replace: bool) {
        use crate::properties::ParsedProperty::*;
        use crate::properties as p;

        if let Marker(SpecifiedValue::Specified(p::Marker(ref v))) = *prop {
            // Since "marker" is a shorthand property, we'll just expand it here
            self.set_property(&MarkerStart(SpecifiedValue::Specified(p::MarkerStart(v.clone()))), replace);
            self.set_property(&MarkerMid(SpecifiedValue::Specified(p::MarkerMid(v.clone()))), replace);
            self.set_property(&MarkerEnd(SpecifiedValue::Specified(p::MarkerEnd(v.clone()))), replace);
        } else {
            self.set_property(prop, replace);
        }
    }

    pub fn set_parsed_property(&mut self, prop: &ParsedProperty) {
        self.set_property_expanding_shorthands(prop, true);
    }

    /* user agent property have less priority than presentation attributes */
    pub fn set_parsed_property_user_agent(&mut self, prop: &ParsedProperty) {
        self.set_property_expanding_shorthands(prop, false);
    }

    pub fn to_computed_values(&self, computed: &mut ComputedValues) {
        macro_rules! compute {
            ($name:ident, $field:ident) => {
                if let Some(index) = self.property_index(PropertyId::$name) {
                    if let &ParsedProperty::$name(ref s) = &self.props[index] {
                        computed.$field = s.compute(&computed.$field, computed);
                    } else {
                        unreachable!();
                    }
                } else {
                    let s = SpecifiedValue::<$name>::Unspecified;
                    computed.$field = s.compute(&computed.$field, computed);
                }
            };
        }

        // First, compute font_size.  It needs to be done before everything
        // else, so that properties that depend on its computed value
        // will be able to use it.  For example, baseline-shift
        // depends on font-size.

        compute!(FontSize, font_size);

        // Then, do all the other properties.

        compute!(BaselineShift, baseline_shift);
        compute!(ClipPath, clip_path);
        compute!(ClipRule, clip_rule);
        compute!(Color, color);
        compute!(ColorInterpolationFilters, color_interpolation_filters);
        compute!(Direction, direction);
        compute!(Display, display);
        compute!(EnableBackground, enable_background);
        compute!(Fill, fill);
        compute!(FillOpacity, fill_opacity);
        compute!(FillRule, fill_rule);
        compute!(Filter, filter);
        compute!(FloodColor, flood_color);
        compute!(FloodOpacity, flood_opacity);
        compute!(FontFamily, font_family);
        compute!(FontStretch, font_stretch);
        compute!(FontStyle, font_style);
        compute!(FontVariant, font_variant);
        compute!(FontWeight, font_weight);
        compute!(LetterSpacing, letter_spacing);
        compute!(LightingColor, lighting_color);
        compute!(MarkerEnd, marker_end);
        compute!(MarkerMid, marker_mid);
        compute!(MarkerStart, marker_start);
        compute!(Mask, mask);
        compute!(Opacity, opacity);
        compute!(Overflow, overflow);
        compute!(ShapeRendering, shape_rendering);
        compute!(StopColor, stop_color);
        compute!(StopOpacity, stop_opacity);
        compute!(Stroke, stroke);
        compute!(StrokeDasharray, stroke_dasharray);
        compute!(StrokeDashoffset, stroke_dashoffset);
        compute!(StrokeLinecap, stroke_line_cap);
        compute!(StrokeLinejoin, stroke_line_join);
        compute!(StrokeOpacity, stroke_opacity);
        compute!(StrokeMiterlimit, stroke_miterlimit);
        compute!(StrokeWidth, stroke_width);
        compute!(TextAnchor, text_anchor);
        compute!(TextDecoration, text_decoration);
        compute!(TextRendering, text_rendering);
        compute!(UnicodeBidi, unicode_bidi);
        compute!(Visibility, visibility);
        compute!(WritingMode, writing_mode);
        compute!(XmlLang, xml_lang);
        compute!(XmlSpace, xml_space);
    }

    pub fn is_overflow(&self) -> bool {
        if let Some(overflow_index) = self.property_index(PropertyId::Overflow) {
            match self.props[overflow_index] {
                ParsedProperty::Overflow(SpecifiedValue::Specified(Overflow::Auto)) => true,
                ParsedProperty::Overflow(SpecifiedValue::Specified(Overflow::Visible)) => true,
                ParsedProperty::Overflow(_) => false,
                _ => unreachable!(),
            }
        } else {
            false
        }
    }

    fn parse_one_presentation_attribute(
        &mut self,
        attr: QualName,
        value: &str,
    ) -> Result<(), ElementError> {
        let mut input = ParserInput::new(value);
        let mut parser = Parser::new(&mut input);

        // Presentation attributes don't accept shorthands, e.g. there is no
        // attribute like marker="#foo" and it needs to be set in the style attribute
        // like style="marker: #foo;".  So, pass false for accept_shorthands here.
        match parse_property(&attr, &mut parser, false) {
            Ok(prop) => self.set_parsed_property(&prop),

            // not a presentation attribute; just ignore it
            Err(ParseError {
                kind: ParseErrorKind::Custom(ValueErrorKind::UnknownProperty),
                ..
            }) => (),

            // https://www.w3.org/TR/CSS2/syndata.html#unsupported-values
            // For all the following cases, ignore illegal values; don't set the whole node to
            // be in error in that case.
            Err(ParseError {
                kind: ParseErrorKind::Basic(BasicParseErrorKind::UnexpectedToken(ref t)),
                ..
            }) => {
                let mut tok = String::new();

                t.to_css(&mut tok).unwrap(); // FIXME: what do we do with a fmt::Error?
                rsvg_log!(
                    "(ignoring invalid presentation attribute {:?}\n    value=\"{}\"\n    \
                     unexpected token '{}')",
                    attr.expanded(),
                    value,
                    tok,
                );
            }

            Err(ParseError {
                kind: ParseErrorKind::Basic(BasicParseErrorKind::EndOfInput),
                ..
            }) => {
                rsvg_log!(
                    "(ignoring invalid presentation attribute {:?}\n    value=\"{}\"\n    \
                     unexpected end of input)",
                    attr.expanded(),
                    value,
                );
            }

            Err(ParseError {
                kind: ParseErrorKind::Basic(_),
                ..
            }) => {
                rsvg_log!(
                    "(ignoring invalid presentation attribute {:?}\n    value=\"{}\"\n    \
                     unexpected error)",
                    attr.expanded(),
                    value,
                );
            }

            Err(ParseError {
                kind: ParseErrorKind::Custom(ref v),
                ..
            }) => {
                rsvg_log!(
                    "(ignoring invalid presentation attribute {:?}\n    value=\"{}\"\n    {})",
                    attr.expanded(),
                    value,
                    v
                );
            }
        }

        Ok(())
    }

    pub fn parse_presentation_attributes(
        &mut self,
        pbag: &PropertyBag<'_>,
    ) -> Result<(), ElementError> {
        for (attr, value) in pbag.iter() {
            match attr.expanded() {
                expanded_name!(xml "lang") => {
                    // xml:lang is a non-presentation attribute and as such cannot have the
                    // "inherit" value.  So, we don't call parse_one_presentation_attribute()
                    // for it, but rather call its parser directly.
                    self.set_parsed_property(&ParsedProperty::XmlLang(SpecifiedValue::Specified(
                        attr.parse(value)?,
                    )));
                }

                expanded_name!(xml "space") => {
                    // xml:space is a non-presentation attribute and as such cannot have the
                    // "inherit" value.  So, we don't call parse_one_presentation_attribute()
                    // for it, but rather call its parser directly.
                    self.set_parsed_property(&ParsedProperty::XmlSpace(SpecifiedValue::Specified(
                        attr.parse(value)?,
                    )));
                }

                _ => self.parse_one_presentation_attribute(attr, value)?,
            }
        }

        Ok(())
    }

    pub fn set_property_from_declaration(
        &mut self,
        declaration: &Declaration,
        origin: Origin,
        important_styles: &mut HashSet<QualName>,
    ) {
        if !declaration.important && important_styles.contains(&declaration.prop_name) {
            return;
        }

        if declaration.important {
            important_styles.insert(declaration.prop_name.clone());
        }

        if origin == Origin::UserAgent {
            self.set_parsed_property_user_agent(&declaration.property);
        } else {
            self.set_parsed_property(&declaration.property);
        }
    }

    pub fn parse_style_declarations(
        &mut self,
        declarations: &str,
        origin: Origin,
        important_styles: &mut HashSet<QualName>,
    ) -> Result<(), ElementError> {
        let mut input = ParserInput::new(declarations);
        let mut parser = Parser::new(&mut input);

        DeclarationListParser::new(&mut parser, DeclParser)
            .filter_map(|r| match r {
                Ok(decl) => Some(decl),
                Err(e) => {
                    rsvg_log!("Invalid declaration; ignoring: {:?}", e);
                    None
                }
            })
            .for_each(|decl| self.set_property_from_declaration(&decl, origin, important_styles));

        Ok(())
    }
}

// Parses the value for the type `T` of the property out of the Parser, including `inherit` values.
fn parse_input<'i, T>(input: &mut Parser<'i, '_>) -> Result<SpecifiedValue<T>, ParseError<'i>>
where
    T: Property<ComputedValues> + Clone + Default + Parse,
{
    if input
        .try_parse(|p| p.expect_ident_matching("inherit"))
        .is_ok()
    {
        Ok(SpecifiedValue::Inherit)
    } else {
        Parse::parse(input).map(SpecifiedValue::Specified)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iri::IRI;
    use crate::length::*;

    #[test]
    fn empty_values_computes_to_defaults() {
        let specified = SpecifiedValues::default();

        let mut computed = ComputedValues::default();
        specified.to_computed_values(&mut computed);

        assert_eq!(computed.stroke_width, StrokeWidth::default());
    }

    #[test]
    fn set_one_property() {
        let length = Length::<Both>::new(42.0, LengthUnit::Px);

        let mut specified = SpecifiedValues::default();
        specified.set_parsed_property(&ParsedProperty::StrokeWidth(SpecifiedValue::Specified(
            StrokeWidth(length),
        )));

        let mut computed = ComputedValues::default();
        specified.to_computed_values(&mut computed);

        assert_eq!(computed.stroke_width, StrokeWidth(length));
    }

    #[test]
    fn replace_existing_property() {
        let length1 = Length::<Both>::new(42.0, LengthUnit::Px);
        let length2 = Length::<Both>::new(42.0, LengthUnit::Px);

        let mut specified = SpecifiedValues::default();

        specified.set_parsed_property(&ParsedProperty::StrokeWidth(SpecifiedValue::Specified(
            StrokeWidth(length1),
        )));

        specified.set_parsed_property(&ParsedProperty::StrokeWidth(SpecifiedValue::Specified(
            StrokeWidth(length2),
        )));

        let mut computed = ComputedValues::default();
        specified.to_computed_values(&mut computed);

        assert_eq!(computed.stroke_width, StrokeWidth(length2));
    }

    #[test]
    fn expands_marker_shorthand() {
        let mut specified = SpecifiedValues::default();
        let iri = IRI::parse_str("url(#foo)").unwrap();

        let marker = Marker(iri.clone());
        specified.set_parsed_property(&ParsedProperty::Marker(SpecifiedValue::Specified(marker)));

        let mut computed = ComputedValues::default();
        specified.to_computed_values(&mut computed);

        assert_eq!(computed.marker_start, MarkerStart(iri.clone()));
        assert_eq!(computed.marker_mid, MarkerMid(iri.clone()));
        assert_eq!(computed.marker_end, MarkerEnd(iri.clone()));
    }

    #[test]
    fn replaces_marker_shorthand() {
        let mut specified = SpecifiedValues::default();
        let iri1 = IRI::parse_str("url(#foo)").unwrap();
        let iri2 = IRI::None;

        let marker1 = Marker(iri1.clone());
        specified.set_parsed_property(&ParsedProperty::Marker(SpecifiedValue::Specified(marker1)));

        let marker2 = Marker(iri2.clone());
        specified.set_parsed_property(&ParsedProperty::Marker(SpecifiedValue::Specified(marker2)));

        let mut computed = ComputedValues::default();
        specified.to_computed_values(&mut computed);

        assert_eq!(computed.marker_start, MarkerStart(iri2.clone()));
        assert_eq!(computed.marker_mid, MarkerMid(iri2.clone()));
        assert_eq!(computed.marker_end, MarkerEnd(iri2.clone()));
    }

    #[test]
    fn computes_property_that_does_not_inherit_automatically() {
        assert_eq!(
            <Opacity as Property<ComputedValues>>::inherits_automatically(),
            false
        );

        let half_opacity = Opacity::parse_str("0.5").unwrap();

        // first level, as specified with opacity

        let mut with_opacity = SpecifiedValues::default();
        with_opacity.set_parsed_property(&ParsedProperty::Opacity(SpecifiedValue::Specified(
            half_opacity.clone(),
        )));

        let mut computed_0_5 = ComputedValues::default();
        with_opacity.to_computed_values(&mut computed_0_5);

        assert_eq!(computed_0_5.opacity, half_opacity.clone());

        // second level, no opacity specified, and it doesn't inherit

        let without_opacity = SpecifiedValues::default();

        let mut computed = computed_0_5.clone();
        without_opacity.to_computed_values(&mut computed);

        assert_eq!(computed.opacity, Opacity::default());

        // another at second level, opacity set to explicitly inherit

        let mut with_inherit_opacity = SpecifiedValues::default();
        with_inherit_opacity.set_parsed_property(&ParsedProperty::Opacity(SpecifiedValue::Inherit));

        let mut computed = computed_0_5.clone();
        with_inherit_opacity.to_computed_values(&mut computed);

        assert_eq!(computed.opacity, half_opacity.clone());
    }
}
