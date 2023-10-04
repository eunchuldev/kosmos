use kosmos_tile::Tilemap;


fn main() {
    let mut map = Tilemap::new(24, 5);
    futures_lite::future::block_on(async move {
        for _ in 0..3 {
            map.tick_without_sync().await;
        }
    })
}
