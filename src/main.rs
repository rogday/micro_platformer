use enigo::{Enigo, KeyboardControllable};

use std::sync::Arc;
use std::time::{Duration, Instant};

use livesplit_hotkey::{Hook, Key, KeyCode};
use parking_lot::Mutex;

#[derive(Debug, Clone)]
enum Action {
    Sleep(std::time::Duration),
    KeyPress(KeyCode),
    KeyRelease(KeyCode),
}

fn to_enigo(key: KeyCode) -> enigo::Key {
    use enigo::Key::*;
    match key {
        KeyCode::Left => LeftArrow,
        KeyCode::Right => RightArrow,
        KeyCode::Up => UpArrow,
        KeyCode::Down => DownArrow,
        KeyCode::Space => Space,
        _ => unreachable!(),
    }
}

fn replay(enigo: &mut Enigo, actions: &[Action]) {
    for action in actions {
        println!("executing {:?}", action);
        match *action {
            Action::Sleep(d) => std::thread::sleep(d),
            Action::KeyPress(key) => enigo.key_down(to_enigo(key)),
            Action::KeyRelease(key) => enigo.key_up(to_enigo(key)),
        }
    }
    clear(enigo);
}

fn clear(enigo: &mut Enigo) {
    use enigo::Key::*;

    [UpArrow, DownArrow, LeftArrow, RightArrow, Space].iter().for_each(|&key| {
        enigo.key_up(key);
    });
}

fn register(
    hook: &Hook,
    keys: &[KeyCode],
    mode: fn(KeyCode) -> Key,
    closure: impl FnMut(KeyCode) + Send + Clone + 'static,
) {
    for &key in keys {
        hook.register(mode(key), {
            let mut closure = closure.clone();
            move || closure(key)
        })
        .unwrap();
    }
}

fn main() {
    let enigo = Arc::new(Mutex::new(Enigo::new()));

    //TODO: common_state to avoid deadlocks
    let saves: Arc<Mutex<Vec<Vec<Action>>>> = Arc::new(Mutex::new(Vec::new()));
    let keys: Arc<Mutex<Vec<Action>>> = Arc::new(Mutex::new(Vec::new()));
    let last_time = Arc::new(Mutex::new(Duration::from_millis(0)));

    let common_time = Arc::new(Instant::now());

    let hook = Hook::new().unwrap();

    register(
        &hook,
        &[KeyCode::Left, KeyCode::Right, KeyCode::Up, KeyCode::Down, KeyCode::Space],
        Key::Press,
        {
            let common_time = Arc::clone(&common_time);
            let last_time = Arc::clone(&last_time);
            let keys = Arc::clone(&keys);

            move |k| {
                //DEADLOCK
                let mut last_time = last_time.lock();
                let mut keys = keys.lock();

                let elapsed = common_time.elapsed();

                if keys.len() > 1 {
                    keys.push(Action::Sleep(elapsed - *last_time));
                }

                keys.push(Action::KeyPress(k));

                *last_time = elapsed;
            }
        },
    );

    register(
        &hook,
        &[KeyCode::Left, KeyCode::Right, KeyCode::Up, KeyCode::Down, KeyCode::Space],
        Key::Release,
        {
            let common_time = Arc::clone(&common_time);
            let last_time = Arc::clone(&last_time);
            let keys = Arc::clone(&keys);

            move |k| {
                let mut last_time = last_time.lock();
                let mut keys = keys.lock();

                let elapsed = common_time.elapsed();

                keys.push(Action::Sleep(elapsed - *last_time));

                *last_time = elapsed;
                keys.push(Action::KeyRelease(k));
            }
        },
    );

    //ResetSave
    hook.register(Key::Press(KeyCode::F1), {
        let keys = Arc::clone(&keys);

        move || keys.lock().clear()
    })
    .unwrap();

    //Save
    hook.register(Key::Press(KeyCode::F2), {
        let saves = Arc::clone(&saves);
        let keys = Arc::clone(&keys);

        move || {
            let mut saves = saves.lock();
            let mut keys = keys.lock();

            saves.push(keys.clone());
            keys.clear();
        }
    })
    .unwrap();

    //Replay
    hook.register(Key::Press(KeyCode::F3), {
        let enigo = Arc::clone(&enigo);
        let saves = Arc::clone(&saves);

        move || {
            let mut enigo = enigo.lock();
            let saves = saves.lock();

            println!("{:?}", saves);
            replay(&mut enigo, &saves.last().unwrap());
        }
    })
    .unwrap();

    //Exit
    hook.register(Key::Press(KeyCode::F4), || panic!()).unwrap();

    std::thread::sleep(Duration::from_secs(6e20 as u64));
}
