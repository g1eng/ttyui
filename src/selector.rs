//! Various selectors for items, numbers, date and times.
//!

use std::io;
use std::io::Write;

use chrono::{DateTime, Days, Duration, Local, Months};
use console::{Key, Term};

/// DateTimeField represents selector field for date and time.
///
#[derive(Clone, Eq, PartialEq, Debug)]
enum DateTimeField {
    Day,
    Month,
    Year,
    Hour,
    Minute,
    Second,
}

/// The ring buffer implementation for date and time, especially for the order of fields.
///
impl DateTimeField {
    fn switch_prev(&mut self) -> Self {
        match self {
            Self::Year => Self::Second,
            Self::Month => Self::Year,
            Self::Day => Self::Month,
            Self::Hour => Self::Day,
            Self::Minute => Self::Hour,
            Self::Second => Self::Minute,
        }
    }
    fn switch_next(&mut self) -> Self {
        match self {
            Self::Year => Self::Month,
            Self::Month => Self::Day,
            Self::Day => Self::Hour,
            Self::Hour => Self::Minute,
            Self::Minute => Self::Second,
            Self::Second => Self::Year,
        }
    }
}

const DEFAULT_DATE_NAME: &str = "due date";

/// The interactive selector interface for date and time.
///
/// By default, `DateSelector::new()` returns a selector for **date**, NOT FOR **date** and **time**.
/// If you want to select date and time with a selector, set DateSelector.has_time true
/// before calling select() method.
///
/// An instance for the date selection must be mutable and the selected date (or datetime) can be
/// extracted within different formats:
///
/// * DateSelector.get_date() -> `chrono::DateTime<Local>`
/// * DateSelector.to_string() -> String
///
/// ```rust
/// use ttyui::selector::DateSelector;
/// let mut d = DateSelector::new();
/// d.has_time = true;
/// println!("selected: {}", d.select().unwrap().to_string());
/// ```
///
#[derive(Clone, Debug)]
pub struct DateSelector {
    /// date name for the selection
    pub name: String,
    /// whether the selector supports time selection or not
    pub has_time: bool,
    /// active (on-cursor) field for the selection
    active_field: DateTimeField,
    /// selected date (datetime)
    date: DateTime<Local>,
    /// terminal instance for reference
    term: Term,
}

impl DateSelector {
    /// Generate selector instance with current date/time
    ///
    pub fn new() -> Self {
        Self {
            name: DEFAULT_DATE_NAME.to_string(),
            active_field: DateTimeField::Day,
            date: Local::now(),
            has_time: false,
            term: Term::stdout(),
        }
    }

    /// Generate selector instance with initial date
    ///
    pub fn from(date: DateTime<Local>) -> Self {
        Self {
            name: DEFAULT_DATE_NAME.to_string(),
            active_field: DateTimeField::Day,
            date,
            has_time: false,
            term: Term::stdout(),
        }
    }

    /// Set date, not interactively.
    ///
    pub fn set_date(&mut self, date: DateTime<Local>) {
        self.date = date;
    }

    /// This method detects whether the instance supports the field under the cursor.
    ///
    /// If the instance has no time range support, (but supports date only), it returns
    /// true for time ranges (Hour | Minute | Second) selected.
    ///
    fn is_out_of_field(&self) -> bool {
        match &self.has_time {
            true => false,
            false => match self.active_field {
                DateTimeField::Year | DateTimeField::Month | DateTimeField::Day => false,
                _ => true,
            },
        }
    }

    /// Move left for ring-bufferish selection field.
    ///
    pub fn left(&mut self) -> io::Result<()> {
        self.active_field = self.active_field.switch_prev();
        if self.is_out_of_field() {
            self.active_field = DateTimeField::Day;
        }
        self.adjust()?;
        Ok(())
    }

    /// Move right for ring-bufferish selection field.
    ///
    pub fn right(&mut self) -> io::Result<()> {
        self.active_field = self.active_field.switch_next();
        if self.is_out_of_field() {
            self.active_field = DateTimeField::Year;
        }
        self.adjust()?;
        Ok(())
    }

