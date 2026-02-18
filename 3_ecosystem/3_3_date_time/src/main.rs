fn main() {
    println!("Implement me!");
}

const NOW: &str = "2019-06-26";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Date {
    year: i32,
    month: u32,
    day: u32,
}

impl Date {
    fn new(year: i32, month: u32, day: u32) -> Option<Self> {
        if !(1..=12).contains(&month) {
            return None;
        }

        let max_day = days_in_month(year, month);
        if !(1..=max_day).contains(&day) {
            return None;
        }

        Some(Self { year, month, day })
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn now_date() -> Date {
    let mut parts = NOW.split('-');
    let (Some(year), Some(month), Some(day), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        unreachable!("NOW is a compile-time constant in YYYY-MM-DD format");
    };

    let (Ok(year), Ok(month), Ok(day)) = (year.parse(), month.parse(), day.parse()) else {
        unreachable!("NOW is a compile-time constant with numeric date components");
    };

    let Some(now) = Date::new(year, month, day) else {
        unreachable!("NOW is a compile-time constant with a valid date");
    };
    now
}

struct User {
    birthdate: Option<Date>,
}

impl User {
    fn with_birthdate(year: i32, month: u32, day: u32) -> Self {
        Self {
            birthdate: Date::new(year, month, day),
        }
    }

    /// Returns current age of [`User`] in years.
    fn age(&self) -> u16 {
        let Some(birthdate) = self.birthdate else {
            return 0;
        };

        let now = now_date();
        if birthdate > now {
            return 0;
        }

        let mut years = now.year - birthdate.year;
        if (now.month, now.day) < (birthdate.month, birthdate.day) {
            years -= 1;
        }

        if years <= 0 {
            return 0;
        }

        match u16::try_from(years) {
            Ok(age) => age,
            Err(_) => u16::MAX,
        }
    }

    /// Checks if [`User`] is 18 years old at the moment.
    fn is_adult(&self) -> bool {
        self.age() >= 18
    }
}

#[cfg(test)]
mod age_spec {
    use super::*;

    #[test]
    fn counts_age() {
        for ((y, m, d), expected) in vec![
            ((1990, 6, 4), 29),
            ((1990, 7, 4), 28),
            ((0, 1, 1), 2019),
            ((1970, 1, 1), 49),
            ((2019, 6, 25), 0),
        ] {
            let user = User::with_birthdate(y, m, d);
            assert_eq!(user.age(), expected);
        }
    }

    #[test]
    fn zero_if_birthdate_in_future() {
        for ((y, m, d), expected) in vec![
            ((2032, 6, 25), 0),
            ((2019, 6, 27), 0),
            ((3000, 6, 27), 0),
            ((9999, 6, 27), 0),
        ] {
            let user = User::with_birthdate(y, m, d);
            assert_eq!(user.age(), expected);
        }
    }

    #[test]
    fn checks_adult_boundary() {
        let not_adult = User::with_birthdate(2001, 6, 27);
        assert!(!not_adult.is_adult());

        let adult_on_birthday = User::with_birthdate(2001, 6, 26);
        assert!(adult_on_birthday.is_adult());
    }

    #[test]
    fn returns_zero_for_invalid_birthdate() {
        for (y, m, d) in vec![(2019, 2, 29), (2019, 13, 1), (2019, 0, 10)] {
            let user = User::with_birthdate(y, m, d);
            assert_eq!(user.age(), 0);
            assert!(!user.is_adult());
        }
    }
}
