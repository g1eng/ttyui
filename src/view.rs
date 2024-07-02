use console::{Key, Term};
use std::io;

const BLANK_LINE: &str = "                                                                      ";

// use super::helper::{clear, clearln, writeln};
use std::fmt::Display;

/// Selection represents a record selection with Picker or Selector view.
///
pub struct Selection {
    pub selected_ids: Vec<usize>,
    pub command_key: Key,
}

/// Cell style category
///
#[derive(PartialEq, Eq)]
pub enum Style {
    Blink,
    Underbar,
    Inverse,
    Slash,
    Red,
    Green,
    Yellow,
    Blue,
    Purple,
    Moss,
    White,
    Emergency,
    Danger,
    Warning,
    BackgroundRed,
    BackgroundGreen,
    BackgroundYellow,
    BackgroundBlue,
    BackgroundPurple,
    BackgroundMoss,
    BackgroundWhite,
    None,
}

impl ToString for Style {
    fn to_string(&self) -> String {
        match self {
            Self::Underbar => "\x1b[4m",
            Self::Blink => "\x1b[5m",
            Self::Inverse => "\x1b[7m",
            Self::Slash => "\x1b[9m",
            Self::Red => "\x1b[31m",
            Self::Green => "\x1b[32m",
            Self::Yellow => "\x1b[33m",
            Self::Blue => "\x1b[34m",
            Self::Purple => "\x1b[35m",
            Self::Moss => "\x1b[36m",
            Self::White => "\x1b[37m",
            Self::Emergency => "\x1b[5m\x1b[41m",
            Self::Danger => "\x1b[5m\x1b[31m",
            Self::Warning => "\x1b[43m\x1b[37m",
            Self::BackgroundRed => "\x1b[41m",
            Self::BackgroundGreen => "\x1b[42m",
            Self::BackgroundYellow => "\x1b[43m",
            Self::BackgroundBlue => "\x1b[44m",
            Self::BackgroundPurple => "\x1b[45m",
            Self::BackgroundMoss => "\x1b[46m",
            Self::BackgroundWhite => "\x1b[47m",
            _ => "",
        }
        .to_string()
    }
}

/// StyleConditioner is a function which detects and return a style for a cell.
///
pub type StyleConditioner = fn(target_field: &str) -> io::Result<Style>;

/// A valiant of StyleConditioner, which acts on the target but refers the second field for
/// the condition checking for the styling.
///
pub type StyleDoubleConditioner = fn(target_field: &str, ref_field: &str) -> io::Result<Style>;

trait StyleFunc {}
impl StyleFunc for StyleConditioner {}
impl StyleFunc for StyleDoubleConditioner {}

pub struct Table {
    data: Vec<Vec<String>>,
    cursor_pos: usize,
}

impl Drop for Table {
    fn drop(&mut self) {
        for i in 0..self.data.len() {
            for j in 0..self.data[i].len() {
                self.data[i][j].clear();
            }
        }
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.data.len() {
            write!(f, "{}\n", self.data[i].join("  "))?;
        }
        Ok(())
    }
}

