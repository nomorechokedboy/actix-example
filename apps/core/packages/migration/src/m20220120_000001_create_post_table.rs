use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Posts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Posts::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Posts::Title).string().not_null())
                    .col(ColumnDef::new(Posts::Text).string().not_null())
                    .to_owned(),
            )
            .await

        // let mut posts: Vec<post::ActiveModel> = Vec::new();
        // let db = &app_state.db_con;

        // for _i in 0..30000 {
        //     posts.push(post::ActiveModel {
        //         text: Set(Paragraph(1..2).fake()),
        //         title: Set(Paragraph(1..2).fake()),
        //         ..Default::default()
        //     });
        // }

        // let res = Post::insert_many(posts).exec(db).await;
        // Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Posts::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Posts {
    Table,
    Id,
    Title,
    Text,
}
