use std::{fs::File, path::Path};

use anyhow::Result;
use csv::Writer;
use glob::glob;
use lazy_static::lazy_static;
use log::{debug, error, info, trace, warn};
use lopdf::Document;
use regex::Regex;
use serde::Serialize;

lazy_static! {
    static ref RE_DATE: Regex = Regex::new(r"\d{2}\.\d{2}\.\d{4}").unwrap();
    static ref RE_NUMBER: Regex = Regex::new(r"\d+,\d+").unwrap();
}
static STARTING_TEXT: &str = "ALTER SALDO";
static ENDING_TEXT: &str = "NEUER SALDO";

#[derive(Debug, PartialEq, Serialize)]
pub struct Transaction {
    pub date: String,
    pub description: String,
    pub amount: f64,
}

fn split_by_regex<'a>(text: &'a str, re: &Regex) -> Vec<&'a str> {
    let mut result = Vec::new();
    let mut last_end = 0;

    for mat in re.find_iter(text) {
        if mat.start() >= last_end {
            let part = &text[last_end..mat.start()];
            if !part.is_empty() {
                result.push(part);
            }
        }
        last_end = mat.start();
    }

    if last_end < text.len() {
        result.push(&text[last_end..]);
    }

    result
}

fn get_transactions(text: &str) -> Vec<Transaction> {
    let mut transactions = Vec::new();

    for part in split_by_regex(text, &RE_DATE) {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        let (date, part) = part.split_once("\n").unwrap_or_default();
        if RE_DATE.find(date).is_none() {
            continue;
        }

        let (description, amount) = RE_NUMBER
            .find_iter(part)
            .last()
            .map(|m| m.start())
            .map(|pos| {
                let (description, amount) = part.split_at(pos);
                (description.trim(), amount.trim())
            })
            .unwrap_or_default();

        let amount = match RE_NUMBER.find(amount) {
            Some(m) => m
                .as_str()
                .replace(",", ".")
                .parse::<f64>()
                .unwrap_or_default(),
            None => 0.0,
        };

        if date.is_empty() || description.is_empty() || amount == 0.0 {
            trace!("Failed to process part: {:?}", part);
            continue;
        }

        let tran = Transaction {
            date: date.trim().to_string(),
            description: description.replace("\n", ", ").trim().to_string(),
            amount,
        };
        debug!("Found transaction: {:?}", tran);
        transactions.push(tran);
    }

    transactions
}

fn get_transactions_from_pdf(document: &Document) -> Vec<Transaction> {
    let mut transactions = Vec::new();
    for page_num in 1..=document.get_pages().len() {
        match document.extract_text(&[page_num as u32]) {
            Ok(text) => {
                let start = text.find(STARTING_TEXT).unwrap_or(0);
                let end = text.find(ENDING_TEXT).unwrap_or(text.len());
                transactions.append(&mut get_transactions(&text[start..end]));
            }
            Err(e) => {
                error!("Failed to extract text from page {}: {}", page_num, e);
            }
        }
    }

    transactions
}

pub fn advanzia2csv(pdf_or_folder: &Path, csv_file: &Path) -> Result<()> {
    let paths = if pdf_or_folder.is_dir() {
        glob(&format!("{}/**/*.pdf", pdf_or_folder.display()))?
            .filter_map(Result::ok)
            .collect()
    } else {
        vec![pdf_or_folder.to_path_buf()]
    };

    let transactions: Vec<Transaction> = paths
        .iter()
        .flat_map(|pdf_path| match Document::load(pdf_path) {
            Ok(document) => {
                info!("Loading transactions from {}", pdf_path.display());
                get_transactions_from_pdf(&document)
            }
            Err(e) => {
                warn!("Failed to load PDF file {:?}: {}", pdf_path, e);
                Vec::new()
            }
        })
        .collect();

    if transactions.is_empty() {
        return Err(anyhow::anyhow!(
            "No transactions found in {}",
            pdf_or_folder.display()
        ));
    }

    info!(
        "{} transactions saved to {}",
        transactions.len(),
        csv_file.display()
    );

    let file = File::create(csv_file)?;
    let mut writer = Writer::from_writer(file);

    for record in transactions {
        writer.serialize(record)?;
    }
    writer.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_transactions() {
        let text = r#"some prefix26.01.2021
IKEA BORLANGE - SEK 111,00 (KURS 11,1111)
BORLANGE
18,30
27.02.2022
FABRIQUE - SEK 1111,00 (KURS 11,1111)
STOCKHOLM
19,23
27.11.2023
Inc. - SEK 111,11 (KURS 11,1111)
UPPLANDS VAS
14,62 some ending"#;
        let transactions = get_transactions(text);
        assert_eq!(transactions.len(), 3);
        assert_eq!(
            transactions[0],
            Transaction {
                date: "26.01.2021".to_string(),
                description: "IKEA BORLANGE - SEK 111,00 (KURS 11,1111), BORLANGE".to_string(),
                amount: 18.30,
            }
        );
        assert_eq!(
            transactions[1],
            Transaction {
                date: "27.02.2022".to_string(),
                description: "FABRIQUE - SEK 1111,00 (KURS 11,1111), STOCKHOLM".to_string(),
                amount: 19.23,
            }
        );
        assert_eq!(
            transactions[2],
            Transaction {
                date: "27.11.2023".to_string(),
                description: "Inc. - SEK 111,11 (KURS 11,1111), UPPLANDS VAS".to_string(),
                amount: 14.62,
            }
        );
    }
}
