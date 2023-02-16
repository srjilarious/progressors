
use progressors::{Progresso, Style, ProgressoBar, ValueDisplay};
use std::thread;

fn main()
{
    let mut style = Style::default_ascii();
    style.value_display = ValueDisplay::Percentage;
    let mut pb = Progresso::new(style);

    pb.set_total(400);
    for i in 0..401 {
        pb.erase();
        pb.set_value(i);
        pb.draw();
        thread::sleep(std::time::Duration::from_millis(25));
    }
}