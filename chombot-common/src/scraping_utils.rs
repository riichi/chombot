use anyhow::anyhow;
use itertools::Itertools;
use scraper::ElementRef;

#[macro_export]
macro_rules! unpack_children {
    ($element:expr, $n:expr) => {
        <[ElementRef; $n]>::try_from(
            $element
                .children()
                .filter_map(ElementRef::wrap)
                .collect::<Vec<ElementRef>>(),
        )
        .map_err(|v| {
            anyhow!(
                "Could not unpack children into {} elements; got {} instead",
                $n,
                v.len()
            )
        })
    };
}

#[macro_export]
macro_rules! select_all {
    ($selector:expr, $obj:expr) => {
        $obj.select(&Selector::parse($selector).expect(concat!("Invalid selector: ", $selector)))
    };
}

#[macro_export]
macro_rules! select_one {
    ($selector:expr, $obj:expr) => {
        select_all!($selector, $obj)
            .next()
            .ok_or(anyhow!(concat!("Could not find any ", $selector)))
    };
}

pub fn first_nonempty_text<'a>(e: &'a ElementRef) -> anyhow::Result<&'a str> {
    let ret = e
        .text()
        .map(str::trim)
        .find(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("No non-empty text nodes found"))?;
    Ok(ret)
}

#[must_use]
pub fn cell_text(e: &ElementRef) -> String {
    e.text().map(str::trim).join(" ").trim().to_owned()
}
