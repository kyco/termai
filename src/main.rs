mod app;
mod db;
mod repository;
mod ui;

use app::App;
use db::SqliteRepository;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let repo = SqliteRepository::new("app.db")?;
    let mut app = App::new(repo)?;
    ui::run(&mut app)?;
    Ok(())
}
