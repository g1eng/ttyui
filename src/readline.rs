//! An alternative readline implementation.
//!
//! This module realizes traditional line editor implementation with Emacs-like shortcuts.
//!
//! ```rust
//! use std::env;
//! use ttyui::readline::Buffer;
//!
//! let mut buf = Buffer::from(&mut String::new());
//! if let Some(x) = env::args().nth(1) {
//!     if x == "-d" || x == "--double" {
//!         buf.double_line_response = true;
//!     } else {
//!         panic!("unknown arguments");
//!     }
//! }
//! buf.read_line().unwrap();
//! println!(
//!     "\n\n[output]\n\x1b[33m{}\x1b[0m",
//!     buf.to_string(),
//! );
//!
//! ```
//!

use console::{Key, Term};
use std::io;
use std::io::Write;

const MAX_PREFIX_CAPACITY: usize = 32;
const DEFAULT_TEXT_CAPACITY: usize = 1024;

/// Buffer of a readline instance.
///
pub struct Buffer<'a> {
    /// Debug mode or not.
    debug: bool,
    /// Whether the read_line method result newline-containing string at the index where an enter key has been pressed.
    pub double_line_response: bool,
    /// Whether the read_line method self.terminates or not when the ArrowUp or ArrowDown key is pressed.
    pub terminate_on_up_down: bool,
    term: Term,
    /// Cursor index for the next character input
    index: usize,
    /// prefix string for the input area
    prefix: String,
    /// Text payload for the buffer
    text: &'a mut String,
}

impl<'a> ToString for Buffer<'a> {
    fn to_string(&self) -> String {
        self.text.clone()
    }
}

impl<'a> Drop for Buffer<'a> {
    fn drop(&mut self) {
        self.double_line_response = false;
        self.index = 0;
        self.text.clear();
    }
}

impl<'a> std::fmt::Debug for Buffer<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Buffer")
            .field("debug", &self.debug)
            .field("double_line_response", &self.double_line_response)
            .field("terminate_on_up_down", &self.terminate_on_up_down)
            .field("index", &self.index)
            .field("text", &self.text);
        Ok(())
    }
}

impl<'a> Buffer<'a> {
    /// Generate buffer with initial text.
    ///
    pub fn from(text: &'a mut String) -> Self {
        Buffer {
            debug: false,
            double_line_response: false,
            terminate_on_up_down: false,
            term: Term::stdout(),
            index: 0,
            prefix: String::with_capacity(MAX_PREFIX_CAPACITY),
            text,
        }
    }

    /// Switch on debug mode
    ///
    pub fn debug(&mut self) {
        self.debug = true;
    }

    fn enter(&mut self) -> io::Result<Key> {
        if self.double_line_response {
            self.text.insert(self.index, '\n');
        }
        Ok(Key::Enter)
    }
    fn home(&mut self) -> io::Result<Key> {
        self.term.move_cursor_left(self.index)?;
        self.index = 0;
        Ok(Key::Home)
    }
    fn end(&mut self) -> io::Result<Key> {
        self.term.move_cursor_right(self.text.len() - self.index)?;
        self.index = self.text.len();
        Ok(Key::End)
    }
    fn char(&mut self, x: char) -> io::Result<Key> {
        self.term.move_cursor_right(self.text.len() - self.index)?;
        self.term.clear_chars(self.text.len() - self.index)?;
        self.text.insert(self.index, x);
        write!(&self.term, "{}", &self.text[self.index..self.text.len()])?;
        self.index += 1;
        self.term.move_cursor_left(self.text.len() - self.index)?;
        Ok(Key::Char(x))
    }
    fn backspace(&mut self) -> io::Result<Key> {
        if self.index > 0 {
            self.term.clear_line()?;
            self.term
                .move_cursor_left(self.text.len() + self.prefix.len())?;
            write!(&self.term, "{}", self.prefix)?;
            self.text.remove(self.index - 1);
            self.index -= 1;
            write!(&self.term, "{}", self.text)?;
            self.term.move_cursor_left(self.text.len() - self.index)?;
        }
        Ok(Key::Backspace)
    }
    fn del(&mut self) -> io::Result<Key> {
        if self.text.len() > 0 && self.text.len() > self.index {
            self.text.remove(self.index);
            self.term.clear_line()?;
            write!(&self.term, "{}", self.text)?;
            self.term.move_cursor_left(self.text.len() - self.index)?;
        }
        Ok(Key::Del)
    }
    fn esc(&mut self) -> io::Result<()> {
        match self.term.read_key()? {
            Key::Char('f') => {
                self.word_forward()?;
            }
            Key::Char('b') => {
                self.word_backword()?;
            }
            Key::Char('d') => {
                self.word_delete()?;
            }
            Key::Backspace => {
                self.word_backspace()?;
            }
            _ => {}
        }
        Ok(())
    }

