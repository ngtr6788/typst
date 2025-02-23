use ecow::{eco_format, EcoString};

use crate::diag::{At, SourceResult};
use crate::engine::Engine;
use crate::foundations::{
    cast, elem, Content, Label, NativeElement, Repr, Show, Smart, StyleChain,
};
use crate::introspection::Location;
use crate::layout::Position;
use crate::text::{Hyphenate, TextElem};

/// Links to a URL or a location in the document.
///
/// By default, links are not styled any different from normal text. However,
/// you can easily apply a style of your choice with a show rule.
///
/// # Example
/// ```example
/// #show link: underline
///
/// https://example.com \
///
/// #link("https://example.com") \
/// #link("https://example.com")[
///   See example.com
/// ]
/// ```
///
/// # Syntax
/// This function also has dedicated syntax: Text that starts with `http://` or
/// `https://` is automatically turned into a link.
#[elem(Show)]
pub struct LinkElem {
    /// The destination the link points to.
    ///
    /// - To link to web pages, `dest` should be a valid URL string. If the URL
    ///   is in the `mailto:` or `tel:` scheme and the `body` parameter is
    ///   omitted, the email address or phone number will be the link's body,
    ///   without the scheme.
    ///
    /// - To link to another part of the document, `dest` can take one of three
    ///   forms:
    ///   - A [label]($label) attached to an element. If you also want automatic
    ///     text for the link based on the element, consider using a
    ///     [reference]($ref) instead.
    ///
    ///   - A [location]($locate) resulting from a [`locate`]($locate) call or
    ///     [`query`]($query).
    ///
    ///   - A dictionary with a `page` key of type [integer]($int) and `x` and
    ///     `y` coordinates of type [length]($length). Pages are counted from
    ///     one, and the coordinates are relative to the page's top left corner.
    ///
    /// ```example
    /// = Introduction <intro>
    /// #link("mailto:hello@typst.app") \
    /// #link(<intro>)[Go to intro] \
    /// #link((page: 1, x: 0pt, y: 0pt))[
    ///   Go to top
    /// ]
    /// ```
    #[required]
    #[parse(
        let dest = args.expect::<LinkTarget>("destination")?;
        dest.clone()
    )]
    pub dest: LinkTarget,

    /// The content that should become a link.
    ///
    /// If `dest` is an URL string, the parameter can be omitted. In this case,
    /// the URL will be shown as the link.
    #[required]
    #[parse(match &dest {
        LinkTarget::Dest(Destination::Url(url)) => match args.eat()? {
            Some(body) => body,
            None => body_from_url(url),
        },
        _ => args.expect("body")?,
    })]
    pub body: Content,
}

impl LinkElem {
    /// Create a link element from a URL with its bare text.
    pub fn from_url(url: EcoString) -> Self {
        let body = body_from_url(&url);
        Self::new(LinkTarget::Dest(Destination::Url(url)), body)
    }
}

impl Show for LinkElem {
    #[typst_macros::time(name = "link", span = self.span())]
    fn show(&self, engine: &mut Engine, _: StyleChain) -> SourceResult<Content> {
        let body = self.body().clone();
        let linked = match self.dest() {
            LinkTarget::Dest(dest) => body.linked(dest.clone()),
            LinkTarget::Label(label) => engine
                .delayed(|engine| {
                    let elem = engine.introspector.query_label(*label).at(self.span())?;
                    let dest = Destination::Location(elem.location().unwrap());
                    Ok(Some(body.clone().linked(dest)))
                })
                .unwrap_or(body),
        };

        Ok(linked.styled(TextElem::set_hyphenate(Hyphenate(Smart::Custom(false)))))
    }
}

fn body_from_url(url: &EcoString) -> Content {
    let mut text = url.as_str();
    for prefix in ["mailto:", "tel:"] {
        text = text.trim_start_matches(prefix);
    }
    let shorter = text.len() < url.len();
    TextElem::packed(if shorter { text.into() } else { url.clone() })
}

/// A target where a link can go.
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum LinkTarget {
    Dest(Destination),
    Label(Label),
}

cast! {
    LinkTarget,
    self => match self {
        Self::Dest(v) => v.into_value(),
        Self::Label(v) => v.into_value(),
    },
    v: Destination => Self::Dest(v),
    v: Label => Self::Label(v),
}

impl From<Destination> for LinkTarget {
    fn from(dest: Destination) -> Self {
        Self::Dest(dest)
    }
}

/// A link destination.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Destination {
    /// A link to a URL.
    Url(EcoString),
    /// A link to a point on a page.
    Position(Position),
    /// An unresolved link to a location in the document.
    Location(Location),
}

impl Repr for Destination {
    fn repr(&self) -> EcoString {
        eco_format!("{self:?}")
    }
}

cast! {
    Destination,
    self => match self {
        Self::Url(v) => v.into_value(),
        Self::Position(v) => v.into_value(),
        Self::Location(v) => v.into_value(),
    },
    v: EcoString => Self::Url(v),
    v: Position => Self::Position(v),
    v: Location => Self::Location(v),
}
