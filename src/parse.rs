use regex::Regex;
use std::io::Write;

pub struct Row {
    pub date: String,
    pub desc: String,
    pub amount: String,
}

pub fn extract_rows(lines: &[String]) -> Vec<Row> {
    let date_re = Regex::new(
        r"(?i)(d{1,2}[/-.]d{1,2}[/-.]d{2,4})"
    ).unwrap();
    let amount_re = Regex::new(
        r"(-?$?d{1,3}(?:,d{3})*(?:.d{2}))"
    ).unwrap();

    let mut rows = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let date = date_re
            .find(trimmed)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();

        let amounts: Vec<String> = amount_re
            .find_iter(trimmed)
            .map(|m| m.as_str().to_string())
            .collect();

        let amount = amounts.last().cloned().unwrap_or_default();

        if date.is_empty() && amount.is_empty() {
            continue;
        }

        let desc = trimmed.to_string();

        rows.push(Row { date, desc, amount });
    }

    rows
}

pub fn write_csv(rows: &[Row], w: &mut impl Write) -> anyhow::Result<()> {
    let mut wtr = csv::Writer::from_writer(w);
    wtr.write_record(["date", "description", "amount"])?;
    for row in rows {
        wtr.write_record([&row.date, &row.desc, &row.amount])?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn write_text(lines: &[String], w: &mut impl Write) -> anyhow::Result<()> {
    for line in lines {
        writeln!(w, "{}", line)?;
    }
    Ok(())
}