    fn word_forward(&mut self) -> io::Result<()> {
        let mut separater_ids = self
            .text
            .match_indices(' ')
            .map(|t| t.0)
            .collect::<Vec<usize>>();
        separater_ids.push(self.text.len());
        for i in separater_ids {
            if i > self.index {
                self.term.move_cursor_right(i - self.index)?;
                self.index = i;
                break;
            }
        }
        Ok(())
    }

    fn word_backword(&mut self) -> io::Result<()> {
        let mut separater_ids = self
            .text
            .match_indices(' ')
            .map(|t| t.0 + 1)
            .collect::<Vec<usize>>();
        separater_ids.insert(0, 0);
        separater_ids.reverse();
        for i in separater_ids {
            if self.index > i {
                self.term.move_cursor_left(self.index - i)?;
                self.index = i;
                break;
            }
        }
        Ok(())
    }

    fn word_backspace(&mut self) -> io::Result<()> {
        let mut separater_ids = self
            .text
            .match_indices(' ')
            .map(|t| t.0)
            .filter(|n| *n < self.index)
            .collect::<Vec<usize>>();
        separater_ids.insert(0, 0);
        separater_ids.reverse();

        if self.text.len() != 0 {
            let target_id = separater_ids[0];
            for i in target_id + 1..self.index {
                self.text.remove(i);
            }
            self.index = target_id;
            self.term.clear_line()?;
            write!(&self.term, "{}", self.text)?;
            self.term.move_cursor_left(self.text.len() - target_id)?;
        }

        Ok(())
    }

    fn word_delete(&mut self) -> io::Result<()> {
        let mut separater_ids = self
            .text
            .match_indices(' ')
            .map(|t| t.0)
            .filter(|n| *n > self.index)
            .collect::<Vec<usize>>();
        separater_ids.push(self.text.len());
        let target_id = separater_ids[0];
        for i in target_id + 1..self.index {
            self.text.remove(i);
        }
        self.term.clear_line()?;
        write!(&self.term, "{}", self.text)?;
        self.term.move_cursor_left(self.text.len() - self.index)?;
        Ok(())
    }

    fn left(&mut self) -> io::Result<Key> {
        if self.index > 0 {
            self.term.move_cursor_left(1)?;
            self.index -= 1;
        }
        Ok(Key::ArrowLeft)
    }
    fn right(&mut self) -> io::Result<Key> {
        if self.index < self.text.len() {
            self.term.move_cursor_right(1)?;
            self.index += 1;
        }
        Ok(Key::ArrowRight)
    }

    /// set prefix for the input area
    pub fn set_prefix(&mut self, prefix: String) {
        self.prefix.clear();
        self.prefix = prefix;
    }

    ///Buffer.read_line provides interactive line editing functionality for a tty, which supports following basic shortcut keys:
    ///
    /// * C-a (Home)
    /// * C-e (End)
    /// * M-d (word delete)
    /// * M-f (word forward)
    /// * M-b (word backward)
    ///
    pub fn read_line(&mut self) -> io::Result<Key> {
        let k: Key;

        write!(&self.term, "{}{}", self.prefix, self.text)?;
        self.term.move_cursor_to(self.prefix.len(), 0)?;
        loop {
            match self.term.read_key()? {
                Key::Enter => {
                    k = self.enter()?;
                    break;
                }
                Key::Home => {
                    self.home()?;
                }
                Key::End => {
                    self.end()?;
                }
                Key::ArrowRight => {
                    self.right()?;
                }
                Key::ArrowLeft => {
                    self.left()?;
                }
                Key::Backspace => {
                    self.backspace()?;
                }
                Key::Del => {
                    self.del()?;
                }
                Key::Char(x) => {
                    self.char(x)?;
                }
                Key::Escape => {
                    self.esc()?;
                }
                Key::ArrowUp => {
                    if self.terminate_on_up_down {
                        k = Key::ArrowUp;
                        break;
                    }
                }
                Key::ArrowDown => {
                    if self.terminate_on_up_down {
                        k = Key::ArrowDown;
                        break;
                    }
                }
                _ => {}
            }
        }
        Ok(k)
    }
}

/// A shortcut to Buffer.read_line()?.to_string.
///
/// Its response contains no newline.
///
/// ```rust
/// use ttyui::readline::read_line;
/// println!("\n\n[output]\n\x1b[33m{}\x1b[0m", read_line().unwrap());
/// ```
///
pub fn read_line() -> io::Result<String> {
    let mut s = &mut String::new();
    let mut buf = Buffer::from(&mut s);
    buf.read_line()?;
    Ok(buf.to_string())
}

