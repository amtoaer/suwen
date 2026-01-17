use chrono::{DateTime, Local};
use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};

use crate::db::SitemapUrl;

pub(crate) fn generate_sitemap(base_url: &str, articles: Vec<SitemapUrl>) -> String {
    let mut writer = Writer::new(Vec::new());
    writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .unwrap();
    let mut urlset = BytesStart::new("urlset");
    urlset.push_attribute(("xmlns", "http://www.sitemaps.org/schemas/sitemap/0.9"));
    writer.write_event(Event::Start(urlset)).unwrap();
    write_url(&mut writer, base_url, "", None, "1.0", "daily");
    write_url(&mut writer, base_url, "archives", None, "0.8", "weekly");
    for article in articles {
        write_url(
            &mut writer,
            base_url,
            &format!("articles/{}", article.slug),
            Some(article.updated_at),
            "0.9",
            "monthly",
        );
    }
    writer
        .write_event(Event::End(BytesEnd::new("urlset")))
        .unwrap();
    String::from_utf8(writer.into_inner()).unwrap()
}

fn write_url(
    writer: &mut Writer<Vec<u8>>,
    base_url: &str,
    path: &str,
    lastmod: Option<DateTime<Local>>,
    priority: &str,
    changefreq: &str,
) {
    writer
        .write_event(Event::Start(BytesStart::new("url")))
        .unwrap();
    writer
        .write_event(Event::Start(BytesStart::new("loc")))
        .unwrap();
    let url = if path.is_empty() {
        base_url.to_string()
    } else {
        format!("{}/{}", base_url.trim_end_matches('/'), path)
    };
    writer
        .write_event(Event::Text(BytesText::new(&url)))
        .unwrap();
    writer
        .write_event(Event::End(BytesEnd::new("loc")))
        .unwrap();
    if let Some(lastmod) = lastmod {
        writer
            .write_event(Event::Start(BytesStart::new("lastmod")))
            .unwrap();
        let date = lastmod.format("%Y-%m-%d").to_string();
        writer
            .write_event(Event::Text(BytesText::new(&date)))
            .unwrap();
        writer
            .write_event(Event::End(BytesEnd::new("lastmod")))
            .unwrap();
    }
    writer
        .write_event(Event::Start(BytesStart::new("changefreq")))
        .unwrap();
    writer
        .write_event(Event::Text(BytesText::new(changefreq)))
        .unwrap();
    writer
        .write_event(Event::End(BytesEnd::new("changefreq")))
        .unwrap();
    writer
        .write_event(Event::Start(BytesStart::new("priority")))
        .unwrap();
    writer
        .write_event(Event::Text(BytesText::new(priority)))
        .unwrap();
    writer
        .write_event(Event::End(BytesEnd::new("priority")))
        .unwrap();
    writer
        .write_event(Event::End(BytesEnd::new("url")))
        .unwrap();
}