    /// Adjust cursor position before selection, after date characters written.
    ///
    fn adjust(&self) -> io::Result<()> {
        let msg_len = self.to_string().len();
        self.term.move_cursor_left(msg_len)?;
        match &self.active_field {
            DateTimeField::Year => self.term.move_cursor_right(3)?,
            DateTimeField::Month => self.term.move_cursor_right(6)?,
            DateTimeField::Day => self.term.move_cursor_right(9)?,
            DateTimeField::Hour => self.term.move_cursor_right(12)?,
            DateTimeField::Minute => self.term.move_cursor_right(15)?,
            DateTimeField::Second => self.term.move_cursor_right(18)?,
        };
        Ok(())
    }

    /// Increment a value under the cursor.
    ///
    pub fn up(&mut self) -> io::Result<()> {
        match &self.active_field {
            DateTimeField::Year => {
                self.date = self.date.checked_add_months(Months::new(12)).unwrap();
            }
            DateTimeField::Month => {
                self.date = self.date.checked_add_months(Months::new(1)).unwrap();
            }
            DateTimeField::Day => {
                self.date = self.date.checked_add_days(Days::new(1)).unwrap();
            }
            _ => {
                match &self.has_time {
                    true => match &self.active_field {
                        DateTimeField::Hour => {
                            self.date = self.date + Duration::hours(1);
                        }
                        DateTimeField::Minute => {
                            self.date = self.date + Duration::minutes(1);
                        }
                        DateTimeField::Second => {
                            self.date = self.date + Duration::seconds(1);
                        }
                        _ => {}
                    },
                    false => {
                        self.active_field = DateTimeField::Day;
                    }
                };
            }
        };
        Ok(())
    }

    /// Decrement a value under the cursor.
    ///
    pub fn down(&mut self) -> io::Result<()> {
        match &self.active_field {
            DateTimeField::Year => {
                self.date = self.date.checked_sub_months(Months::new(12)).unwrap();
            }
            DateTimeField::Month => {
                self.date = self.date.checked_sub_months(Months::new(1)).unwrap();
            }
            DateTimeField::Day => {
                self.date = self.date.checked_sub_days(Days::new(1)).unwrap();
            }
            _ => {
                match self.has_time {
                    true => match &self.active_field {
                        DateTimeField::Hour => {
                            self.date = self.date - Duration::hours(1);
                        }
                        DateTimeField::Minute => {
                            self.date = self.date - Duration::minutes(1);
                        }
                        DateTimeField::Second => {
                            self.date = self.date - Duration::seconds(1);
                        }
                        _ => {}
                    },
                    false => {
                        self.active_field = DateTimeField::Day;
                    }
                };
            }
        };
        Ok(())
    }

    /// Return selected date.
    ///
    pub fn get_date(&self) -> DateTime<Local> {
        self.date.clone()
    }

    /// Select date interactively.
    ///
    /// ```rust
    /// use ttyui::selector::DateSelector;
    /// let mut d = DateSelector::new();
    /// d.has_time = true;
    /// println!("selected: {}", d.select().unwrap().to_string());
    /// ```
    ///
    pub fn select(&mut self) -> io::Result<&mut Self> {
        loop {
            self.term.clear_screen()?;
            write!(&self.term, "{}: {}", self.name, self.to_string())?;
            self.adjust()?;

            match self.term.read_key()? {
                Key::ArrowLeft => {
                    self.left()?;
                    self.adjust()?;
                }
                Key::ArrowRight => {
                    self.right()?;
                    self.adjust()?;
                }
                Key::ArrowUp => {
                    self.up()?;
                }
                Key::ArrowDown => {
                    self.down()?;
                }
                Key::Enter => break,
                _ => {}
            };
        }
        self.term.clear_screen()?;
        Ok(self)
    }
}

impl ToString for DateSelector {
    fn to_string(&self) -> String {
        match self.has_time {
            true => format!("{}", self.date.format("%Y-%m-%d %H:%M:%S")),
            false => format!("{}", self.date.format("%Y-%m-%d")),
        }
    }
}

/// Select date with default conditions
///
pub fn select_date(initial_date: DateTime<Local>) -> io::Result<DateTime<Local>> {
    // println!("input {:?}", initial_date);
    Ok(DateSelector::from(initial_date).select()?.get_date())
}

