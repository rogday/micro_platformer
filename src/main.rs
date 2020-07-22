use enigo::{Enigo, Key, KeyboardControllable};
use std::thread;
use std::time::Duration;

fn sleep_ms(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}

fn clear(enigo: &mut Enigo) {
    enigo.key_up(Key::Space);
    enigo.key_up(Key::UpArrow);
    enigo.key_up(Key::DownArrow);
    enigo.key_up(Key::LeftArrow);
    enigo.key_up(Key::RightArrow);
}

fn main() {
    let mut enigo = Enigo::new();

    macro_rules! press {
        ($key:tt) => {
            enigo.key_down(map!($key));
        };
    }

    macro_rules! release {
        ($key:tt) => {
            enigo.key_up(map!($key));
        };
    }

    macro_rules! map {
        (up) => {
            Key::UpArrow
        };
        (down) => {
            Key::DownArrow
        };
        (left) => {
            Key::LeftArrow
        };
        (right) => {
            Key::RightArrow
        };
        (dash) => {
            Key::Space
        };
    }

    sleep_ms(5_000);
    clear(&mut enigo);

    press![right];
    press![up];
    sleep_ms(65);
    release![up];
    sleep_ms(70);
    press![down];
    sleep_ms(300);
    release![right];
    sleep_ms(100);

    //floor
    press![right];
    press![up];
    sleep_ms(630);
    press![down];
    release![up];
    sleep_ms(500);

    //wall
    press![left];
    press![down];
    sleep_ms(110);
    release![left];
    press![up];
    sleep_ms(300);
    press![up];
    sleep_ms(80);
    press![left];
    sleep_ms(120);
    release![up];
    sleep_ms(100);
    press![down];
    sleep_ms(500);

    //wall2
    release![left];
    sleep_ms(10);
    press![up];
    sleep_ms(180);
    press![left];
    sleep_ms(200);
    press![down];
    sleep_ms(100);
    press![down];
    sleep_ms(50);
    release![left];

    sleep_ms(500);
    release![left];
    release![right];
    release![up];
    release![down];

    clear(&mut enigo);
}
