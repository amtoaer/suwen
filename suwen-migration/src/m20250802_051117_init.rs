use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(pk_auto(User::Id))
                    .col(text(User::Email))
                    .col(string_len(User::Username, 10).unique_key())
                    .col(string_len(User::PasswordHash, 256))
                    .col(string_len(User::DisplayName, 10))
                    .col(text(User::AvatarUrl))
                    .col(date_time(User::CreatedAt).default(Expr::current_timestamp()))
                    .col(date_time(User::UpdatedAt).default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Site::Table)
                    .if_not_exists()
                    .col(pk_auto(Site::Id))
                    .col(integer(Site::OwnerId).unique_key())
                    .col(text(Site::SiteName))
                    .col(text(Site::Intro))
                    .col(text(Site::Tabs))
                    .col(text(Site::RelatedLinks))
                    .col(date_time(Site::CreatedAt).default(Expr::current_timestamp()))
                    .col(date_time(Site::UpdatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_site_owner")
                            .from(Site::Table, Site::OwnerId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(ContentMetadata::Table)
                    .if_not_exists()
                    .col(pk_auto(ContentMetadata::Id))
                    .col(text(ContentMetadata::Slug).unique_key())
                    .col(text(ContentMetadata::CoverImages))
                    .col(text(ContentMetadata::Tags))
                    .col(text(ContentMetadata::ContentType))
                    .col(text(ContentMetadata::OriginalLang).default("zh-CN"))
                    .col(integer(ContentMetadata::ViewCount).default(0))
                    .col(integer(ContentMetadata::CommentCount).default(0))
                    .col(integer(ContentMetadata::LikeCount).default(0))
                    .col(date_time_null(ContentMetadata::PublishedAt))
                    .col(date_time(ContentMetadata::CreatedAt).default(Expr::current_timestamp()))
                    .col(date_time(ContentMetadata::UpdatedAt).default(Expr::current_timestamp()))
                    .check(Expr::col(ContentMetadata::ContentType).is_in(["article", "gallery"]))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Content::Table)
                    .if_not_exists()
                    .col(pk_auto(Content::Id))
                    .col(integer(Content::ContentMetadataId))
                    .col(text(Content::LangCode).default("zh-CN"))
                    .col(string_null(Content::Summary))
                    .col(string_null(Content::Intro))
                    .col(text(Content::Title))
                    .col(text(Content::OriginalText))
                    .col(string_null(Content::RenderedHtml))
                    .col(string_null(Content::Toc))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_content_metadata")
                            .from(Content::Table, Content::ContentMetadataId)
                            .to(ContentMetadata::Table, ContentMetadata::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_content_unique")
                            .col(Content::ContentMetadataId)
                            .col(Content::LangCode)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Tag::Table)
                    .if_not_exists()
                    .col(pk_auto(Tag::Id))
                    .col(text(Tag::TagName).unique_key())
                    .col(date_time(Tag::CreatedAt).default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(ContentMetadataTag::Table)
                    .if_not_exists()
                    .col(integer(ContentMetadataTag::ContentMetadataId))
                    .col(integer(ContentMetadataTag::TagId))
                    .primary_key(
                        Index::create()
                            .col(ContentMetadataTag::ContentMetadataId)
                            .col(ContentMetadataTag::TagId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_content_metadata_tag_content")
                            .from(
                                ContentMetadataTag::Table,
                                ContentMetadataTag::ContentMetadataId,
                            )
                            .to(ContentMetadata::Table, ContentMetadata::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_content_metadata_tag_tag")
                            .from(ContentMetadataTag::Table, ContentMetadataTag::TagId)
                            .to(Tag::Table, Tag::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Identity::Table)
                    .if_not_exists()
                    .col(pk_auto(Identity::Id))
                    .col(uuid_null(Identity::Uuid).unique_key())
                    .col(integer_null(Identity::UserId))
                    .col(date_time(Identity::CreatedAt).default(Expr::current_timestamp()))
                    .col(date_time(Identity::UpdatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_identity_user")
                            .from(Identity::Table, Identity::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Like::Table)
                    .if_not_exists()
                    .col(pk_auto(Like::Id))
                    .col(integer(Like::IdentityId))
                    .col(integer_null(Like::ContentMetadataId))
                    .col(date_time(Like::CreatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_like_identity")
                            .from(Like::Table, Like::IdentityId)
                            .to(Identity::Table, Identity::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_like_content_metadata")
                            .from(Like::Table, Like::ContentMetadataId)
                            .to(ContentMetadata::Table, ContentMetadata::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Comment::Table)
                    .if_not_exists()
                    .col(pk_auto(Comment::Id))
                    .col(integer(Comment::IdentityId))
                    .col(integer(Comment::ContentMetadataId))
                    .col(integer_null(Comment::ParentId))
                    .col(text(Comment::Content))
                    .col(boolean(Comment::IsDeleted).default(false))
                    .col(date_time(Comment::CreatedAt).default(Expr::current_timestamp()))
                    .col(date_time(Comment::UpdatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_comment_identity")
                            .from(Comment::Table, Comment::IdentityId)
                            .to(Identity::Table, Identity::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_comment_content_metadata")
                            .from(Comment::Table, Comment::ContentMetadataId)
                            .to(ContentMetadata::Table, ContentMetadata::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_comment_parent")
                            .from(Comment::Table, Comment::ParentId)
                            .to(Comment::Table, Comment::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_content_metadata__type_published")
                    .table(ContentMetadata::Table)
                    .col(ContentMetadata::ContentType)
                    .col((ContentMetadata::PublishedAt, IndexOrder::Desc))
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_content_metadata__view")
                    .table(ContentMetadata::Table)
                    .col((ContentMetadata::ViewCount, IndexOrder::Desc))
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_content_metadata__comment")
                    .table(ContentMetadata::Table)
                    .col((ContentMetadata::CommentCount, IndexOrder::Desc))
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_content__metadata_lang")
                    .table(Content::Table)
                    .col(Content::ContentMetadataId)
                    .col(Content::LangCode)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_content_metadata_tag__metadata")
                    .table(ContentMetadataTag::Table)
                    .col(ContentMetadataTag::ContentMetadataId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_content_metadata_tag__tag")
                    .table(ContentMetadataTag::Table)
                    .col(ContentMetadataTag::TagId)
                    .to_owned(),
            )
            .await?;
        // 新增索引
        manager
            .create_index(
                Index::create()
                    .name("idx_identity_uuid")
                    .table(Identity::Table)
                    .col(Identity::Uuid)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_like_identity")
                    .table(Like::Table)
                    .col(Like::IdentityId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_like_content_metadata")
                    .table(Like::Table)
                    .col(Like::ContentMetadataId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_comment_content_metadata")
                    .table(Comment::Table)
                    .col(Comment::ContentMetadataId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_comment_parent")
                    .table(Comment::Table)
                    .col(Comment::ParentId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Comment::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Like::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Identity::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ContentMetadataTag::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Tag::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Content::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ContentMetadata::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Site::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Email,
    Username,
    PasswordHash,
    DisplayName,
    AvatarUrl,
    CreatedAt,
    UpdatedAt,
}

#[allow(clippy::enum_variant_names)]
#[derive(DeriveIden)]
enum Site {
    Table,
    Id,
    OwnerId,
    SiteName,
    Intro,
    Tabs,
    RelatedLinks,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ContentMetadata {
    Table,
    Id,
    Slug,
    CoverImages,
    Tags,
    ContentType,
    OriginalLang,
    ViewCount,
    CommentCount,
    LikeCount,
    PublishedAt,
    CreatedAt,
    UpdatedAt,
}

#[allow(clippy::enum_variant_names)]
#[derive(DeriveIden)]
enum Content {
    Table,
    Id,
    ContentMetadataId,
    LangCode,
    Summary,
    Intro,
    Title,
    OriginalText,
    RenderedHtml,
    Toc,
}

#[allow(clippy::enum_variant_names)]
#[derive(DeriveIden)]
enum Tag {
    Table,
    Id,
    TagName,
    CreatedAt,
}

#[derive(DeriveIden)]
enum ContentMetadataTag {
    Table,
    ContentMetadataId,
    TagId,
}

#[derive(DeriveIden)]
enum Identity {
    Table,
    Id,
    Uuid,
    UserId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Like {
    Table,
    Id,
    IdentityId,
    ContentMetadataId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Comment {
    Table,
    Id,
    IdentityId,
    ContentMetadataId,
    ParentId,
    Content,
    IsDeleted,
    CreatedAt,
    UpdatedAt,
}
