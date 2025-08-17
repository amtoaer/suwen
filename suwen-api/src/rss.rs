use rss::{CategoryBuilder, ChannelBuilder, GuidBuilder, ImageBuilder};

use crate::db;

pub(crate) fn generate_rss(site: db::Site, articles: Vec<db::ArticleForRSS>) -> String {
    let (mut published_at, mut latest_modified) = (site.created_at, site.updated_at);
    for article in &articles {
        if article.published_at > published_at {
            published_at = article.published_at;
        }
        if article.updated_at > latest_modified {
            latest_modified = article.updated_at;
        }
    }
    let channel = ChannelBuilder::default()
        .title(site.site_name.clone())
        .description(site.intro)
        .image(
            ImageBuilder::default()
                .url(site.avatar_url)
                .title(site.site_name)
                .build(),
        )
        .pub_date(published_at.to_rfc2822())
        .last_build_date(latest_modified.to_rfc2822())
        .generator("Suwen".to_string())
        .items(
            articles
                .into_iter()
                .map(|article| {
                    rss::ItemBuilder::default()
                        .title(article.title)
                        .guid(
                            GuidBuilder::default()
                                .permalink(false)
                                .value(article.slug)
                                .build(),
                        )
                        .categories(
                            article
                                .tags
                                .0
                                .into_iter()
                                .map(|tag| CategoryBuilder::default().name(tag).build())
                                .collect::<Vec<_>>(),
                        )
                        .description(article.intro.or(article.summary))
                        .content(article.rendered_html)
                        .pub_date(article.published_at.to_rfc2822())
                        .author(site.display_name.clone())
                        .build()
                })
                .collect::<Vec<_>>(),
        )
        .build();
    channel.to_string()
}
