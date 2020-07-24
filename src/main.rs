use enigo::{Enigo, KeyboardControllable};

use std::sync::Arc;
use std::time::{Duration, Instant};

use livesplit_hotkey::{Hook, Key, KeyCode};
use parking_lot::FairMutex;

const CONTROLS: [KeyCode; 5] =
    [KeyCode::Left, KeyCode::Right, KeyCode::Up, KeyCode::Down, KeyCode::Space];

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
    println!("========================Replay in progress========================");
    //first element is sleep(), so we skip it
    for action in actions.iter().skip(1) {
        println!("executing {:?}", action);
        match *action {
            Action::Sleep(d) => std::thread::sleep(d),
            Action::KeyPress(key) => enigo.key_down(to_enigo(key)),
            Action::KeyRelease(key) => enigo.key_up(to_enigo(key)),
        }
    }
    println!("========================Replay is done========================");
}

fn register(
    hook: &Hook,
    keys: &[KeyCode],
    mode: fn(KeyCode) -> Key,
    action: fn(KeyCode) -> Action,
    closure: impl FnMut(Action) + Send + Clone + 'static,
) {
    for &key in keys {
        hook.register(mode(key), {
            let mut closure = closure.clone();
            move || closure(action(key))
        })
        .unwrap();
    }
}

#[derive(Default)]
struct SharedState {
    saves:       Vec<Vec<Action>>,
    actions:     Vec<Action>,
    last_update: Duration,
    enigo:       Enigo,
}

fn main() {
    let state = Arc::new(FairMutex::new(SharedState::default()));
    let common_time = Arc::new(Instant::now());

    let hook = Hook::new().unwrap();

    let get_callback = || {
        let state = Arc::clone(&state);
        let common_time = Arc::clone(&common_time);

        move |k| {
            let SharedState { ref mut last_update, ref mut actions, .. } = &mut *state.lock();
            let elapsed = common_time.elapsed();

            actions.extend_from_slice(&[Action::Sleep(elapsed - *last_update), k]);
            *last_update = elapsed;
        }
    };

    register(&hook, &CONTROLS, Key::Press, Action::KeyPress, get_callback());
    register(&hook, &CONTROLS, Key::Release, Action::KeyRelease, get_callback());

    //ResetSave - after every replay this should be initiated
    hook.register(Key::Press(KeyCode::F1), {
        let state = Arc::clone(&state);

        move || state.lock().actions.clear()
    })
    .unwrap();

    //Save
    hook.register(Key::Press(KeyCode::F2), {
        let state = Arc::clone(&state);

        move || {
            let SharedState { ref mut saves, ref mut actions, .. } = &mut *state.lock();

            saves.push(actions.clone());
            actions.clear();
        }
    })
    .unwrap();

    //Replay
    hook.register(Key::Press(KeyCode::F3), {
        let state = Arc::clone(&state);

        move || {
            let SharedState { saves, ref mut enigo, .. } = &mut *state.lock();

            println!("{:?}", saves);
            replay(enigo, &saves.last().unwrap());
        }
    })
    .unwrap();

    //Exit
    hook.register(Key::Press(KeyCode::F4), || panic!()).unwrap();

    std::thread::sleep(Duration::from_secs(6e20 as u64));
}
