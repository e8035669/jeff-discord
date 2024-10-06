use anyhow::{anyhow, Result};
use jeff_discord::entities::prelude::*;
use jeff_discord::entities::*;
use sea_orm::prelude::*;
use sea_orm::ActiveValue::Set;
use sea_orm::{Database, EntityTrait, IntoActiveModel, QueryFilter};

async fn main1() -> Result<()> {
    let db = Database::connect("sqlite://test_db.sqlite?mode=rwc").await?;

    let ret1 = color_random_data::Entity::delete_many().exec(&db).await?;
    println!("DeleteResult: {:?}", ret1);

    let d1 = color_random_data::ActiveModel {
        guild: Set(99999),
        role: Set(9999),
        shift: Set(0),
        ..Default::default()
    };

    let ret2 = ColorRandomData::insert(d1).exec(&db).await?;
    println!("InsertResult: {:?}", ret2);

    let d2 = ColorRandomData::find().all(&db).await?;
    println!("Find all: {:?}", d2);

    let d3 = color_random_data::ActiveModel {
        guild: Set(1000),
        role: Set(1000),
        shift: Set(0),
        ..Default::default()
    };

    let d4 = color_random_data::ActiveModel {
        guild: Set(50),
        role: Set(50),
        shift: Set(0),
        ..Default::default()
    };

    let ret3 = ColorRandomData::insert_many([d3, d4]).exec(&db).await?;
    println!("Insert many: {:?}", ret3);

    let d5 = ColorRandomData::find()
        .filter(color_random_data::Column::Guild.eq(1000))
        .filter(color_random_data::Column::Role.eq(100))
        .one(&db)
        .await?;
    println!("Find one: {:?}", d5);

    let d6 = d5.ok_or(anyhow!("Not found model"))?;
    let mut d6 = d6.into_active_model();

    println!("D6 {:?}", d6);

    d6.shift = Set(d6.shift.take().ok_or_else(|| anyhow!("No value"))? + 1);

    println!("D6 {:?}", d6);

    let ret4 = ColorRandomData::update(d6).exec(&db).await?;
    println!("Update: {:?}", ret4);

    let d7 = ColorRandomData::find().all(&db).await?;
    println!("Find all");
    for m in &d7 {
        println!("{:?}", m);
    }

    db.close().await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let ret = main1().await;
    if let Err(e) = ret {
        println!("Found Error: {:?}", e);
    }
}

