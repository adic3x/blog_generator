use crate::trim_offset::TrimOffsets as _;

#[derive(Debug, PartialEq)]
pub struct Article {
    pub ts: Datetime,
    name:   std::ops::Range<usize>,
    body:   Option<std::ops::Range<usize>>,
    data:   String,
}

impl Article {
    pub fn new(s: String) -> Result<Self, &'static str> {
        if 
            let Some(delimiter) = s.find(' ') &&
            let Some(newline) = s.find('\n') &&
            let Some(ts) = s.get(..delimiter) &&
            let Some(name) = s.get(delimiter + 1..newline) &&
            let name_trim = name.trim_offsets() &&
            name_trim.length > 0 &&
            let Ok(ts) = ts.parse::<u64>()
        {
            let name_start = delimiter + 1 + name_trim.left;
            let name_range = name_start..name_start + name_trim.length;
            Ok(
                Self {
                    ts: Datetime(ts),
                    name: name_range.clone(),
                    body: s.get(newline + 1..).and_then(|s| {
                        let body_trim = s.trim_offsets();
                        match body_trim.length {
                            0 => None,
                            n => {
                                let body_start = newline + 1 + body_trim.left;
                                Some(body_start..body_start + n)
                            },
                        }
                    }),
                    data: s,
                }
            )
        } else {
            Err("can't parse article, expected '123456789 Article name\\nArticle Body'")
        }       
    }

    pub fn name(&self) -> &str {
        self.data.get(self.name.clone()).unwrap()
    }

    pub fn body(&self) -> Option<&str> {
        self.body.as_ref().and_then(|r| self.data.get(r.clone()))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Datetime(u64);

impl std::fmt::Display for Datetime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const SECONDS_PER_DAY: u64 = 86400;
        const DAYS_TO_0000_03_01: i64 = 719468;
        const DAYS_PER_ERA: i64 = 146097;
        const MONTHS: [&'static str; 12] = ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"];

        let total_days = (self.0 / SECONDS_PER_DAY) as i64 + DAYS_TO_0000_03_01;
        let era = total_days / DAYS_PER_ERA;
        let day_of_era = (total_days - era * DAYS_PER_ERA) as u32;
        let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36524 - day_of_era / 146096) / 365;
        let mut y = (year_of_era as i32) + (era as i32 * 400);
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let month_prime = (5 * day_of_year + 2) / 153;
        let d = day_of_year - (153 * month_prime + 2) / 5 + 1;
        let m = if month_prime < 10 { month_prime + 3 } else { month_prime - 9 };
        if m <= 2 {
            y += 1;
        }

        write!(f, "{d:02} {} {y}", MONTHS[(m - 1) as usize])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_article_parsing() {
        let cases = [
            ("1674547200 My First Post\nContent here", Some((1674547200, "My First Post", Some("Content here")))),
            ("999 Header Only\nBody with \n newlines", Some((999,        "Header Only",   Some("Body with \n newlines")))),
            ("777 Non ascїї\nNon ascїї",               Some((777,        "Non ascїї",     Some("Non ascїї")))),
            ("1337  Extra spaces  \n\n  Text  \n\n\n", Some((1337,       "Extra spaces",  Some("Text")))),
            ("123 SimpleTitle\n",                      Some((123,        "SimpleTitle",   None))),
            ("NoTimestamp\n",                          None),
            ("invalid_ts Name\nContent",               None),
            ("123 \nNo Name",                          None),
            ("123      \nNo Name",                     None),
            ("123 NoNewline",                          None),
        ];

        cases.into_iter().enumerate().for_each(|(i, (input, expected))| {
            let result = Article::new(input.to_string());

            match (&result, expected) {
                (Err(_), None) => (), 
                (Ok(a), Some((ts, name, body))) if a.ts == Datetime(ts) && a.name() == name && a.body() == body => (),
                _ => panic!("\n[Test {i} failed]\nInput: {input:?}\nResult: {result:?}\nExpected matches: {expected:?}\n"),
            }
        });
    }

        #[test]
    fn test_datetime_formatting() {
        assert_eq!(&format!("{}", Datetime(1770203100)), "04 feb 2026")
    }
}