/// A shortcut to Buffer.read_line()?.to_string, but returns double line (which contains newline character in the response).
///
/// ```rust
/// use ttyui::readline::read_line2;
/// println!("\n\n[output]\n\x1b[33m{}\x1b[0m", read_line2().unwrap());
/// ```
///
pub fn read_line2() -> io::Result<String> {
    let mut s = &mut String::new();
    let mut buf = Buffer::from(&mut s);
    buf.double_line_response = true;
    buf.read_line()?;
    Ok(buf.to_string())
}

#[cfg(test)]
mod tests {
    use crate::readline::*;

    const DUMMY_WORD: &str = "kabukiza";
    const DUMMY_TEXT: &str = "okachimachi koshigaya inogashira suidobashi ochanomidzu";
    const DUMMY_INDEX: usize = 19;

    #[test]
    fn test_new() {
        let mut s = DUMMY_TEXT.to_string();
        let mut b = Buffer::from(&mut s);
        b.index = DUMMY_INDEX;
        assert!(!b.debug);
        assert!(!b.double_line_response);
    }

    #[test]
    fn test_home() {
        let mut s = DUMMY_TEXT.to_string();
        let mut b = Buffer::from(&mut s);
        b.index = DUMMY_INDEX;
        let h = b.home().unwrap();
        assert_eq!(h, Key::Home);
        assert_eq!(b.index, 0);
    }

    #[test]
    fn test_end() {
        let mut s = DUMMY_TEXT.to_string();
        let mut b = Buffer::from(&mut s);
        b.index = DUMMY_INDEX;
        let k = b.end().unwrap();
        assert_eq!(k, Key::End);
        assert_eq!(b.index, b.text.len());
    }

    #[test]
    fn test_char_input_at_start_results_a_char() {
        let mut s = String::new();
        let mut b = Buffer::from(&mut s);
        let k = b.char('g').unwrap();
        assert_eq!(k, Key::Char('g'));
        assert_eq!(b.index, 1);
        assert_eq!(*b.text, "g".to_string());
    }

    #[test]
    fn test_char_input_before_word_results_inserted_char() {
        let mut s = DUMMY_WORD.to_string();
        let mut b = Buffer::from(&mut s);

        let mut text_swap = b.text.clone();
        let k = b.char('@').unwrap();
        assert_eq!(k, Key::Char('@'));
        assert_eq!(b.index, 1);
        text_swap.insert(0, '@');
        assert_eq!(*b.text, text_swap);
    }

    #[test]
    fn test_char_input_between_characters_inserted_char() {
        let mut s = DUMMY_TEXT.to_string();
        let mut b = Buffer::from(&mut s);
        b.index = DUMMY_INDEX;
        let idx_init = b.index;
        let mut text_swap = b.text.clone();
        let k = b.char('g').unwrap();
        assert_eq!(k, Key::Char('g'));
        assert_eq!(b.index, idx_init + 1);
        text_swap.insert(idx_init, 'g');
        assert_eq!(*b.text, text_swap);
    }

    #[test]
    fn test_string_input_results_modified_word() {
        let mut s = DUMMY_TEXT.to_string();
        let mut b = Buffer::from(&mut s);
        b.index = DUMMY_INDEX;
        let idx_init = b.index;
        let mut text_swap = b.text.clone();
        b.char('i').unwrap();
        b.char('t').unwrap();
        b.char('a').unwrap();
        b.char('i').unwrap();
        assert_eq!(b.index, idx_init + "itai".len());
        text_swap.insert_str(idx_init, "itai");
        assert_eq!(
            *b.text,
            "okachimachi koshigaitaiya inogashira suidobashi ochanomidzu".to_string()
        );
    }

    #[test]
    fn test_backspace_after_characters_removes_char() {
        let mut s = DUMMY_TEXT.to_string();
        let mut b = Buffer::from(&mut s);
        b.index = DUMMY_INDEX;
        let idx_init = b.index;
        let mut text_swap = b.text.clone();
        b.backspace().unwrap();
        assert_eq!(b.index, idx_init - 1);
        text_swap.remove(idx_init - 1);
        assert_eq!(*b.text, text_swap);
    }

    #[test]
    fn test_backspace_before_characters_has_no_effect() {
        let mut s = DUMMY_WORD.to_string();
        let mut b = Buffer::from(&mut s);
        let idx_init = b.index;
        let text_swap = b.text.clone();
        b.backspace().unwrap();
        assert_eq!(b.index, idx_init);
        assert_eq!(*b.text, text_swap);
    }

    #[test]
    fn test_delete_before_characters_results_shortened_string() {
        let mut s = DUMMY_WORD.to_string();
        let mut b = Buffer::from(&mut s);
        let idx_init = b.index;
        let text_init = b.text.clone();
        b.del().unwrap();
        assert_eq!(b.index, idx_init);
        assert_eq!(*b.text, text_init.as_str()[1..text_init.len()]);
    }

