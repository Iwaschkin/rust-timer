use super::{Command, RunFailure, Step, TerminalError, combine, command};
use ratatui::crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use std::io;

fn press(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new(code, modifiers))
}

fn with_kind(code: KeyCode, kind: KeyEventKind) -> Event {
    Event::Key(KeyEvent::new_with_kind(code, KeyModifiers::NONE, kind))
}

#[test]
fn keys_map_to_commands() {
    let cases = [
        (
            press(KeyCode::Char(' '), KeyModifiers::NONE),
            Command::StartPause,
        ),
        (press(KeyCode::Char('s'), KeyModifiers::NONE), Command::Skip),
        (
            press(KeyCode::Char('S'), KeyModifiers::SHIFT),
            Command::Skip,
        ),
        (press(KeyCode::Char('q'), KeyModifiers::NONE), Command::Quit),
        (
            press(KeyCode::Char('Q'), KeyModifiers::SHIFT),
            Command::Quit,
        ),
        (press(KeyCode::Esc, KeyModifiers::NONE), Command::Quit),
        (
            press(KeyCode::Char('c'), KeyModifiers::CONTROL),
            Command::Quit,
        ),
    ];
    for (event, expected) in cases {
        assert_eq!(command(&event), Some(expected), "{event:?}");
    }
}

#[test]
fn ignores_key_release_and_repeat() {
    for kind in [KeyEventKind::Release, KeyEventKind::Repeat] {
        for code in [
            KeyCode::Char(' '),
            KeyCode::Char('s'),
            KeyCode::Char('S'),
            KeyCode::Char('q'),
            KeyCode::Char('Q'),
            KeyCode::Esc,
        ] {
            let event = with_kind(code, kind);
            assert_eq!(command(&event), None, "{event:?}");
        }
    }
}

#[test]
fn unbound_input_is_ignored() {
    let mouse = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 3,
        row: 2,
        modifiers: KeyModifiers::NONE,
    };
    let events = [
        press(KeyCode::Char('c'), KeyModifiers::NONE),
        press(KeyCode::Char('s'), KeyModifiers::CONTROL),
        press(KeyCode::Char('q'), KeyModifiers::ALT),
        press(KeyCode::Char('x'), KeyModifiers::NONE),
        press(KeyCode::Enter, KeyModifiers::NONE),
        Event::Mouse(mouse),
        Event::FocusGained,
        Event::FocusLost,
        Event::Paste(String::from("q")),
        Event::Resize(80, 24),
    ];
    for event in events {
        assert_eq!(command(&event), None, "{event:?}");
    }
}

fn failed(step: Step) -> TerminalError {
    TerminalError::Failed {
        step,
        source: io::Error::other("broken pipe"),
    }
}

#[test]
fn restore_failure_follows_run_failure() {
    assert!(combine(Ok(()), Ok(())).is_ok());
    let Err(RunFailure::Run(TerminalError::Failed {
        step: Step::Draw, ..
    })) = combine(Err(failed(Step::Draw)), Ok(()))
    else {
        panic!("a failed run with a clean restore reports the run only");
    };
    let Err(RunFailure::Restore(TerminalError::Failed {
        step: Step::Restore,
        ..
    })) = combine(Ok(()), Err(io::Error::other("tty gone")))
    else {
        panic!("a clean run with a failed restore reports the restore");
    };
    let Err(failure) = combine(
        Err(failed(Step::ReadInput)),
        Err(io::Error::other("tty gone")),
    ) else {
        panic!("both failures are reported");
    };
    let steps: Vec<String> = failure.errors().iter().map(ToString::to_string).collect();
    assert_eq!(
        steps,
        ["could not read input", "could not restore the terminal"]
    );
}