/// Select date with time range
///
pub fn select_datetime(initial_date: DateTime<Local>) -> io::Result<DateTime<Local>> {
    // println!("input {:?}", initial_date);
    let mut t = DateSelector::from(initial_date);
    t.has_time = true;
    Ok(t.select()?.get_date())
}

/// Select date with custom date title
///
pub fn select_date_with_title(
    initial_date: DateTime<Local>,
    title: &str,
) -> io::Result<DateTime<Local>> {
    let mut t = DateSelector::from(initial_date);
    t.has_time = false;
    t.name = title.to_string();
    Ok(t.select()?.get_date())
}

/// Select date with time range and custom date title
///
pub fn select_datetime_with_title(
    initial_date: DateTime<Local>,
    title: &str,
) -> io::Result<DateTime<Local>> {
    // println!("input {:?}", initial_date);
    let mut t = DateSelector::from(initial_date);
    t.has_time = true;
    t.name = title.to_string();
    Ok(t.select()?.get_date())
}

/// A traditional selector to tell user something and requests `y` or `n`.
///
pub fn ask_yes_no(question_msg: &str) -> io::Result<bool> {
    let mut term = Term::stdout();
    let mut msg = format!("{}: ", question_msg);

    write!(term, "{}", msg)?;
    loop {
        match term.read_key().unwrap() {
            Key::Char('Y') | Key::Char('y') => {
                write!(term, "y\n")?;
                return Ok(true);
            }
            Key::Char('N') | Key::Char('n') => {
                write!(term, "n\n")?;
                return Ok(false);
            }
            _ => {
                term.clear_chars(msg.len())?;
                term.move_cursor_left(msg.len())?;
                msg = "Answer with y or n: ".to_string();
                write!(term, "{}", msg)?;
                continue;
            }
        }
    }
}

/// Item selection interface for a slice of descriptions.
///
/// This method returns a selected line with new String literal, or io::Error::Other for `Q` or escape key pressed.
///
/// ```rust
/// use ttyui::selector::select_word_from_words;
///
/// let animals = [
///     "Elephant",
///     "Horse",
///     "Whale",
///     "Tiger",
///     "Panda",
/// ];
/// println!("selected: {}",select_word_from_words("your favorite animal", &animals).unwrap());
/// ```

pub fn select_word_from_words(description: &str, items: &[&str]) -> io::Result<String> {
    let term = Term::stdout();
    term.clear_line()?;
    let mut seq = 0;
    let word_count = items.len();
    let mut table: Vec<&str> = Vec::with_capacity(word_count);
    table.push("\x1b[32m*\x1b[0m");
    for _ in 0..word_count - 1 {
        table.push(" ");
    }
    loop {
        term.clear_screen()?;
        term.write_line(description)?;
        for i in 0..word_count {
            write!(&term, "{} {}\n", table[i], items[i])?;
        }
        seq = match term.read_key().unwrap() {
            Key::ArrowUp | Key::Char('k') => {
                if seq == 0 {
                    word_count - 1
                } else {
                    seq - 1
                }
            }
            Key::ArrowDown | Key::Char('j') => {
                if seq == word_count - 1 {
                    0
                } else {
                    seq + 1
                }
            }
            Key::Char('q') | Key::Char('Q') | Key::Escape => {
                term.clear_screen()?;
                break;
            }
            Key::Enter => {
                term.clear_screen()?;
                return Ok(String::from(items[seq]));
            }
            _ => seq,
        };

        for i in 0..word_count {
            if i == seq {
                table[i] = "\x1b[32m*\x1b[0m";
            } else {
                table[i] = " ";
            }
        }
    }
    Err(io::Error::new(io::ErrorKind::Other, "quit"))
}

#[cfg(test)]
mod date_selector_tests {
    use crate::selector::*;
    use chrono::{Duration, Months};
    use std::thread::sleep;
    use std::time;

    fn date_init() -> (DateSelector, DateSelector) {
        let o = DateSelector::new();
        (o.clone(), o)
    }

    fn datetime_init() -> (DateSelector, DateSelector) {
        let mut o = DateSelector::new();
        o.has_time = true;
        (o.clone(), o)
    }