    #[test]
    fn test_delete_all_characters_results_blank_string() {
        let mut s = DUMMY_WORD.to_string();
        let mut b = Buffer::from(&mut s);
        for _ in 0..100 {
            b.del().unwrap();
        }
        assert_eq!(b.index, 0);
        assert_eq!(*b.text, "".to_string());
    }

    #[test]
    fn test_delete_many_after_a_character_results_trimmed_string() {
        let mut s = DUMMY_TEXT.to_string();
        let mut b = Buffer::from(&mut s);
        b.index = DUMMY_INDEX;
        let idx_init = b.index;
        let text_init = b.text.clone();
        for _ in 0..100 {
            b.del().unwrap();
        }
        assert_eq!(b.index, idx_init);
        assert_eq!(*b.text, text_init.as_str()[0..idx_init]);
    }

    #[test]
    fn test_go_word_foward_rearrange_cursor_to_next_word_separator() {
        let mut s = DUMMY_TEXT.to_string();
        let mut b = Buffer::from(&mut s);
        b.index = DUMMY_INDEX;
        b.word_forward().unwrap();
        assert_eq!(
            b.index,
            DUMMY_TEXT.match_indices(' ').map(|t| t.0).nth(1).unwrap()
        );
    }

    #[test]
    fn test_go_word_backward_rearrange_cursor_to_previous_word_head() {
        let mut s = DUMMY_TEXT.to_string();
        let mut b = Buffer::from(&mut s);
        b.index = DUMMY_INDEX;
        b.word_backword().unwrap();
        assert_eq!(
            b.index,
            DUMMY_TEXT.match_indices(' ').map(|t| t.0).nth(0).unwrap() + 1
        );
    }

    #[test]
    fn test_word_backspace_removes_partial_string_from_current_word() {
        let mut s = DUMMY_TEXT.to_string();
        let mut b = Buffer::from(&mut s);
        b.index = DUMMY_INDEX;
        let idx_init = b.index;
        let text_init = b.text.clone();
        let idx_prev_space: usize = DUMMY_TEXT
            .match_indices(' ')
            .map(|t| t.0)
            .filter(|n| *n < idx_init)
            .last()
            .unwrap();
        b.word_backspace().unwrap();
        assert_eq!(b.index, idx_prev_space);
        assert_eq!(
            *b.text,
            format!(
                "{}{}",
                &text_init[0..idx_prev_space],
                &text_init[idx_init..text_init.len()]
            )
        );
    }

    #[test]
    fn test_word_delete_removes_partial_string_from_current_word() {
        let mut s = DUMMY_TEXT.to_string();
        let mut b = Buffer::from(&mut s);
        b.index = DUMMY_INDEX;
        let idx_init = b.index;
        let text_init = b.text.clone();
        let idx_next_space: usize = DUMMY_TEXT
            .match_indices(' ')
            .map(|t| t.0)
            .filter(|n| *n >= idx_init)
            .nth(0)
            .unwrap();
        b.word_delete().unwrap();
        assert_eq!(b.index, idx_init);
        assert_eq!(
            *b.text,
            format!(
                "{}{}",
                &text_init[0..idx_init],
                &text_init[idx_next_space..text_init.len()]
            )
        );
    }

    #[test]
    fn test_left_key_after_characters_results_cursor_shift() {
        let mut s = DUMMY_TEXT.to_string();
        let mut b = Buffer::from(&mut s);
        b.index = DUMMY_INDEX;
        let idx_init = b.index;
        b.left().unwrap();
        assert_eq!(b.index, idx_init - 1);
    }

    #[test]
    fn test_right_key_after_all_character_results_cursor_shift() {
        let mut s = DUMMY_WORD.to_string();
        let mut b = Buffer::from(&mut s);
        b.index = b.text.len();
        let idx_init = b.text.len();
        b.right().unwrap();
        assert_eq!(b.index, idx_init);
    }

    #[test]
    fn test_right_key_before_characters_results_cursor_shift() {
        let mut s = DUMMY_WORD.to_string();
        let mut b = Buffer::from(&mut s);
        let idx_init = b.index;
        b.right().unwrap();
        assert_eq!(b.index, idx_init + 1);
    }

    #[test]
    fn test_left_key_before_characters_has_no_effect() {
        let mut s = DUMMY_WORD.to_string();
        let mut b = Buffer::from(&mut s);
        let idx_init = b.index;
        b.left().unwrap();
        assert_eq!(b.index, idx_init);
    }

    #[test]
    fn test_set_prefix() {
        let mut s = DUMMY_WORD.to_string();
        let mut b = Buffer::from(&mut s);
        let data = "korekara";
        assert_eq!(b.prefix.len(), 0);
        b.set_prefix(data.to_string());
        assert_eq!(b.prefix.len(), data.len());
    }
}
