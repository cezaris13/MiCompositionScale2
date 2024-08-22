mod scale_metrics;

fn main() {
    println!("Hello, world!");
    println!(
        "{}",
        scale_metrics::check_value_overflow(5 as f32, 1 as f32, 100 as f32)
    );
    println!(
        "{}",
        scale_metrics::check_value_overflow(127 as f32, 1 as f32, 100 as f32)
    );
    println!(
        "{}",
        scale_metrics::check_value_overflow(0 as f32, 1 as f32, 100 as f32)
    );
    println!(
        "{}",
        scale_metrics::get_fat_percentage(
            450 as f32,
            84 as f32,
            scale_metrics::Gender::Male,
            23,
            184 as f32
        )
    );
}