impl Table {
    /// Generate a new View instance
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(8),
            cursor_pos: 1,
        }
    }
    /// Generate a View instance with pre-defined data table
    ///
    pub fn from(data: Vec<Vec<String>>) -> Self {
        let mut t = Self {
            data,
            cursor_pos: 1,
        };
        t.prettify().unwrap();
        t
    }

    pub fn with_capacity(u: usize) -> Self {
        Self {
            data: Vec::with_capacity(u),
            cursor_pos: 1,
        }
    }

    /// format tables with cell padding calculated with maximum lengths of row record strings
    ///
    pub fn prettify(&mut self) -> io::Result<&mut Self> {
        let mut row_width_max = Vec::with_capacity(self.data[0].len());
        for row in 0..self.data[0].len() {
            row_width_max.push(self.data[0][row].len());
            for col in 0..self.data.len() {
                if self.data[col][row].is_ascii() {
                    if self.data[col][row].len() > row_width_max[row] {
                        row_width_max[row] = self.data[col][row].len();
                        if row_width_max[row] > BLANK_LINE.len() {
                            row_width_max[row] = BLANK_LINE.len();
                        }
                    }
                } else {
                    let str_width = self.data[col][row].as_str().encode_utf16().count();
                    if str_width > row_width_max[row] {
                        row_width_max[row] = str_width;
                        if row_width_max[row] > BLANK_LINE.len() {
                            row_width_max[row] = BLANK_LINE.len();
                        }
                    }
                }
            }
        }
        for col in 0..self.data.len() {
            for row in 0..row_width_max.len() {
                let str_len = match self.data[col][row].is_ascii() {
                    true => self.data[col][row].len(),
                    false => self.data[col][row].encode_utf16().count() * 2,
                };
                if str_len > BLANK_LINE.len() {
                    let td_clone =
                        self.data[col][row].clone()[0..BLANK_LINE.len() - 3].to_string() + "...";
                    self.data[col][row].clear();
                    self.data[col][row] = td_clone;
                } else if str_len < row_width_max[row] {
                    self.data[col][row].push_str(&BLANK_LINE[str_len..row_width_max[row]]);
                }
            }
        }
        // eprintln!("row_width_max: {:?}", row_width_max);
        Ok(self)
    }

    /// get row sequence with complete matching of the `pattern`, if exist.
    ///
    fn get_row_id(&self, pattern: &str) -> io::Result<usize> {
        let mut seq = 0;
        for i in 0..self.data[0].len() {
            if self.data[0][i].as_str().trim() == pattern {
                seq = i;
                break;
            }
        }
        if seq == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("no such row category: {}", pattern).as_str(),
            ));
        }
        return Ok(seq);
    }

    /// Apply style for each cell of a row with a style callback (`StyleModifier`).
    ///
    /// Row id is selected with its header value. If the row with the specified header does not exist, it
    /// returns error.
    ///
    pub fn apply_style(&mut self, row_name: &str, f: StyleConditioner) -> io::Result<&mut Self> {
        let row_id = self.get_row_id(row_name)?;
        for i in 1..self.data.len() {
            let prefix = f(self.data[i][row_id].as_str().trim())?;
            if prefix != Style::None {
                self.data[i][row_id].insert_str(0, &prefix.to_string());
                self.data[i][row_id].push_str("\x1b[0m");
            }
        }
        Ok(self)
    }

    /// Apply style for each cell of a row, but use reference row with the second parameter.
    ///
    pub fn apply_style_double(
        &mut self,
        row_name_target: &str,
        row_name_refered: &str,
        f: StyleDoubleConditioner,
    ) -> io::Result<&mut Self> {
        let row_id = self.get_row_id(row_name_target)?;
        let reffered_row_id = self.get_row_id(row_name_refered)?;
        for i in 1..self.data.len() {
            let style = f(
                self.data[i][row_id].as_str().trim(),
                self.data[i][reffered_row_id].as_str().trim(),
            )?;
            if style != Style::None {
                self.data[i][row_id].insert_str(0, &style.to_string());
                self.data[i][row_id].push_str("\x1b[0m");
            }
        }
        Ok(self)
    }

    pub fn show(&self) {
        for col in 0..self.data.len() {
            println!("{}", self.data[col].join("  "));
        }
    }

    /// selecct one item from registered records
    ///
    pub fn select_one(&mut self) -> io::Result<Selection> {
        self.selectn(1)
    }

    /// select multiple items from registered records
    ///
    pub fn select(&mut self) -> io::Result<Selection> {
        if self.data.len() <= 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "too few data to select",
            ));
        }
        self.selectn(self.data.len() - 1)
    }

    /// select multiple items from registered records, up to `max_count`
    ///
    pub fn selectn(&mut self, max_count: usize) -> io::Result<Selection> {
        if self.data.len() <= 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "no data to select",
            ));
        } else if max_count == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "max selection must be greater than 0",
            ));
        }
        let term = Term::stdout();
        self.prettify()?;

        let mut selected_ids = Vec::with_capacity(self.data.len());
        self.data[0].insert(0, String::new());
        if max_count == 1 {
            for i in 0..self.data.len() {
                self.data[i].insert(0, " ".to_string());
            }
        } else if max_count > 1 {
            for i in 1..self.data.len() {
                self.data[i].insert(0, " ".to_string());
                self.data[i].insert(0, "[ ]".to_string());
            }
        }

        loop {
            term.clear_screen()?;

            self.data[self.cursor_pos][0].clear();
            self.data[self.cursor_pos][0] = String::from("\x1b[32m>\x1b[0m");
            self.show();

            match term.read_key()? {
                Key::ArrowUp | Key::Char('k') => {
                    self.data[self.cursor_pos][0].clear();
                    self.data[self.cursor_pos][0] = " ".to_string();
                    self.cursor_pos = match self.cursor_pos == 1 {
                        true => self.data.len() - 1,
                        false => self.cursor_pos - 1,
                    }
                }
                Key::ArrowDown | Key::Char('j') => {
                    self.data[self.cursor_pos][0].clear();
                    self.data[self.cursor_pos][0] = " ".to_string();
                    self.cursor_pos = match self.cursor_pos == self.data.len() - 1 {
                        true => 1,
                        false => self.cursor_pos + 1,
                    }
                }
                x => match max_count {
                    1 => match x {
                        Key::Enter => {
                            if selected_ids.len() == 0 {
                                continue;
                            }
                            return Ok(Selection {
                                selected_ids,
                                command_key: Key::Enter,
                            });
                        }
                        Key::Char('u') | Key::Char('d') | Key::Char('i') | Key::Char('a') => {
                            return Ok(Selection {
                                selected_ids: [self.cursor_pos - 1].to_vec(),
                                command_key: x,
                            })
                        }
                        Key::Char('b') => {
                            return Ok(Selection {
                                selected_ids: Vec::with_capacity(0),
                                command_key: Key::Char('b'),
                            })
                        }
                        _ => {}
                    },
                    1.. => match x {
                        Key::Char(' ') => {
                            let pos = self.cursor_pos - 1;
                            if selected_ids.contains(&pos) {
                                self.data[self.cursor_pos][1].clear();
                                self.data[self.cursor_pos][1] = "[ ]".to_string();
                                selected_ids.remove(selected_ids.binary_search(&pos).unwrap());
                            } else {
                                self.data[self.cursor_pos][1].clear();
                                self.data[self.cursor_pos][1] = "[*]".to_string();
                                selected_ids.push(pos);
                            }
                        }
                        Key::Enter => {
                            return Ok(Selection {
                                selected_ids: [self.cursor_pos - 1].to_vec(),
                                command_key: Key::Enter,
                            });
                        }
                        Key::Escape => {
                            return Ok(Selection {
                                selected_ids: Vec::with_capacity(0),
                                command_key: Key::Escape,
                            })
                        }
                        _ => {}
                    },
                    _ => panic!("unknown condition"),
                },
            };
        }
    }
}

#[cfg(test)]
mod test {
    use crate::view::*;
    use std::io;

    fn init_data() -> Vec<Vec<String>> {
        [
            [1, 2, 3, 4, 5].map(|d| d.to_string()),
            [12345, 23456, 34567, 45678, 56789].map(|d| d.to_string()),
            ["ana", "ini", "unu", "ene", "ono"].map(|d| d.to_string()),
            ["A", "B", "C", "D", "E"].map(|d| d.to_string()),
        ]
        .map(|e| e.to_vec())
        .to_vec()
    }

    #[test]
    fn test_from() {
        Table::from(init_data());
    }

    #[test]
    fn test_apply_style() {
        let mut t = Table::from(init_data());
        t.apply_style("2", |d: &str| -> io::Result<Style> {
            if d.len() > 3 {
                Ok(Style::Red)
            } else {
                Ok(Style::None)
            }
        })
        .unwrap();
    }

    #[test]
    fn test_apply_style_double() {
        let mut t = Table::from(init_data());
        t.apply_style_double("2", "3", |d: &str, r: &str| -> io::Result<Style> {
            if d.len() != 1 && !r.contains("4") {
                Ok(Style::Green)
            } else {
                Ok(Style::None)
            }
        })
        .unwrap();
    }
}