    #[test]
    fn date_up_increments_day_by_default() {
        let (mut t, s) = date_init();
        t.up().unwrap();
        assert_eq!(t.get_date(), s.get_date() + Duration::days(1))
    }

    #[test]
    fn date_down_decrements_day_by_default() {
        let (mut t, s) = date_init();
        t.down().unwrap();
        assert_eq!(t.get_date(), s.get_date() - Duration::days(1))
    }

    #[test]
    fn date_left_down2_decrements_months() {
        let (mut t, s) = date_init();
        t.left().unwrap();
        t.down().unwrap();
        t.down().unwrap();
        assert_eq!(t.get_date(), s.get_date() - Months::new(2))
    }

    #[test]
    fn date_left_up_down_results_same_date() {
        let (mut t, s) = date_init();
        t.left().unwrap();
        t.up().unwrap();
        t.down().unwrap();
        assert_eq!(t.get_date(), s.get_date())
    }

    #[test]
    fn date_left2_down_decrements_year() {
        let (mut t, s) = date_init();
        t.left().unwrap();
        t.left().unwrap();
        t.down().unwrap();
        assert_eq!(t.get_date(), s.get_date() - Months::new(12))
    }

    #[test]
    fn date_left3_down_decrements_day() {
        let (mut t, s) = date_init();
        t.left().unwrap();
        t.left().unwrap();
        t.left().unwrap();
        t.down().unwrap();
        assert_eq!(t.active_field, s.active_field);
        assert_eq!(t.get_date(), s.get_date() - Duration::days(1));
    }

    #[test]
    fn datetime_left3_down_decrements_second() {
        let (mut t, s) = datetime_init();
        t.left().unwrap();
        t.left().unwrap();
        t.left().unwrap();
        t.down().unwrap();
        assert_eq!(t.get_date(), s.get_date() - Duration::seconds(1))
    }

    #[test]
    fn date_right_left_up_increments_day() {
        let (mut t, s) = date_init();
        t.right().unwrap();
        t.left().unwrap();
        t.up().unwrap();
        assert_eq!(t.get_date(), s.get_date() + Duration::days(1))
    }

    #[test]
    fn datetime_right_left_up_increments_day() {
        let (mut t, s) = datetime_init();
        t.right().unwrap();
        t.left().unwrap();
        t.up().unwrap();
        assert_eq!(t.get_date(), s.get_date() + Duration::days(1))
    }

    #[test]
    fn date_right_up_increments_year() {
        let (mut t, s) = date_init();
        t.right().unwrap();
        t.up().unwrap();
        assert_eq!(t.get_date(), s.get_date() + Months::new(12))
    }

    #[test]
    fn datetime_right_up_increments_hour() {
        let (mut t, s) = datetime_init();
        t.right().unwrap();
        t.up().unwrap();
        assert_eq!(t.get_date(), s.get_date() + Duration::hours(1))
    }

    #[test]
    fn datetime_right2_down2_decrements_minutes() {
        let (mut t, s) = datetime_init();
        t.right().unwrap();
        t.right().unwrap();
        t.down().unwrap();
        t.down().unwrap();
        assert_eq!(t.get_date(), s.get_date() - Duration::minutes(2))
    }

    #[test]
    fn datetime_right3_up_increments_second() {
        let (mut t, s) = datetime_init();
        t.right().unwrap();
        t.right().unwrap();
        t.right().unwrap();
        t.up().unwrap();
        assert_eq!(t.get_date(), s.get_date() + Duration::seconds(1))
    }

    #[test]
    fn datetime_right4_down_decrements_year() {
        let (mut t, s) = datetime_init();
        t.right().unwrap();
        t.right().unwrap();
        t.right().unwrap();
        t.right().unwrap();
        t.down().unwrap();
        assert_eq!(t.get_date(), s.get_date() - Months::new(12))
    }

    #[test]
    fn test_date_set_date() {
        let (mut t, s) = date_init();
        sleep(time::Duration::from_secs(1));
        t.set_date(Local::now());
        assert_ne!(t.get_date(), s.get_date())
    }
}
